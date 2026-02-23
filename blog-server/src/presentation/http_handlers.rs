use axum::{Json, Router, routing::get};
use serde::Serialize;

use super::{AppState, routes};

pub(crate) fn routes(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health_handler))
        .merge(routes::router(state.clone()))
        .with_state(state)
}

#[derive(Debug, Serialize)]
struct HealthzResponse {
    status: &'static str,
}

async fn health_handler() -> Json<HealthzResponse> {
    Json(HealthzResponse { status: "ok" })
}
