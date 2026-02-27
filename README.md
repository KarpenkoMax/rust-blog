# rust-blog

Учебный blog-проект на Rust в формате workspace:
- `blog-server`: backend (HTTP(axum) + gRPC, Postgres, JWT)
- `blog-client`: библиотека клиента (HTTP/gRPC)
- `blog-cli`: CLI поверх `blog-client`
- `blog-wasm`: фронтенд на Leptos (WASM, HTTP-only)

## Оглавление
- [Quick Start](#quick-start)
- [Архитектура Проекта](#архитектура-проекта)
- [Требования Для Разработки](#требования-для-разработки)
- [Настройка Окружения](#настройка-окружения)
- [Сборка И Запуск Компонентов](#сборка-и-запуск-компонентов)
- [SQLx Offline Mode](#sqlx-offline-mode)
- [Полезные Команды](#полезные-команды)
- [Сценарии Проверки](#сценарии-проверки)
- [Типичные Проблемы](#типичные-проблемы)

## Quick Start
Минимальный путь, чтобы запустить всё локально.

1. Поднимите PostgreSQL:
```bash
docker compose up -d postgres
```

2. Создайте env-файл сервера:
```bash
cp blog-server/.env.example blog-server/.env
```

3. (Опционально) задайте безопасный JWT secret:
```bash
openssl rand -hex 32
```
Скопируйте значение в `blog-server/.env` как `JWT_SECRET`.

4. Прогон миграций:
```bash
set -a; source blog-server/.env; set +a
cargo sqlx migrate run --source blog-server/migrations
```

5. Запуск сервера:
```bash
set -a; source blog-server/.env; set +a
cargo run -p blog-server --bin blog-server
```

6. В другом терминале запустите фронт:
```bash
cd blog-wasm
trunk serve --open --port 3000
```

По умолчанию:
- HTTP API: `http://127.0.0.1:8080`
- gRPC: `http://127.0.0.1:50051`
- WASM UI: `http://127.0.0.1:3000`

## Архитектура Проекта

### Крейты
- `blog-server`
  - `domain`: модели и доменные ошибки
  - `application`: бизнес-логика (`AuthService`, `BlogService`)
  - `data`: репозитории
  - `infrastructure`: DB/JWT/settings/logging
  - `presentation`: HTTP (axum) и gRPC (tonic)
- `blog-client`
  - единый API `BlogClient` с `Transport::Http/Grpc`
  - хранение токена + CRUD/Auth
  - централизованный маппинг ошибок
- `blog-cli`
  - CLI команды `register/login/create/get/update/delete/list`
  - токен-файл `.blog_token`
- `blog-wasm`
  - Leptos CSR UI
  - HTTP-запросы через `gloo-net`
  - localStorage для токена/пользователя

### Как связаны компоненты
- `blog-wasm` и `blog-cli` ходят в `blog-server`.
- `blog-cli` использует `blog-client`.
- `blog-wasm` не использует gRPC и не зависит от `blog-client` (browser HTTP-only).

## Требования Для Разработки
- Rust toolchain (`rustup`, `cargo`)
- Docker + Docker Compose
- `protoc` (для tonic/protobuf codegen)
- `trunk` (для Leptos frontend)
- WASM target:
```bash
rustup target add wasm32-unknown-unknown
```

Рекомендуемые утилиты:
- `sqlx-cli`

Пример установки:
```bash
cargo install trunk
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

## Настройка Окружения

### 1. PostgreSQL
```bash
docker compose up -d postgres
```

### 2. Переменные окружения сервера
Скопируй шаблон:
```bash
cp blog-server/.env.example blog-server/.env
```

Ключевые параметры в `blog-server/.env`:
- `DATABASE_URL=postgres://blog:blog@localhost:5432/blog_db`
- `JWT_SECRET=<не короче 32 символов>`
- `HTTP_ADDR=0.0.0.0:8080`
- `GRPC_ADDR=0.0.0.0:50051`
- `CORS_ORIGINS=http://localhost:3000,http://127.0.0.1:3000`

### 3. Миграции
```bash
set -a; source blog-server/.env; set +a
cargo sqlx migrate run --source blog-server/migrations
```

## Сборка И Запуск Компонентов

### Сборка всего workspace
С живой БД:
```bash
set -a; source blog-server/.env; set +a
cargo build --workspace
```

С offline SQLx:
```bash
SQLX_OFFLINE=true cargo build --workspace
```

### Server
```bash
set -a; source blog-server/.env; set +a
cargo run -p blog-server --bin blog-server
```

### Client library
```bash
cargo test -p blog-client
```

Smoke (нужен запущенный server + DB):
```bash
cargo test -p blog-client -- --ignored
```

### CLI
```bash
cargo run -p blog-cli -- --help
```

### WASM frontend
```bash
cd blog-wasm
trunk serve --open --port 3000
```

`blog-wasm` поддерживает compile-time переменную `WASM_API_BASE_URL`.
Для dev это тоже применимо, потому что `trunk serve` компилирует проект перед запуском.

По умолчанию используется `http://127.0.0.1:8080`.  
Если нужен другой API в dev:
```bash
cd blog-wasm
WASM_API_BASE_URL=http://127.0.0.1:8081 trunk serve --open --port 3000
```

## SQLx Offline Mode
`sqlx::query!`/`query_as!` используют metadata из `.sqlx/`, если выставлен `SQLX_OFFLINE=true`.

### Подготовка snapshot
```bash
set -a; source blog-server/.env; set +a
cargo sqlx prepare --workspace -- --all-targets
```

После этого закоммить `.sqlx/`.

### Проверка offline-сборки
```bash
SQLX_OFFLINE=true cargo check --workspace
SQLX_OFFLINE=true cargo test --workspace --no-run
```

Когда нужно обновлять `.sqlx/`:
- после изменения SQL-запросов в `query!`/`query_as!`
- после изменения миграций
- после изменения схемы таблиц

## Полезные Команды
```bash
# форматирование
cargo fmt --all

# линтер
cargo clippy --workspace --all-targets

# быстрый smoke на wasm-таргете
cargo check -p blog-wasm --target wasm32-unknown-unknown

# помощь по CLI
cargo run -p blog-cli -- --help
```

## Сценарии Проверки

### 1) HTTP через curl
Регистрация:
```bash
curl -sS -X POST http://127.0.0.1:8080/api/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","email":"alice@example.com","password":"password123"}'
```

Логин:
```bash
curl -sS -X POST http://127.0.0.1:8080/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"password123"}'
```

Создать пост (подставь token):
```bash
curl -sS -X POST http://127.0.0.1:8080/api/posts \
  -H "Authorization: Bearer <TOKEN>" \
  -H 'Content-Type: application/json' \
  -d '{"title":"First post","content":"Hello"}'
```

Список постов:
```bash
curl -sS "http://127.0.0.1:8080/api/posts?limit=10&offset=0"
```

### 2) CLI сценарий
HTTP (по умолчанию):
```bash
cargo run -p blog-cli -- register --username "alice" --email "alice@example.com" --password "password123"
cargo run -p blog-cli -- login --username "alice" --password "password123"
cargo run -p blog-cli -- create --title "My post" --content "Hello from CLI"
cargo run -p blog-cli -- list --limit 10 --offset 0
```

gRPC:
```bash
cargo run -p blog-cli -- --grpc login --username "alice" --password "password123"
cargo run -p blog-cli -- --grpc list --limit 10 --offset 0
```

### 3) Браузерный сценарий (WASM)
1. Запустите server и `trunk serve`.
2. Откройте `http://127.0.0.1:3000`.
3. `Register` или `Login`.
4. Создайте пост.
5. Обновите/удалите свой пост.
6. Нажмите F5: auth должен сохраниться (token/user из localStorage).
7. Logout: auth сбрасывается.

## Типичные Проблемы

### `trunk: command not found`
```bash
cargo install trunk
export PATH="$HOME/.cargo/bin:$PATH"
```

### `can't find crate for std/core` для wasm
```bash
rustup target add wasm32-unknown-unknown
```

### SQLx не может подключиться к БД при сборке
- либо поднимите БД и проверь `DATABASE_URL`
- либо используйте offline:
```bash
SQLX_OFFLINE=true cargo check --workspace
```

### Порт занят
```bash
cd blog-wasm
trunk serve --port 3000
```

## Docker Compose Full Setup
Полный стек в Docker запускается через `docker-compose.full.yml` и использует переменные из корневого `.env`.

### 1. Подготовка
Создайте корневой `.env`:
```bash
cp blog-server/.env.example ./.env
```

Затем добавьте в `./.env` недостающие переменные одним блоком:

```env
# --- Compose runtime ports on host ---
COMPOSE_POSTGRES_PORT=5432
COMPOSE_HTTP_PORT=8081
COMPOSE_GRPC_PORT=50052
COMPOSE_WASM_PORT=3000

# --- PostgreSQL container settings ---
POSTGRES_USER=blog
POSTGRES_PASSWORD=blog
POSTGRES_DB=blog_db

# --- blog-server settings ---
# IMPORTANT: inside compose network use service host 'postgres', not localhost
DATABASE_URL=postgres://blog:blog@postgres:5432/blog_db

JWT_SECRET=replace_me_with_long_random_secret_at_least_32_chars
JWT_TTL_SECONDS=3600

HTTP_ADDR=0.0.0.0:8080
GRPC_ADDR=0.0.0.0:50051

CORS_ORIGINS=http://localhost:3000,http://127.0.0.1:3000
RUST_LOG=info,blog_server=debug,sqlx=warn,tower_http=info

HTTP_REQUEST_BODY_LIMIT_BYTES=1048576
HTTP_CONCURRENCY_LIMIT=256
HTTP_REQUEST_TIMEOUT_SECS=10

GRPC_CONCURRENCY_LIMIT=256
GRPC_REQUEST_TIMEOUT_SECS=10
GRPC_MAX_DECODING_MESSAGE_SIZE_BYTES=4194304
GRPC_MAX_ENCODING_MESSAGE_SIZE_BYTES=4194304

# --- blog-wasm build arg ---
WASM_API_BASE_URL=""
```

### 2. Сборка и запуск
```bash
docker compose --env-file .env -f docker-compose.full.yml up -d --build
```

### 3. Проверка
- WASM frontend: `http://127.0.0.1:${COMPOSE_WASM_PORT}`
- HTTP API: `http://127.0.0.1:${COMPOSE_HTTP_PORT}`
- gRPC: `127.0.0.1:${COMPOSE_GRPC_PORT}`

Пример health-check HTTP:
```bash
curl -i "http://127.0.0.1:${COMPOSE_HTTP_PORT}/api/posts?limit=1&offset=0"
```

### 4. Остановка
```bash
docker compose --env-file .env -f docker-compose.full.yml down
```

С удалением volume БД:
```bash
docker compose --env-file .env -f docker-compose.full.yml down -v
```
