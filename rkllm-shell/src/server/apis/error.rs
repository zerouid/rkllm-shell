use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorDetails,
}

#[derive(Serialize)]
struct ErrorDetails {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
    code: Option<String>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match &self {
            ApiError::InvalidRequest(msg) => (
                        StatusCode::BAD_REQUEST,
                        "invalid_request_error",
                        msg.clone(),
                    ),
            ApiError::AuthenticationError(msg) => (
                        StatusCode::UNAUTHORIZED,
                        "authentication_error", 
                        msg.clone(),
                    ),
            ApiError::ModelNotFound(msg) => (
                        StatusCode::NOT_FOUND,
                        "model_not_found",
                        msg.clone(), 
                    ),
            ApiError::RateLimitExceeded(msg) => (
                        StatusCode::TOO_MANY_REQUESTS,
                        "rate_limit_exceeded",
                        msg.clone(),
                    ),
            ApiError::InternalError(msg) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal_server_error",
                        msg.clone(),
                    ),
            ApiError::Internal(msg) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal_error",
                        msg.clone(),
                    ),
        };

        let body = ErrorResponse {
            error: ErrorDetails {
                message,
                error_type: error_type.to_string(),
                code: None,
            },
        };

        (status, Json(body)).into_response()
    }
}

impl From<hf_hub::api::sync::ApiError> for ApiError {
    fn from(err: hf_hub::api::sync::ApiError) -> Self {
        ApiError::InternalError(err.to_string())
    }
}