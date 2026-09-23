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
   token. Enable the Message Content intent for Social Embeds (and for optional
   prefix commands).
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

The complete syntax and description for every command is in the
[command reference](docs/commands.md).

| Module ID | Enabled by default | Purpose |
| --- | --- | --- |
| `core` | Required | Help, module activation, permissions, prefixes, and server configuration |
| `general` | Yes | Ping, avatars, and user, server, and role information |
| `utility` | Yes | Choices, polls, recurring reminders, private bookmarks, and timestamps |
| `admin` | No | Announcements, slowmode, nicknames, and role assignment |
| `moderation` | No | Kick, ban, timeout, purge, warnings, and auditable cases |
| `music` | No | Voice playback, queues, controls, repeat, shuffle, and saved playlists |
| `download` | No | Bounded downloads, media inspection, conversion, and Discord reuploads |
| `image` | No | Safe image search with interactive result navigation |
| `torbox` | No | Cached-magnet file links and subscription-gated video streams |
| `social-embeds` | No | Webhook-based embeds for X, Instagram, TikTok, Reddit, Threads, and Bluesky |

Core is always enabled. Optional modules can be changed per server with
`/module enable <module>` and `/module disable <module>`. Download is disabled
by default because it consumes host and Discord bandwidth. Slash downloads
honor Discord's reported attachment limit; prefix downloads use the server
boost tier limit because message commands do not carry an attachment-limit
field.
Right-click or long-press a message and open **Apps** for `Bookmark Message`,
`Remind Me in 1 Hour`, `Download Media`, and `Repair Social Embed`. Each action
still follows its module's enablement and permission policy; bookmark results
and reminder confirmations are private.
Image Search is disabled by default because DuckDuckGo does not publish a
supported image-search API. It requires no account or API key; repeated
identical queries are cached briefly to reduce upstream traffic. Each search
keeps up to ten results available for five minutes of button navigation.
Social Embeds is disabled by default. It requires the Message Content intent
and the View Channel, Manage Messages, Manage Webhooks, and Send Messages
permissions. Enable it per server with `/module enable social-embeds`. Momo
labels webhook names with `via Momo`, disables mentions in reposted content,
and deletes the original only after the corrected webhook post succeeds. Each
repost starts with links to the untouched original URL, corrected provider URL,
and the content creator's platform profile when the source exposes that author.
Messages containing uploads, stickers, or polls are left untouched. Add the
standalone word `fxignore`, wrap a link in `< >`, or put it in inline/fenced
code to opt out. Instagram providers are tried in configured order and must
return usable Open Graph metadata before Momo replaces the original message.
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
