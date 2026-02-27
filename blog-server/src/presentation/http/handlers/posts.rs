use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::domain::post::{Post, UpdatePostRequest};
use crate::presentation::AppState;
use crate::presentation::http::app_error::AppResult;
use crate::presentation::http::middleware::auth::AuthenticatedUser;
use crate::{application::blog_service::ListPostsResult, domain::post::CreatePostRequest};

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub(crate) struct CreatePostDto {
    #[validate(length(min = 1, max = 255))]
    pub(crate) title: String,
    #[validate(length(min = 1))]
    pub(crate) content: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub(crate) struct UpdatePostDto {
    #[validate(length(min = 1, max = 255))]
    pub(crate) title: String,
    #[validate(length(min = 1))]
    pub(crate) content: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub(crate) struct PaginationQuery {
    #[validate(range(min = 1, max = 100))]
    pub(crate) limit: Option<u32>,
    pub(crate) offset: Option<u32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct PostDto {
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) author_id: i64,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub(crate) struct ListPostsResponseDto {
    pub(crate) posts: Vec<PostDto>,
    pub(crate) limit: u32,
    pub(crate) offset: u32,
    pub(crate) total: i64,
}

impl From<Post> for PostDto {
    fn from(post: Post) -> Self {
        Self {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            created_at: post.created_at,
            updated_at: post.updated_at,
        }
    }
}

impl From<ListPostsResult> for ListPostsResponseDto {
    fn from(result: ListPostsResult) -> Self {
        let offset = result
            .page
            .saturating_sub(1)
            .saturating_mul(result.page_size);
        Self {
            posts: result.posts.into_iter().map(PostDto::from).collect(),
            limit: result.page_size,
            offset,
            total: result.total,
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/posts",
    tag = "posts",
    params(
        ("limit" = Option<u32>, Query, description = "Items per page (1..=100)"),
        ("offset" = Option<u32>, Query, description = "Offset from the beginning (>= 0)")
    ),
    responses(
        (status = 200, description = "Posts listed", body = ListPostsResponseDto),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal error")
    )
)]
pub(crate) async fn list_posts(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> AppResult<(StatusCode, Json<ListPostsResponseDto>)> {
    query.validate()?;
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    let page = (offset / limit) + 1;
    let page_size = limit;

    let result = state.blog_service.list_posts(page, page_size).await?;

    Ok((StatusCode::OK, Json(ListPostsResponseDto::from(result))))
}

#[utoipa::path(
    get,
    path = "/api/posts/{id}",
    tag = "posts",
    params(
        ("id" = i64, Path, description = "Post id")
    ),
    responses(
        (status = 200, description = "Post found", body = PostDto),
        (status = 404, description = "Post not found"),
        (status = 500, description = "Internal error")
    )
)]
pub(crate) async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<(StatusCode, Json<PostDto>)> {
    let result = state.blog_service.get_post(id).await?;

    Ok((StatusCode::OK, Json(PostDto::from(result))))
}

#[utoipa::path(
    post,
    path = "/api/posts",
    tag = "posts",
    security(
        ("bearer_auth" = [])
    ),
    request_body = CreatePostDto,
    responses(
        (status = 201, description = "Post created", body = PostDto),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal error")
    )
)]
pub(crate) async fn create_post(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(dto): Json<CreatePostDto>,
) -> AppResult<(StatusCode, Json<PostDto>)> {
    dto.validate()?;
    let req = CreatePostRequest {
        title: dto.title,
        content: dto.content,
    };

    let result = state.blog_service.create_post(auth.user_id, req).await?;
    Ok((StatusCode::CREATED, Json(PostDto::from(result))))
}

#[utoipa::path(
    put,
    path = "/api/posts/{id}",
    tag = "posts",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = i64, Path, description = "Post id")
    ),
    request_body = UpdatePostDto,
    responses(
        (status = 200, description = "Post updated", body = PostDto),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Post not found"),
        (status = 500, description = "Internal error")
    )
)]
pub(crate) async fn update_post(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<i64>,
    Json(dto): Json<UpdatePostDto>,
) -> AppResult<(StatusCode, Json<PostDto>)> {
    dto.validate()?;
    let req = UpdatePostRequest {
        title: dto.title,
        content: dto.content,
    };

    let result = state
        .blog_service
        .update_post(auth.user_id, id, req)
        .await?;
    Ok((StatusCode::OK, Json(PostDto::from(result))))
}

#[utoipa::path(
    delete,
    path = "/api/posts/{id}",
    tag = "posts",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("id" = i64, Path, description = "Post id")
    ),
    responses(
        (status = 204, description = "Post deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Post not found"),
        (status = 500, description = "Internal error")
    )
)]
pub(crate) async fn delete_post(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<i64>,
) -> AppResult<StatusCode> {
    state.blog_service.delete_post(auth.user_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
