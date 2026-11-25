FROM rust:1.98.1-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends cmake pkg-config libopus-dev libssl-dev python3 && rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
COPY migrations ./migrations
COPY vendor ./vendor
COPY scripts/collect_rust_notices.py ./scripts/collect_rust_notices.py
COPY scripts/collect_toolchain_notices.py ./scripts/collect_toolchain_notices.py
COPY LICENSE ./LICENSE
COPY licenses ./licenses
ARG REVISION=unknown
RUN cargo build --locked --release -p bot-app
RUN python3 scripts/collect_toolchain_notices.py --expected-release 1.98.1 --output /build-toolchain-notices
RUN target=$(rustc -vV | sed -n 's/^host: //p') && \
    cargo metadata --locked --format-version 1 --filter-platform "$target" > /tmp/cargo-metadata.json && \
    python3 scripts/collect_rust_notices.py --metadata /tmp/cargo-metadata.json \
      --lockfile Cargo.lock --cargo-home "$CARGO_HOME" --output /build-notices \
      --target "$target" --revision "$REVISION"

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
ARG YT_DLP_EJS_VERSION=0.8.0
ARG BGUTIL_PROVIDER_VERSION=2.0.2
RUN python3 -m venv /opt/ytdlp && /opt/ytdlp/bin/pip install --no-cache-dir \
    "yt-dlp[default]==${YT_DLP_VERSION}" \
    "yt-dlp-ejs==${YT_DLP_EJS_VERSION}" \
    "bgutil-ytdlp-pot-provider==${BGUTIL_PROVIDER_VERSION}"
COPY --from=deno /deno /usr/local/bin/deno
RUN useradd --system --uid 10001 --create-home bot && mkdir -p /data/backups /app/config && chown -R bot:bot /data /app
COPY --from=builder /build/target/release/bot-app /app/bot
COPY config/example.toml /app/config/example.toml
COPY LICENSE THIRD_PARTY_NOTICES.md /usr/share/doc/momokun/
COPY licenses/Apache-2.0.txt /usr/share/doc/momokun/licenses/Apache-2.0.txt
COPY licenses/MIT-legacy.txt /usr/share/doc/momokun/licenses/MIT-legacy.txt
COPY licenses/upstream/denoland--deno/14eea3160ae5834476aa3b9d317b8d41d991b982/LICENSE.md /usr/share/doc/momokun/licenses/Deno-2.9.4-MIT.txt
COPY licenses/upstream/brainicism--bgutil-ytdlp-pot-provider/26475e9d3665b972c4aeb83b4dbaafcf64b88438/LICENSE /usr/share/doc/momokun/licenses/BgUtils-2.0.2-GPL-3.0.txt
COPY licenses/runtime-supplements.json /usr/share/doc/momokun/licenses/runtime-supplements.json
COPY licenses/upstream/npm /usr/share/doc/momokun/licenses/npm
COPY licenses/ejs-dependencies.json /usr/share/doc/momokun/licenses/ejs-dependencies.json
COPY licenses/deno-2.9.4-notices.tar.gz licenses/deno-2.9.4-notices.tar.gz.sha256 /usr/share/doc/momokun/licenses/
RUN cd /usr/share/doc/momokun/licenses && \
    sha256sum -c deno-2.9.4-notices.tar.gz.sha256 && \
    mkdir deno-2.9.4 && tar --no-same-owner -xzf deno-2.9.4-notices.tar.gz -C deno-2.9.4 && \
    rm deno-2.9.4-notices.tar.gz
COPY licenses/upstream/brainicism--bgutil-ytdlp-pot-provider/26475e9d3665b972c4aeb83b4dbaafcf64b88438/source.tar.gz /usr/share/doc/momokun/licenses/BgUtils-2.0.2-source.tar.gz
COPY scripts/verify_bgutils_source.py /usr/local/libexec/momokun/verify_bgutils_source.py
RUN /opt/ytdlp/bin/python /usr/local/libexec/momokun/verify_bgutils_source.py \
    --archive /usr/share/doc/momokun/licenses/BgUtils-2.0.2-source.tar.gz \
    --sha256 a8144eab6a4f38f320063688531f461afe18c0a58be2dd3152b780f4851e954d \
    --installed /opt/ytdlp --version "${BGUTIL_PROVIDER_VERSION}" \
    --output /usr/share/doc/momokun/licenses/BgUtils-source-verification.json
COPY licenses/MPL-2.0.txt /usr/share/doc/momokun/bundle/licenses/MPL-2.0.txt
COPY --from=builder /build-notices /usr/share/doc/momokun/bundle
COPY --from=builder /build-toolchain-notices /usr/share/doc/momokun/toolchain-notices
COPY vendor/davey/LICENSE /usr/share/doc/momokun/licenses/davey-MIT.txt
COPY vendor/openmls_basic_credential/LICENSE /usr/share/doc/momokun/licenses/openmls-basic-credential-MIT.txt
COPY vendor/openmls_rust_crypto/LICENSE /usr/share/doc/momokun/licenses/openmls-rust-crypto-MIT.txt
COPY vendor/ringbuf/LICENSE-MIT /usr/share/doc/momokun/licenses/ringbuf-LICENSE-MIT
COPY scripts/collect_image_notices.py /usr/local/libexec/momokun/collect_image_notices.py
RUN python3 /usr/local/libexec/momokun/collect_image_notices.py --output /usr/share/doc/momokun/image-notices
ENV PATH="/opt/ytdlp/bin:${PATH}" DATABASE_URL=sqlite:///data/bot.db DENO_NO_UPDATE_CHECK=1 DENO_NO_PROMPT=1
USER bot
WORKDIR /app
VOLUME ["/data"]
ENTRYPOINT ["/app/bot"]

