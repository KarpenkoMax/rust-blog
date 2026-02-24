use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::presentation::http::handlers::auth::{AuthResponseDto, LoginDto, RegisterDto, UserDto};
use crate::presentation::http::handlers::posts::{
    CreatePostDto, ListPostsResponseDto, PaginationQuery, PostDto, UpdatePostDto,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::presentation::http::handlers::auth::register,
        crate::presentation::http::handlers::auth::login,
        crate::presentation::http::handlers::posts::list_posts,
        crate::presentation::http::handlers::posts::get_post,
        crate::presentation::http::handlers::posts::create_post,
        crate::presentation::http::handlers::posts::update_post,
        crate::presentation::http::handlers::posts::delete_post
    ),
    components(
        schemas(
            RegisterDto,
            LoginDto,
            AuthResponseDto,
            UserDto,
            CreatePostDto,
            UpdatePostDto,
            PaginationQuery,
            PostDto,
            ListPostsResponseDto
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "posts", description = "Post endpoints")
    ),
    modifiers(&SecurityAddon)
)]
pub(crate) struct ApiDoc;

pub(crate) struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let mut components = openapi.components.take().unwrap_or_default();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
        openapi.components = Some(components);
    }
}
