# Configuration reference

Generated from serialized `AppConfig::default()` and reviewed source descriptions. The bot accepts an optional TOML file through `--config PATH`; it does **not** automatically load `config/config.toml` or `.env`. Docker Compose passes the selected file and environment. Environment overrides apply after TOML, then validation runs. Unknown TOML fields are rejected.

All TOML fields are optional: omitted fields take these runtime defaults. Connecting to Discord requires `DISCORD_TOKEN`. Optional API services require only their own credentials. Keep secrets in your private environment or Compose `.env`, never TOML or Git. None of the TOML fields is a credential; database paths, owner IDs, guild IDs and hostnames can still reveal private operational information.

Use `docker compose run --rm --no-deps bot --config /app/config/config.toml --check` for the source Compose installation, or the equivalent documented container configuration path. `--check` opens SQLite, applies migrations, constructs services and validates the registry, but does not connect to Discord. Back up existing data before validating an upgrade.

## `[commands]`

| Option | Runtime default / example | Purpose and accepted values |
| --- | --- | --- |
| `development_guilds` | `[]` | Positive Discord server IDs. Required and nonempty when mode is `guild`; ignored for global synchronization. |
| `mode` | `"none"` | Command registration: `none`, `guild`, or `global`. `none` does not publish commands. The example TOML uses `guild`, unlike this runtime default. |

## `[database]`

| Option | Runtime default / example | Purpose and accepted values |
| --- | --- | --- |
| `max_connections` | `5` | SQLite connection pool size: integer 1–10. |
| `url` | `"sqlite://data/bot.db"` | SQLite URL beginning `sqlite:`. Custom parent directories must already exist. Keep the database private. |

## `[discord]`

| Option | Runtime default / example | Purpose and accepted values |
| --- | --- | --- |
| `administrator_bypass` | `true` | Allow Discord administrators to bypass custom permission rules; boolean. Discord action and hierarchy checks still apply. |
| `default_prefix` | `"!"` | Initial server prefix; 1–16 characters and not blank. Server administrators can override it. |
| `owners` | `[]` | Bot owner user IDs (decimal integers); empty means no configured owners. Does not skip permission checks. |
| `prefix_commands` | `false` | Enable message commands; boolean. Requires Message Content intent in the Developer Portal. |

## `[download]`

| Option | Runtime default / example | Purpose and accepted values |
| --- | --- | --- |
| `absolute_max_bytes` | `524288000` | Absolute byte ceiling: integer 1048576–2147483648. Discord's upload limit can be lower. |
| `allowed_domains` | `["youtube.com", "youtu.be", "tiktok.com", "x.com", "twitter.com", "reddit.com", "v.redd.it", "instagram.com"]` | Nonempty lowercase hostname array; no slashes, leading dots or trailing dots. This controls supported website resolution, not arbitrary private-network access. |
| `max_duration_seconds` | `900` | Maximum accepted media duration in seconds: 1–86400. |
| `scan_messages` | `25` | Recent messages searched for omitted URLs: integer 1–100. |
| `timeout_seconds` | `180` | Download/conversion timeout in seconds: 10–900. |
| `yt_dlp_path` | `"yt-dlp"` | Path or executable name for yt-dlp used for downloading supported websites. |

## `[logging]`

| Option | Runtime default / example | Purpose and accepted values |
| --- | --- | --- |
| `format` | `"pretty"` | Log format: `pretty` or `json`. |

## `[music]`

| Option | Runtime default / example | Purpose and accepted values |
| --- | --- | --- |
| `default_volume_percent` | `100` | Initial volume percentage: integer 0–200. |
| `easynews_enabled` | `false` | Enable optional Easynews audio provider; boolean. Requires subscription credentials in environment. |
| `empty_channel_disconnect_seconds` | `120` | Initial empty voice channel disconnect timeout in seconds: 10–86400. |
| `idle_disconnect_seconds` | `300` | Initial server idle disconnect timeout in seconds: 10–86400. |
| `match_min_confidence` | `0.85` | Easynews/TorBox metadata match confidence: finite number 0.5–1.0. |
| `max_playlist_import` | `100` | Maximum playlist import count: 1 through max_queue_length. |
| `max_queue_length` | `500` | Process queue ceiling: integer 1–5000; also initial per-server queue limit. |
| `provider_cooldown_seconds` | `120` | Circuit breaker cooldown in seconds after provider failures: 1–3600. |
| `provider_priority` | `["youtube", "easynews", "torbox"]` | Nonempty ordered array of unique provider IDs: `youtube`, `easynews`, `torbox`. Disabled providers are skipped; public direct audio is also supported. |
| `provider_timeout_seconds` | `30` | Per-provider attempt timeout in seconds: 1–120. |
| `resolver_timeout_seconds` | `30` | yt-dlp resolution timeout in seconds: 1–120. |
| `torbox_cached_only` | `true` | Must remain `true`; uncached TorBox music requests are deliberately unsupported. |
| `torbox_enabled` | `false` | Enable optional cached TorBox music provider; boolean. Requires API key in environment. |
| `yt_dlp_path` | `"yt-dlp"` | Path or executable name for yt-dlp. No automatic installation by the native binary. |

## `[social_embeds]`

