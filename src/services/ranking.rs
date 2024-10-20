use std::collections::HashMap;
use crate::error::Result;
use actix_web::{get, HttpResponse, Responder};
use actix_web::web::Data;
use anyhow::anyhow;
use chrono::Duration;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use chrono::prelude::*;
use itertools::Itertools;
use log::{info, log};
use serde::Serialize;
use crate::AppState;
use crate::entities::{ballot, jogador};
use crate::entities::ballot::Column::Vote;
use crate::entities::prelude::{Ballot, Jogador};
use crate::templates::TEMPLATES;

pub const WEEK_IN_SECONDS: i64 = 604800;

pub fn get_last_reset() -> DateTime<Utc> {

    //reference reset date / time

    let past_reset : DateTime<Utc> = Local.with_ymd_and_hms(2024, 1, 7, 19, 0, 0).unwrap().with_timezone(&Utc);

    //current time
    let now: DateTime<Utc> = Utc::now();

    //get Duration since reference reset
    let d = now - past_reset;

    // divide by a week, and get the remainder
    let rem = d.num_seconds() % WEEK_IN_SECONDS;

    //subtract the remainder from current time
    let target = now - Duration::seconds(rem);

    //last reset / datetime
    target
}
#[get("/debugRanking")]
pub async fn debug_ranking(state: Data<AppState>) -> Result<impl Responder> {
    let db = &state.db;

    // Get the start of the week, ie. the last monday
    let start_of_week = get_last_reset();



    // COMPUTE RANKING
    // Get all votes this week
    let votes = Ballot::find()
        .filter(ballot::Column::State.contains("closed"))
        .filter(ballot::Column::Date.gt(start_of_week))
        .all(db)
        .await?;

    #[derive(Debug, Serialize, Clone)]
    struct RankingEntry {
        pos: i32,
        nome: String,
        media: f32,
        votos: i32,
        desvio_padrao: f32,
    }

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
        let non_zero_weight_votes = v.iter().filter(|(vote, weight)| *weight > 0.).count();
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
                        desvio_padrao: *std_dev,
                    }
                })
                .collect::<Vec<_>>();
    for (i, player) in players_mentioned.iter_mut().sorted_by(|a, b| b.media.partial_cmp(&a.media).unwrap()).enumerate() {
        player.pos = (i + 1) as i32;
    }

    let players_mentioned = players_mentioned.iter().sorted_by(|a, b| a.pos.cmp(&b.pos)).collect_vec();







    let mut context = tera::Context::new();
    context.insert("votes", &votes.iter().count());
    context.insert("last_reset", &start_of_week.with_timezone(&Local).format("%d/%m/%Y %H:%M:%S").to_string());
    context.insert("now", &Utc::now().with_timezone(&Local).format("%d/%m/%Y %H:%M:%S").to_string());
    context.insert("ranking", &players_mentioned);
    let page_content = TEMPLATES.render("debug_ranking.html", &context)?;
    Ok(HttpResponse::Ok().body(page_content))
}