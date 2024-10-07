use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder, ResponseError};
use env_logger::Env;
use lazy_static::lazy_static;
use tera::Tera;

mod list;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to render template:\n{0}")]
    TemplateError(#[from] tera::Error),
    #[error("an unexpected error occurred: {0}")]
    UnexpectedError(#[from] anyhow::Error),
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

lazy_static! {
    static ref TEMPLATES: Tera = {
        let source = "templates/**/*.html";
        let mut tera = Tera::new(source).expect("failed to compile template");
        tera.autoescape_on(vec![".html", ".sql"]);
        tera
    };
}

#[get("/")]
async fn index() -> Result<impl Responder> {
    let context = tera::Context::new();
    let page_content = TEMPLATES.render("index.html", &context)?;
    Ok(HttpResponse::Ok().body(page_content))
}

#[get("/jogadores")]
async fn jogadores() -> Result<impl Responder> {
    let mut context = tera::Context::new();
    context.insert("jogadores", &list::get_list());
    context.insert("max_jogadores", &list::get_max_jogadores());
    let page_content = TEMPLATES.render("shards/jogadores.html", &context)?;
    Ok(HttpResponse::Ok().body(page_content))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    HttpServer::new(|| {
        App::new()
        .wrap(Logger::default())
        .wrap(Logger::new("%a %{User-Agent}i"))
        .service(index)
        .service(jogadores)
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
