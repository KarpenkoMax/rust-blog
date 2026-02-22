use async_trait::async_trait;

use crate::domain::error::DomainError;
use crate::domain::post::Post;

#[derive(Debug, Clone)]
pub(crate) struct NewPost {
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) author_id: i64,
}

#[derive(Debug, Clone)]
pub(crate) struct PostPatch {
    pub(crate) title: String,
    pub(crate) content: String,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Pagination {
    pub(crate) page: u32,
    pub(crate) page_size: u32,
}

#[async_trait]
pub(crate) trait PostRepository: Send + Sync {
    async fn create_post(&self, input: NewPost) -> Result<Post, DomainError>;
    async fn get_post(&self, id: i64) -> Result<Option<Post>, DomainError>;
    async fn update_post_owned(
        &self,
        post_id: i64,
        owner_id: i64,
        patch: PostPatch,
    ) -> Result<Option<Post>, DomainError>;
    async fn delete_post(&self, id: i64) -> Result<bool, DomainError>;
    async fn list_posts(&self, pagination: Pagination) -> Result<Vec<Post>, DomainError>;
    async fn total_posts(&self) -> Result<i64, DomainError>;
}
