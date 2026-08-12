use std::io;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("config error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("server error: {0}")]
    Server(String),
    #[error("network error: {0}")]
    Network(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match &self {
            Error::Config(_) => StatusCode::BAD_REQUEST,
            Error::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Server(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Network(_) => StatusCode::BAD_GATEWAY,
        };
        let body = Json(json!({
            "error": self.to_string(),
        }));
        (status, body).into_response()
    }
}
