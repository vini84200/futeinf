use crate::entities::{prelude::*, *};
use crate::error::Result;
use crate::templates::TEMPLATES;
use crate::AppState;
use actix_identity::Identity;
use actix_web::http::header::CacheControl;
use actix_web::web::{Data, Path};
use actix_web::{get, post, web, HttpMessage, HttpResponse, Responder};
use argon2;
use argon2::PasswordVerifier;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};
use secrecy::{ExposeSecret, SecretBox, SecretString};
use serde::Deserialize;

use crate::error::create_bad_request;
use futures_util::StreamExt as _;

#[derive(Deserialize, Debug)]
struct LoginData {
    email: String,
    password: SecretString,
}

#[tracing::instrument(name = "Render Login Form")]
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
            Ok(create_bad_request("Invalid username or password"))
        }
    } else {
        tracing::info!("User not found");
        Ok(create_bad_request("Invalid username or password"))
    }
}

#[tracing::instrument(name = "Logout User", skip(identity))]
#[post("/logout")]
pub async fn logout(identity: Option<Identity>) -> impl Responder {
    if let Some(identity) = identity {
        identity.logout();
    }
    HttpResponse::Ok()
        .append_header(("HX-Refresh", "true"))
        .body("Logged out")
}

#[tracing::instrument(name = "Upload Image", skip(app_state, payload, identity))]
#[post("/upload_image")]
pub async fn upload_image(
    app_state: Data<AppState>,
    mut payload: actix_multipart::Multipart,
    identity: Option<Identity>,
) -> Result<HttpResponse> {
    let db = &app_state.db;
    let mut imagem = None;
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| anyhow::anyhow!("Erro lendo arquivo"))?;
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();
        let mut data = Vec::new();
        while let Some(chunk) = field.next().await {
            let chunk = chunk.map_err(|e| anyhow::anyhow!("Erro lendo arquivo"))?;
            data.extend_from_slice(&chunk);
        }
        imagem = Some(data);
    }
    if let Some(imagem) = imagem {
        let user = Jogador::find()
            .filter(jogador::Column::Email.eq(identity.unwrap().id().unwrap()))
            .one(db)
            .await?;
        let user = user.ok_or(anyhow::anyhow!("User not found"))?;
        let mut user = user.into_active_model();
        user.imagem = ActiveValue::set(Some(imagem));
        user.save(db).await?;

        Ok(HttpResponse::Ok().body("Image uploaded"))
    } else {
        Ok(create_bad_request("No image uploaded"))
    }
}

#[tracing::instrument(
    name = "Get Image",
    skip(app_state),
    fields(id = %id)
)]
#[get("/image/{id}")]
pub async fn get_image(app_state: Data<AppState>, id: Path<i32>) -> Result<HttpResponse> {
    let db = &app_state.db;
    let user = Jogador::find()
        .filter(jogador::Column::Id.eq(id.into_inner()))
        .one(db)
        .await?;
    if let Some(user) = user {
        let imagem = user.imagem;
        if let Some(imagem) = imagem {
            return Ok(HttpResponse::Ok()
                .insert_header(CacheControl(vec![
                    actix_web::http::header::CacheDirective::Public,
                    actix_web::http::header::CacheDirective::MaxAge(360),
                ]))
                .body(imagem));
        }
        else {
            Ok(HttpResponse::NotFound().body("Image not found"))
        }
    
    } else {
        Ok(create_bad_request("User not found"))
    }
}
