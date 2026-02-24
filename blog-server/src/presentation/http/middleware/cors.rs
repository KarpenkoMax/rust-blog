use anyhow::{Result, anyhow};
use axum::Router;
use axum::http::{Method, header};
use tower_http::cors::{Any, CorsLayer};

use crate::infrastructure::settings::Settings;

pub(crate) fn build_cors_layer(settings: &Settings) -> Result<CorsLayer> {
    let layer = if settings.cors_origins.iter().any(|origin| origin == "*") {
        CorsLayer::new().allow_origin(Any)
    } else {
        let origins = settings
            .cors_origins
            .iter()
            .map(|origin| origin.parse())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| anyhow!("invalid CORS origin: {err}"))?;

        CorsLayer::new().allow_origin(origins)
    };

    Ok(layer
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT]))
}

pub(crate) fn apply_cors(router: Router, settings: &Settings) -> Result<Router> {
    let cors = build_cors_layer(settings)?;
    Ok(router.layer(cors))
}
