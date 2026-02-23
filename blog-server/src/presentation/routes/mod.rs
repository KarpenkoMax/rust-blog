use axum::Router;

use super::AppState;

pub(crate) mod auth;
pub(crate) mod posts;

pub(crate) fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/api/auth", auth::router())
        .nest("/api/posts", posts::router(state))
}
