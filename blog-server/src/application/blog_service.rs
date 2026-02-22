use crate::data::post_repository::{NewPost, Pagination, PostPatch, PostRepository};
use crate::domain::error::DomainError;
use crate::domain::post::{CreatePostRequest, Post, UpdatePostRequest};

#[derive(Debug, Clone)]
pub(crate) struct ListPostsResult {
    pub(crate) posts: Vec<Post>,
    pub(crate) page: u32,
    pub(crate) page_size: u32,
    pub(crate) total: i64,
}

pub(crate) struct BlogService<R: PostRepository> {
    repo: R,
}

impl<R: PostRepository> BlogService<R> {
    pub(crate) fn new(repo: R) -> Self {
        Self { repo }
    }

    pub(crate) async fn create_post(
        &self,
        author_id: i64,
        req: CreatePostRequest,
    ) -> Result<Post, DomainError> {
        let req = req.validate()?;

        let new_post = NewPost {
            title: req.title,
            content: req.content,
            author_id,
        };
        self.repo.create_post(new_post).await
    }

    pub(crate) async fn get_post(&self, id: i64) -> Result<Post, DomainError> {
        self.repo
            .get_post(id)
            .await?
            .ok_or(DomainError::NotFound(format!("post id: {id}").into()))
    }

    pub(crate) async fn update_post(
        &self,
        actor_user_id: i64,
        post_id: i64,
        req: UpdatePostRequest,
    ) -> Result<Post, DomainError> {
        let req = req.validate()?;
        let patch = PostPatch {
            title: req.title,
            content: req.content,
        };
        self.repo
            .update_post_owned(post_id, actor_user_id, patch)
            .await?
            .ok_or(DomainError::NotFound(format!("post id: {post_id}").into()))
    }

    pub(crate) async fn delete_post(
        &self,
        actor_user_id: i64,
        post_id: i64,
    ) -> Result<(), DomainError> {
        let original_post = self
            .repo
            .get_post(post_id)
            .await?
            .ok_or(DomainError::NotFound(format!("post id: {post_id}").into()))?;

        if original_post.author_id != actor_user_id {
            return Err(DomainError::Forbidden);
        }

        let deleted = self.repo.delete_post(post_id).await?;
        if !deleted {
            return Err(DomainError::NotFound(format!("post id: {post_id}")));
        }
        Ok(())
    }

