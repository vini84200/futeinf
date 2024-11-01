use std::collections::HashMap;

use actix_web::web::Data;
use anyhow::ensure;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use log::info;
use rand::Rng;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};
use serde::{Deserialize, Serialize};

use crate::entities::prelude::{Apuracao, Ballot};
use crate::error::Result;
use crate::timings::ref_point_from_id;
use crate::{
    entities::{
        apuracao::{self},
        ballot,
        prelude::Jogador
    },
    services::ranking,
    AppState,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RankingEntry {
    pub pos: i32,
    pub nome: String,
    pub media: f32,
    pub votos: i32,
    pub desvio_padrao: Option<f32>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ranking {
    pub entries: Vec<RankingEntry>,
    pub timestamp: DateTime<Utc>,
    pub votes: i32,
}

const APURACAO_COMPLETE: &str = "complete";
const APURACAO_STARTED: &str = "started";

pub async fn prepare_apurar(state: Data<AppState>, id: i32) -> Result<()> {
    info!("Preparing apurar for event {}", id);
    // Fecha ballots
    let db = &state.db;
    let ballots = Ballot::find()
        .filter(ballot::Column::State.eq("open"))
        .filter(ballot::Column::FuteId.eq(id))
        .all(db)
        .await?;

    info!("Found {} open ballots", ballots.len());
    for ballot in ballots {
        let mut am = ballot.into_active_model();
        am.state = ActiveValue::Set("closed".to_string());
        am.save(db).await?;
    }
    info!("Closed all ballots");

    info!("Apurar prepared");

    Ok(())
}

pub async fn has_apurado(state: Data<AppState>, id: i32) -> Result<bool> {
    let db = &state.db;
    let apuracao = Apuracao::find()
        .filter(apuracao::Column::WeekId.eq(id))
        .one(db)
        .await?;

    Ok(apuracao.is_some())
}

pub async fn get_or_create_apuracao(state: Data<AppState>, id: i32) -> Result<Ranking> {
    let db = &state.db;
    let apuracao = Apuracao::find()
        .filter(apuracao::Column::WeekId.eq(id))
        .one(db)
        .await?;

    if let Some(apuracao) = apuracao {
        if apuracao.state != APURACAO_COMPLETE {
            return Err(anyhow::anyhow!("Apuração não está completa").into());
        }
        let ranking = apuracao.results;
        let ranking = serde_json::from_value::<Ranking>(ranking)?;
        Ok(ranking)
    } else {
        apurar_complete(state.clone(), id).await
    }
}

pub async fn apurar_complete(state: Data<AppState>, id: i32) -> Result<Ranking> {
    // Cria instancia de apuração, como id é unique key, se já existir, retorna erro
    let db = &state.clone().db;

    let rand_id = rand::thread_rng().gen::<i32>();

    let nova_apuracao = apuracao::ActiveModel {
        week_id: ActiveValue::Set(id),
        random_id: ActiveValue::Set(rand_id.to_string()),
        results: ActiveValue::Set(serde_json::to_value(Ranking {
            entries: vec![],
            timestamp: Utc::now(),
            votes: 0,
        })?),
        state: ActiveValue::Set(APURACAO_STARTED.to_string()),
        ..Default::default()
    };
    let mut nova_apuracao = nova_apuracao.save(db).await?;

    // Prepara apuração
    prepare_apurar(state.clone(), id).await?;

    let ranking = calculate_ranking(state, id).await?;

    nova_apuracao.results = ActiveValue::Set(serde_json::to_value(&ranking)?);
    nova_apuracao.state = ActiveValue::Set(APURACAO_COMPLETE.to_string());
    nova_apuracao.save(db).await?;

    Ok(ranking)
}

pub async fn calculate_ranking(state: Data<AppState>, id: i32) -> Result<Ranking> {
    let db = &state.clone().db;
    // Get the start of the week, ie. the last monday
    let start_of_week = ref_point_from_id(id);

    // COMPUTE RANKING
    // Get all votes this week
    let votes = Ballot::find()
        .filter(ballot::Column::State.contains("closed"))
        .filter(ballot::Column::FuteId.eq(id))
        .all(db)
        .await?;


    // Count how many votes each voter has cast
    let mut votes_per_voter = HashMap::new();
    for vote in &votes {
        let voter = vote.voter.clone();
        let count = votes_per_voter.entry(voter).or_insert(0);
        *count += 1;
    }

    info!("Votes per voter: {:?}", votes_per_voter);

    const MAX_ACC_VOTING_POWER: f32 = 2.5;
    let vote_power = votes_per_voter
        .iter()
        .map(|(v, count)| {
            if (*count as f32) <= MAX_ACC_VOTING_POWER {
                // Voting power can be 1 for each vote and the sum of the votes
                // can't be greater than MAX_ACC_VOTING_POWER
                (v, 1.)
            } else {
                // Cap the voting power to MAX_ACC_VOTING_POWER/number_of_votes
                (v, MAX_ACC_VOTING_POWER / (*count as f32))
            }
        })
        .collect::<HashMap<_, _>>();

    info!("Vote power: {:?}", vote_power);

    struct Vote {
        vote: f32,
        weight: f32,
    }

    // Compute the ranking
    let mut votes_per_player: HashMap<i32, Vec<(f32, f32)>> = HashMap::new();
    for vote in &votes {
        // Two categories of votes: "ranked" and "unranked"
        // Ranked votes are counted as beat/4 points
        // Where beat is the number of players that are ranked lower than the player plus the unranked votes
        // Unranked votes are counted as (number_of_unranked_votes - 1)/(4*2) points
        let ranked_votes = vote.vote.as_array().unwrap().iter().map(|x| x.as_i64().unwrap() as i32).collect::<Vec<_>>();
        let all_players =
            vote.players
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_i64().unwrap() as i32)
                .collect::<Vec<_>>();
        let unranked_votes = all_players.iter().filter(|x| !ranked_votes.contains(x)).collect::<Vec<_>>();
        let unranked_votes_count = unranked_votes.len() as f32;
        const NUMBER_OF_PLAYERS_IN_VOTE: usize = 4;
        let weight = vote_power.get(&vote.voter).unwrap();
        let mut ranked_votes = ranked_votes.iter().enumerate().map(|(i, player)| {
            let beat = NUMBER_OF_PLAYERS_IN_VOTE - i; //
            let vote = (beat as f32) / (NUMBER_OF_PLAYERS_IN_VOTE as f32);
            (*player, Vote { vote, weight: *weight })
        }).collect::<Vec<_>>();

        let unranked_votes_value = (unranked_votes_count - 1.) / (NUMBER_OF_PLAYERS_IN_VOTE as f32 * 2.);
        for player in unranked_votes {
            ranked_votes.push((*player, Vote { vote: unranked_votes_value, weight: *weight }));
        }

        for (player, vote) in ranked_votes {
            let entry = votes_per_player.entry(player).or_insert(vec![]);
            entry.push((vote.vote, vote.weight));
        }

    }

    info!("Votes per player: {:?}", votes_per_player);

    let mean : HashMap<i32, f32> = votes_per_player.iter().map(|(k, v)| {
        let sum = v.iter().map(|(vote, weight)| vote * weight).sum::<f32>();
        let weight = v.iter().map(|(_, weight)| weight).sum::<f32>();
        (*k, sum / weight)
    }).collect();

    info!("Mean: {:?}", mean);

    let std_dev : HashMap<i32, f32> = votes_per_player.iter().map(|(k, v)| {
        let mean = mean.get(k).unwrap();
        let sum = v.iter().map(|(vote, weight)| (vote - mean).powi(2) * weight).sum::<f32>();
        let weight = v.iter().map(|(_, weight)| weight).sum::<f32>();
        let non_zero_weight_votes = v.iter().filter(|(_, weight)| *weight > 0.).count();
        if non_zero_weight_votes == 0 {
            (*k, 0.)
        } else {
            let denominator = ((non_zero_weight_votes as f32 - 1.) * weight)/non_zero_weight_votes as f32;
            (*k, (sum / denominator).sqrt())
        }
    }).collect();

    info!("Std Dev: {:?}", std_dev);

    let all_players = Jogador::find().all(db).await?;

    let mut players_mentioned =
            all_players
                .iter()
                .filter(|x| votes_per_player.contains_key(&x.id))
                .map(|x| {
                    let mean = mean.get(&x.id).unwrap();
                    let std_dev = std_dev.get(&x.id).unwrap();
                    let votes = votes_per_player.get(&x.id).unwrap().iter().count();
                    RankingEntry {
                        pos: 0,
                        nome: x.nome.clone(),
                        media: *mean,
                        votos: votes as i32,
                        desvio_padrao: std_dev.is_finite().then(|| *std_dev)
                    }
                })
                .collect::<Vec<_>>();
    for (i, player) in players_mentioned.iter_mut().sorted_by(|a, b| b.media.partial_cmp(&a.media).unwrap()).enumerate() {
        player.pos = (i + 1) as i32;
    }

    let players_mentioned = players_mentioned.iter()
    .sorted_by(|a, b| a.pos.cmp(&b.pos))
    .cloned()
    .collect_vec();
    
    let ranking = Ranking {
        entries: players_mentioned,
        timestamp: Utc::now(),
        votes: votes.len() as i32,
    };

    Ok(
        ranking
    )
}
