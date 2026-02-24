use chrono::{DateTime, Utc};
use prost_types::Timestamp;

use crate::application::auth_service::AuthResult;
use crate::application::blog_service::ListPostsResult;
use crate::domain::post::{
    CreatePostRequest as DomainCreatePostRequest, Post as DomainPost,
    UpdatePostRequest as DomainUpdatePostRequest,
};
use crate::domain::user::{
    LoginRequest as DomainLoginRequest, RegisterRequest as DomainRegisterRequest,
    User as DomainUser,
};

use super::proto::{
    AuthResponse, CreatePostRequest, ListPostsResponse, LoginRequest, Post, RegisterRequest,
    UpdatePostRequest, User,
};

pub(crate) fn to_domain_register_request(input: RegisterRequest) -> DomainRegisterRequest {
    DomainRegisterRequest {
        username: input.username,
        email: input.email,
        password: input.password,
    }
}

pub(crate) fn to_domain_login_request(input: LoginRequest) -> DomainLoginRequest {
    DomainLoginRequest {
        username: input.username,
        password: input.password,
    }
}

pub(crate) fn to_domain_create_post_request(input: CreatePostRequest) -> DomainCreatePostRequest {
    DomainCreatePostRequest {
        title: input.title,
        content: input.content,
    }
}

pub(crate) fn to_domain_update_post_request(input: UpdatePostRequest) -> DomainUpdatePostRequest {
    DomainUpdatePostRequest {
        title: input.title,
        content: input.content,
    }
}

pub(crate) fn to_proto_auth_response(result: AuthResult) -> AuthResponse {
    AuthResponse {
        access_token: result.access_token,
        user: Some(to_proto_user(result.user)),
    }
}

pub(crate) fn to_proto_user(user: DomainUser) -> User {
    User {
        id: user.id,
        username: user.username,
        email: user.email,
        created_at: Some(to_proto_timestamp(user.created_at)),
    }
}

pub(crate) fn to_proto_post(post: DomainPost) -> Post {
    Post {
        id: post.id,
        title: post.title,
        content: post.content,
        author_id: post.author_id,
        created_at: Some(to_proto_timestamp(post.created_at)),
        updated_at: Some(to_proto_timestamp(post.updated_at)),
    }
}

pub(crate) fn to_proto_list_posts_response(result: ListPostsResult) -> ListPostsResponse {
    ListPostsResponse {
        posts: result.posts.into_iter().map(to_proto_post).collect(),
        page: result.page,
        page_size: result.page_size,
        total: result.total.max(0) as u64,
    }
}

fn to_proto_timestamp(value: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: value.timestamp(),
        nanos: value.timestamp_subsec_nanos() as i32,
    }
}