    pub(crate) async fn list_posts(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<ListPostsResult, DomainError> {
        let pagination = Pagination { page, page_size };
        let posts = self.repo.list_posts(pagination).await?;
        let total = self.repo.total_posts().await?;

        Ok(ListPostsResult {
            posts,
            page,
            page_size,
            total,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use chrono::Utc;

    use super::BlogService;
    use crate::data::post_repository::{NewPost, Pagination, PostPatch, PostRepository};
    use crate::domain::error::DomainError;
    use crate::domain::post::{CreatePostRequest, Post, UpdatePostRequest};

    #[derive(Clone)]
    struct FakePostRepo {
        created_input: Arc<Mutex<Option<NewPost>>>,
        post_for_get: Arc<Mutex<Option<Post>>>,
        update_owned_result: Arc<Mutex<Option<Post>>>,
        update_owned_call: Arc<Mutex<Option<(i64, i64, PostPatch)>>>,
        delete_result: Arc<Mutex<bool>>,
        list_result: Arc<Mutex<Vec<Post>>>,
        total_result: Arc<Mutex<i64>>,
    }

    impl FakePostRepo {
        fn new() -> Self {
            Self {
                created_input: Arc::new(Mutex::new(None)),
                post_for_get: Arc::new(Mutex::new(None)),
                update_owned_result: Arc::new(Mutex::new(None)),
                update_owned_call: Arc::new(Mutex::new(None)),
                delete_result: Arc::new(Mutex::new(true)),
                list_result: Arc::new(Mutex::new(Vec::new())),
                total_result: Arc::new(Mutex::new(0)),
            }
        }
    }

    #[async_trait]
    impl PostRepository for FakePostRepo {
        async fn create_post(&self, input: NewPost) -> Result<Post, DomainError> {
            *self
                .created_input
                .lock()
                .expect("created_input mutex poisoned") = Some(input.clone());
            Ok(sample_post(
                1,
                &input.title,
                &input.content,
                input.author_id,
            ))
        }

        async fn get_post(&self, _id: i64) -> Result<Option<Post>, DomainError> {
            Ok(self
                .post_for_get
                .lock()
                .expect("post_for_get mutex poisoned")
                .clone())
        }

        async fn update_post_owned(
            &self,
            post_id: i64,
            owner_id: i64,
            patch: PostPatch,
        ) -> Result<Option<Post>, DomainError> {
            *self
                .update_owned_call
                .lock()
                .expect("update_owned_call mutex poisoned") = Some((post_id, owner_id, patch));
            Ok(self
                .update_owned_result
                .lock()
                .expect("update_owned_result mutex poisoned")
                .clone())
        }

        async fn delete_post(&self, _id: i64) -> Result<bool, DomainError> {
            Ok(*self
                .delete_result
                .lock()
                .expect("delete_result mutex poisoned"))
        }

        async fn list_posts(&self, _pagination: Pagination) -> Result<Vec<Post>, DomainError> {
            Ok(self
                .list_result
                .lock()
                .expect("list_result mutex poisoned")
                .clone())
        }

        async fn total_posts(&self) -> Result<i64, DomainError> {
            Ok(*self
                .total_result
                .lock()
                .expect("total_result mutex poisoned"))
        }
    }

    #[tokio::test]
    async fn create_post_normalizes_request_before_repo_call() {
        let repo = FakePostRepo::new();
        let service = BlogService::new(repo.clone());

        let req = CreatePostRequest {
            title: "  title  ".to_string(),
            content: "  content  ".to_string(),
        };

        let created = service
            .create_post(10, req)
            .await
            .expect("create_post must succeed");

        assert_eq!(created.title, "title");
        assert_eq!(created.content, "content");

        let input = repo
            .created_input
            .lock()
            .expect("created_input mutex poisoned")
            .clone()
            .expect("repo input must be captured");
        assert_eq!(input.title, "title");
        assert_eq!(input.content, "content");
        assert_eq!(input.author_id, 10);
    }

    #[tokio::test]
    async fn get_post_returns_not_found_when_missing() {
        let repo = FakePostRepo::new();
        let service = BlogService::new(repo);

        let err = service
            .get_post(42)
            .await
            .expect_err("post must be missing");
        assert!(matches!(err, DomainError::NotFound(_)));
    }

    #[tokio::test]
    async fn update_post_uses_update_post_owned_contract() {
        let repo = FakePostRepo::new();
        *repo
            .update_owned_result
            .lock()
            .expect("update_owned_result mutex poisoned") = Some(sample_post(7, "new", "body", 10));

        let service = BlogService::new(repo.clone());
        let req = UpdatePostRequest {
            title: "  new  ".to_string(),
            content: "  body  ".to_string(),
        };

        let updated = service
            .update_post(10, 7, req)
            .await
            .expect("update must succeed");
        assert_eq!(updated.id, 7);

        let call = repo
            .update_owned_call
            .lock()
            .expect("update_owned_call mutex poisoned")
            .clone()
            .expect("update call must be captured");
        assert_eq!(call.0, 7);
        assert_eq!(call.1, 10);
        assert_eq!(call.2.title, "new");
        assert_eq!(call.2.content, "body");
    }

    #[tokio::test]
    async fn delete_post_returns_forbidden_for_non_owner() {
        let repo = FakePostRepo::new();
        *repo
            .post_for_get
            .lock()
            .expect("post_for_get mutex poisoned") = Some(sample_post(7, "title", "body", 99));

        let service = BlogService::new(repo);
        let err = service
            .delete_post(10, 7)
            .await
            .expect_err("must be forbidden");
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn list_posts_returns_posts_and_total() {
        let repo = FakePostRepo::new();
        *repo.list_result.lock().expect("list_result mutex poisoned") =
            vec![sample_post(1, "a", "b", 10)];
        *repo
            .total_result
            .lock()
            .expect("total_result mutex poisoned") = 1;

        let service = BlogService::new(repo);
        let result = service
            .list_posts(1, 10)
            .await
            .expect("list_posts must succeed");

        assert_eq!(result.page, 1);
        assert_eq!(result.page_size, 10);
        assert_eq!(result.total, 1);
        assert_eq!(result.posts.len(), 1);
    }

    fn sample_post(id: i64, title: &str, content: &str, author_id: i64) -> Post {
        Post::new(
            id,
            title.to_string(),
            content.to_string(),
            author_id,
            Utc::now(),
            Utc::now(),
        )
        .expect("sample post must be valid")
    }
}
