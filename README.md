# Momo Discord Bot

Momo is a self-hosted, modular Discord bot written in Rust. It provides
per-server module activation, deterministic command policy, moderation audit
cases, persistent reminders, utility commands, and isolated music players with
saved playlists.

This repository currently identifies as `0.2.0`. The implementation follows
the checked-in [technical specification](docs/specification.md). Live Discord
acceptance testing is still required before calling it a production `1.0.0`
release; see [release status](docs/release-status.md).

## First-time setup

1. Create a Discord application and bot in the Developer Portal. Copy the bot
   token. Enable Message Content only if prefix commands are enabled.
2. Copy `config/example.toml` to `config/config.toml` and add a private test
   guild ID under `commands.development_guilds`.
3. Set `DISCORD_TOKEN` in your environment. Never put it in the TOML file.
   Set `TORBOX_API_KEY` as well when using the optional TorBox module.
4. Start with Docker Compose:

   ```powershell
   $env:DISCORD_TOKEN = 'your-token'
   docker compose up --build -d
   ```

   The SQLite database lives in the named `momo-data` volume. The deployment
   includes pinned yt-dlp and Deno releases, yt-dlp's matching EJS challenge
   scripts, an internal PO-token provider for YouTube playback requests, and
   FFmpeg for bounded video/audio merges.

On a Linux deployment host, keep the token in a private `.env` file and make
the bind-mounted configuration readable by the container's unprivileged user:

```sh
umask 077
printf 'DISCORD_TOKEN=%s\n' "$DISCORD_TOKEN" > .env
chmod 600 .env
chmod 644 config/config.toml
docker compose up --build -d
```

The TOML configuration contains no secrets. Docker Compose reads `.env`
automatically; do not commit that file.

For local Windows development, `scripts/dev.ps1` uses the workspace-local Rust,
LLVM, CMake, and Ninja tools installed during project creation:

```powershell
Copy-Item config/example.toml config/config.toml
$env:DISCORD_TOKEN = 'your-token'
./scripts/dev.ps1 run -p bot-app -- --config config/config.toml
```

Use `commands.mode = "guild"` while developing. Change to `global` for a
production release. Command synchronization uses a content hash and runs only
when the command tree changes; pass `--sync-commands` to force it.

## Commands and modules

- Core: `/help`, `/about`, `/module`, `/permissions`, `/config`, `/prefix`
- General: `/ping`, `/avatar`, `/user info`, `/server info`, `/role info`
- Administration: `/announce`, `/slowmode`, `/nick`, `/role add|remove`
- Moderation: kick, ban, timeout, purge, warnings, and auditable cases
- Utility: choose, polls, persisted reminders, and timestamps
- Music: play, transport controls, queue editing, repeat, shuffle, buttons, and
  guild-local playlists
- TorBox: cached-magnet file links and subscription-gated video streams
- Download: reupload public videos from an explicit URL, replied message, or
  recent channel history
- Image Search: `/image` searches DuckDuckGo Images with strict SafeSearch,
  falls back to Bing when DuckDuckGo blocks the request, and provides owner-only
  Previous/Next buttons for cycling through cached results

Core is always enabled. General and Utility are enabled by default. Admin,
Moderation, Music, and TorBox require `/module enable <module>` in each server.
Download is also disabled by default because it consumes host and Discord
bandwidth. Slash downloads honor Discord's reported attachment limit; prefix
downloads use the server boost tier limit because message commands do not carry
an attachment-limit field.
Image Search is disabled by default because DuckDuckGo does not publish a
supported image-search API. It requires no account or API key; repeated
identical queries are cached briefly to reduce upstream traffic. Each search
keeps up to ten results available for five minutes of button navigation.
TorBox also requires `TORBOX_API_KEY`. Restrict it to trusted roles with
`/permissions` before enabling it because users share the operator's TorBox
account and rate limits.

## Operations

Run all quality gates before merging:

```powershell
./scripts/dev.ps1 fmt --all -- --check
./scripts/dev.ps1 clippy --locked --workspace --all-targets --all-features -- -D warnings
./scripts/dev.ps1 test --locked --workspace
./scripts/dev.ps1 deny check
./scripts/dev.ps1 run --locked -p bot-app -- --check
```

Back up a running database with SQLite's online backup command:

```sh
sqlite3 /data/bot.db ".backup '/data/backups/bot-$(date +%Y%m%d-%H%M%S).db'"
```

Restore only while the bot is stopped, keeping the original database until the
restored file passes `bot-app --check`. Schema migrations run before Discord
connects. Review [security policy](docs/security.md) before dependency changes.

## Revision policy

- `main` stays releasable and is protected by CI.
- Versions follow semantic versioning in the workspace `Cargo.toml`.
- User-visible changes go in `CHANGELOG.md`.
- Database changes are forward-only numbered files in `migrations/`.
- Dependency updates arrive weekly through Dependabot and must pass cargo-deny.
- Release tags use `vMAJOR.MINOR.PATCH` after the acceptance checklist passes.

The repository contains no Discord token, local configuration, database, or
signed media URLs. Those paths are ignored by Git.
