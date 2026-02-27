use std::fs;
use std::io;
use std::path::Path;
use std::process;

use anyhow::{Context, Result};
use blog_client::{AuthResponse, BlogClient, BlogClientError, ListPostsResponse, Post, Transport};
use clap::{Parser, Subcommand};

const TOKEN_FILE: &str = ".blog_token";
const DEFAULT_HTTP_SERVER: &str = "http://127.0.0.1:8080";
const DEFAULT_GRPC_SERVER: &str = "http://127.0.0.1:50051";

#[derive(Debug, Parser)]
#[command(name = "blog-cli", version, about = "CLI клиент для blog-server")]
struct Cli {
    /// Использовать gRPC транспорт (по умолчанию HTTP).
    #[arg(long, global = true)]
    grpc: bool,

    /// Адрес сервера (для HTTP или gRPC, в зависимости от --grpc).
    #[arg(long, global = true)]
    server: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Регистрация пользователя.
    Register {
        #[arg(long)]
        username: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
    },
    /// Вход пользователя.
    Login {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: String,
    },
    /// Создание поста (требует токен).
    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
    },
    /// Получение поста по id.
    Get {
        #[arg(long)]
        id: i64,
    },
    /// Обновление поста (требует токен).
    ///
    /// Если `--content` не указан, используется текущее содержимое поста.
    Update {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: Option<String>,
    },
    /// Удаление поста (требует токен).
    Delete {
        #[arg(long)]
        id: i64,
    },
    /// Список постов.
    List {
        #[arg(long, default_value_t = 10)]
        limit: u32,
        #[arg(long, default_value_t = 0)]
        offset: u32,
    },
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Ошибка: {err}");
        process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    let transport = resolve_transport(cli.grpc, cli.server);
    let mut client = BlogClient::new(transport);

    if let Some(token) = load_token().context("не удалось прочитать .blog_token")?
    {
        client.set_token(token);
    }

    match cli.command {
        Command::Register {
            username,
            email,
            password,
        } => {
            let auth = client
                .register(&username, &email, &password)
                .await
                .map_err(map_client_error)?;
            persist_token(&client).context("не удалось сохранить токен")?;
            print_auth("Регистрация успешна", &auth);
        }
        Command::Login { username, password } => {
            let auth = client
                .login(&username, &password)
                .await
                .map_err(map_client_error)?;
            persist_token(&client).context("не удалось сохранить токен")?;
            print_auth("Вход выполнен", &auth);
        }
        Command::Create { title, content } => {
            let post = client
                .create_post(&title, &content)
                .await
                .map_err(map_client_error)?;
            print_post("Пост создан", &post);
        }
        Command::Get { id } => {
            let post = client.get_post(id).await.map_err(map_client_error)?;
            print_post("Пост", &post);
        }
        Command::Update { id, title, content } => {
            // Если пользователь не передал --content, сохраняем текущее содержимое поста.
            let content = match content {
                Some(content) => content,
                None => client.get_post(id).await.map_err(map_client_error)?.content,
            };

            let post = client
                .update_post(id, &title, &content)
                .await
                .map_err(map_client_error)?;
            print_post("Пост обновлён", &post);
        }
        Command::Delete { id } => {
            client.delete_post(id).await.map_err(map_client_error)?;
            println!("Пост удалён: id={id}");
        }
        Command::List { limit, offset } => {
            let list = client
                .list_posts(limit, offset)
                .await
                .map_err(map_client_error)?;
            print_list(&list);
        }
    }

    Ok(())
}

fn resolve_transport(grpc: bool, server: Option<String>) -> Transport {
    let default = if grpc {
        DEFAULT_GRPC_SERVER
    } else {
        DEFAULT_HTTP_SERVER
    };
    let raw = server.unwrap_or_else(|| default.to_string());
    let normalized = normalize_server(raw);

    if grpc {
        Transport::Grpc(normalized)
    } else {
        Transport::Http(normalized)
    }
}

fn normalize_server(server: String) -> String {
    if server.starts_with("http://") || server.starts_with("https://") {
        return server;
    }

    format!("http://{server}")
}

