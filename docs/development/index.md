# Develop and contribute

Start with [architecture](architecture.md), `AGENTS.md`, and the relevant crate.
Read `docs/security.md` before dependency, network, media, or credential changes.
The technical specification is an engineering contract, not a list of completed
features. The source registry and feature guides describe implemented behavior.

## Set up a development environment

Install stable Rust (the workspace requires at least Rust 1.88), Git, CMake,
pkg-config and the platform's OpenSSL/Opus development libraries. Linux CI uses:

```sh
sudo apt-get update
sudo apt-get install -y cmake pkg-config libopus-dev libssl-dev
cargo build --locked -p bot-app
DATABASE_URL=sqlite::memory: cargo run --locked -p bot-app -- --check
```

For playback/download testing also install the versions of yt-dlp, Deno,
FFmpeg/FFprobe and the PO-token integration specified by the Dockerfile. The
[native guide](../self-hosting/native.md) explains runtime setup. Local Windows
developers with the existing workspace tool bundle can use `scripts/dev.ps1`;
the bundle is not downloaded or shipped by that script. A new contributor can
use a normal supported Rust installation or Docker.

Use a private Discord application and test server. Copy the example configuration,
set `commands.mode = "guild"` and your test server ID, and provide your own token
in the environment. The native executable does not read `.env` automatically.
Offline validation and reference generation require no real token.

## Add a command

1. Put behavior in the relevant feature crate. Follow existing Poise patterns.
2. Declare slash and prefix support when safe. Sensitive arguments and private
   interactions should remain slash-only; explain exceptions in the feature guide.
3. Use shared reply, guild, effective-permission, channel-permission and hierarchy
   helpers. Suppress mentions. Handle missing cache data and invalid input safely.
4. Register it in the module's command list, or declare an extension for a parent
   owned elsewhere. Paths must remain unique across the composed registry.
5. Add meaningful tests for policy/state transitions, regenerate the references,
   and update feature examples, limitations, and release notes.

## Add a module

Create a `module-*` crate exporting `module() -> RegisteredModule`. Choose a stable
lowercase ID, default activation, required services, minimal intents and event
subscriptions. Wire root workspace dependencies, `bot-app/Cargo.toml` and
`bot-app::compose`; update composition tests. Keep mutable long-lived guild state
under one owner and use supervised cancellable work. Add its feature page and
sidebar entry in the same change.

## Configuration and storage

Configuration uses Serde defaults and rejects unknown fields. Add options to the
typed configuration, validation, example file, generated reference and operator
instructions together. Secrets belong in environment variables. See the
[configuration reference](../reference/configuration.md) for precedence.

Add a monotonically numbered migration; never edit an applied migration. Extend
typed database methods and test reopen/retry semantics. A migration checksum
mismatch requires investigation and backup, not editing migration history or
deleting the database.

## Validate and debug

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace
DATABASE_URL=sqlite::memory: cargo run --locked -p bot-app -- --check
cargo deny check
```

On the bundled Windows environment, prefix Cargo arguments with
`./scripts/dev.ps1 --%` (for example, `./scripts/dev.ps1 --% test --locked --workspace`).
The stop-parsing token preserves the `--` separator and flags. Focused crate tests
are useful during development; run all relevant gates before integration. CI
also builds the Docker image. A green unit test does not establish live Discord
rendering, permission, command-sync or voice acceptance.

Use normal privacy-safe logs first. Debug-level third-party logging can reveal
URLs and account data; inspect privately and redact reports. Reproduce storage
issues against disposable databases and provider issues against mocked services.
Never use production credentials in CI. Preserve `#![forbid(unsafe_code)]`, existing
rate limits, timeouts, queue ceilings and mention suppression.

See [contributing](../project/contributing.md), [documentation maintenance](documentation.md)
and [release procedures](releases.md) before opening a pull request.
