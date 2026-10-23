FROM rust:1.99 AS builder

WORKDIR /app

RUN cargo install --locked topcoat-cli

COPY Cargo.toml Cargo.lock ./
COPY build.rs styles.css ./
RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN cargo build --release
RUN rm -rf src target/release/deps/notion2cal* target/release/notion2cal*

COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx
ENV SQLX_OFFLINE=true
RUN cargo build --release

COPY assets ./assets
RUN topcoat asset bundle --release --bin notion2cal --out /app/bundle

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/notion2cal /usr/local/bin/
COPY --from=builder /app/bundle /usr/local/bin/assets
ENV HOST=0.0.0.0
ENV PORT=3001
CMD ["notion2cal"]
