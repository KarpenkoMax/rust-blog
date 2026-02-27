use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Публичная модель пользователя.
pub struct User {
    /// Идентификатор пользователя.
    pub id: i64,
    /// Логин.
    pub username: String,
    /// Email.
    pub email: String,
    /// Дата и время создания пользователя (UTC).
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Публичная модель поста.
pub struct Post {
    /// Идентификатор поста.
    pub id: i64,
    /// Заголовок поста.
    pub title: String,
    /// Содержимое поста.
    pub content: String,
    /// Идентификатор автора.
    pub author_id: i64,
    /// Дата и время создания поста (UTC).
    pub created_at: DateTime<Utc>,
    /// Дата и время последнего обновления поста (UTC).
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Ответ после успешной регистрации или входа.
pub struct AuthResponse {
    /// JWT access token.
    pub access_token: String,
    /// Данные пользователя.
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Ответ списка постов с параметрами пагинации.
pub struct ListPostsResponse {
    /// Список постов на текущей странице.
    pub posts: Vec<Post>,
    /// Размер страницы.
    pub limit: u32,
    /// Смещение от начала выборки.
    pub offset: u32,
    /// Общее количество постов.
    pub total: u64,
}
