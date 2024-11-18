use crate::templates::TEMPLATES;
use crate::timings::{can_cast_vote, can_create_ballot, get_end_elegible_check, get_ref_point_of, get_start_elegible_check, ref_point_from_id, ref_point_id};
use crate::{entities, list, AppState};
use sqlx::{prelude::*, query};
use actix_identity::Identity;
use actix_web::web::Data;
use actix_web::{get, post, web, HttpResponse, Responder};
use anyhow::anyhow;
use log::info;
use rand::prelude::SliceRandom;
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use entities::{prelude::*, *};
use crate::error::{Error, Result};

#[get("/voting")]
pub async fn voting(state: Data<AppState>) -> Result<impl Responder> {
    let context = tera::Context::new();
    let page_content = TEMPLATES.render("voting.html", &context)?;
    Ok(HttpResponse::Ok().body(page_content))
}

#[get("/elegible")]
pub async fn get_elegible_players(
    state: Data<AppState>,
    identity: Option<Identity>
) -> Result<impl Responder> {
    let identity = identity.ok_or(anyhow!("Not logged in"))?;
    let db = &state.db;
    let now = chrono::Utc::now();

    let start_elegible_check = get_start_elegible_check(now);
    let end_elegible_check = get_end_elegible_check(now);

    let all_players : Vec<jogador::Model> = Jogador::find().all(db).await?;


    // Filter for players that have been active in the last 30 days
    let players: Vec<_> = query!(
        "select distinct email_address as email FROM ticket WHERE event_id IN 
        (SELECT id FROM event WHERE start_ts between $1 and $2)
         and email_address is not null;",
        start_elegible_check,
        end_elegible_check
    ).fetch_all(&state.alfio_db).await?;

    let extra_players = ListaExtra::find()
        .filter(lista_extra::Column::Data.gt(start_elegible_check))
        .filter(lista_extra::Column::Data.lt(end_elegible_check))
        .all(db)
        .await?;

    let extra_players = extra_players.iter().map(|f| f.jogador_id).collect::<Vec<_>>();


    let players = players.iter().map(|f| f.email.to_owned()).collect::<Option<Vec<_>>>().unwrap_or_default();
    info!("Players: {:?}", players);
    let elegible_players = all_players.iter().filter(|x| players.contains(&x.email) || extra_players.contains(&x.id)).collect::<Vec<_>>();

    info!("Elegible players({}): {:?}", elegible_players.len(), elegible_players);

    Ok(
        HttpResponse::Ok()
            .json(elegible_players.iter().map(|x| x.apelido.to_owned()).collect::<Vec<String>>())
    )
}

#[post("/voting/create")]
pub async fn voting_create(
    state: Data<AppState>, 
    identity: Option<Identity>
) -> Result<impl Responder> {

    let identity = identity.ok_or(anyhow!("Not logged in"))?;

    let db = &state.db;

    let now = chrono::Utc::now();
    let ref_point = get_ref_point_of(now);
    let event_id = ref_point_id(now);

    // Check if a open ballot exists for the current week
    let alredy_open_ballot = Ballot::find()
        .filter(ballot::Column::FuteId.eq(event_id))
        .filter(ballot::Column::State.eq("open"))
        .filter(ballot::Column::Voter.eq(identity.id().unwrap()))
        .one(db)
        .await?
    ;

    if let Some(b) = alredy_open_ballot {
        // Send the user to the voting page
        return Ok(HttpResponse::Ok()
            .append_header(("HX-Redirect", format!("/voting/{}", b.id)))
            .body("Voting created")
        )
    }

    // Check if we can create a ballot

    let can_create_ballot = can_create_ballot(now);
    if !can_create_ballot {
        return Ok(HttpResponse::BadRequest().body("Cannot create a ballot at this time"));
    }
    let start_elegible_check = get_start_elegible_check(now);
    let end_elegible_check = get_end_elegible_check(now);

    let all_players : Vec<jogador::Model> = Jogador::find().all(db).await?;


    // Filter for players that have been active in the last 30 days
    let players: Vec<_> = query!(
        "select distinct email_address as email FROM ticket WHERE event_id IN 
        (SELECT id FROM event WHERE start_ts between $1 and $2)
         and email_address is not null;",
        start_elegible_check,
        end_elegible_check
    ).fetch_all(&state.alfio_db).await?;

    let extra_players = ListaExtra::find()
        .filter(lista_extra::Column::Data.gt(start_elegible_check))
        .filter(lista_extra::Column::Data.lt(end_elegible_check))
        .all(db)
        .await?;

    let extra_players = extra_players.iter().map(|f| f.jogador_id).collect::<Vec<_>>();


    let players = players.iter().map(|f| f.email.to_owned()).collect::<Option<Vec<_>>>().unwrap_or_default();
    info!("Players: {:?}", players);
    let elegible_players = all_players.iter().filter(|x| players.contains(&x.email) || extra_players.contains(&x.id)).collect::<Vec<_>>();

    info!("Elegible players({}): {:?}", elegible_players.len(), elegible_players);

    if elegible_players.len() < 5 {
        return Ok(HttpResponse::BadRequest().body("Not enough players to create a ballot"));    
    }

    // Get 5 random players
    let players = elegible_players.choose_multiple(&mut rand::thread_rng(), 5)
        .cloned()
        .collect::<Vec<&jogador::Model>>();

    let players_json = serde_json::json!(players.iter().map(|x| x.id).collect::<Vec<i32>>());


    // TODO: Check if the event exists and has voting enabled
    let ballot = ballot::ActiveModel {
        players: ActiveValue::Set(players_json),
        vote: ActiveValue::Set(serde_json::json!([])),
        date: ActiveValue::Set(chrono::Utc::now()),
        voter: ActiveValue::Set(identity.id().unwrap().to_string()),
        fute_id: ActiveValue::Set(event_id),
        state: ActiveValue::Set("open".to_string()), // TODO: Change to enum.
        ..Default::default()
    };

    let ballot = ballot.save(db).await?;
    Ok(HttpResponse::Ok()
        .append_header(("HX-Redirect", format!("/voting/{}", ballot.id.unwrap())))
        .body("Voting created")
    )
}