| Option | Runtime default / example | Purpose and accepted values |
| --- | --- | --- |
| `instagram_providers` | `["kkinstagram.com", "oginstagram.com", "d.oginstagram.com", "vxinstagram.com"]` | Ordered array of 1–5 approved hosts: oginstagram.com, d.oginstagram.com, vxinstagram.com, kkinstagram.com, instagram7.com, zzinstagram.com. |
| `probe_timeout_seconds` | `6` | Embed provider probe timeout in seconds: 1–15. |

## Environment variables

These values are read by the bot or its logging/terminal libraries. Compose reads `.env` for interpolation; a native launch must export variables explicitly. Boolean overrides accept Rust `true`/`false` values. Never paste real secrets into diagnostics or issue reports.

| Name | Purpose / module | Required | Default | Accepted value or safe example | Secret |
| --- | --- | --- | --- | --- | --- |
| `DISCORD_TOKEN` | Discord bot login token | Required to connect; not required for offline export or --check | `None` | `Bot token from Developer Portal` | Yes |
| `DATABASE_URL` | Override database.url | Optional | `TOML/default URL` | `sqlite://data/bot.db` | No |
| `YT_DLP_PATH` | Override both music/download yt-dlp paths | Optional | `TOML/default path` | `/usr/local/bin/yt-dlp` | No |
| `EASYNEWS_ENABLED` | Override music.easynews_enabled | Optional | `false` | `true or false` | No |
| `EASYNEWS_USERNAME` | Easynews subscription username | Required only for enabled Easynews music | `None` | `your-username` | Yes |
| `EASYNEWS_PASSWORD` | Easynews subscription password | Required only for enabled Easynews music | `None` | `your-password` | Yes |
| `TORBOX_ENABLED` | Override music.torbox_enabled; separate from the TorBox command module | Optional | `false` | `true or false` | No |
| `TORBOX_CACHED_ONLY` | Override music.torbox_cached_only | Optional; false is rejected | `true` | `true` | No |
| `TORBOX_API_KEY` | TorBox API credential for commands and optional music provider | Required only when using TorBox | `None` | `your-api-key` | Yes |
| `MUSIC_PROVIDER_PRIORITY` | Override music.provider_priority | Optional | `youtube,easynews,torbox` | `Comma-separated unique provider IDs` | No |
| `MUSIC_PROVIDER_TIMEOUT` | Override music.provider_timeout_seconds | Optional | `30` | `Integer 1–120` | No |
| `MUSIC_PROVIDER_COOLDOWN` | Override music.provider_cooldown_seconds | Optional | `120` | `Integer 1–3600` | No |
| `MUSIC_MATCH_MIN_CONFIDENCE` | Override music.match_min_confidence | Optional | `0.85` | `Number 0.5–1.0` | No |
| `TMDB_READ_ACCESS_TOKEN` | TMDB API Read Access Token for movie/TV results; takes precedence over API key | Required for movie/TV if API key absent | `None` | `your-read-access-token` | Yes |
| `TMDB_API_KEY` | TMDB v3 API key fallback | Required for movie/TV if read access token absent | `None` | `your-api-key` | Yes |
| `RUST_LOG` | Tracing EnvFilter log directives | Optional | `info,serenity=warn,songbird=warn,sqlx=warn` | `info,serenity=warn` | No |
| `NO_COLOR` | Disable terminal banner color when set | Optional | `Unset` | `Any set value` | No |
| `TERM` | Terminal type; dumb disables banner color | Optional; normally supplied by terminal | `Host terminal` | `dumb` | No |

## Per-server settings

These are stored in SQLite rather than environment variables. Manage Server is required for `/config show`, `/config set <key> <value>`, and `/config reset`. Server values survive restarts; reset restores process defaults. `/config set` supports:

| Key | Accepted value | Initial value |
| --- | --- | --- |
| `volume` | Integer 0–200 | `music.default_volume_percent` |
| `queue_limit` | Integer 1 through process max_queue_length | `music.max_queue_length` |
| `idle_seconds` | Integer 10–86400 | `music.idle_disconnect_seconds` |
| `empty_seconds` | Integer 10–86400 | `music.empty_channel_disconnect_seconds` |
| `dj_role` | Decimal Discord role ID, or `none` | `none` |
| `announce` | `true` or `false` | `true` |

Module enablement, module allowed-channel lists, prefix overrides and custom permission rules are also per-server database settings. Use `/module`, `/prefix` and `/permissions` rather than editing SQLite manually.

## Command-line switches

| Switch | Behavior |
| --- | --- |
| `--config PATH` | Load an explicit TOML file. |
| `--check` | Validate configuration, migrations, services and registry without Discord. |
| `--sync-commands` | Force configured guild/global command registration on startup. |
| `--export-command-reference PATH` | Write default config, modules and composed command metadata to JSON before loading configuration or database. No credentials required. |

## Optional dependency requirements

SQLite initializes and migrates automatically. Music website resolution needs yt-dlp; voice decoding/playback uses Songbird. Download conversions need FFmpeg and ffprobe. Source/native installs must provide these executables for the corresponding feature. Containers include these tools. AniList and image search need no API key. Movie/TV requires a TMDB credential. Easynews and TorBox are optional paid services; disabling them leaves other features usable. See the [feature guides](../features/index.md).