fn parse_token_content(raw: &str) -> Option<String> {
    let token = raw.trim().to_string();
    if token.is_empty() {
        return None;
    }
    Some(token)
}

fn load_token() -> io::Result<Option<String>> {
    if !Path::new(TOKEN_FILE).exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(TOKEN_FILE)?;
    Ok(parse_token_content(&raw))
}

fn persist_token(client: &BlogClient) -> io::Result<()> {
    if let Some(token) = client.get_token() {
        fs::write(TOKEN_FILE, token)?;
    }
    Ok(())
}

fn map_client_error(err: BlogClientError) -> anyhow::Error {
    let message = match err {
        BlogClientError::Unauthorized => {
            "требуется авторизация: выполните `blog-cli login ...` или `blog-cli register ...`"
                .to_string()
        }
        BlogClientError::NotFound => "ресурс не найден".to_string(),
        BlogClientError::InvalidRequest(message) => format!("некорректный запрос: {message}"),
        BlogClientError::Http(err) => format!("ошибка HTTP: {err}"),
        BlogClientError::GrpcStatus(status) => {
            format!(
                "ошибка gRPC: code={:?}, message={}",
                status.code(),
                status.message()
            )
        }
        BlogClientError::GrpcTransport(err) => format!("ошибка gRPC соединения: {err}"),
    };
    anyhow::anyhow!(message)
}

fn print_auth(title: &str, auth: &AuthResponse) {
    println!("{title}");
    println!("token: {}", auth.access_token);
    println!("user:");
    println!("  id: {}", auth.user.id);
    println!("  username: {}", auth.user.username);
    println!("  email: {}", auth.user.email);
    println!("  created_at: {}", auth.user.created_at);
}

fn print_post(title: &str, post: &Post) {
    println!("{title}");
    println!("id: {}", post.id);
    println!("title: {}", post.title);
    println!("content: {}", post.content);
    println!("author_id: {}", post.author_id);
    println!("created_at: {}", post.created_at);
    println!("updated_at: {}", post.updated_at);
}

fn print_list(list: &ListPostsResponse) {
    println!(
        "Постов: {} (limit={}, offset={}, total={})",
        list.posts.len(),
        list.limit,
        list.offset,
        list.total
    );

    for post in &list.posts {
        println!(
            "- [{}] {} (author_id={})",
            post.id, post.title, post.author_id
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_server_keeps_scheme() {
        let s = normalize_server("https://example.com:8080".to_string());
        assert_eq!(s, "https://example.com:8080");
    }

    #[test]
    fn normalize_server_adds_http_scheme() {
        let s = normalize_server("127.0.0.1:50051".to_string());
        assert_eq!(s, "http://127.0.0.1:50051");
    }

    #[test]
    fn resolve_transport_defaults_to_http() {
        let transport = resolve_transport(false, None);
        match transport {
            Transport::Http(url) => assert_eq!(url, DEFAULT_HTTP_SERVER),
            Transport::Grpc(_) => panic!("expected HTTP transport"),
        }
    }

    #[test]
    fn resolve_transport_uses_grpc_when_flag_enabled() {
        let transport = resolve_transport(true, None);
        match transport {
            Transport::Grpc(url) => assert_eq!(url, DEFAULT_GRPC_SERVER),
            Transport::Http(_) => panic!("expected gRPC transport"),
        }
    }

    #[test]
    fn resolve_transport_uses_custom_server() {
        let transport = resolve_transport(false, Some("localhost:9999".to_string()));
        match transport {
            Transport::Http(url) => assert_eq!(url, "http://localhost:9999"),
            Transport::Grpc(_) => panic!("expected HTTP transport"),
        }
    }

    #[test]
    fn parse_token_content_trims_whitespace() {
        let token = parse_token_content("  abc.def.ghi  ");
        assert_eq!(token.as_deref(), Some("abc.def.ghi"));
    }

    #[test]
    fn parse_token_content_rejects_blank() {
        let token = parse_token_content("   ");
        assert!(token.is_none());
    }
}
