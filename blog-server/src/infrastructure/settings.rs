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
    pub http_request_body_limit_bytes: usize,
    pub http_concurrency_limit: usize,
    pub http_request_timeout_secs: u64,
    pub grpc_concurrency_limit: usize,
    pub grpc_request_timeout_secs: u64,
    pub grpc_max_decoding_message_size_bytes: usize,
    pub grpc_max_encoding_message_size_bytes: usize,
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
        let http_request_body_limit_bytes =
            parse_usize_env("HTTP_REQUEST_BODY_LIMIT_BYTES", 1024 * 1024)?;
        let http_concurrency_limit = parse_usize_env("HTTP_CONCURRENCY_LIMIT", 256)?;
        let http_request_timeout_secs = parse_u64_env("HTTP_REQUEST_TIMEOUT_SECS", 10)?;
        let grpc_concurrency_limit = parse_usize_env("GRPC_CONCURRENCY_LIMIT", 256)?;
        let grpc_request_timeout_secs = parse_u64_env("GRPC_REQUEST_TIMEOUT_SECS", 10)?;
        let grpc_max_decoding_message_size_bytes =
            parse_usize_env("GRPC_MAX_DECODING_MESSAGE_SIZE_BYTES", 4 * 1024 * 1024)?;
        let grpc_max_encoding_message_size_bytes =
            parse_usize_env("GRPC_MAX_ENCODING_MESSAGE_SIZE_BYTES", 4 * 1024 * 1024)?;

        Ok(Self {
            database_url,
            jwt_secret,
            jwt_ttl_seconds,
            http_addr,
            grpc_addr,
            cors_origins,
            log_level,
            http_request_body_limit_bytes,
            http_concurrency_limit,
            http_request_timeout_secs,
            grpc_concurrency_limit,
            grpc_request_timeout_secs,
            grpc_max_decoding_message_size_bytes,
            grpc_max_encoding_message_size_bytes,
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

fn parse_usize_env(key: &str, default: usize) -> Result<usize> {
    let value = std::env::var(key)
        .unwrap_or_else(|_| default.to_string())
        .parse::<usize>()
        .with_context(|| format!("Failed to parse {key}, expecting positive integer"))?;

    if value == 0 {
        return Err(anyhow!("{key} must be > 0"));
    }
    Ok(value)
}

fn parse_u64_env(key: &str, default: u64) -> Result<u64> {
    let value = std::env::var(key)
        .unwrap_or_else(|_| default.to_string())
        .parse::<u64>()
        .with_context(|| format!("Failed to parse {key}, expecting positive integer"))?;

    if value == 0 {
        return Err(anyhow!("{key} must be > 0"));
    }
    Ok(value)
}
