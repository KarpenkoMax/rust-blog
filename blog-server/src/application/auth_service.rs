use argon2::{
    password_hash::{Error as PasswordHashError, PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
    Argon2, Algorithm, Version, Params,
};

use crate::data::user_repository::{NewUser, UserRepository};
use crate::domain::error::DomainError;
use crate::domain::user::{LoginRequest, RegisterRequest, User};
use crate::infrastructure::jwt::JwtService;

#[derive(Debug, Clone)]
pub(crate) struct AuthResult {
    pub(crate) user: User,
    pub(crate) access_token: String,
}

pub(crate) struct AuthService<R: UserRepository> {
    repo: R,
    jwt: JwtService,
}

impl<R: UserRepository> AuthService<R> {
    const DUMMY_PASSWORD_HASH: &'static str =
        "$argon2id$v=19$m=19456,t=2,p=1$MDEyMzQ1Njc4OWFiY2RlZg$gwN6hT1sNdk9kI95f7n2Gl3fL0qRmBf2Ffkj2r90/0M";

    pub(crate) fn new(repo: R, jwt: JwtService) -> Self {
        Self { repo, jwt }
    }

    pub(crate) async fn register(&self, req: RegisterRequest) -> Result<AuthResult, DomainError> {

        req.validate()?;

        let password_hash = self.hash_password(&req.password)?;

        let new_user = Self::into_new_user(req, password_hash);
        let user = self.repo.create_user(new_user).await?;

        let access_token = self.jwt.generate_token(user.id, &user.username)
            .map_err(|err| DomainError::Unexpected(err.to_string()))?;

        Ok(AuthResult {
            user,
            access_token,
        })
    }

    pub(crate) async fn login(&self, req: LoginRequest) -> Result<AuthResult, DomainError> {

        req.validate()?;
        let username = req.username.trim().to_string();

        let user_creds = match self.repo.find_by_username(&username).await? {
            Some(user_creds) => user_creds,
            None => {
                // стремимся к одинаковому времени проверки если user не найден
                match self.verify_password(&req.password, Self::DUMMY_PASSWORD_HASH) {
                    Ok(()) | Err(DomainError::InvalidCredentials) => {}
                    Err(err) => return Err(err),
                }
                return Err(DomainError::InvalidCredentials);
            }
        };

        self.verify_password(&req.password, &user_creds.password_hash)?;

        let access_token = self.jwt.generate_token(user_creds.user.id, &user_creds.user.username)
            .map_err(|err| DomainError::Unexpected(err.to_string()))?;

        Ok(AuthResult {
            user: user_creds.user,
            access_token,
        })
    }

    pub(crate) fn hash_password(&self, raw_password: &str) -> Result<String, DomainError> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Self::argon2()?.hash_password(raw_password.as_bytes(), &salt)
            .map_err(|err| DomainError::Unexpected(err.to_string()))?;
        Ok(password_hash.to_string())
    }

    pub(crate) fn verify_password(
        &self,
        raw_password: &str,
        password_hash: &str,
    ) -> Result<(), DomainError> {
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|err| DomainError::Unexpected(err.to_string()))?;
        Self::argon2()?
            .verify_password(raw_password.as_bytes(), &parsed_hash)
            .map_err(|err| match err {
                PasswordHashError::Password => DomainError::InvalidCredentials,
                _ => DomainError::Unexpected(err.to_string()),
            })?;

        Ok(())
    }

    pub(crate) fn into_new_user(
        req: RegisterRequest,
        password_hash: String,
    ) -> NewUser {
        NewUser {
            username: req.username.trim().to_string(),
            email: req.email.trim().to_lowercase(),
            password_hash,
        }
    }

    fn argon2() -> Result<Argon2<'static>, DomainError> {
        let params = Params::new(19 * 1024, 2, 1, None)
            .map_err(|err| DomainError::Unexpected(err.to_string()))?;
        Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
    }
}
