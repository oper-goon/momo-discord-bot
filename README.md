# Momokun

A modular Discord bot you host yourself. Music queues and playlists, moderation
cases, reminders, private bookmarks, media tools, and per-server feature controls.
Written in Rust; internal crate and executable names still use Momo.

[![CI](https://github.com/oper-goon/momo-discord-bot/actions/workflows/ci.yml/badge.svg)](https://github.com/oper-goon/momo-discord-bot/actions/workflows/ci.yml)
[![Documentation](https://github.com/oper-goon/momo-discord-bot/actions/workflows/docs.yml/badge.svg)](https://github.com/oper-goon/momo-discord-bot/actions/workflows/docs.yml)

**Preparing 1.0.** The latest source and documentation are public. The owner has
accepted the bot for 1.0 after months of use and reviewed the recorded dependency
attribution exceptions. The workspace version remains 0.9.0 until the release is
tagged. Prebuilt container images are not published.

[Documentation sources](docs/index.md) ·
[Documentation website](https://oper-goon.github.io/momo-discord-bot/) ·
[Commands](docs/reference/commands.md) · [Troubleshooting](docs/troubleshooting.md)

## Self-host with Docker Compose

Install Git, Docker and its Compose plugin. Create your Discord application and
bot, enable Message Content Intent, and invite it with `bot` and
`applications.commands` scopes. Follow [Getting started](docs/getting-started/index.md)
for permissions and account setup.

```sh
git clone https://github.com/oper-goon/momo-discord-bot.git
cd momo-discord-bot
cp .env.example .env
cp config/example.toml config/config.toml
```

Edit `.env` to replace the Discord token placeholder. Keep it private (`chmod 600
.env` on Linux/macOS). Edit `config/config.toml`: use `commands.mode = "global"`,
or set your private server ID in `commands.development_guilds` for guild mode.
The unchanged example intentionally requires this choice.

```sh
docker compose build
docker compose run --rm --no-deps bot --config /app/config/config.toml --check
docker compose up -d
docker compose logs --tail 100 bot
```

Confirm `Bot ready`, then try `/ping` and `/help`. SQLite initializes automatically
in the `momo-data` volume. Never run `docker compose down -v` against data you need.
Only Discord credentials are required for core features. Public prebuilt images
are pending approval and validation; source builds are the current method.

[Detailed Docker guide and PowerShell steps](docs/self-hosting/index.md) ·
[Backups and updates](docs/self-hosting/operations.md) ·
[Configuration](docs/reference/configuration.md)

## Features

- Core: per-server modules, command policies, channel restrictions and prefixes.
- General and Utility: profiles/server info, polls, choices, timestamps,
  persistent reminders and private bookmarks. Enabled by default with Core.
- Optional: administration, moderation, music, downloads/conversions, anime/film/TV
  lookup, image search, TorBox cached files/streams and social embed repair.
- Music: YouTube, SoundCloud, Bandcamp and public HTTP audio; optional paid
  Easynews and TorBox accounts. yt-dlp, Deno and FFmpeg ship in the Docker image.
- Media lookup: credential-free AniList; TMDB movie/TV lookup requires a token/key.
  Image search needs no key but relies on unsupported upstream interfaces.

Enable optional modules with `/module enable <id>`. Credentials do not enable
modules automatically. See [all feature guides](docs/features/index.md) for
limitations, subscription requirements, permissions and privacy behavior.

## Contribute and maintain

Read [CONTRIBUTING.md](CONTRIBUTING.md), [architecture](docs/development/architecture.md)
and [development instructions](docs/development/index.md). Documentation builds
locally with Python 3.12 and `requirements-docs.txt`; see the
[preview and validation guide](docs/development/documentation.md).

Report vulnerabilities privately using [SECURITY.md](SECURITY.md). Keep tokens,
private configurations, logs, database backups and signed media URLs out of issues.

## License and identity

This release candidate uses [PolyForm Noncommercial 1.0.0](LICENSE) for
first-party code, following the owner's confirmation of licensing rights.
It is source-available; commercial use requires separate permission.
See the [licensing review](docs/project/license.md) and [third-party notices](THIRD_PARTY_NOTICES.md).
Earlier MIT grants and all third-party licenses remain valid. No CLA has been
imposed. Branding derivatives are integrated and owner-approved; see
[branding guidelines](docs/project/branding.md).
