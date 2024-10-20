use actix_web::{
    middleware::Logger, web::Data, App, HttpResponse, HttpServer, ResponseError,
};
use dotenv::dotenv;
use env_logger::Env;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

mod db;
mod list;
mod services;
mod templates;
mod entities;
mod error;


pub struct AppState {
    alfio_db: Pool<Postgres>,
    db: DatabaseConnection,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // Database

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Error building a connection pool");
    let db = Database::connect("sqlite://db.sqlite?mode=rwc").await.expect("Error connecting to the database");
    Migrator::up(&db, None).await.expect("Error running migrations");
    HttpServer::new(move || {
        App::new()
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
            .service(services::ranking::debug_ranking)
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
