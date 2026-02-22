use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::ValidateEmail;

use super::error::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RegisterRequest {
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) password: String,
}

impl RegisterRequest {
    pub(crate) fn validate(self) -> Result<Self, DomainError> {
        let username = normalize_register_username(&self.username)?;
        let email = normalize_email(&self.email)?;
        let password_len = self.password.chars().count();
        if password_len < 8 || password_len > 128 {
            return Err(DomainError::Validation {
                field: "password",
                message: "must be 8..128 chars",
            });
        }
        Ok(Self {
            username,
            email,
            password: self.password,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LoginRequest {
    pub(crate) username: String,
    pub(crate) password: String,
}

impl LoginRequest {
    pub(crate) fn validate(self) -> Result<Self, DomainError> {
        let username = self.username.trim();
        if username.is_empty() || username.len() > 64 {
            return Err(DomainError::Validation {
                field: "username",
                message: "must be 1..64 chars",
            });
        }

        if self.password.is_empty() {
            return Err(DomainError::Validation {
                field: "password",
                message: "must not be empty",
            });
        }
        Ok(Self {
            username: username.to_string(),
            password: self.password,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct User {
    pub(crate) id: i64,
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) created_at: DateTime<Utc>,
}

impl User {
    pub(crate) fn new(
        id: i64,
        username: impl Into<String>,
        email: impl Into<String>,
        created_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if id <= 0 {
            return Err(DomainError::Validation {
                field: "id",
                message: "must be > 0",
            });
        }
        let username = normalize_register_username(&username.into())?;
        let email = normalize_email(&email.into())?;

        Ok(Self {
            id,
            username,
            email,
            created_at,
        })
    }
}

fn normalize_register_username(username: &str) -> Result<String, DomainError> {
    let username = username.trim();
    if username.len() < 3 || username.len() > 64 {
        return Err(DomainError::Validation {
            field: "username",
            message: "must be 3..64 chars",
        });
    }
    Ok(username.to_string())
}

fn normalize_email(email: &str) -> Result<String, DomainError> {
    let email = email.trim().to_lowercase();
    if !email.validate_email() {
        return Err(DomainError::Validation {
            field: "email",
            message: "must be a valid email",
        });
    }
    Ok(email)
}

#[cfg(test)]
mod tests {
    use super::{RegisterRequest, User, normalize_email, normalize_register_username};
    use chrono::Utc;

    #[test]
    fn user_new_rejects_non_positive_id() {
        let result = User::new(0, "valid_user", "test@example.com", Utc::now());
        assert!(result.is_err());
    }

    #[test]
    fn normalize_email_trims_and_lowercases() {
        let value = normalize_email("  TeSt@Example.COM ").expect("must be valid");
        assert_eq!(value, "test@example.com");
    }

    #[test]
    fn register_username_rules_are_applied() {
        assert!(normalize_register_username("ab").is_err());
        assert!(normalize_register_username("valid_user").is_ok());
    }

    #[test]
    fn register_password_length_is_checked() {
        let short = RegisterRequest {
            username: "valid_user".to_string(),
            email: "test@example.com".to_string(),
            password: "short".to_string(),
        };
        assert!(short.validate().is_err());

        let ok = RegisterRequest {
            username: "valid_user".to_string(),
            email: "test@example.com".to_string(),
            password: "very-secure-password".to_string(),
        };
        let validated = ok.validate().expect("must be valid");
        assert_eq!(validated.username, "valid_user");
        assert_eq!(validated.email, "test@example.com");
    }
}
