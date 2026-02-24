pub(crate) mod pb {
    tonic::include_proto!("blog");
}

pub(crate) use pb::blog_service_server::{BlogService, BlogServiceServer};
pub(crate) use pb::{
    AuthResponse, CreatePostRequest, DeletePostRequest, GetPostRequest, ListPostsRequest,
    ListPostsResponse, LoginRequest, Post, RegisterRequest, UpdatePostRequest, User,
};
