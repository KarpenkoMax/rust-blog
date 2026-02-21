use anyhow::Result;
use axum::Router;
use tokio::net::TcpListener;
use tracing::info;

mod infrastructure;

use infrastructure::database::{create_pool, run_migrations};
use infrastructure::settings::Settings;


#[tokio::main]
async fn main() -> Result<()> {

    dotenvy::dotenv().ok();
    let settings = Settings::from_env()?;

    let pool = create_pool(&settings.database_url).await?;
    run_migrations(&pool).await?;

    Ok(())
}
