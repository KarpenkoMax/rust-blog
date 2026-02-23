use crate::domain::error::DomainError;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub(crate) enum AppError {
    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error("validation error: {0}")]
    Validation(#[from] ValidationErrors),

    #[error("not found")]
    NotFound,

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}

pub(crate) type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AppError::Domain(err) => {
                let (status, msg) = match &err {
                    DomainError::Validation { .. } => (StatusCode::BAD_REQUEST, err.to_string()),
                    DomainError::AlreadyExists(_) => (StatusCode::CONFLICT, err.to_string()),
                    DomainError::InvalidCredentials => (StatusCode::UNAUTHORIZED, err.to_string()),
                    DomainError::NotFound(_) => (StatusCode::NOT_FOUND, err.to_string()),
                    DomainError::Forbidden => (StatusCode::FORBIDDEN, err.to_string()),
                    DomainError::Unexpected(_) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal error".to_string(),
                    ),
                };
                (status, msg)
            }
            AppError::Validation(err) => (StatusCode::BAD_REQUEST, err.to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".to_string()),
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error".to_string(),
            ),
        };

        (status, Json(ErrorBody { error: msg })).into_response()
    }
}
