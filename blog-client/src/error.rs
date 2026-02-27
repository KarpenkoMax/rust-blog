use thiserror::Error;
use tonic::Code;

#[derive(Debug, Error)]
/// Ошибки клиентской библиотеки `blog-client`.
pub enum BlogClientError {
    /// Ошибка HTTP-транспорта (`reqwest`).
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// Ошибка gRPC-метода (`tonic::Status`).
    #[error("grpc status error: {0}")]
    GrpcStatus(#[from] tonic::Status),

    /// Ошибка подключения/канала gRPC (`tonic::transport::Error`).
    #[error("grpc transport error: {0}")]
    GrpcTransport(#[from] tonic::transport::Error),

    /// Требуется авторизация (отсутствует/некорректен токен).
    #[error("unauthorized")]
    Unauthorized,

    /// Запрошенный ресурс не найден.
    #[error("not found")]
    NotFound,

    /// Некорректный запрос или бизнес-ошибка валидации.
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

/// Результат операций `blog-client`.
pub type BlogClientResult<T> = Result<T, BlogClientError>;

impl BlogClientError {
    pub(crate) fn from_http_status(status: reqwest::StatusCode, message: Option<String>) -> Self {
        match status {
            reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
                Self::Unauthorized
            }
            reqwest::StatusCode::NOT_FOUND => Self::NotFound,
            _ => {
                let message = message.unwrap_or_else(|| format!("http status {status}"));
                Self::InvalidRequest(message)
            }
        }
    }

    pub(crate) fn from_reqwest(err: reqwest::Error) -> Self {
        if let Some(status) = err.status() {
            return Self::from_http_status(status, None);
        }
        Self::Http(err)
    }

    pub(crate) fn from_grpc_status(status: tonic::Status) -> Self {
        match status.code() {
            Code::Unauthenticated | Code::PermissionDenied => Self::Unauthorized,
            Code::NotFound => Self::NotFound,
            Code::InvalidArgument | Code::AlreadyExists | Code::FailedPrecondition => {
                Self::InvalidRequest(status.message().to_string())
            }
            _ => Self::GrpcStatus(status),
        }
    }
}
