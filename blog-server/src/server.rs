use axum::Router;
use tokio::net::TcpListener;
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::infrastructure::settings::Settings;
use crate::presentation::middleware::cors::apply_cors;
use crate::presentation::middleware::trace::apply_trace;
use crate::presentation::openapi::ApiDoc;
use crate::presentation::{AppState, http_handlers};

pub(crate) async fn run_http(settings: &Settings, state: AppState) -> anyhow::Result<()> {
    let app = build_router(state);
    let app = apply_trace(app);
    let app = apply_cors(app, settings)?;

    let listener = TcpListener::bind(&settings.http_addr).await?;

    info!("HTTP server listening on {}", settings.http_addr);
    axum::serve(listener, app).await?;
    Ok(())
}

pub(crate) fn build_router(state: AppState) -> Router {
    http_handlers::routes(state)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
