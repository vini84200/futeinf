use crate::list;
use crate::{templates::TEMPLATES, AppState, Result};
use actix_web::{get, web::Data, HttpResponse, Responder};

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
