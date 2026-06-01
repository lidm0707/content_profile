FROM rust:bookworm AS builder

RUN apt-get update && apt-get install -y cmake && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY content_sdk/Cargo.toml content_sdk/Cargo.toml
COPY content_proxy/Cargo.toml content_proxy/Cargo.toml
COPY supabase_client/Cargo.toml supabase_client/Cargo.toml
COPY content_ui/Cargo.toml content_ui/Cargo.toml

RUN mkdir -p content_sdk/src && echo "" > content_sdk/src/lib.rs \
    && mkdir -p content_proxy/src && echo "fn main() {}" > content_proxy/src/main.rs \
    && mkdir -p supabase_client/src && echo "" > supabase_client/src/lib.rs \
    && mkdir -p content_ui/src && echo "fn main() {}" > content_ui/src/main.rs

RUN cargo build --release -p content_proxy 2>/dev/null || true

COPY content_sdk/src/ content_sdk/src/
COPY content_proxy/src/ content_proxy/src/
COPY supabase_client/src/ supabase_client/src/

RUN touch content_proxy/src/main.rs && cargo build --release -p content_proxy

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/content_proxy /usr/local/bin/content_proxy

EXPOSE 6190

CMD ["content_proxy"]
