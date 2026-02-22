use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{
        Error as PasswordHashError, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        rand_core::OsRng,
    },
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
    const DUMMY_PASSWORD_HASH: &'static str = "$argon2id$v=19$m=19456,t=2,p=1$MDEyMzQ1Njc4OWFiY2RlZg$gwN6hT1sNdk9kI95f7n2Gl3fL0qRmBf2Ffkj2r90/0M";

    pub(crate) fn new(repo: R, jwt: JwtService) -> Self {
        Self { repo, jwt }
    }

    pub(crate) async fn register(&self, req: RegisterRequest) -> Result<AuthResult, DomainError> {
        let req = req.validate()?;

        let password_hash = self.hash_password(&req.password)?;

        let new_user = Self::into_new_user(req, password_hash);
        let user = self.repo.create_user(new_user).await?;

        let access_token = self
            .jwt
            .generate_token(user.id, &user.username)
            .map_err(|err| DomainError::Unexpected(err.to_string()))?;

        Ok(AuthResult { user, access_token })
    }

    pub(crate) async fn login(&self, req: LoginRequest) -> Result<AuthResult, DomainError> {
        let req = req.validate()?;
        let username = req.username.to_string();

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

        let access_token = self
            .jwt
            .generate_token(user_creds.user.id, &user_creds.user.username)
            .map_err(|err| DomainError::Unexpected(err.to_string()))?;

        Ok(AuthResult {
            user: user_creds.user,
            access_token,
        })
    }

    pub(crate) fn hash_password(&self, raw_password: &str) -> Result<String, DomainError> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Self::argon2()?
            .hash_password(raw_password.as_bytes(), &salt)
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

    pub(crate) fn into_new_user(req: RegisterRequest, password_hash: String) -> NewUser {
        NewUser {
            username: req.username,
            email: req.email,
            password_hash,
        }
    }

    fn argon2() -> Result<Argon2<'static>, DomainError> {
        let params = Params::new(19 * 1024, 2, 1, None)
            .map_err(|err| DomainError::Unexpected(err.to_string()))?;
        Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use chrono::Utc;

    use super::AuthService;
    use crate::data::user_repository::{NewUser, UserCredentials, UserRepository};
    use crate::domain::error::DomainError;
    use crate::domain::user::{LoginRequest, RegisterRequest, User};
    use crate::infrastructure::jwt::JwtService;

    #[derive(Clone)]
    struct FakeUserRepo {
        created_input: Arc<Mutex<Option<NewUser>>>,
        login_credentials: Arc<Mutex<Option<UserCredentials>>>,
        create_user_out: User,
    }

    impl FakeUserRepo {
        fn new(create_user_out: User) -> Self {
            Self {
                created_input: Arc::new(Mutex::new(None)),
                login_credentials: Arc::new(Mutex::new(None)),
                create_user_out,
            }
        }

        fn set_login_credentials(&self, creds: Option<UserCredentials>) {
            *self
                .login_credentials
                .lock()
                .expect("login credentials mutex poisoned") = creds;
        }

        fn take_created_input(&self) -> Option<NewUser> {
            self.created_input
                .lock()
                .expect("created input mutex poisoned")
                .take()
        }
    }

    #[async_trait]
    impl UserRepository for FakeUserRepo {
        async fn create_user(&self, input: NewUser) -> Result<User, DomainError> {
            *self
                .created_input
                .lock()
                .expect("created input mutex poisoned") = Some(input);
            Ok(self.create_user_out.clone())
        }

        async fn find_by_username(
            &self,
            _username: &str,
        ) -> Result<Option<UserCredentials>, DomainError> {
            Ok(self
                .login_credentials
                .lock()
                .expect("login credentials mutex poisoned")
                .clone())
        }

        async fn find_by_email(
            &self,
            _email: &str,
        ) -> Result<Option<UserCredentials>, DomainError> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn register_creates_user_and_returns_token() {
        let repo = FakeUserRepo::new(sample_user(1, "valid_user", "valid@example.com"));
        let service = AuthService::new(repo.clone(), test_jwt());

        let req = RegisterRequest {
            username: "  valid_user  ".to_string(),
            email: "  VALID@EXAMPLE.COM  ".to_string(),
            password: "very-secure-password".to_string(),
        };

        let result = service.register(req).await.expect("register must succeed");

        assert_eq!(result.user.username, "valid_user");
        assert!(!result.access_token.is_empty());

        let created = repo
            .take_created_input()
            .expect("create_user must be called");
        assert_eq!(created.username, "valid_user");
        assert_eq!(created.email, "valid@example.com");
        assert!(!created.password_hash.is_empty());
    }

    #[tokio::test]
    async fn login_returns_invalid_credentials_for_missing_user() {
        let repo = FakeUserRepo::new(sample_user(1, "valid_user", "valid@example.com"));
        repo.set_login_credentials(None);
        let service = AuthService::new(repo, test_jwt());

        let req = LoginRequest {
            username: "valid_user".to_string(),
            password: "some-password".to_string(),
        };

        let err = service.login(req).await.expect_err("login must fail");
        assert!(matches!(err, DomainError::InvalidCredentials));
    }

    #[tokio::test]
    async fn login_returns_invalid_credentials_for_wrong_password() {
        let repo = FakeUserRepo::new(sample_user(1, "valid_user", "valid@example.com"));
        let service = AuthService::new(repo.clone(), test_jwt());

        let hash = service
            .hash_password("correct-password")
            .expect("hash must be created");
        repo.set_login_credentials(Some(UserCredentials {
            user: sample_user(1, "valid_user", "valid@example.com"),
            password_hash: hash,
        }));

        let req = LoginRequest {
            username: "valid_user".to_string(),
            password: "wrong-password".to_string(),
        };

        let err = service.login(req).await.expect_err("login must fail");
        assert!(matches!(err, DomainError::InvalidCredentials));
    }

    #[tokio::test]
    async fn login_returns_token_for_valid_credentials() {
        let repo = FakeUserRepo::new(sample_user(1, "valid_user", "valid@example.com"));
        let service = AuthService::new(repo.clone(), test_jwt());

        let hash = service
            .hash_password("correct-password")
            .expect("hash must be created");
        repo.set_login_credentials(Some(UserCredentials {
            user: sample_user(1, "valid_user", "valid@example.com"),
            password_hash: hash,
        }));

        let req = LoginRequest {
            username: "valid_user".to_string(),
            password: "correct-password".to_string(),
        };

        let result = service.login(req).await.expect("login must succeed");
        assert_eq!(result.user.id, 1);
        assert!(!result.access_token.is_empty());
    }

    fn sample_user(id: i64, username: &str, email: &str) -> User {
        User::new(id, username.to_string(), email.to_string(), Utc::now())
            .expect("sample user must be valid")
    }

    fn test_jwt() -> JwtService {
        JwtService::new("0123456789abcdef0123456789abcdef", 3600)
    }
}
