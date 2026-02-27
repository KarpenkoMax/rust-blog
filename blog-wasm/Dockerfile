FROM rust:1.93-slim-trixie AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install trunk
RUN rustup target add wasm32-unknown-unknown

COPY . .

# По умолчанию фронт собран для прямого доступа к локальному серверу.
# Для compose ниже подставляется /api и запросы идут через nginx proxy_pass.
ARG WASM_API_BASE_URL=http://127.0.0.1:8080
ENV WASM_API_BASE_URL=${WASM_API_BASE_URL}

WORKDIR /app/blog-wasm
RUN trunk build --release

FROM nginx:1.29.5-alpine AS runtime

COPY blog-wasm/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/blog-wasm/dist/ /usr/share/nginx/html/

EXPOSE 80
