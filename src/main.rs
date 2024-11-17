use std::{env, time::Duration};

use actix_web::{
    cookie::{Key}, middleware::Logger, web::Data, App, HttpResponse, HttpServer, ResponseError
};
use base64::{engine::general_purpose::STANDARD, Engine};
use dotenv::dotenv;
use env_logger::Env;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use actix_identity::IdentityMiddleware;
use actix_session::{
    storage::CookieSessionStore,
    SessionMiddleware,
};

mod db;
mod list;
mod services;
mod templates;
mod entities;
mod error;
mod timings;
mod ranking;


pub struct AppState {
    alfio_db: Pool<Postgres>,
    db: DatabaseConnection,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init_from_env(Env::default().default_filter_or("info"));
    let cookie_key_master_str = std::env::var("COOKIE_KEY_MASTER").expect("COOKIE_KEY_MASTER must be set");
    let cookie_key_master = STANDARD.decode( cookie_key_master_str.as_bytes() )
        .expect("Error decoding COOKIE_KEY_MASTER");
    if cookie_key_master.len() != 32 {
        panic!("COOKIE_KEY_MASTER must be 32 bytes long");
    }   
    let cookie_key = Key::derive_from(&cookie_key_master);



    // Database

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Error building a connection pool");
    let local_db_url = std::env::var("LOCAL_DATABASE_URL").expect("LOCAL_DATABASE_URL must be set");
    let db = Database::connect(local_db_url).await.expect("Error connecting to the database");
    Migrator::up(&db, None).await.expect("Error running migrations");
    HttpServer::new(move || {
        let identity = IdentityMiddleware::builder()
            .visit_deadline(Some(Duration::from_secs(60 * 60 * 24 * 7)))
            .login_deadline(Some(Duration::from_secs(60 * 60 * 24 * 31)))
            .build();

        App::new()
            // Install the identity framework first.
            .wrap(identity)
            // The identity system is built on top of sessions. You must install the session
            // middleware to leverage `actix-identity`. The session middleware must be mounted
            // AFTER the identity middleware: `actix-web` invokes middleware in the OPPOSITE
            // order of registration when it receives an incoming request.
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                cookie_key.clone(),
            ))

            .app_data(Data::new(AppState {
                alfio_db: pool.clone(),
                db: db.clone(),
            }))
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
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
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
