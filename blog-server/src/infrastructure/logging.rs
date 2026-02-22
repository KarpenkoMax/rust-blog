use anyhow::{Result, anyhow};
use tracing_subscriber::{EnvFilter, fmt};

pub fn init_logging(default_level: &str) -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(default_level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .compact()
        .try_init()
        .map_err(|e| anyhow!("failed to init logging: {e}"))?;

    Ok(())
}
