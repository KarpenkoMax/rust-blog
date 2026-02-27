use std::time::{SystemTime, UNIX_EPOCH};

use blog_client::{BlogClient, BlogClientError, Transport};

fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after unix epoch")
        .as_nanos();
    format!("{nanos}")
}

#[tokio::test]
#[ignore = "requires running HTTP server and database"]
async fn http_smoke_flow() {
    let base_url =
        std::env::var("BLOG_HTTP_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
    let mut client = BlogClient::new(Transport::Http(base_url));

    let suffix = unique_suffix();
    let username = format!("http_user_{suffix}");
    let email = format!("http_{suffix}@example.com");
    let password = "password123";

    let register = client
        .register(&username, &email, password)
        .await
        .expect("register must succeed");
    assert!(!register.access_token.is_empty());
    assert_eq!(register.user.username, username);
    assert!(client.get_token().is_some());

    let login = client
        .login(&username, password)
        .await
        .expect("login must succeed");
    assert!(!login.access_token.is_empty());
    assert_eq!(login.user.username, username);
    assert!(client.get_token().is_some());

    let created = client
        .create_post("http title", "http content")
        .await
        .expect("create_post must succeed");
    assert_eq!(created.title, "http title");

    let fetched = client
        .get_post(created.id)
        .await
        .expect("get_post must succeed");
    assert_eq!(fetched.id, created.id);

    let listed = client
        .list_posts(20, 0)
        .await
        .expect("list_posts must succeed");
    assert!(listed.posts.iter().any(|post| post.id == created.id));

    let updated = client
        .update_post(created.id, "http title updated", "http content updated")
        .await
        .expect("update_post must succeed");
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.title, "http title updated");

    client
        .delete_post(created.id)
        .await
        .expect("delete_post must succeed");

    let after_delete = client.get_post(created.id).await;
    assert!(matches!(after_delete, Err(BlogClientError::NotFound)));
}

#[tokio::test]
#[ignore = "requires running gRPC server and database"]
async fn grpc_smoke_flow() {
    let endpoint = std::env::var("BLOG_GRPC_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());
    let mut client = BlogClient::new(Transport::Grpc(endpoint));

    let suffix = unique_suffix();
    let username = format!("grpc_user_{suffix}");
    let email = format!("grpc_{suffix}@example.com");
    let password = "password123";

    let register = client
        .register(&username, &email, password)
        .await
        .expect("register must succeed");
    assert!(!register.access_token.is_empty());
    assert_eq!(register.user.username, username);
    assert!(client.get_token().is_some());

    let login = client
        .login(&username, password)
        .await
        .expect("login must succeed");
    assert!(!login.access_token.is_empty());
    assert_eq!(login.user.username, username);
    assert!(client.get_token().is_some());

    let created = client
        .create_post("grpc title", "grpc content")
        .await
        .expect("create_post must succeed");
    assert_eq!(created.title, "grpc title");

    let fetched = client
        .get_post(created.id)
        .await
        .expect("get_post must succeed");
    assert_eq!(fetched.id, created.id);

    let listed = client
        .list_posts(20, 0)
        .await
        .expect("list_posts must succeed");
    assert!(listed.posts.iter().any(|post| post.id == created.id));

    let updated = client
        .update_post(created.id, "grpc title updated", "grpc content updated")
        .await
        .expect("update_post must succeed");
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.title, "grpc title updated");

    client
        .delete_post(created.id)
        .await
        .expect("delete_post must succeed");

    let after_delete = client.get_post(created.id).await;
    assert!(matches!(after_delete, Err(BlogClientError::NotFound)));
}
