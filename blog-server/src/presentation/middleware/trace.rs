use axum::Router;
use tower_http::trace::TraceLayer;

pub(crate) fn apply_trace(router: Router) -> Router {
    router.layer(TraceLayer::new_for_http())
}
