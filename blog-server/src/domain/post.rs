use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::error::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Post {
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) author_id: i64,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CreatePostRequest {
    pub(crate) title: String,
    pub(crate) content: String,
}

impl CreatePostRequest {
    pub(crate) fn validate(self) -> Result<Self, DomainError> {
        Ok(Self {
            title: normalize_title(&self.title)?,
            content: normalize_content(&self.content)?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct UpdatePostRequest {
    pub(crate) title: String,
    pub(crate) content: String,
}

impl UpdatePostRequest {
    pub(crate) fn validate(self) -> Result<Self, DomainError> {
        Ok(Self {
            title: normalize_title(&self.title)?,
            content: normalize_content(&self.content)?,
        })
    }
}

impl Post {
    pub(crate) fn new(
        id: i64,
        title: impl Into<String>,
        content: impl Into<String>,
        author_id: i64,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        validate_positive_i64("id", id)?;
        validate_positive_i64("author_id", author_id)?;
        let title = normalize_title(&title.into())?;
        let content = normalize_content(&content.into())?;

        if updated_at < created_at {
            return Err(DomainError::Validation {
                field: "updated_at",
                message: "must be >= created_at",
            });
        }

        Ok(Self {
            id,
            title,
            content,
            author_id,
            created_at,
            updated_at,
        })
    }
}

fn validate_positive_i64(field: &'static str, value: i64) -> Result<(), DomainError> {
    if value <= 0 {
        return Err(DomainError::Validation {
            field,
            message: "must be > 0",
        });
    }
    Ok(())
}

fn normalize_title(title: &str) -> Result<String, DomainError> {
    let title = title.trim();
    if title.is_empty() || title.len() > 255 {
        return Err(DomainError::Validation {
            field: "title",
            message: "must be 1..255 chars",
        });
    }
    Ok(title.to_string())
}

fn normalize_content(content: &str) -> Result<String, DomainError> {
    let content = content.trim();
    if content.is_empty() {
        return Err(DomainError::Validation {
            field: "content",
            message: "must not be empty",
        });
    }
    Ok(content.to_string())
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::{CreatePostRequest, DomainError, Post, UpdatePostRequest};

    #[test]
    fn create_post_request_validate_rejects_empty_title() {
        let req = CreatePostRequest {
            title: "   ".to_string(),
            content: "valid content".to_string(),
        };

        let err = req.validate().expect_err("title must be rejected");
        assert_validation_field(err, "title");
    }

    #[test]
    fn update_post_request_validate_rejects_empty_content() {
        let req = UpdatePostRequest {
            title: "valid title".to_string(),
            content: "   ".to_string(),
        };

        let err = req.validate().expect_err("content must be rejected");
        assert_validation_field(err, "content");
    }

    #[test]
    fn create_post_request_validate_normalizes_fields() {
        let req = CreatePostRequest {
            title: "  title  ".to_string(),
            content: "  content  ".to_string(),
        };

        let validated = req.validate().expect("must validate");
        assert_eq!(validated.title, "title");
        assert_eq!(validated.content, "content");
    }

    #[test]
    fn post_new_normalizes_and_builds_post() {
        let created_at = Utc::now();
        let updated_at = created_at + Duration::seconds(1);

        let post = Post::new(1, "  Title  ", "  Content  ", 10, created_at, updated_at)
            .expect("post should be created");

        assert_eq!(post.id, 1);
        assert_eq!(post.author_id, 10);
        assert_eq!(post.title, "Title");
        assert_eq!(post.content, "Content");
    }

    #[test]
    fn post_new_rejects_non_positive_author_id() {
        let now = Utc::now();
        let err =
            Post::new(1, "Title", "Content", 0, now, now).expect_err("author_id must be > 0");
        assert_validation_field(err, "author_id");
    }

    #[test]
    fn post_new_rejects_updated_before_created() {
        let updated_at = Utc::now();
        let created_at = updated_at + Duration::seconds(1);

        let err = Post::new(1, "Title", "Content", 10, created_at, updated_at)
            .expect_err("updated_at < created_at must fail");
        assert_validation_field(err, "updated_at");
    }

    fn assert_validation_field(err: DomainError, expected_field: &'static str) {
        match err {
            DomainError::Validation { field, .. } => assert_eq!(field, expected_field),
            _ => panic!("expected DomainError::Validation"),
        }
    }
}
