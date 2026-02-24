use axum::{Router, routing::post};

use crate::presentation::AppState;
use crate::presentation::http::handlers::auth::{login, register};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}
