use reqwest::{Client, Method};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::time::Duration;

use crate::error::{BlogClientError, BlogClientResult};
use crate::models::{AuthResponse, ListPostsResponse, Post, User};

#[derive(Debug, Serialize)]
struct RegisterRequestDto<'a> {
    username: &'a str,
    email: &'a str,
    password: &'a str,
}

#[derive(Debug, Serialize)]
struct LoginRequestDto<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Debug, Serialize)]
struct CreatePostRequestDto<'a> {
    title: &'a str,
    content: &'a str,
}

#[derive(Debug, Serialize)]
struct UpdatePostRequestDto<'a> {
    title: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct ErrorResponseDto {
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AuthResponseDto {
    access_token: String,
    user: UserDto,
}

#[derive(Debug, Deserialize)]
struct UserDto {
    id: i64,
    username: String,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct PostDto {
    id: i64,
    title: String,
    content: String,
    author_id: i64,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct ListPostsResponseDto {
    posts: Vec<PostDto>,
    limit: u32,
    offset: u32,
    total: i64,
}

#[derive(Serialize)]
struct ListPostsQuery {
    limit: u32,
    offset: u32,
}

impl From<AuthResponseDto> for AuthResponse {
    fn from(value: AuthResponseDto) -> Self {
        Self {
            access_token: value.access_token,
            user: User {
                id: value.user.id,
                username: value.user.username,
                email: value.user.email,
                created_at: value.user.created_at,
            },
        }
    }
}

impl From<PostDto> for Post {
    fn from(value: PostDto) -> Self {
        Self {
            id: value.id,
            title: value.title,
            content: value.content,
            author_id: value.author_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<ListPostsResponseDto> for ListPostsResponse {
    fn from(value: ListPostsResponseDto) -> Self {
        Self {
            posts: value.posts.into_iter().map(Post::from).collect(),
            limit: value.limit,
            offset: value.offset,
            total: value.total.max(0) as u64,
        }
    }
}

#[derive(Debug, Clone)]
/// HTTP-клиент для работы с REST API `blog-server`.
pub struct HttpClient {
    base_url: String,
    client: Client,
}

impl HttpClient {
    /// Создаёт новый HTTP-клиент с базовым URL сервера.
    pub fn new(base_url: impl Into<String>) -> Self {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15))
            .build()
            .expect("failed to build reqwest client");

        Self {
            base_url: base_url.into(),
            client,
        }
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    async fn decode_error(response: reqwest::Response) -> BlogClientError {
        let status = response.status();

        let message = match response.json::<ErrorResponseDto>().await {
            Ok(body) => body
                .error
                .unwrap_or_else(|| format!("http status {status}")),
            Err(_) => format!("http status {status}"),
        };
        BlogClientError::from_http_status(status, Some(message))
    }

    /// универсальный helper для отправки запросов с json-payload
    async fn send_json<TReq, TRes>(
        &self,
        method: Method,
        path: &str,
        body: &TReq,
        token: Option<&str>,
    ) -> BlogClientResult<TRes>
    where
        TReq: Serialize,
        TRes: DeserializeOwned,
    {
        let url = self.endpoint(path);

        let mut request = self.client.request(method, url).json(body);
        if let Some(token) = token {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .map_err(BlogClientError::from_reqwest)?;
        if !response.status().is_success() {
            return Err(Self::decode_error(response).await);
        }

        response
            .json::<TRes>()
            .await
            .map_err(BlogClientError::from_reqwest)
    }

    /// Регистрирует пользователя и возвращает JWT + данные пользователя.
    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> BlogClientResult<AuthResponse> {
        let payload = RegisterRequestDto {
            username,
            email,
            password,
        };
        let dto: AuthResponseDto = self
            .send_json(Method::POST, "/api/auth/register", &payload, None)
            .await?;
        Ok(dto.into())
    }

    /// Выполняет авторизацию пользователя и возвращает JWT + данные пользователя.
    pub async fn login(&self, username: &str, password: &str) -> BlogClientResult<AuthResponse> {
        let payload = LoginRequestDto { username, password };
        let dto: AuthResponseDto = self
            .send_json(Method::POST, "/api/auth/login", &payload, None)
            .await?;
        Ok(dto.into())
    }

    /// Создаёт пост от имени авторизованного пользователя.
    ///
    /// Требует валидный JWT-токен.
    pub async fn create_post(
        &self,
        token: &str,
        title: &str,
        content: &str,
    ) -> BlogClientResult<Post> {
        let payload = CreatePostRequestDto { title, content };
        let dto: PostDto = self
            .send_json(Method::POST, "/api/posts", &payload, Some(token))
            .await?;

        Ok(dto.into())
    }

    /// Получает пост по идентификатору.
    pub async fn get_post(&self, id: i64) -> BlogClientResult<Post> {
        let url = self.endpoint(&format!("/api/posts/{id}"));

        let request = self.client.request(Method::GET, url);

        let response = request
            .send()
            .await
            .map_err(BlogClientError::from_reqwest)?;
        if !response.status().is_success() {
            return Err(Self::decode_error(response).await);
        }

        let dto = response
            .json::<PostDto>()
            .await
            .map_err(BlogClientError::from_reqwest)?;
        Ok(dto.into())
    }

    /// Обновляет пост по идентификатору.
    ///
    /// Требует валидный JWT-токен.
    pub async fn update_post(
        &self,
        token: &str,
        id: i64,
        title: &str,
        content: &str,
    ) -> BlogClientResult<Post> {
        let payload = UpdatePostRequestDto { title, content };
        let dto: PostDto = self
            .send_json(
                Method::PUT,
                &format!("/api/posts/{id}"),
                &payload,
                Some(token),
            )
            .await?;

        Ok(dto.into())
    }

    /// Удаляет пост по идентификатору.
    ///
    /// Требует валидный JWT-токен.
    pub async fn delete_post(&self, token: &str, id: i64) -> BlogClientResult<()> {
        let url = self.endpoint(&format!("/api/posts/{id}"));

        let request = self.client.request(Method::DELETE, url).bearer_auth(token);

        let response = request
            .send()
            .await
            .map_err(BlogClientError::from_reqwest)?;
        if !response.status().is_success() {
            return Err(Self::decode_error(response).await);
        }

        Ok(())
    }

    /// Возвращает список постов с пагинацией `limit/offset`.
    pub async fn list_posts(&self, limit: u32, offset: u32) -> BlogClientResult<ListPostsResponse> {
        let url = self.endpoint("/api/posts");

        let query = ListPostsQuery { limit, offset };

        let request = self.client.request(Method::GET, url).query(&query);

        let response = request
            .send()
            .await
            .map_err(BlogClientError::from_reqwest)?;
        if !response.status().is_success() {
            return Err(Self::decode_error(response).await);
        }

        let dto = response
            .json::<ListPostsResponseDto>()
            .await
            .map_err(BlogClientError::from_reqwest)?;
        Ok(dto.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn endpoint_normalizes_slashes() {
        let client = HttpClient::new("http://localhost:8080/");
        let full = client.endpoint("/api/posts");
        assert_eq!(full, "http://localhost:8080/api/posts");
    }

    #[test]
    fn list_posts_response_keeps_limit_and_offset() {
        let dto = ListPostsResponseDto {
            posts: vec![],
            limit: 20,
            offset: 40,
            total: 42,
        };

        let mapped = ListPostsResponse::from(dto);
        assert_eq!(mapped.limit, 20);
        assert_eq!(mapped.offset, 40);
        assert_eq!(mapped.total, 42);
    }

    #[test]
    fn list_posts_response_clamps_negative_total() {
        let dto = ListPostsResponseDto {
            posts: vec![PostDto {
                id: 1,
                title: "t".to_string(),
                content: "c".to_string(),
                author_id: 2,
                created_at: Utc.timestamp_opt(10, 0).single().expect("valid ts"),
                updated_at: Utc.timestamp_opt(20, 0).single().expect("valid ts"),
            }],
            limit: 10,
            offset: 0,
            total: -7,
        };

        let mapped = ListPostsResponse::from(dto);
        assert_eq!(mapped.total, 0);
        assert_eq!(mapped.posts.len(), 1);
        assert_eq!(mapped.posts[0].id, 1);
    }
}
