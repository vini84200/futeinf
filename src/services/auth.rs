use crate::entities::{prelude::*, *};
use crate::error::Result;
use crate::templates::TEMPLATES;
use crate::AppState;
use actix_identity::Identity;
use actix_web::web::Data;
use actix_web::{get, post, web, HttpMessage, HttpResponse, Responder};
use argon2;
use argon2::PasswordVerifier;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use secrecy::{ExposeSecret, SecretBox, SecretString};
use crate::error::create_bad_request;

#[derive(Deserialize, Debug)]
struct LoginData {
    email: String,
    password: SecretString
}

#[tracing::instrument(
    name = "Render Login Form",
)]
#[get("/login")]
pub async fn login_form() -> impl Responder {
    let context = tera::Context::new();
    let page_content = TEMPLATES.render("login.html", &context).unwrap();
    HttpResponse::Ok().body(page_content)
}

#[tracing::instrument(
    name = "Login User",
    skip(app_state),
    fields(email = %login_data.email)
)]
#[post("/login")]
pub async fn login(
    app_state: Data<AppState>,
    login_data: web::Form<LoginData>,
    request: actix_web::HttpRequest,
) -> Result<impl Responder> {
    let db = &app_state.db;
    let user: Option<jogador::Model> = Jogador::find()
        .filter(jogador::Column::Email.eq(&login_data.email))
        .one(db)
        .await?;
    if let Some(user) = user {
        tracing::info!("Found user {}", user.email);
        let parsed_hash = argon2::PasswordHash::new(&user.senha_hash)
            .map_err(|_| anyhow::anyhow!("Failed to parse hash"))?;
        if argon2::Argon2::default()
            .verify_password(login_data.password.expose_secret().as_bytes(), &parsed_hash)
            .is_ok()
        {
            tracing::info!("Logged in user {}", user.email);
            Identity::login(&request.extensions(), user.email.clone()).unwrap();
            Ok(HttpResponse::Ok()
                .append_header(("HX-Redirect", "/"))
                .body("Logged in"))
        } else {
            tracing::info!("Invalid password");
            Ok(
                create_bad_request("Invalid username or password")
            )
        }
    } else {
        tracing::info!("User not found");
        Ok(
            create_bad_request("Invalid username or password")
        )
    }
}

#[tracing::instrument(
    name = "Logout User",
    skip(identity),
)]
#[post("/logout")]
pub async fn logout(identity: Option<Identity>) -> impl Responder {
    if let Some(identity) = identity {
        identity.logout();
    }
    HttpResponse::Ok()
        .append_header(("HX-Refresh", "true"))
        .body("Logged out")
}
