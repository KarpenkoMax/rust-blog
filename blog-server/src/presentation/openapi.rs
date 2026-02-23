use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::presentation::handlers::auth::{AuthResponseDto, LoginDto, RegisterDto, UserDto};
use crate::presentation::handlers::posts::{
    CreatePostDto, ListPostsResponseDto, PaginationQuery, PostDto, UpdatePostDto,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::presentation::handlers::auth::register,
        crate::presentation::handlers::auth::login,
        crate::presentation::handlers::posts::list_posts,
        crate::presentation::handlers::posts::get_post,
        crate::presentation::handlers::posts::create_post,
        crate::presentation::handlers::posts::update_post,
        crate::presentation::handlers::posts::delete_post
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
