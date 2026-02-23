use sqlx::PgPool;
use std::sync::Arc;

use crate::application::auth_service::AuthService;
use crate::application::blog_service::BlogService;
use crate::data::repositories::postgres::post_repository::PostgresPostRepository;
use crate::data::repositories::postgres::user_repository::PostgresUserRepository;
use crate::infrastructure::jwt::JwtService;

pub(crate) mod app_error;
pub(crate) mod grpc_service;
pub(crate) mod handlers;
pub(crate) mod http_handlers;
pub(crate) mod middleware;
pub(crate) mod openapi;
pub(crate) mod routes;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) pool: PgPool,
    pub(crate) auth_service: Arc<AuthService<PostgresUserRepository>>,
    pub(crate) blog_service: Arc<BlogService<PostgresPostRepository>>,
    pub(crate) jwt: Arc<JwtService>,
}

impl AppState {
    pub(crate) fn new(
        pool: PgPool,
        auth_service: Arc<AuthService<PostgresUserRepository>>,
        blog_service: Arc<BlogService<PostgresPostRepository>>,
        jwt: Arc<JwtService>,
    ) -> Self {
        Self {
            pool,
            auth_service,
            blog_service,
            jwt,
        }
    }
}
