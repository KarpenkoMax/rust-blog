use async_trait::async_trait;

use crate::domain::error::DomainError;
use crate::domain::user::User;

#[derive(Debug, Clone)]
pub(crate) struct UserCredentials {
    pub(crate) user: User,
    pub(crate) password_hash: String,
}

#[derive(Debug, Clone)]
pub(crate) struct NewUser {
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) password_hash: String,
}

#[async_trait]
pub(crate) trait UserRepository: Send + Sync {
    async fn create_user(&self, input: NewUser) -> Result<User, DomainError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<UserCredentials>, DomainError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<UserCredentials>, DomainError>;
}
