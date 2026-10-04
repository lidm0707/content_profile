# content_backend axum server (native binary).
# Build context is the repo root (same as ui.Dockerfile).

FROM rust:1-slim AS build
WORKDIR /app
COPY . .
RUN cargo build --release -p content_backend --bin content_backend

FROM debian:bookworm-slim
# ca-certificates: reqwest (rustls) needs roots for outbound Supabase sync
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/content_backend /usr/local/bin/content_backend
# sqlx::migrate! embeds migrations at compile time; nothing else to copy

ENV HTTP_ADDR=0.0.0.0:8080
EXPOSE 8080
ENTRYPOINT ["content_backend"]
