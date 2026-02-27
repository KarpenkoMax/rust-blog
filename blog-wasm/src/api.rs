use gloo_net::http::Request;
use serde::de::DeserializeOwned;

use crate::models::{
    AuthResponse, CreatePostRequest, ListPostsResponse, LoginRequest, Post, RegisterRequest,
    UpdatePostRequest,
};

const API_BASE_URL: &str = match option_env!("WASM_API_BASE_URL") {
    Some(value) => value,
    None => "http://127.0.0.1:8080",
};

#[derive(Debug, Clone)]
pub(crate) enum ApiError {
    Network(String),
    Http { status: u16, message: String },
    Decode(String),
}

impl core::fmt::Display for ApiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Network(msg) => write!(f, "network error: {msg}"),
            Self::Http { status, message } => write!(f, "http error {status}: {message}"),
            Self::Decode(msg) => write!(f, "decode error: {msg}"),
        }
    }
}

fn endpoint(path: &str) -> String {
    format!(
        "{}/{}",
        API_BASE_URL.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

async fn parse_json<T: DeserializeOwned>(response: gloo_net::http::Response) -> Result<T, ApiError> {
    response
        .json::<T>()
        .await
        .map_err(|err| ApiError::Decode(err.to_string()))
}

async fn parse_error_body(response: gloo_net::http::Response) -> ApiError {
    let status = response.status();
    let text = response
        .text()
        .await
        .unwrap_or_else(|_| "request failed".to_string());

    let fallback = match status {
        400 => "Некорректный запрос".to_string(),
        401 => "Требуется авторизация".to_string(),
        403 => "Недостаточно прав для этой операции".to_string(),
        404 => "Ресурс не найден".to_string(),
        409 => "Конфликт данных (например, пользователь уже существует)".to_string(),
        500..=599 => "Ошибка сервера".to_string(),
        _ => format!("HTTP ошибка {status}"),
    };

    let message = if text.trim().is_empty() { fallback } else { text };

    ApiError::Http { status, message }
}

pub(crate) async fn register(username: &str, email: &str, password: &str) -> Result<AuthResponse, ApiError> {
    let payload = RegisterRequest {
        username: username.to_string(),
        email: email.to_string(),
        password: password.to_string(),
    };

    let response = Request::post(&endpoint("/api/auth/register"))
        .json(&payload)
        .map_err(|err| ApiError::Network(err.to_string()))?
        .send()
        .await
        .map_err(|err| ApiError::Network(err.to_string()))?;

    if !response.ok() {
        return Err(parse_error_body(response).await);
    }

    parse_json(response).await
}

pub(crate) async fn login(username: &str, password: &str) -> Result<AuthResponse, ApiError> {
    let payload = LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
    };

    let response = Request::post(&endpoint("/api/auth/login"))
        .json(&payload)
        .map_err(|err| ApiError::Network(err.to_string()))?
        .send()
        .await
        .map_err(|err| ApiError::Network(err.to_string()))?;

    if !response.ok() {
        return Err(parse_error_body(response).await);
    }

    parse_json(response).await
}

pub(crate) async fn list_posts(limit: u32, offset: u32) -> Result<ListPostsResponse, ApiError> {
    let page_size = limit.max(1);
    let page = (offset / page_size) + 1;
    let url = endpoint(&format!("/api/posts?page={page}&page_size={page_size}"));

    let response = Request::get(&url)
        .send()
        .await
        .map_err(|err| ApiError::Network(err.to_string()))?;

    if !response.ok() {
        return Err(parse_error_body(response).await);
    }

    parse_json(response).await
}

pub(crate) async fn create_post(token: &str, title: &str, content: &str) -> Result<Post, ApiError> {
    let payload = CreatePostRequest {
        title: title.to_string(),
        content: content.to_string(),
    };

    let response = Request::post(&endpoint("/api/posts"))
        .header("Authorization", &format!("Bearer {token}"))
        .json(&payload)
        .map_err(|err| ApiError::Network(err.to_string()))?
        .send()
        .await
        .map_err(|err| ApiError::Network(err.to_string()))?;

    if !response.ok() {
        return Err(parse_error_body(response).await);
    }

    parse_json(response).await
}

pub(crate) async fn update_post(
    token: &str,
    id: i64,
    title: &str,
    content: &str,
) -> Result<Post, ApiError> {
    let payload = UpdatePostRequest {
        title: title.to_string(),
        content: content.to_string(),
    };

    let response = Request::put(&endpoint(&format!("/api/posts/{id}")))
        .header("Authorization", &format!("Bearer {token}"))
        .json(&payload)
        .map_err(|err| ApiError::Network(err.to_string()))?
        .send()
        .await
        .map_err(|err| ApiError::Network(err.to_string()))?;

    if !response.ok() {
        return Err(parse_error_body(response).await);
    }

    parse_json(response).await
}

pub(crate) async fn delete_post(token: &str, id: i64) -> Result<(), ApiError> {
    let response = Request::delete(&endpoint(&format!("/api/posts/{id}")))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|err| ApiError::Network(err.to_string()))?;

    if !response.ok() {
        return Err(parse_error_body(response).await);
    }

    Ok(())
}
