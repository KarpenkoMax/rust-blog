use axum::Router;
use axum::middleware;
use axum::routing::{get, post, put};

use crate::presentation::AppState;
use crate::presentation::handlers::posts::{
    create_post, delete_post, get_post, list_posts, update_post,
};
use crate::presentation::middleware::auth::jwt_auth_middleware;

pub(crate) fn router(state: AppState) -> Router<AppState> {
    let public = Router::new()
        .route("/", get(list_posts))
        .route("/{id}", get(get_post));

    let protected = Router::new()
        .route("/", post(create_post))
        .route("/{id}", put(update_post).delete(delete_post))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            jwt_auth_middleware,
        ));

    public.merge(protected)
}
