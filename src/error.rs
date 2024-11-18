use crate::templates::TEMPLATES;
use actix_web::{HttpResponse, ResponseError};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to render template:\n{0} \n{0:?}")]
    TemplateError(#[from] tera::Error),
    #[error("an unexpected error occurred: {0}")]
    UnexpectedError(#[from] anyhow::Error),
    #[error("alfio database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("sea_orm database error: {0}")]
    SeaOrmError(#[from] sea_orm::error::DbErr),
    #[error("A serialization error occurred: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Integer Parse error: {0}")]
    ParseError(#[from] std::num::ParseIntError),
}

impl ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        tracing::error!("{}", self);
        let mut context = tera::Context::new();
        context.insert("error", &self.to_string());
        let page_content = TEMPLATES.render("error.html", &context).unwrap();

        HttpResponse::build(self.status_code()).body(page_content)
    }
}
