use crate::templates::TEMPLATES;
use crate::timings::ref_point_id;
use crate::{list, AppState};
use actix_identity::Identity;
use actix_web::web::Data;
use actix_web::{get, HttpResponse, Responder};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use crate::entities::{prelude::*, *};
use chrono::Utc;
use anyhow::anyhow;

#[tracing::instrument(name = "Render Index", skip(identity, state))]
#[get("/")]
pub async fn index(identity: Option<Identity>, 
    state: Data<AppState>
) -> crate::error::Result<impl Responder> {
    let mut context = tera::Context::new();
    if let Some(identity) = identity {
        context.insert("logged_in", &true);
        let user: Option<jogador::Model> = Jogador::find()
            .filter(jogador::Column::Email.eq(identity.id().unwrap()))
            .one(&state.db.clone())
            .await?;
        let user = user.ok_or(anyhow!("User not found"))?;
        context.insert("username", &user.nome);
        context.insert("imagem", &user.imagem);


    } else {
        context.insert("logged_in", &false);
        context.insert("username", &"");
    }
    let week_id = ref_point_id(Utc::now());
    let last_week = week_id - 1;
    context.insert("last_week", &last_week);
    let page_content = TEMPLATES.render("index.html", &context)?;
    Ok(HttpResponse::Ok().body(page_content))
}

#[tracing::instrument(name = "Render Jogadores", skip(state))]
#[get("/jogadores")]
pub async fn jogadores(state: Data<AppState>) -> crate::error::Result<impl Responder> {
    let mut context = tera::Context::new();
    context.insert("jogadores", &list::get_list(state.alfio_db.clone()).await?);
    context.insert("max_jogadores", &list::get_max_jogadores());
    let page_content = TEMPLATES.render("shards/jogadores.html", &context)?;
    Ok(HttpResponse::Ok().body(page_content))
}
