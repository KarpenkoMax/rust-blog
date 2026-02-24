use axum::{
    extract::{FromRequestParts, Request, State},
    http::{header, request::Parts},
    middleware::Next,
    response::Response,
};

use crate::presentation::AppState;
use crate::presentation::http::app_error::AppError;

#[derive(Debug, Clone)]
pub(crate) struct AuthenticatedUser {
    pub(crate) user_id: i64,
    // pub(crate) username: String,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or(AppError::Unauthorized)
    }
}

pub(crate) async fn jwt_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let mut parts = auth_header.split_whitespace();
    let scheme = parts.next().ok_or(AppError::Unauthorized)?;
    let token = parts.next().ok_or(AppError::Unauthorized)?;
    if parts.next().is_some() {
        return Err(AppError::Unauthorized);
    }
    if !scheme.eq_ignore_ascii_case("bearer") {
        return Err(AppError::Unauthorized);
    }
    if token.trim().is_empty() {
        return Err(AppError::Unauthorized);
    }

    let claims = state
        .jwt
        .verify_token(token.trim())
        .map_err(|_| AppError::Unauthorized)?;

    request.extensions_mut().insert(AuthenticatedUser {
        user_id: claims.user_id,
        // username: claims.username,
    });

    Ok(next.run(request).await)
}
