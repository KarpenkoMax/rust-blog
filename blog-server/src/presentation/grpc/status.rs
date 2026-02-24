use tonic::Status;

use crate::domain::error::DomainError;

pub(crate) fn map_domain_error(err: DomainError) -> Status {
    match err {
        DomainError::Validation { .. } => Status::invalid_argument(err.to_string()),
        DomainError::AlreadyExists(_) => Status::already_exists(err.to_string()),
        DomainError::InvalidCredentials => Status::unauthenticated(err.to_string()),
        DomainError::NotFound(_) => Status::not_found(err.to_string()),
        DomainError::Forbidden => Status::permission_denied(err.to_string()),
        DomainError::Unexpected(_) => Status::internal("internal error"),
    }
}
