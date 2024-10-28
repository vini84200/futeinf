use crate::templates::TEMPLATES;
use crate::{entities, list, AppState, error::Result};
use actix_identity::Identity;
use actix_web::web::Data;
use actix_web::{get, post, web, HttpResponse, Responder};
use anyhow::anyhow;
use log::info;
use rand::prelude::SliceRandom;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};
use serde::{Deserialize, Deserializer};
use sqlx::types::chrono;
use entities::{prelude::*, *};
use crate::error::Error;

#[get("/voting")]
pub async fn voting(state: Data<AppState>) -> crate::error::Result<impl Responder> {
    let context = tera::Context::new();
    let page_content = TEMPLATES.render("voting.html", &context)?;
    Ok(HttpResponse::Ok().body(page_content))
}

#[post("/voting/create/{event_id}")]
pub async fn voting_create(
    state: Data<AppState>, path: web::Path<u32>,
    identity: Option<Identity>
) -> crate::error::Result<impl Responder> {

    let identity = identity.ok_or(anyhow!("Not logged in"))?;

    let event_id = path.into_inner();

    let db = &state.db;

    let all_players : Vec<jogador::Model> = Jogador::find().all(db).await?;

    // Get 5 random players
    let players = all_players.choose_multiple(&mut rand::thread_rng(), 5)
        .collect::<Vec<&jogador::Model>>();

    let players_json = serde_json::json!(players.iter().map(|x| x.id).collect::<Vec<i32>>());

    // TODO: Check if the event exists and has voting enabled
    let ballot = ballot::ActiveModel {
        players: ActiveValue::Set(players_json),
        vote: ActiveValue::Set(serde_json::json!([])),
        date: ActiveValue::Set(chrono::Utc::now()),
        voter: ActiveValue::Set(identity.id().unwrap().to_string()),
        fute_id: ActiveValue::Set(event_id as i32),
        state: ActiveValue::Set("open".to_string()), // TODO: Change to enum.
        ..Default::default()
    };

    let ballot = ballot.save(db).await?;
    Ok(HttpResponse::Ok()
        .append_header(("HX-Redirect", format!("/voting/{}", ballot.id.unwrap())))
        .await
    )
}

#[get("/voting/{ballot_id}")]
pub async fn vote(state: Data<AppState>, path: web::Path<u32>,
    identity: Option<Identity>
) -> crate::error::Result<impl Responder> {
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
        let page_content = TEMPLATES.render("voting.html", &context)?;
        Ok(HttpResponse::Ok().body(page_content))
    } else {
        return Ok(HttpResponse::NotFound().body("Ballot not found"));
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
) -> crate::error::Result<impl Responder> {
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
        return Ok(HttpResponse::NotFound().body("Ballot not found"));
    }
}
