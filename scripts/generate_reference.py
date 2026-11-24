"""Render checked-in reference pages from the offline, composed Poise registry.

Update: cargo run --locked -p bot-app -- --export-command-reference docs/reference/commands.json
        python scripts/generate_reference.py
CI:     python scripts/generate_reference.py --registry /tmp/commands.json --check
Only Python's standard library is required. No credentials or bot connection.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
SNAPSHOT = ROOT / "docs/reference/commands.json"

# Every entry is reviewed against config.rs. Missing/new fields fail generation.
CONFIG = {
    "discord.owners": "Bot owner user IDs (decimal integers); empty means no configured owners. Does not skip permission checks.",
    "discord.default_prefix": "Initial server prefix; 1–16 characters and not blank. Server administrators can override it.",
    "discord.prefix_commands": "Enable message commands; boolean. Requires Message Content intent in the Developer Portal.",
    "discord.administrator_bypass": "Allow Discord administrators to bypass custom permission rules; boolean. Discord action and hierarchy checks still apply.",
    "commands.mode": "Command registration: `none`, `guild`, or `global`. `none` does not publish commands. The example TOML uses `guild`, unlike this runtime default.",
    "commands.development_guilds": "Positive Discord server IDs. Required and nonempty when mode is `guild`; ignored for global synchronization.",
    "database.url": "SQLite URL beginning `sqlite:`. Custom parent directories must already exist. Keep the database private.",
    "database.max_connections": "SQLite connection pool size: integer 1–10.",
    "music.easynews_enabled": "Enable optional Easynews audio provider; boolean. Requires subscription credentials in environment.",
    "music.torbox_enabled": "Enable optional cached TorBox music provider; boolean. Requires API key in environment.",
    "music.torbox_cached_only": "Must remain `true`; uncached TorBox music requests are deliberately unsupported.",
    "music.provider_priority": "Nonempty ordered array of unique provider IDs: `youtube`, `easynews`, `torbox`. Disabled providers are skipped; public direct audio is also supported.",
    "music.provider_timeout_seconds": "Per-provider attempt timeout in seconds: 1–120.",
    "music.provider_cooldown_seconds": "Circuit breaker cooldown in seconds after provider failures: 1–3600.",
    "music.match_min_confidence": "Easynews/TorBox metadata match confidence: finite number 0.5–1.0.",
    "music.yt_dlp_path": "Path or executable name for yt-dlp. No automatic installation by the native binary.",
    "music.resolver_timeout_seconds": "yt-dlp resolution timeout in seconds: 1–120.",
    "music.idle_disconnect_seconds": "Initial server idle disconnect timeout in seconds: 10–86400.",
    "music.empty_channel_disconnect_seconds": "Initial empty voice channel disconnect timeout in seconds: 10–86400.",
    "music.default_volume_percent": "Initial volume percentage: integer 0–200.",
    "music.max_queue_length": "Process queue ceiling: integer 1–5000; also initial per-server queue limit.",
    "music.max_playlist_import": "Maximum playlist import count: 1 through max_queue_length.",
    "download.yt_dlp_path": "Path or executable name for yt-dlp used for downloading supported websites.",
    "download.scan_messages": "Recent messages searched for omitted URLs: integer 1–100.",
    "download.timeout_seconds": "Download/conversion timeout in seconds: 10–900.",
    "download.max_duration_seconds": "Maximum accepted media duration in seconds: 1–86400.",
    "download.absolute_max_bytes": "Absolute byte ceiling: integer 1048576–2147483648. Discord's upload limit can be lower.",
    "download.allowed_domains": "Nonempty lowercase hostname array; no slashes, leading dots or trailing dots. This controls supported website resolution, not arbitrary private-network access.",
    "social_embeds.probe_timeout_seconds": "Embed provider probe timeout in seconds: 1–15.",
    "social_embeds.instagram_providers": "Ordered array of 1–5 approved hosts: oginstagram.com, d.oginstagram.com, vxinstagram.com, kkinstagram.com, instagram7.com, zzinstagram.com.",
    "logging.format": "Log format: `pretty` or `json`.",
}

# Name: purpose, requirement, default, accepted/example, sensitive
ENV = {
    "DISCORD_TOKEN": ("Discord bot login token", "Required to connect; not required for offline export or --check", "None", "Bot token from Developer Portal", True),
    "DATABASE_URL": ("Override database.url", "Optional", "TOML/default URL", "sqlite://data/bot.db", False),
    "YT_DLP_PATH": ("Override both music/download yt-dlp paths", "Optional", "TOML/default path", "/usr/local/bin/yt-dlp", False),
    "EASYNEWS_ENABLED": ("Override music.easynews_enabled", "Optional", "false", "true or false", False),
    "EASYNEWS_USERNAME": ("Easynews subscription username", "Required only for enabled Easynews music", "None", "your-username", True),
    "EASYNEWS_PASSWORD": ("Easynews subscription password", "Required only for enabled Easynews music", "None", "your-password", True),
    "TORBOX_ENABLED": ("Override music.torbox_enabled; separate from the TorBox command module", "Optional", "false", "true or false", False),
    "TORBOX_CACHED_ONLY": ("Override music.torbox_cached_only", "Optional; false is rejected", "true", "true", False),
    "TORBOX_API_KEY": ("TorBox API credential for commands and optional music provider", "Required only when using TorBox", "None", "your-api-key", True),
    "MUSIC_PROVIDER_PRIORITY": ("Override music.provider_priority", "Optional", "youtube,easynews,torbox", "Comma-separated unique provider IDs", False),
    "MUSIC_PROVIDER_TIMEOUT": ("Override music.provider_timeout_seconds", "Optional", "30", "Integer 1–120", False),
    "MUSIC_PROVIDER_COOLDOWN": ("Override music.provider_cooldown_seconds", "Optional", "120", "Integer 1–3600", False),
    "MUSIC_MATCH_MIN_CONFIDENCE": ("Override music.match_min_confidence", "Optional", "0.85", "Number 0.5–1.0", False),
    "TMDB_READ_ACCESS_TOKEN": ("TMDB API Read Access Token for movie/TV results; takes precedence over API key", "Required for movie/TV if API key absent", "None", "your-read-access-token", True),
    "TMDB_API_KEY": ("TMDB v3 API key fallback", "Required for movie/TV if read access token absent", "None", "your-api-key", True),
    "RUST_LOG": ("Tracing EnvFilter log directives", "Optional", "info,serenity=warn,songbird=warn,sqlx=warn", "info,serenity=warn", False),
    "NO_COLOR": ("Disable terminal banner color when set", "Optional", "Unset", "Any set value", False),
    "TERM": ("Terminal type; dumb disables banner color", "Optional; normally supplied by terminal", "Host terminal", "dumb", False),
}

ARG_HELP = {
    "user": "Discord user/member (unban accepts a decimal user ID)", "owner": "Playlist owner (Discord user)", "channel": "Discord channel", "role": "Discord role",
    "module": "Stable module ID, for example music", "command": "Command path, for example queue move", "query": "Search words or title",
    "url": "Public supported media URL", "magnet": "Private magnet URI", "file_number": "One-based file number",
    "reason": "Reason recorded with the action", "duration": "Human duration, for example 10m", "interval": "Repeat duration, for example 1h",
    "message": "Message text", "question": "Poll question", "options": "Choices separated by |", "count": "Number of messages",
    "seconds": "Duration or playback position in seconds", "percent": "Volume percentage", "position": "One-based queue/playlist position",
    "from": "One-based source position", "to": "One-based destination position", "name": "Playlist name", "id": "Record ID shown by its list command",
    "note": "Private bookmark note", "nickname": "Replacement nickname", "delete_days": "Days of message history to delete, 0–7",
    "target": "Command or module policy target", "effect": "Permission effect", "subject": "Permission subject", "value": "Setting value",
    "key": "Supported setting key", "number": "Moderation case number", "unix_seconds": "Unix timestamp in seconds", "mode": "Repeat mode",
    "input": "Supported website URL, public direct audio URL, or search words", "why": "Audit reason, 1–400 characters",
}

OPTIONAL = {
    ("help", "command"): "List commands when omitted.", ("avatar", "user"): "You.", ("user.info", "user"): "You.",
    ("nick", "nickname"): "Clear nickname.", ("ban", "delete_days"): "0.",
    ("permissions.list", "target"): "List all targets.", ("permissions.explain", "user"): "You.",
    ("reminder.edit", "message"): "Keep existing message.", ("purge", "user"): "All authors.",
    ("permissions.set", "channel"): "Rule applies across channels.", ("permissions.reset", "channel"): "Remove matching rule without channel restriction.",
}
NOTES = {
    "help": "Results are filtered by your effective access; pass a command path for details.",
    "module.channels.remove": "Removing the last allowed channel restores access in all channels.",
    "module.channels.reset": "Clear the module's channel allowlist, restoring access in all channels.",
    "permissions.set": "Target is a stable module ID or dot-separated command path (for example queue.move). Subject accepts everyone, user:ID, role:ID or channel:ID. A channel argument limits the rule's scope.",
    "permissions.reset": "Remove the exact matching target, subject and optional channel rule.",
    "config.set": "Keys are volume, queue_limit, idle_seconds, empty_seconds, dj_role and announce. Accepted values are in the configuration reference.",
    "prefix.enable": "Enables the server preference; the process must also have prefix_commands enabled and Message Content intent available.",
    "role.add": "Actor and bot must outrank the target member and role; managed roles cannot be assigned. This changes the member's roles.",
    "role.remove": "Actor and bot must outrank the target member and role; managed roles cannot be removed.",
    "nick": "Actor and bot hierarchy checks apply. Omit nickname to clear it.",
    "kick": "Checks hierarchy, allocates an auditable case, then attempts the Discord action. Only use on a member you intend to remove.",
    "ban": "Checks hierarchy and records an auditable case. Optional delete_days is 0–7; omitting it preserves message history.",
    "timeout": "Checks hierarchy; maximum duration is 28 days. A case records the outcome.",
    "untimeout": "Checks hierarchy and records removal of the timeout as a moderation case.",
    "warn": "Records a warning in SQLite after hierarchy checks; does not kick, ban or timeout the user.",
    "purge": "Deletes up to 100 eligible recent messages. Discord cannot bulk-delete messages older than 14 days; optional user filters by author.",
    "choose": "Accepts 2–25 pipe-separated options, at most 1800 bytes overall.",
    "poll.create": "Creates a native Discord poll lasting 24 hours. Accepts 2–10 nonempty choices and a question up to 300 characters.",
    "remind": "Durations use a number and s, m, h or d (for example 10m); maximum 365 days. Delivery is to the saved channel and can mention you, even when confirmation is private.",
    "reminder.repeat": "Recurring reminder interval is persisted. Disabling Utility pauses server delivery; the worker checks due reminders every 15 seconds.",
    "reminder.edit": "Only your own reminder can be edited; omitted message preserves its current text.",
    "reminder.snooze": "Only your own reminder can be rescheduled; the message text is preserved.",
    "reminder.cancel": "Deletes only a reminder owned by you.",
    "bookmark.list": "Returns your ten most recent private bookmarks.",
    "bookmark.search": "Searches your own saved excerpts and notes; results remain private.",
    "bookmark.note": "Updates only a bookmark you own.",
    "bookmark.delete": "Deletes only a bookmark you own.",
    "play": "Joins your voice channel and queues resolved media. Queue, resolver timeout, provider concurrency and configured voice/DJ policy apply. Service availability and website restrictions can prevent playback.",
    "join": "Join your voice channel; the bot needs View Channel, Connect and Speak.",
    "leave": "Stops playback, clears pending tracks and disconnects the server player.",
    "stop": "Stops playback and clears the queue while remaining in voice.",
    "seek": "Only works for a seekable current track.",
    "queue.show": "Shows the current track and at most 15 pending tracks.",
    "queue.clear": "Clears pending tracks without stopping the current track.",
    "playlist.play": "Imports the saved playlist subject to playlist import and queue limits. Owner defaults to you.",
    "playlist.delete": "Deleting another owner's playlist requires the implementation's server administration check.",
    "playlist.move": "Reordering another owner's playlist requires the server administration check.",
    "playlist.remove": "Editing another owner's playlist requires the server administration check.",
    "torbox.download": "Requires TORBOX_API_KEY and cached content. Omit file_number for a private picker (up to 250 safe files, 25 per page, five-minute expiry). ZIP preparation requires all files to be safe; generated links are temporary and sensitive.",
    "torbox.stream": "Requires cached safe video and operator credentials. Returns a temporary HLS link; stream creation can require a TorBox Pro subscription. The picker belongs to you and expires after five minutes.",
    "download": "When URL is omitted, inspect a replied message and then recent channel history. Download size, duration, supported-host and Discord upload limits apply.",
    "media.gif": "Converts the first ten seconds to GIF, subject to the shared download/upload limits.",
    "image": "Uses safe image search and cached Previous/Next buttons. Provider results and availability are external; safe search is not a guarantee of suitable content.",
    "repair_embed": "Repairs supported social URLs on the selected message. Repairing another user's message requires Manage Messages.",
}
EXAMPLES = {"module.enable": "music", "module.disable": "music", "module.info": "music", "play": "lofi hip hop", "image": "black cat", "anime": "Cowboy Bebop", "movie": "The Matrix", "tv": "Breaking Bad", "animesearch": "Naruto", "moviesearch": "Dune", "tvsearch": "The Office", "choose": "tea | coffee", "remind": "10m Take a break", "reminder.repeat": "1h Stretch", "poll.create": "Lunch? pizza | sushi", "config.set": "volume 80", "prefix.set": "!", "timestamp": "1700000000", "repeat": "off"}


def esc(value):
    return str(value).replace("|", "\\|").replace("\n", " ")


def source_functions():
    found = {}
    for file in sorted((ROOT / "crates").glob("*/src/**/*.rs")):
        text = file.read_text(encoding="utf-8")
        for match in re.finditer(r"(?:pub )?async fn (\w+)\s*\(", text):
            found.setdefault(match[1], []).append((file, text.count("\n", 0, match.start()) + 1))
    return found


def command_page(data):
    lines = ["# Command reference", "", "Generated from the actual composed command registry, including cross-module extensions and message actions. Do not edit this page directly; edit `scripts/generate_reference.py` or command source and regenerate.", "", "`<argument>` is required; `[argument]` is optional. Slash commands use Discord's labeled argument fields. Prefix examples use the default `!`; prefix commands require both configuration and Message Content intent. Optional modules must be enabled per server. All commands also pass module/channel policy, custom permissions, shutdown and concurrency checks.", "", "Replies normally remain private for slash commands. Prefix replies appear in channel history. Bookmark and TorBox command inputs intentionally remain slash-only. Context actions are under a message's **Apps** menu.", "", "Permission values below are framework metadata; feature pages describe additional runtime target-channel, role hierarchy and voice/DJ checks. Empty metadata means no extra framework flag, not unrestricted access. Command groups organize subcommands. Optional arguments have no universal framework default; their omitted behavior is listed where applicable.", ""]
    sources = source_functions()
    for module in data["modules"]:
        mid = module["id"]
        lines += [f'## {module["name"]} (`{mid}`)', "", f'[Feature guide](../features/{mid}.md). ' + ("Always enabled." if module["required"] else "Enabled by default." if module["default_enabled"] else f'Enable with `/module enable {mid}` (Manage Server).'), ""]
        for row in (r for r in data["commands"] if r["module"] == mid):
            path = row["path"].replace(".", " ")
            label = row["context_menu"] or "/" + path
            lines += [f"### `{label}`", "", row["description"] or "Message action available from the selected message's Apps menu.", ""]
            syntax = " ".join(("<" if a["required"] else "[") + a["name"] + (">" if a["required"] else "]") for a in row["arguments"])
            modes = []
            if row["slash"]: modes.append(f"`/{path} {syntax}`".replace(" `", "`"))
            if row["prefix"]: modes.append(f"`!{path} {syntax}`".replace(" `", "`"))
            if row["context_menu"]: modes.append(f'Message → Apps → **{row["context_menu"]}**')
            lines += ["**Syntax:** " + "; ".join(modes) + ".", ""]
            if row["group"]:
                children = ["`/" + r["path"].replace(".", " ") + "`" for r in data["commands"] if r["path"].rsplit(".", 1)[0] == row["path"] and r["path"] != row["path"]]
                lines += ["**Subcommands:** " + ", ".join(children) + ".", ""]
            lines += [f'**User permissions:** {row["required_permissions"] or "No additional framework permission"}. **Bot permissions:** {row["required_bot_permissions"] or "No additional framework permission"}.', ""]
            if row["default_member_permissions"]: lines += [f'**Discord default visibility permission:** {row["default_member_permissions"]}.', ""]
            restrictions = [name for key, name in [("guild_only", "Server only"), ("dm_only", "Direct messages only"), ("owners_only", "Configured owners only"), ("nsfw_only", "Age-restricted channels only")] if row[key]]
            lines += ["**Restrictions:** " + ("; ".join(restrictions) if restrictions else "No extra location restriction in command metadata") + ". " + "; ".join(f"{scope}: {seconds:g}s cooldown" for scope, seconds in row["cooldowns_seconds"].items() if seconds is not None) + ".", ""]
            if row["arguments"]:
                lines += ["| Argument | Required | Meaning / accepted values | Omitted behavior |", "| --- | --- | --- | --- |"]
                for arg in row["arguments"]:
                    schema = arg["schema"] or {}
                    detail = arg["description"] or ARG_HELP.get(arg["name"], arg["name"].replace("_", " ").capitalize())
                    kind = {3: "text", 4: "integer", 5: "boolean", 6: "Discord user", 7: "Discord channel", 8: "Discord role", 9: "mentionable", 10: "number", 11: "attachment"}.get(schema.get("type"))
                    if kind: detail += f" ({kind})"
                    if row["context_menu"] and arg["name"] == "message": detail = "The message you selected in Discord; no typed argument"
                    choices = schema.get("choices") or []
                    if choices: detail += "; choices: " + ", ".join(str(c["name"]) for c in choices)
                    for bound in ["min_value", "max_value", "min_length", "max_length"]:
                        if schema.get(bound) is not None: detail += f'; {bound}: {schema[bound]}'
                    default = "Not applicable" if arg["required"] else OPTIONAL.get((row["path"], arg["name"]), "No reason provided." if arg["name"] in {"reason", "why"} else "You." if arg["name"] == "owner" else "Inspect replied message, then recent history." if arg["name"] == "url" else "Open private file picker." if arg["name"] == "file_number" else "See behavior and feature guide; no fixed default in registry.")
                    lines += [f'| `{arg["name"]}` | {"Yes" if arg["required"] else "No"} | {esc(detail)} | {esc(default)} |']
                lines += [""]
            if not row["group"] and not row["context_menu"]:
                example = EXAMPLES.get(row["path"])
                if example is None:
                    values = {"user": "@member", "channel": "#general", "role": "@Members", "module": "music", "command": "ping", "owner": "@member", "seconds": "10", "percent": "80", "count": "5", "position": "1", "from": "1", "to": "2", "number": "1", "id": "1", "name": "favorites", "reason": "Please follow the server rules", "why": "Please follow the server rules", "message": "Take a break", "query": "black cat", "target": "music", "effect": "Deny", "subject": "everyone", "note": "Read later", "duration": "10m", "interval": "1h", "value": "!", "key": "volume", "magnet": "magnet:?xt=urn:btih:<your-content-hash>"}
                    example = " ".join(values.get(a["name"], f'<{a["name"]}>') for a in row["arguments"] if a["required"])
                lines += [f'**Example:** `/{path}{" " + esc(example) if example else ""}`. Replace user, role, channel and record references with values from your server.', ""]
            candidates = sources.get(row["source_function"], [])
            if row["path"] in NOTES: lines += ["**Behavior:** " + NOTES[row["path"]], ""]
            preferred = [x for x in candidates if x[0].relative_to(ROOT).parts[1] == ("bot-core" if mid == "core" else "module-" + mid)]
            file, line = (preferred or candidates)[0]
            # Link to source using the public repository's current branch-independent tree.
            lines += [f'**Implementation and detailed behavior:** `{file.relative_to(ROOT).as_posix()}:{line}` (`{row["source_function"]}`). Related commands are grouped above; read the [feature guide](../features/{mid}.md) for limits and practical workflows.', ""]
    return "\n".join(lines) + "\n"


def configuration_page(data):
    flat = {f"{section}.{key}": value for section, fields in data["config_defaults"].items() for key, value in fields.items()}
    if set(flat) != set(CONFIG):
        raise ValueError(f"Config coverage mismatch: undocumented={set(flat)-set(CONFIG)}, removed={set(CONFIG)-set(flat)}")
    # Detect newly added environment names even if only used inside provider code.
    texts = "\n".join(p.read_text(encoding="utf-8") for p in (ROOT / "crates").glob("*/src/**/*.rs"))
    names = set(re.findall(r'(?:std::)?env::var(?:_os)?\(\s*"([A-Z][A-Z0-9_]+)"', texts))
    # Dynamically paired env setters are in config.rs, not literal env::var calls.
    config_text = (ROOT / "crates/bot-core/src/config.rs").read_text(encoding="utf-8")
    names |= set(re.findall(r'"((?:EASYNEWS|TORBOX|MUSIC)_[A-Z_]+)"', config_text))
    if names - set(ENV): raise ValueError(f"Undocumented environment variables: {names-set(ENV)}")
    lines = ["# Configuration reference", "", "Generated from serialized `AppConfig::default()` and reviewed source descriptions. The bot accepts an optional TOML file through `--config PATH`; it does **not** automatically load `config/config.toml` or `.env`. Docker Compose passes the selected file and environment. Environment overrides apply after TOML, then validation runs. Unknown TOML fields are rejected.", "", "All TOML fields are optional: omitted fields take these runtime defaults. Connecting to Discord requires `DISCORD_TOKEN`. Optional API services require only their own credentials. Keep secrets in your private environment or Compose `.env`, never TOML or Git. None of the TOML fields is a credential; database paths, owner IDs, guild IDs and hostnames can still reveal private operational information.", "", "Use `docker compose run --rm --no-deps bot --config /app/config/config.toml --check` for the source Compose installation, or the equivalent documented container configuration path. `--check` opens SQLite, applies migrations, constructs services and validates the registry, but does not connect to Discord. Back up existing data before validating an upgrade.", ""]
    for section, fields in data["config_defaults"].items():
        lines += [f"## `[{section}]`", "", "| Option | Runtime default / example | Purpose and accepted values |", "| --- | --- | --- |"]
        for key, value in fields.items():
            lines += [f"| `{key}` | `{esc(json.dumps(value, ensure_ascii=False))}` | {CONFIG[section+'.'+key]} |"]
        lines += [""]
    lines += ["## Environment variables", "", "These values are read by the bot or its logging/terminal libraries. Compose reads `.env` for interpolation; a native launch must export variables explicitly. Boolean overrides accept Rust `true`/`false` values. Never paste real secrets into diagnostics or issue reports.", "", "| Name | Purpose / module | Required | Default | Accepted value or safe example | Secret |", "| --- | --- | --- | --- | --- | --- |"]
    for name, (purpose, required, default, example, sensitive) in ENV.items():
        lines += [f"| `{name}` | {purpose} | {required} | `{esc(default)}` | `{esc(example)}` | {'Yes' if sensitive else 'No'} |"]
    lines += ["", "## Per-server settings", "", "These are stored in SQLite rather than environment variables. Manage Server is required for `/config show`, `/config set <key> <value>`, and `/config reset`. Server values survive restarts; reset restores process defaults. `/config set` supports:", "", "| Key | Accepted value | Initial value |", "| --- | --- | --- |", "| `volume` | Integer 0–200 | `music.default_volume_percent` |", "| `queue_limit` | Integer 1 through process max_queue_length | `music.max_queue_length` |", "| `idle_seconds` | Integer 10–86400 | `music.idle_disconnect_seconds` |", "| `empty_seconds` | Integer 10–86400 | `music.empty_channel_disconnect_seconds` |", "| `dj_role` | Decimal Discord role ID, or `none` | `none` |", "| `announce` | `true` or `false` | `true` |", "", "Module enablement, module allowed-channel lists, prefix overrides and custom permission rules are also per-server database settings. Use `/module`, `/prefix` and `/permissions` rather than editing SQLite manually.", "", "## Command-line switches", "", "| Switch | Behavior |", "| --- | --- |", "| `--config PATH` | Load an explicit TOML file. |", "| `--check` | Validate configuration, migrations, services and registry without Discord. |", "| `--sync-commands` | Force configured guild/global command registration on startup. |", "| `--export-command-reference PATH` | Write default config, modules and composed command metadata to JSON before loading configuration or database. No credentials required. |", "", "## Optional dependency requirements", "", "SQLite initializes and migrates automatically. Music website resolution needs yt-dlp; voice decoding/playback uses Songbird. Download conversions need FFmpeg and ffprobe. Source/native installs must provide these executables for the corresponding feature. Containers include these tools. AniList and image search need no API key. Movie/TV requires a TMDB credential. Easynews and TorBox are optional paid services; disabling them leaves other features usable. See the [feature guides](../features/index.md).", ""]
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--registry", type=Path, default=SNAPSHOT)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    data = json.loads(args.registry.read_text(encoding="utf-8"))
    if args.check and data != json.loads(SNAPSHOT.read_text(encoding="utf-8")):
        sys.exit("Offline registry snapshot is stale. Export docs/reference/commands.json and regenerate references.")
    outputs = {ROOT / "docs/reference/commands.md": command_page(data), ROOT / "docs/reference/configuration.md": configuration_page(data)}
    commands = data["commands"]
    report = {"registry_entries": len(commands), "leaf_commands": sum(not r["group"] for r in commands), "command_groups": sum(r["group"] for r in commands), "slash_entries": sum(r["slash"] for r in commands), "prefix_entries": sum(r["prefix"] for r in commands), "context_actions": sum(bool(r["context_menu"]) for r in commands), "modules": len(data["modules"]), "config_fields": len(CONFIG), "environment_variables": len(ENV), "documented_registry_entries": len(commands), "documented_config_fields": len(CONFIG), "limitations": ["Registry cannot express target-dependent hierarchy, DJ checks, omitted-argument behavior or all string parser rules. Curated feature guides and argument notes document those behaviors.", "Coverage checks compare the fresh registry export in CI; Python alone cannot verify runtime composition without that export."]}
    outputs[ROOT / "docs/reference/coverage.json"] = json.dumps(report, indent=2) + "\n"
    failed = []
    for path, content in outputs.items():
        content = content.replace("\r\n", "\n")
        if args.check:
            if not path.exists() or path.read_text(encoding="utf-8") != content: failed.append(path.relative_to(ROOT).as_posix())
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8", newline="\n")
    if failed: sys.exit("Outdated generated references: " + ", ".join(failed))
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
