use anyhow::{Context, Result, anyhow};

#[derive(Debug, Clone)]
pub struct Settings {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_ttl_seconds: i64,
    pub http_addr: String,
    pub grpc_addr: String,
    pub cors_origins: Vec<String>,
    pub log_level: String,
}

impl Settings {
    pub fn from_env() -> Result<Self> {
        let database_url = get_required("DATABASE_URL").context("DATABASE_URL is required")?;
        let jwt_secret = get_required("JWT_SECRET").context("JWT_SECRET is required")?;
        let jwt_ttl_seconds: i64 = std::env::var("JWT_TTL_SECONDS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .context("Failed to parse JWT_TTL_SECONDS, expecting integer")?;

        if jwt_secret.chars().count() < 32 {
            return Err(anyhow!("JWT_SECRET must be at least 32 characters"));
        }

        let http_addr = std::env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        let grpc_addr = std::env::var("GRPC_ADDR").unwrap_or_else(|_| "0.0.0.0:50051".to_string());
        let cors_origins = parse_cors_origins(
            std::env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:8000,http://127.0.0.1:8000".to_string()),
        );
        let log_level = std::env::var("LOG_LEVEL")
            .or_else(|_| std::env::var("RUST_LOG"))
            .unwrap_or_else(|_| "info".to_string());

        Ok(Self {
            database_url,
            jwt_secret,
            jwt_ttl_seconds,
            http_addr,
            grpc_addr,
            cors_origins,
            log_level,
        })
    }
}

fn get_required(key: &str) -> Result<String> {
    let value = std::env::var(key)?;
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(anyhow!("{key} must not be empty"));
    }
    Ok(value)
}

fn parse_cors_origins(raw: String) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(str::to_string)
        .collect()
}
