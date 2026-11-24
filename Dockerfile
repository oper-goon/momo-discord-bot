FROM rust:1.98-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends cmake pkg-config libopus-dev libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
COPY migrations ./migrations
COPY vendor ./vendor
RUN cargo build --locked --release -p bot-app

FROM denoland/deno:bin-2.9.4 AS deno

FROM debian:bookworm-slim
ARG VERSION=development
ARG REVISION=unknown
# First-party licensing; third-party components retain their own terms.
LABEL org.opencontainers.image.title="Momokun" \
      org.opencontainers.image.description="Self-hosted modular Discord bot" \
      org.opencontainers.image.source="https://github.com/oper-goon/momo-discord-bot" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${REVISION}" \
      org.opencontainers.image.licenses="PolyForm-Noncommercial-1.0.0"
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates ffmpeg libssl3 libopus0 python3 python3-venv sqlite3 && rm -rf /var/lib/apt/lists/*
ARG YT_DLP_VERSION=2026.10.26
ARG BGUTIL_PROVIDER_VERSION=2.0.2
RUN python3 -m venv /opt/ytdlp && /opt/ytdlp/bin/pip install --no-cache-dir \
    "yt-dlp[default]==${YT_DLP_VERSION}" \
    "bgutil-ytdlp-pot-provider==${BGUTIL_PROVIDER_VERSION}"
COPY --from=deno /deno /usr/local/bin/deno
RUN useradd --system --uid 10001 --create-home bot && mkdir -p /data/backups /app/config && chown -R bot:bot /data /app
COPY --from=builder /build/target/release/bot-app /app/bot
COPY config/example.toml /app/config/example.toml
COPY LICENSE THIRD_PARTY_NOTICES.md /usr/share/doc/momokun/
COPY licenses/Apache-2.0.txt /usr/share/doc/momokun/licenses/Apache-2.0.txt
COPY licenses/MIT-legacy.txt /usr/share/doc/momokun/licenses/MIT-legacy.txt
COPY vendor/davey/LICENSE /usr/share/doc/momokun/licenses/davey-MIT.txt
COPY vendor/openmls_basic_credential/LICENSE /usr/share/doc/momokun/licenses/openmls-basic-credential-MIT.txt
COPY vendor/openmls_rust_crypto/LICENSE /usr/share/doc/momokun/licenses/openmls-rust-crypto-MIT.txt
COPY vendor/ringbuf/LICENSE-MIT /usr/share/doc/momokun/licenses/ringbuf-LICENSE-MIT
ENV PATH="/opt/ytdlp/bin:${PATH}" DATABASE_URL=sqlite:///data/bot.db DENO_NO_UPDATE_CHECK=1 DENO_NO_PROMPT=1
USER bot
WORKDIR /app
VOLUME ["/data"]
ENTRYPOINT ["/app/bot"]