#[get("/voting/{ballot_id}")]
pub async fn vote(state: Data<AppState>, path: web::Path<u32>,
    identity: Option<Identity>
) -> Result<impl Responder> {
    let identity = identity.ok_or(anyhow!("Not logged in"))?;
    let ballot_id = path.into_inner();
    // Get players and add them to the context
    let mut players: Vec<list::Jogador> = vec![];

    let db = &state.db;
    let ballot: Option<ballot::Model> = Ballot::find_by_id(ballot_id as i32).one(db).await?;
    if let Some(ballot) = ballot {
        if ballot.voter != identity.id().unwrap() {
            return Ok(HttpResponse::Unauthorized().body("You are not allowed to vote on this ballot"));
        }
        let mut context = tera::Context::new();
        context.insert("ballot_id", &ballot_id);
        let players_ids: Vec<i32> = serde_json::from_value(ballot.players)?;
        for player_id in players_ids {
            let player = Jogador::find_by_id(player_id).one(db).await?.ok_or(anyhow!("Player not found"))?;

            players.push(list::Jogador {
                id: player.id,
                nome: player.nome,
                apelido: player.apelido,
            });
        }
        context.insert("players", &players);
        let semana = ref_point_from_id(ballot.fute_id);
        context.insert("semana", &semana.format("%d/%m/%Y").to_string());
        context.insert("week_id", &ballot.fute_id);

        let page_content = TEMPLATES.render("voting.html", &context)?;
        Ok(HttpResponse::Ok().body(page_content))
    } else {
        Ok(HttpResponse::NotFound().body("Ballot not found"))
    }


}

#[derive(Deserialize, Debug)]
struct CastVote {
    players: Vec<String>,
}

#[post("/voting/{ballot_id}/")]
pub async fn vote_submit(
    state: Data<AppState>,
    path: web::Path<u32>,
    cast_vote: web::Json<CastVote>,
    identity: Option<Identity>,
) -> Result<impl Responder> {
    let ballot_id = path.into_inner();
    let identity = identity.ok_or(anyhow!("Not logged in"))?;


    let db = &state.db;
    let ballot: ballot::Model = Ballot::find_by_id(ballot_id as i32).one(db).await?.ok_or(anyhow!("Ballot not found"))?;
    if ballot.voter != identity.id().unwrap() {
        return Ok(HttpResponse::Unauthorized().body("You are not allowed to vote on this ballot"));
    }
    if ballot.state != "open" {
        return Ok(HttpResponse::BadRequest().body("Ballot is not open"));
    }
    let now = chrono::Utc::now();
    let ballot_rt = ref_point_from_id(ballot.fute_id as i32);
    if !can_cast_vote(ballot_rt) {
        return Ok(HttpResponse::BadRequest().body("Voting is closed"));
    }

    info!("Ballot: {:?}", ballot);
    info!("Cast vote: {:?}", cast_vote);
    let cast_vote = cast_vote.players.iter()

        .map(|x| x.parse::<i32>().map_err( |_|
            Error::UnexpectedError(
                anyhow::Error::msg("Failed to parse vote")
            )
        ))
        .collect::<Result<Vec<i32>>>()?;
    info!("Cast vote: {:?}", cast_vote);
    let v = serde_json::to_value(cast_vote.clone())?;
    ballot::ActiveModel {
        vote: ActiveValue::Set(v),
        state: ActiveValue::Set("closed".to_string()),
        id: ActiveValue::Set(ballot_id as i32),
        ..Default::default()
    }.update(db).await?;
    info!("Ballot updated");
    let mut context = tera::Context::new();
    context.insert("ballot_id", &ballot_id);
    context.insert("votes", &cast_vote.iter().map(|x| x.to_string()).collect::<Vec<String>>());
    println!("Votes: {:?}", cast_vote);
    Ok(HttpResponse::Ok().body("Voto computado"))
}

#[get("/voting/{ballot_id}/success")]
pub async fn vote_success(state: Data<AppState>, path: web::Path<u32>,
    identity: Option<Identity>
) -> Result<impl Responder> {
    let identity = identity.ok_or(anyhow!("Not logged in"))?;
    let ballot_id = path.into_inner();
    let db = &state.db;
    let ballot: Option<ballot::Model> = Ballot::find_by_id(ballot_id as i32).one(db).await?;
    if let Some(ballot) = ballot {
        if ballot.voter != identity.id().unwrap() {
            return Ok(HttpResponse::Unauthorized().body("You are not allowed to vote on this ballot"));
        }
        let mut context = tera::Context::new();
        context.insert("ballot_id", &ballot_id);
        let votes: Vec<i32> = serde_json::from_value(ballot.vote)?;
        let mut players: Vec<list::Jogador> = vec![];
        for player_id in votes {
            let player = Jogador::find_by_id(player_id).one(db).await?.ok_or(anyhow!("Player not found"))?;
            players.push(list::Jogador {
                id: player.id,
                nome: player.nome,
                apelido: player.apelido,
            });
        }
        context.insert("players", &players);
        let page_content = TEMPLATES.render("voting_success.html", &context)?;
        Ok(HttpResponse::Ok().body(page_content))
    } else {
        Ok(HttpResponse::NotFound().body("Ballot not found"))
    }
}
