use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::data::user_repository::{NewUser, UserCredentials, UserRepository};
use crate::domain::error::DomainError;
use crate::domain::user::User;

#[derive(Debug, Clone)]
pub(crate) struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct UserRow {
    id: i64,
    username: String,
    email: String,
    created_at: DateTime<Utc>,
}

struct UserCredentialsRow {
    id: i64,
    username: String,
    email: String,
    password_hash: String,
    created_at: DateTime<Utc>,
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create_user(&self, input: NewUser) -> Result<User, DomainError> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, username, email, created_at
            "#,
            input.username,
            input.email,
            input.password_hash,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_user_db_error)?;

        User::new(row.id, row.username, row.email, row.created_at)
            .map_err(|err| DomainError::Unexpected(err.to_string()))
    }

    async fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Option<UserCredentials>, DomainError> {
        let row = sqlx::query_as!(
            UserCredentialsRow,
            r#"
            SELECT
            id,
            username,
            email,
            password_hash,
            created_at
            FROM users
            WHERE username = $1
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_user_db_error)?;
        if let Some(r) = row {
            let user = User::new(r.id, r.username, r.email, r.created_at)
                .map_err(|err| DomainError::Unexpected(err.to_string()))?;

            Ok(Some(UserCredentials {
                user,
                password_hash: r.password_hash,
            }))
        } else {
            Ok(None)
        }
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<UserCredentials>, DomainError> {
        let row = sqlx::query_as!(
            UserCredentialsRow,
            r#"
            SELECT
            id,
            username,
            email,
            password_hash,
            created_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_user_db_error)?;
        if let Some(r) = row {
            let user = User::new(r.id, r.username, r.email, r.created_at)
                .map_err(|err| DomainError::Unexpected(err.to_string()))?;

            Ok(Some(UserCredentials {
                user,
                password_hash: r.password_hash,
            }))
        } else {
            Ok(None)
        }
    }
}

fn map_user_db_error(err: sqlx::Error) -> DomainError {
    if let sqlx::Error::Database(db_err) = &err
        && db_err.code().as_deref() == Some("23505") {
            let resource = match db_err.constraint() {
                Some("users_username_key") => "username",
                Some("users_email_key") => "email",
                _ => "user",
            };
            return DomainError::AlreadyExists(resource.to_string());
        }
    DomainError::Unexpected(err.to_string())
}
