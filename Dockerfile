FROM rust:1.98-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends cmake pkg-config libopus-dev libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
COPY migrations ./migrations
COPY vendor ./vendor
RUN cargo build --locked --release -p bot-app

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 libopus0 python3 python3-venv sqlite3 && rm -rf /var/lib/apt/lists/*
ARG YT_DLP_VERSION=2026.8.19
RUN python3 -m venv /opt/ytdlp && /opt/ytdlp/bin/pip install --no-cache-dir "yt-dlp==${YT_DLP_VERSION}"
RUN useradd --system --uid 10001 --create-home bot && mkdir -p /data/backups /app/config && chown -R bot:bot /data /app
COPY --from=builder /build/target/release/bot-app /app/bot
COPY config/example.toml /app/config/example.toml
ENV PATH="/opt/ytdlp/bin:${PATH}" DATABASE_URL=sqlite:///data/bot.db
USER bot
WORKDIR /app
VOLUME ["/data"]
ENTRYPOINT ["/app/bot"]

