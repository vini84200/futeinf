use actix_identity::Identity;
use actix_web::{get, post, web, HttpMessage, HttpResponse, Responder};
use actix_web::web::Data;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use crate::AppState;
use crate::error::Result;
use crate::entities::{prelude::*, *};
use argon2;
use argon2::PasswordVerifier;
use serde::Deserialize;
use crate::templates::TEMPLATES;

#[derive(Deserialize)]
struct LoginData {
    email: String,
    password: String,
}

#[get("/login")]
pub async fn login_form() -> impl Responder {
    let context = tera::Context::new();
    let page_content = TEMPLATES.render("login.html", &context).unwrap();
    HttpResponse::Ok().body(page_content)
}

#[post("/login")]
pub async fn login(
    app_state: Data<AppState>,
    login_data: web::Form<LoginData>,
    request: actix_web::HttpRequest,
) -> Result<impl Responder> {
    let db = &app_state.db;
    let user : Option<jogador::Model>= Jogador::find()
        .filter(jogador::Column::Email.eq(&login_data.email))
        .one(db)
        .await?;
    if let Some(user) = user {
        let parsed_hash = argon2::PasswordHash::new(&user.senha_hash)
            .map_err(|_| anyhow::anyhow!("Failed to parse hash"))?;
        if argon2::Argon2::default().verify_password(
            login_data.password.as_bytes(),
            &parsed_hash,
        ).is_ok() {
            Identity::login(
                &request.extensions(),
                user.email.clone(),
            ).unwrap();
            Ok(
                HttpResponse::Ok()
                    .append_header(("HX-Redirect", "/"))
                    .body("Logged in")
            )
        } else {
            Ok(
                HttpResponse::BadRequest()
                    .body("Invalid username or password")
            )
        }
    }
    else {
        Ok(HttpResponse::BadRequest()
            .body("Invalid username or password"))
    }
}

#[post("/logout")]
pub async fn logout(
    identity: Option<Identity>,
) -> impl Responder {
    if let Some(identity) = identity {
        identity.logout();
    }
    HttpResponse::Ok()
        .append_header(("HX-Refresh", "true")) 
    .body("Logged out")
}