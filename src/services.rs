use crate::list;
use crate::list::Jogador;
use crate::{templates::TEMPLATES, AppState, Result};
use actix_web::{post, web, FromRequest, HttpMessage, HttpRequest};
use actix_web::{get, web::Data, HttpResponse, Responder};
use serde::Deserialize;

#[get("/")]
pub async fn index() -> Result<impl Responder> {
    let context = tera::Context::new();
    let page_content = TEMPLATES.render("index.html", &context)?;
    Ok(HttpResponse::Ok().body(page_content))
}

#[get("/jogadores")]
pub async fn jogadores(state: Data<AppState>) -> Result<impl Responder> {
    let mut context = tera::Context::new();
    context.insert("jogadores", &list::get_list(state.db.clone()).await?);
    context.insert("max_jogadores", &list::get_max_jogadores());
    let page_content = TEMPLATES.render("shards/jogadores.html", &context)?;
    Ok(HttpResponse::Ok().body(page_content))
}

#[get("/voting/{ballot_id}")]
pub async fn vote(state: Data<AppState>, path: web::Path<u32>) -> Result<impl Responder> {
    let (ballot_id) = path.into_inner();
    let mut context = tera::Context::new();
    context.insert("ballot_id", &ballot_id);
    // Get players and add them to the context
    let mut players: Vec<list::Jogador> = vec![];
    players.push(Jogador {
        nome: "Ronaldo".to_string(),
        id: 1,
        apelido: "Fenômeno".to_string(),
    });
    players.push(Jogador {
        nome: "Messi".to_string(),
        id: 2,
        apelido: "Messi".to_string(),
    });
    players.push(Jogador {
        nome: "Neymar".to_string(),
        id: 3,
        apelido: "Neymar".to_string(),
    });
    context.insert("players", &players);
    let page_content = TEMPLATES.render("voting.html", &context)?;
    Ok(HttpResponse::Ok().body(page_content))
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
) -> Result<impl Responder> {
    

    let (ballot_id) = path.into_inner();
    let mut context = tera::Context::new();
    context.insert("ballot_id", &ballot_id);
    context.insert("votes", &cast_vote.players.iter().map(|x| x.parse::<i32>().unwrap()).collect::<Vec<i32>>());
    println!("Votes: {:?}", cast_vote.players);
    Ok(HttpResponse::Ok().body({ "Voto computado" }))
}
