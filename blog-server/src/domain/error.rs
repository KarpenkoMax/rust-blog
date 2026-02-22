use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum DomainError {
    #[error("validation failed for '{field}': {message}")]
    Validation {
        field: &'static str,
        message: &'static str,
    },

    #[error("resource not found: {0}")]
    NotFound(String),

    #[error("resource already exists: {0}")]
    AlreadyExists(String),

    #[error("forbidden")]
    Forbidden,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("unexpected domain error: {0}")]
    Unexpected(String),
}
