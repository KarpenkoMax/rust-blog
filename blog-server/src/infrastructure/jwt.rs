use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum JwtError {
    #[error("token encode failed")]
    Encode(#[source] jsonwebtoken::errors::Error),

    #[error("token decode/validation failed")]
    Decode(#[source] jsonwebtoken::errors::Error),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Claims {
    pub(crate) user_id: i64,
    pub(crate) username: String,
    pub(crate) exp: i64,
}

pub(crate) struct JwtService {
    pub(crate) secret: String,
    pub(crate) ttl_seconds: i64,
}

impl JwtService {
    const DEFAULT_TTL_SECONDS: i64 = 24 * 60 * 60;

    pub(crate) fn new(secret: &str, ttl_seconds: i64) -> Self {
        let ttl_seconds = if ttl_seconds > 0 {
            ttl_seconds
        } else {
            Self::DEFAULT_TTL_SECONDS
        };

        JwtService {
            secret: secret.into(),
            ttl_seconds,
        }
    }

    pub(crate) fn generate_token(&self, user_id: i64, username: &str) -> Result<String, JwtError> {
        let exp = (Utc::now() + Duration::seconds(self.ttl_seconds)).timestamp();

        let claims = Claims {
            user_id,
            username: username.into(),
            exp,
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| JwtError::Encode(e))
    }

    pub(crate) fn verify_token(&self, token: &str) -> Result<Claims, JwtError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.leeway = 10;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(|e| JwtError::Decode(e))?;

        Ok(token_data.claims)
    }
}
