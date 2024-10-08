use actix_web::{
    cookie::time::error, get, middleware::Logger, web::{self, Data}, App, HttpResponse, HttpServer, Responder, ResponseError
};
use dotenv::dotenv;
use env_logger::Env;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

mod db;
mod list;
mod services;
mod templates;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to render template:\n{0}")]
    TemplateError(#[from] tera::Error),
    #[error("an unexpected error occurred: {0}")]
    UnexpectedError(#[from] anyhow::Error),
    #[error("database error: {0}")]
    DatabaseError(#[from] sqlx::Error)
}

impl ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

pub struct AppState {
    db: Pool<Postgres>,
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
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState { db: pool.clone() }))
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(services::index)
            .service(services::jogadores)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
