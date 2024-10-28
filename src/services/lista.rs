use crate::templates::TEMPLATES;
use crate::{list, AppState};
use actix_identity::Identity;
use actix_web::web::Data;
use actix_web::{get, HttpResponse, Responder};

#[get("/")]
pub async fn index(
    identity: Option<Identity>,
) -> crate::error::Result<impl Responder> {
    let mut context = tera::Context::new();
    if let Some(identity) = identity {
        context.insert("logged_in", &true);
        context.insert("username", &identity.id().unwrap());
    } else {
        context.insert("logged_in", &false);
        context.insert("username", &"");
    }
    let page_content = TEMPLATES.render("index.html", &context)?;
    Ok(HttpResponse::Ok().body(page_content))
}

#[get("/jogadores")]
pub async fn jogadores(state: Data<AppState>) -> crate::error::Result<impl Responder> {
    let mut context = tera::Context::new();
    context.insert("jogadores", &list::get_list(state.alfio_db.clone()).await?);
    context.insert("max_jogadores", &list::get_max_jogadores());
    let page_content = TEMPLATES.render("shards/jogadores.html", &context)?;
    Ok(HttpResponse::Ok().body(page_content))
}
