use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub user: User,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListPostsResponse {
    pub posts: Vec<Post>,
    pub limit: u32,
    pub offset: u32,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdatePostRequest {
    pub title: String,
    pub content: String,
}
