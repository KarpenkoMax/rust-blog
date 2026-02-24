use anyhow::Result;
use std::sync::Arc;

mod application;
mod data;
mod domain;
mod infrastructure;
mod presentation;
mod server;

use application::auth_service::AuthService;
use application::blog_service::BlogService;
use data::repositories::postgres::post_repository::PostgresPostRepository;
use data::repositories::postgres::user_repository::PostgresUserRepository;
use infrastructure::database::{create_pool, run_migrations};
use infrastructure::jwt::JwtService;
use infrastructure::logging::init_logging;
use infrastructure::settings::Settings;
use presentation::AppState;
use server::{run_grpc, run_http};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let settings = Settings::from_env()?;

    init_logging(&settings.log_level)?;

    let pool = create_pool(&settings.database_url).await?;
    run_migrations(&pool).await?;

    let user_repo = PostgresUserRepository::new(pool.clone());
    let post_repo = PostgresPostRepository::new(pool.clone());
    let jwt = Arc::new(JwtService::new(
        &settings.jwt_secret,
        settings.jwt_ttl_seconds,
    ));
    let auth_service = Arc::new(AuthService::new(
        user_repo,
        JwtService::new(&settings.jwt_secret, 24 * 60 * 60),
    ));
    let blog_service = Arc::new(BlogService::new(post_repo));

    let state = AppState::new(pool, auth_service, blog_service, jwt);

    tokio::try_join!(
        run_http(&settings, state.clone()),
        run_grpc(&settings, state)
    )?;
    Ok(())
}
