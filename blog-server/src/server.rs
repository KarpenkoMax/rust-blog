use axum::BoxError;
use axum::Router;
use axum::error_handling::HandleErrorLayer;
use axum::http::StatusCode;
use std::time::Duration;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower::limit::ConcurrencyLimitLayer;
use tower::timeout::TimeoutLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::infrastructure::settings::Settings;
use crate::presentation::AppState;
use crate::presentation::grpc::GrpcBlogService;
use crate::presentation::http::middleware::cors::apply_cors;
use crate::presentation::http::middleware::trace::apply_trace;
use crate::presentation::http::openapi::ApiDoc;
use crate::presentation::http::router as http_router;

use tonic::transport::Server;

pub(crate) async fn run_http(settings: &Settings, state: AppState) -> anyhow::Result<()> {
    let app = build_router(state);
    let app = apply_trace(app);
    let app = apply_cors(app, settings)?;
    let app = app.layer(
        ServiceBuilder::new()
            .layer(HandleErrorLayer::new(handle_http_middleware_error))
            .layer(RequestBodyLimitLayer::new(
                settings.http_request_body_limit_bytes,
            ))
            .layer(ConcurrencyLimitLayer::new(settings.http_concurrency_limit))
            .layer(TimeoutLayer::new(Duration::from_secs(
                settings.http_request_timeout_secs,
            ))),
    );

    let listener = TcpListener::bind(&settings.http_addr).await?;

    info!("HTTP server listening on {}", settings.http_addr);
    axum::serve(listener, app).await?;
    Ok(())
}

pub(crate) fn build_router(state: AppState) -> Router {
    http_router::routes(state)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

pub(crate) async fn run_grpc(settings: &Settings, state: AppState) -> anyhow::Result<()> {
    let grpc = GrpcBlogService::new(state)
        .into_server()
        .max_decoding_message_size(settings.grpc_max_decoding_message_size_bytes)
        .max_encoding_message_size(settings.grpc_max_encoding_message_size_bytes);

    let addr = settings.grpc_addr.parse()?;

    info!("gRPC server listening on {}", settings.grpc_addr);

    Server::builder()
        .layer(ConcurrencyLimitLayer::new(settings.grpc_concurrency_limit))
        .layer(TimeoutLayer::new(Duration::from_secs(
            settings.grpc_request_timeout_secs,
        )))
        .layer(TraceLayer::new_for_grpc())
        .add_service(grpc)
        .serve(addr)
        .await?;

    Ok(())
}

async fn handle_http_middleware_error(err: BoxError) -> StatusCode {
    if err.is::<tower::timeout::error::Elapsed>() {
        StatusCode::REQUEST_TIMEOUT
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
