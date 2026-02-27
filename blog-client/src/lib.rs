//! Клиентская библиотека для работы с blog-server по HTTP или gRPC.
//!
//! Предоставляет единый API (`BlogClient`) поверх двух транспортов:
//! - HTTP (`reqwest`)
//! - gRPC (`tonic`)
//!
//! Клиент хранит JWT-токен после `register`/`login` и автоматически использует
//! его в защищённых операциях.
#![warn(missing_docs)]

mod error;
mod grpc_client;
mod http_client;
mod models;

pub use error::{BlogClientError, BlogClientResult};
pub use models::{AuthResponse, ListPostsResponse, Post, User};

use grpc_client::GrpcClient;
use http_client::HttpClient;

#[derive(Debug, Clone)]
/// Транспорт, через который `BlogClient` отправляет запросы.
pub enum Transport {
    /// HTTP-транспорт с базовым URL, например `http://127.0.0.1:8080`.
    Http(String),
    /// gRPC-транспорт с endpoint, например `http://127.0.0.1:50051`.
    Grpc(String),
}

#[derive(Debug, Clone)]
/// Унифицированный клиент для работы с блог-сервисом через HTTP или gRPC.
pub struct BlogClient {
    transport: Transport,
    http_client: Option<HttpClient>,
    grpc_client: Option<GrpcClient>,
    token: Option<String>,
}

impl BlogClient {
    /// Создаёт клиент с выбранным транспортом и инициализирует внутренний
    /// HTTP/gRPC-клиент.
    pub fn new(transport: Transport) -> Self {
        let (http_client, grpc_client) = match &transport {
            Transport::Http(base_url) => (Some(HttpClient::new(base_url.clone())), None),
            Transport::Grpc(endpoint) => (None, Some(GrpcClient::new(endpoint.clone()))),
        };

        Self {
            transport,
            http_client,
            grpc_client,
            token: None,
        }
    }

    /// Устанавливает JWT-токен вручную.
    pub fn set_token(&mut self, token: impl Into<String>) {
        self.token = Some(token.into());
    }

    /// Возвращает текущий JWT-токен, если он установлен.
    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// Очищает сохранённый JWT-токен.
    pub fn clear_token(&mut self) {
        self.token = None;
    }

    /// Регистрирует пользователя и сохраняет полученный JWT-токен в клиенте.
    pub async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> BlogClientResult<AuthResponse> {
        let result = match &self.transport {
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .ok_or_else(|| {
                        BlogClientError::InvalidRequest(
                            "http client is not initialized".to_string(),
                        )
                    })?
                    .register(username, email, password)
                    .await?
            }
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_ref()
                    .ok_or_else(|| {
                        BlogClientError::InvalidRequest(
                            "grpc client is not initialized".to_string(),
                        )
                    })?
                    .register(username, email, password)
                    .await?
            }
        };
        self.token = Some(result.access_token.clone());
        Ok(result)
    }

    /// Выполняет вход пользователя и сохраняет полученный JWT-токен в клиенте.
    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> BlogClientResult<AuthResponse> {
        let result = match &self.transport {
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .ok_or_else(|| {
                        BlogClientError::InvalidRequest(
                            "http client is not initialized".to_string(),
                        )
                    })?
                    .login(username, password)
                    .await?
            }
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_ref()
                    .ok_or_else(|| {
                        BlogClientError::InvalidRequest(
                            "grpc client is not initialized".to_string(),
                        )
                    })?
                    .login(username, password)
                    .await?
            }
        };

        self.token = Some(result.access_token.clone());
        Ok(result)
    }

    /// Создаёт новый пост.
    ///
    /// Требует установленный JWT-токен.
    pub async fn create_post(&self, title: &str, content: &str) -> BlogClientResult<Post> {
        let token = self.require_token()?;
        match &self.transport {
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .ok_or_else(|| {
                        BlogClientError::InvalidRequest(
                            "http client is not initialized".to_string(),
                        )
                    })?
                    .create_post(token, title, content)
                    .await
            }
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_ref()
                    .ok_or_else(|| {
                        BlogClientError::InvalidRequest(
                            "grpc client is not initialized".to_string(),
                        )
                    })?
                    .create_post(token, title, content)
                    .await
            }
        }
    }

    /// Возвращает пост по идентификатору.
    pub async fn get_post(&self, id: i64) -> BlogClientResult<Post> {
        match &self.transport {
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .ok_or_else(|| {
                        BlogClientError::InvalidRequest(
                            "http client is not initialized".to_string(),
                        )
                    })?
                    .get_post(id)
                    .await
            }
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_ref()
                    .ok_or_else(|| {
                        BlogClientError::InvalidRequest(
                            "grpc client is not initialized".to_string(),
                        )
                    })?
                    .get_post(id)
                    .await
            }
        }
    }

    /// Обновляет пост по идентификатору.
    ///
    /// Требует установленный JWT-токен.
    pub async fn update_post(&self, id: i64, title: &str, content: &str) -> BlogClientResult<Post> {
        let token = self.require_token()?;
        match &self.transport {
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .ok_or_else(|| {
                        BlogClientError::InvalidRequest(
                            "http client is not initialized".to_string(),
                        )
                    })?
                    .update_post(token, id, title, content)
                    .await
            }
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_ref()
                    .ok_or_else(|| {
                        BlogClientError::InvalidRequest(
                            "grpc client is not initialized".to_string(),
                        )
                    })?
                    .update_post(token, id, title, content)
                    .await
            }
        }
    }

    /// Удаляет пост по идентификатору.
    ///
    /// Требует установленный JWT-токен.
    pub async fn delete_post(&self, id: i64) -> BlogClientResult<()> {
        let token = self.require_token()?;
        match &self.transport {
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .ok_or_else(|| {
                        BlogClientError::InvalidRequest(
                            "http client is not initialized".to_string(),
                        )
                    })?
                    .delete_post(token, id)
                    .await
            }
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_ref()
                    .ok_or_else(|| {
                        BlogClientError::InvalidRequest(
                            "grpc client is not initialized".to_string(),
                        )
                    })?
                    .delete_post(token, id)
                    .await
            }
        }
    }

    /// Возвращает список постов с пагинацией.
    ///
    /// `limit` и `offset` задаются в клиентских терминах. Для серверного API:
    /// - `page = offset / limit + 1`
    /// - `page_size = limit`
    ///
    /// При `limit == 0` используется `page = 1`, `page_size = 0`.
    pub async fn list_posts(&self, limit: u32, offset: u32) -> BlogClientResult<ListPostsResponse> {
        match &self.transport {
            Transport::Http(_) => {
                self.http_client
                    .as_ref()
                    .ok_or_else(|| {
                        BlogClientError::InvalidRequest(
                            "http client is not initialized".to_string(),
                        )
                    })?
                    .list_posts(limit, offset)
                    .await
            }
            Transport::Grpc(_) => {
                self.grpc_client
                    .as_ref()
                    .ok_or_else(|| {
                        BlogClientError::InvalidRequest(
                            "grpc client is not initialized".to_string(),
                        )
                    })?
                    .list_posts(limit, offset)
                    .await
            }
        }
    }

    fn require_token(&self) -> BlogClientResult<&str> {
        self.token.as_deref().ok_or(BlogClientError::Unauthorized)
    }
}
