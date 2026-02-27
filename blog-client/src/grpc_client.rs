use std::time::Duration;
use tonic::metadata::MetadataValue;
use tonic::transport::{Channel, Endpoint};

use crate::error::{BlogClientError, BlogClientResult};
use crate::models::{AuthResponse, ListPostsResponse, Post, User};

pub mod pb {
    tonic::include_proto!("blog");
}

#[derive(Debug)]
struct AuthResponseDto {
    access_token: String,
    user: UserDto,
}

#[derive(Debug)]
struct UserDto {
    id: i64,
    username: String,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
struct PostDto {
    id: i64,
    title: String,
    content: String,
    author_id: i64,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
struct ListPostsResponseDto {
    posts: Vec<PostDto>,
    limit: u32,
    offset: u32,
    total: u64,
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
            total: value.total,
        }
    }
}

#[derive(Debug, Clone)]
/// gRPC-клиент для работы с API `blog-server`.
pub struct GrpcClient {
    endpoint: String,
}

impl GrpcClient {
    /// Создаёт новый gRPC-клиент с endpoint сервера.
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }

    /// Регистрирует пользователя и возвращает JWT + данные пользователя.
    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> BlogClientResult<AuthResponse> {
        let mut client = self.connect().await?;
        let request = tonic::Request::new(pb::RegisterRequest {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        });
        let response = client
            .register(request)
            .await
            .map_err(BlogClientError::from_grpc_status)?;

        let dto = Self::map_auth_response(response.into_inner())?;
        Ok(dto.into())
    }

    /// Выполняет авторизацию пользователя и возвращает JWT + данные пользователя.
    pub async fn login(&self, username: &str, password: &str) -> BlogClientResult<AuthResponse> {
        let mut client = self.connect().await?;
        let request = tonic::Request::new(pb::LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        });
        let response = client
            .login(request)
            .await
            .map_err(BlogClientError::from_grpc_status)?;

        let dto = Self::map_auth_response(response.into_inner())?;
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
        let mut client = self.connect().await?;
        let request = tonic::Request::new(pb::CreatePostRequest {
            title: title.to_string(),
            content: content.to_string(),
        });
        let request = Self::attach_bearer_token(request, token)?;

        let response = client
            .create_post(request)
            .await
            .map_err(BlogClientError::from_grpc_status)?;

        let dto = Self::map_post(response.into_inner())?;
        Ok(dto.into())
    }

    /// Получает пост по идентификатору.
    pub async fn get_post(&self, id: i64) -> BlogClientResult<Post> {
        let mut client = self.connect().await?;
        let request = tonic::Request::new(pb::GetPostRequest { id });

        let response = client
            .get_post(request)
            .await
            .map_err(BlogClientError::from_grpc_status)?;
        let dto = Self::map_post(response.into_inner())?;
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
        let mut client = self.connect().await?;
        let request = tonic::Request::new(pb::UpdatePostRequest {
            id,
            title: title.to_string(),
            content: content.to_string(),
        });
        let request = Self::attach_bearer_token(request, token)?;

        let response = client
            .update_post(request)
            .await
            .map_err(BlogClientError::from_grpc_status)?;
        let dto = Self::map_post(response.into_inner())?;
        Ok(dto.into())
    }

    /// Удаляет пост по идентификатору.
    ///
    /// Требует валидный JWT-токен.
    pub async fn delete_post(&self, token: &str, id: i64) -> BlogClientResult<()> {
        let mut client = self.connect().await?;
        let request = tonic::Request::new(pb::DeletePostRequest { id });
        let request = Self::attach_bearer_token(request, token)?;

        client
            .delete_post(request)
            .await
            .map_err(BlogClientError::from_grpc_status)?;
        Ok(())
    }

    /// Возвращает список постов с клиентской пагинацией `limit/offset`.
    ///
    /// gRPC API сервера принимает пагинацию в виде `page/page_size`,
    /// поэтому внутри выполняется преобразование:
    /// - `page_size = max(limit, 1)`
    /// - `page = (offset / page_size) + 1`
    ///
    /// Примеры:
    /// - `limit=20, offset=0`  -> `page=1, page_size=20`
    /// - `limit=20, offset=40` -> `page=3, page_size=20`
    pub async fn list_posts(&self, limit: u32, offset: u32) -> BlogClientResult<ListPostsResponse> {
        let mut client = self.connect().await?;
        let page_size = limit.max(1);
        let page = (offset / page_size) + 1;
        let request = tonic::Request::new(pb::ListPostsRequest { page, page_size });

        let response = client
            .list_posts(request)
            .await
            .map_err(BlogClientError::from_grpc_status)?;
        let dto = Self::map_list_posts_response(response.into_inner())?;
        Ok(dto.into())
    }

    async fn connect(
        &self,
    ) -> BlogClientResult<pb::blog_service_client::BlogServiceClient<Channel>> {
        let endpoint =
            if self.endpoint.starts_with("http://") || self.endpoint.starts_with("https://") {
                self.endpoint.clone()
            } else {
                format!("http://{}", self.endpoint)
            };

        let channel = Endpoint::from_shared(endpoint)
            .map_err(|err| {
                BlogClientError::InvalidRequest(format!("invalid grpc endpoint: {err}"))
            })?
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15))
            .connect()
            .await
            .map_err(BlogClientError::GrpcTransport)?;
        Ok(pb::blog_service_client::BlogServiceClient::new(channel))
    }

    fn map_auth_response(proto: pb::AuthResponse) -> BlogClientResult<AuthResponseDto> {
        let user = proto.user.ok_or_else(|| {
            BlogClientError::InvalidRequest("grpc auth response is missing user".to_string())
        })?;
        let user = Self::map_user(user)?;

        Ok(AuthResponseDto {
            access_token: proto.access_token,
            user,
        })
    }

    fn map_user(proto: pb::User) -> BlogClientResult<UserDto> {
        let created_at = proto.created_at.ok_or_else(|| {
            BlogClientError::InvalidRequest("grpc user is missing created_at".to_string())
        })?;
        let created_at = Self::map_timestamp(created_at, "user.created_at")?;

        Ok(UserDto {
            id: proto.id,
            username: proto.username,
            email: proto.email,
            created_at,
        })
    }

    fn map_post(proto: pb::Post) -> BlogClientResult<PostDto> {
        let created_at = proto.created_at.ok_or_else(|| {
            BlogClientError::InvalidRequest("grpc post is missing created_at".to_string())
        })?;
        let updated_at = proto.updated_at.ok_or_else(|| {
            BlogClientError::InvalidRequest("grpc post is missing updated_at".to_string())
        })?;

        Ok(PostDto {
            id: proto.id,
            title: proto.title,
            content: proto.content,
            author_id: proto.author_id,
            created_at: Self::map_timestamp(created_at, "post.created_at")?,
            updated_at: Self::map_timestamp(updated_at, "post.updated_at")?,
        })
    }

    fn map_list_posts_response(
        proto: pb::ListPostsResponse,
    ) -> BlogClientResult<ListPostsResponseDto> {
        let posts = proto
            .posts
            .into_iter()
            .map(Self::map_post)
            .collect::<BlogClientResult<Vec<_>>>()?;

        Ok(ListPostsResponseDto {
            posts,
            limit: proto.page_size,
            offset: proto.page.saturating_sub(1).saturating_mul(proto.page_size),
            total: proto.total,
        })
    }

    fn map_timestamp(
        ts: prost_types::Timestamp,
        field_name: &str,
    ) -> BlogClientResult<chrono::DateTime<chrono::Utc>> {
        chrono::DateTime::from_timestamp(ts.seconds, ts.nanos.max(0) as u32).ok_or_else(|| {
            BlogClientError::InvalidRequest(format!("invalid grpc timestamp in {field_name}"))
        })
    }

    fn attach_bearer_token<T>(
        mut request: tonic::Request<T>,
        token: &str,
    ) -> BlogClientResult<tonic::Request<T>> {
        let token = token.trim();
        if token.is_empty() {
            return Err(BlogClientError::Unauthorized);
        }

        let header = MetadataValue::try_from(format!("Bearer {token}")).map_err(|_| {
            BlogClientError::InvalidRequest("invalid token format for grpc metadata".to_string())
        })?;

        request.metadata_mut().insert("authorization", header);
        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    #[test]
    fn attach_bearer_token_sets_authorization_metadata() {
        let request = tonic::Request::new(());
        let request = GrpcClient::attach_bearer_token(request, "token123")
            .expect("token must be accepted");

        let auth = request
            .metadata()
            .get("authorization")
            .expect("authorization metadata must exist")
            .to_str()
            .expect("metadata must be valid ascii");
        assert_eq!(auth, "Bearer token123");
    }

    #[test]
    fn attach_bearer_token_rejects_empty_token() {
        let request = tonic::Request::new(());
        let err = GrpcClient::attach_bearer_token(request, "   ").expect_err("must fail");
        assert!(matches!(err, BlogClientError::Unauthorized));
    }

    #[test]
    fn map_auth_response_returns_error_without_user() {
        let proto = pb::AuthResponse {
            access_token: "jwt".to_string(),
            user: None,
        };

        let err = GrpcClient::map_auth_response(proto).expect_err("must fail");
        match err {
            BlogClientError::InvalidRequest(msg) => assert!(msg.contains("missing user")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn map_list_posts_response_maps_page_to_offset() {
        let proto = pb::ListPostsResponse {
            posts: vec![],
            page: 3,
            page_size: 20,
            total: 7,
        };

        let dto = GrpcClient::map_list_posts_response(proto).expect("must map");
        assert_eq!(dto.limit, 20);
        assert_eq!(dto.offset, 40);
        assert_eq!(dto.total, 7);
    }

    #[test]
    fn grpc_status_mapping_covers_common_business_errors() {
        let unauth = BlogClientError::from_grpc_status(tonic::Status::new(Code::Unauthenticated, ""));
        assert!(matches!(unauth, BlogClientError::Unauthorized));

        let not_found = BlogClientError::from_grpc_status(tonic::Status::new(Code::NotFound, ""));
        assert!(matches!(not_found, BlogClientError::NotFound));

        let invalid = BlogClientError::from_grpc_status(tonic::Status::new(Code::InvalidArgument, "bad input"));
        match invalid {
            BlogClientError::InvalidRequest(msg) => assert_eq!(msg, "bad input"),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
