use axum::{Json, extract::State, http::StatusCode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::domain::user::{LoginRequest, RegisterRequest, User};
use crate::presentation::AppState;
use crate::presentation::app_error::AppResult;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub(crate) struct RegisterDto {
    #[validate(length(min = 3, max = 64))]
    pub(crate) username: String,
    #[validate(email)]
    pub(crate) email: String,
    #[validate(length(min = 8, max = 128))]
    pub(crate) password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub(crate) struct LoginDto {
    #[validate(length(min = 1, max = 64))]
    pub(crate) username: String,
    #[validate(length(min = 1))]
    pub(crate) password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct AuthResponseDto {
    pub(crate) access_token: String,
    pub(crate) user: UserDto,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct UserDto {
    pub(crate) id: i64,
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) created_at: DateTime<Utc>,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "auth",
    request_body = RegisterDto,
    responses(
        (status = 201, description = "Registered successfully", body = AuthResponseDto),
        (status = 400, description = "Validation error"),
        (status = 409, description = "User already exists"),
        (status = 500, description = "Internal error")
    )
)]
pub(crate) async fn register(
    State(state): State<AppState>,
    Json(dto): Json<RegisterDto>,
) -> AppResult<(StatusCode, Json<AuthResponseDto>)> {
    dto.validate()?;

    let req = RegisterRequest {
        username: dto.username,
        email: dto.email,
        password: dto.password,
    };

    let result = state.auth_service.register(req).await?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponseDto {
            access_token: result.access_token,
            user: result.user.into(),
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    request_body = LoginDto,
    responses(
        (status = 200, description = "Login successful", body = AuthResponseDto),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal error")
    )
)]
pub(crate) async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginDto>,
) -> AppResult<(StatusCode, Json<AuthResponseDto>)> {
    dto.validate()?;

    let req = LoginRequest {
        username: dto.username,
        password: dto.password,
    };

    let result = state.auth_service.login(req).await?;

    Ok((
        StatusCode::OK,
        Json(AuthResponseDto {
            access_token: result.access_token,
            user: result.user.into(),
        }),
    ))
}
