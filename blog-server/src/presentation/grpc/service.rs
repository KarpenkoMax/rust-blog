use tonic::{Request, Response, Status};

use crate::presentation::{
    AppState,
    grpc::mappers::{
        to_domain_create_post_request, to_domain_login_request, to_domain_register_request,
        to_domain_update_post_request, to_proto_auth_response, to_proto_list_posts_response,
        to_proto_post,
    },
};

use super::interceptors::authenticate_request;
use super::proto::{
    AuthResponse, BlogService, BlogServiceServer, CreatePostRequest, DeletePostRequest,
    GetPostRequest, ListPostsRequest, ListPostsResponse, LoginRequest, Post, RegisterRequest,
    UpdatePostRequest,
};
use super::status::map_domain_error;

#[derive(Clone)]
pub(crate) struct GrpcBlogService {
    state: AppState,
}

impl GrpcBlogService {
    pub(crate) fn new(state: AppState) -> Self {
        Self { state }
    }

    pub(crate) fn into_server(self) -> BlogServiceServer<Self> {
        BlogServiceServer::new(self)
    }

    pub(crate) fn state(&self) -> &AppState {
        &self.state
    }
}

#[tonic::async_trait]
impl BlogService for GrpcBlogService {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = to_domain_register_request(request.into_inner());

        let result = self
            .state
            .auth_service
            .register(req)
            .await
            .map_err(map_domain_error)?;
        let response = to_proto_auth_response(result);
        Ok(Response::new(response))
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = to_domain_login_request(request.into_inner());

        let result = self
            .state
            .auth_service
            .login(req)
            .await
            .map_err(map_domain_error)?;

        let response = to_proto_auth_response(result);
        Ok(Response::new(response))
    }

    async fn create_post(
        &self,
        request: Request<CreatePostRequest>,
    ) -> Result<Response<Post>, Status> {
        let auth = authenticate_request(self.state().jwt.as_ref(), request.metadata())?;

        let req = to_domain_create_post_request(request.into_inner());

        let result = self
            .state
            .blog_service
            .create_post(auth.user_id, req)
            .await
            .map_err(map_domain_error)?;

        let response = to_proto_post(result);
        Ok(Response::new(response))
    }

    async fn get_post(&self, request: Request<GetPostRequest>) -> Result<Response<Post>, Status> {
        let result = self
            .state
            .blog_service
            .get_post(request.into_inner().id)
            .await
            .map_err(map_domain_error)?;

        let response = to_proto_post(result);
        Ok(Response::new(response))
    }

    async fn update_post(
        &self,
        request: Request<UpdatePostRequest>,
    ) -> Result<Response<Post>, Status> {
        let auth = authenticate_request(self.state().jwt.as_ref(), request.metadata())?;

        let input = request.into_inner();
        let post_id = input.id;
        let req = to_domain_update_post_request(input);
        let result = self
            .state
            .blog_service
            .update_post(auth.user_id, post_id, req)
            .await
            .map_err(map_domain_error)?;

        let response = to_proto_post(result);
        Ok(Response::new(response))
    }

    async fn delete_post(
        &self,
        request: Request<DeletePostRequest>,
    ) -> Result<Response<()>, Status> {
        let auth = authenticate_request(self.state().jwt.as_ref(), request.metadata())?;

        self.state
            .blog_service
            .delete_post(auth.user_id, request.into_inner().id)
            .await
            .map_err(map_domain_error)?;

        Ok(Response::new(()))
    }

    async fn list_posts(
        &self,
        request: Request<ListPostsRequest>,
    ) -> Result<Response<ListPostsResponse>, Status> {
        const DEFAULT_LIMIT: u32 = 20;
        const MAX_LIMIT: u32 = 100;

        let input = request.into_inner();
        let limit = if input.limit == 0 {
            DEFAULT_LIMIT
        } else {
            input.limit
        };
        let offset = input.offset;

        if limit > MAX_LIMIT {
            return Err(Status::invalid_argument(format!(
                "limit must be in 1..={MAX_LIMIT}"
            )));
        }
        let page = (offset / limit) + 1;
        let page_size = limit;

        let result = self
            .state
            .blog_service
            .list_posts(page, page_size)
            .await
            .map_err(map_domain_error)?;

        Ok(Response::new(to_proto_list_posts_response(result)))
    }
}
