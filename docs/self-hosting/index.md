# Self-host with Docker Compose

Docker Compose is the recommended method. It runs the bot and a private YouTube PO-token provider. The provider has no published host port and receives no Discord credentials. SQLite storage is created and migrated automatically before the bot connects to Discord. You do not need PostgreSQL, Redis, a reverse proxy, or a web server.

!!! warning "Release validation"
    The files are prepared for source builds. Docker is unavailable in the implementation environment, so this revised installation has not been built or started here. Prebuilt public images have not been published.

## Quick start

Install Git, Docker, and Docker Compose first. Create a Discord bot using [Getting started](../getting-started/index.md).

=== "Linux / macOS"

    ```sh
    git clone https://github.com/oper-goon/momo-discord-bot.git
    cd momo-discord-bot
    umask 077
    cp .env.example .env
    cp config/example.toml config/config.toml
    chmod 600 .env
    chmod 644 config/config.toml
    ```

=== "PowerShell"

    ```powershell
    git clone https://github.com/oper-goon/momo-discord-bot.git
    Set-Location momo-discord-bot
    Copy-Item .env.example .env
    Copy-Item config/example.toml config/config.toml
    ```

Edit `.env` with a text editor and replace the `DISCORD_TOKEN` placeholder with your token. On shared machines restrict access to this file. Compose passes the private `.env` values into the bot container, including optional provider settings. Do not commit or share it. Quote values according to Compose environment-file syntax when they contain special characters; a single-quoted value preserves literal `$` characters.

Edit `config/config.toml`. The supplied example deliberately uses guild registration with an empty server list, which fails validation until you choose one of these:

```toml
[commands]
mode = "global"
development_guilds = []
```

For private testing, keep `mode = "guild"` and set `development_guilds = [123456789012345678]` using your own server ID. Enable Discord Developer Mode to copy server IDs. Configure `discord.owners` only with accounts you trust to operate the bot. Leave optional integrations disabled unless needed.

Build and validate:

```sh
docker compose build
docker compose run --rm --no-deps bot --config /app/config/config.toml --check
docker compose up -d
docker compose ps
docker compose logs --tail 100 bot
```

The offline `--check` validates configuration, applies migrations, and assembles services and commands. It writes to the database but does not validate Discord credentials or external-service availability. It is not a running-container health check. Startup success is the `Bot ready` log followed by a working `/ping` in Discord. Avoid sharing unredacted logs or `docker compose config` output, which can expose environment values.

An optional `python scripts/setup.py` requires Python 3.11 or newer. It checks Docker/Compose availability, creates missing local files without overwriting existing files, and reports missing/placeholder tokens and invalid command-registration settings without printing private values. It checks `.env` directly; it does not evaluate variable substitutions. Edit the indicated files and rerun until those checks pass. It never starts services or authenticates credentials; the bot's offline `--check` remains the complete configuration validator.

## What persists and what connects

The bot runs as UID 10001. The named `momo-data` volume holds `/data/bot.db`, its SQLite WAL files, and backups. The TOML file is mounted read-only; credentials stay outside the image. Compose forces `DATABASE_URL=sqlite:///data/bot.db` to keep data in the volume. No host ports are published. Allow outbound DNS, HTTPS for Discord and enabled providers, Discord gateway WebSocket connections, and outbound UDP needed by Discord voice. The PO-token provider communicates only over the Compose network.

The image includes FFmpeg/FFprobe, Python, yt-dlp, Deno and SQLite. Optional integrations require no extra containers. The existing provider container supports YouTube handling; upstream site changes can still break extraction. Container logs rotate at 10 MB with three files; the provider's logging is disabled to avoid logging temporary tokens. The bot receives a 20-second shutdown grace period for its existing graceful shutdown.

Read [operations](operations.md) for updates and backups, [native installation](native.md) for development, and [image publishing status](images.md) for prebuilt-image plans.

