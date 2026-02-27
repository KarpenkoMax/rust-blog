FROM rust:1.93-slim-trixie AS builder

WORKDIR /app

# Для codegen tonic/protobuf в build.rs
RUN apt-get update \
    && apt-get install -y --no-install-recommends protobuf-compiler libprotobuf-dev pkg-config ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Копируем весь workspace, чтобы корректно собрать crate с workspace dependencies.
COPY . .

# SQLx offline mode, чтобы сборка контейнера не зависела от живой БД.
ENV SQLX_OFFLINE=true

RUN cargo build -p blog-server --release

FROM debian:trixie-slim AS runtime

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/blog-server /usr/local/bin/blog-server

EXPOSE 8080 50051

CMD ["blog-server"]
