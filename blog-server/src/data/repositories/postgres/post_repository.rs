use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::data::post_repository::{NewPost, Pagination, PostPatch, PostRepository};
use crate::domain::error::DomainError;
use crate::domain::post::Post;

#[derive(Debug, Clone)]
pub(crate) struct PostgresPostRepository {
    pool: PgPool,
}

impl PostgresPostRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct PostRow {
    id: i64,
    title: String,
    content: String,
    author_id: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    async fn create_post(&self, input: NewPost) -> Result<Post, DomainError> {
        let row = sqlx::query_as!(
            PostRow,
            r#"
            INSERT INTO posts (title, content, author_id)
            VALUES ($1, $2, $3)
            RETURNING id, title as "title!", content, author_id, created_at, updated_at
            "#,
            input.title,
            input.content,
            input.author_id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_post_db_error)?;

        Post::new(
            row.id,
            row.title,
            row.content,
            row.author_id,
            row.created_at,
            row.updated_at,
        )
        .map_err(|err| DomainError::Unexpected(err.to_string()))
    }

    async fn get_post(&self, id: i64) -> Result<Option<Post>, DomainError> {
        let row = sqlx::query_as!(
            PostRow,
            r#"
            SELECT
            id,
            title,
            content,
            author_id,
            created_at,
            updated_at
            FROM posts
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_post_db_error)?;

        row.map(map_row_to_post).transpose()
    }

    async fn update_post_owned(
        &self,
        post_id: i64,
        owner_id: i64,
        patch: PostPatch,
    ) -> Result<Option<Post>, DomainError> {
        let row = sqlx::query_as!(
            PostRow,
            r#"
            UPDATE posts
            SET title = $3,
                content = $4,
                updated_at = NOW()
            WHERE id = $1 AND author_id = $2
            RETURNING id, title as "title!", content, author_id, created_at, updated_at
            "#,
            post_id,
            owner_id,
            patch.title,
            patch.content,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_post_db_error)?;

        row.map(map_row_to_post).transpose()
    }

    async fn delete_post(&self, id: i64) -> Result<bool, DomainError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM posts
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(map_post_db_error)?;

        Ok(result.rows_affected() > 0)
    }

    async fn list_posts(&self, pagination: Pagination) -> Result<Vec<Post>, DomainError> {
        let limit = pagination.page_size as i64;
        let offset = (pagination.page.saturating_sub(1) as i64) * limit;

        let rows = sqlx::query_as!(
            PostRow,
            r#"
            SELECT
                id,
                title,
                content,
                author_id,
                created_at,
                updated_at
            FROM posts
            ORDER BY created_at DESC, id DESC
            LIMIT $1
            OFFSET $2
            "#,
            limit,
            offset,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_post_db_error)?;

        rows.into_iter().map(map_row_to_post).collect()
    }

    async fn total_posts(&self) -> Result<i64, DomainError> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM posts
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_post_db_error)?;

        Ok(row.count)
    }
}

fn map_row_to_post(row: PostRow) -> Result<Post, DomainError> {
    Post::new(
        row.id,
        row.title,
        row.content,
        row.author_id,
        row.created_at,
        row.updated_at,
    )
    .map_err(|err| DomainError::Unexpected(err.to_string()))
}

fn map_post_db_error(err: sqlx::Error) -> DomainError {
    if let sqlx::Error::Database(db_err) = &err
        && db_err.code().as_deref() == Some("23503") {
            return DomainError::NotFound("author".to_string());
        }
    DomainError::Unexpected(err.to_string())
}
