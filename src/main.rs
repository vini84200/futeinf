use std::time::Duration;

use actix_web::{cookie::Key, web::Data, App, HttpServer};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::serde::ts_microseconds_option::serialize;
use dotenv::dotenv;
use env_logger::Env;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use secrecy::{ExposeSecret, SecretBox, SecretString};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use tracing::{subscriber, Instrument};
use tracing_actix_web::TracingLogger;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing::subscriber::set_global_default;

mod db;
mod entities;
mod error;
mod list;
mod ranking;
mod services;
mod templates;
mod timings;

pub struct AppState {
    alfio_db: Pool<Postgres>,
    db: DatabaseConnection,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("futeinf".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    set_global_default(subscriber).expect("Error setting global default");

    let cookie_key_master_str = SecretBox::from(
        std::env::var("COOKIE_KEY_MASTER").expect("COOKIE_KEY_MASTER must be set")
    );
    let cookie_key_master = SecretBox::from(STANDARD
        .decode(cookie_key_master_str.expose_secret().as_bytes())
        .expect("Error decoding COOKIE_KEY_MASTER"));
    if cookie_key_master.expose_secret().len() != 32 {
        panic!("COOKIE_KEY_MASTER must be 32 bytes long");
    }
    let cookie_key = 
        Key::derive_from(&cookie_key_master.expose_secret());

    // Database

    let database_url = SecretBox::from(std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"));
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url.expose_secret())
        .await
        .expect("Error building a connection pool");
    let local_db_url = SecretBox::from(std::env::var("LOCAL_DATABASE_URL").expect("LOCAL_DATABASE_URL must be set"));
    let db = Database::connect(local_db_url.expose_secret())
        .instrument(tracing::info_span!("local db connection"))
        .await
        .expect("Error connecting to the database");

    
    Migrator::up(&db, None)
        .instrument(tracing::info_span!("migrations"))
        .await
        .expect("Error running migrations");

    HttpServer::new(move || {
        let identity = IdentityMiddleware::builder()
            .visit_deadline(Some(Duration::from_secs(60 * 60 * 24 * 7)))
            .login_deadline(Some(Duration::from_secs(60 * 60 * 24 * 31)))
            .build();

        let cookie_middle = SessionMiddleware::builder(
            CookieSessionStore::default(),
            cookie_key.clone(),
        );

        App::new()
            // Install the identity framework first.
            .wrap(identity)
            // The identity system is built on top of sessions. You must install the session
            // middleware to leverage `actix-identity`. The session middleware must be mounted
            // AFTER the identity middleware: `actix-web` invokes middleware in the OPPOSITE
            // order of registration when it receives an incoming request.
            .wrap( cookie_middle.build())
            .app_data(Data::new(AppState {
                alfio_db: pool.clone(),
                db: db.clone(),
            }))
            .wrap(TracingLogger::default())
            .service(services::lista::index)
            .service(services::lista::jogadores)
            .service(services::voting::vote)
            .service(services::voting::vote_submit)
            .service(services::voting::voting_create)
            .service(services::voting::vote_success)
            .service(services::voting::get_elegible_players)
            .service(services::ranking::debug_ranking)
            .service(services::ranking::week_ranking)
            .service(services::auth::login_form)
            .service(services::auth::login)
            .service(services::auth::logout)
            .service(services::auth::upload_image)
            .service(services::auth::get_image)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
