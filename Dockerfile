FROM rust:1.99 AS builder
WORKDIR /app

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo install --locked topcoat-cli

COPY Cargo.toml Cargo.lock build.rs styles.css ./
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx
COPY assets ./assets

ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked \
 && cp target/release/notion2cal /app/notion2cal \
 && topcoat asset bundle --release --bin notion2cal --out /app/bundle


FROM debian:trixie-slim
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/* \
  && useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin app


COPY --from=builder /app/notion2cal /usr/local/bin/
COPY --from=builder /app/bundle /usr/local/bin/assets

ENV HOST=0.0.0.0 PORT=3001
USER 10001
EXPOSE 3001
CMD ["notion2cal"]
