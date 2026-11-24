# Public-release implementation plan and discovery inventory

Review date: 2026-11-24. This report is excluded from the documentation website.

## Scope and isolation

Worktree: `work/public-release`, branch `feature/public-release-docs`, based on
`ac327f3`. The source worktree was already dirty on `feature/music-providers`.
Its tracked edits and new media crate/channel-policy/migration files were copied
into this worktree so the release documents the actual supplied implementation.
They are pre-existing development, not changes authored by this initiative.
`input-snapshot.json` records hashes of the original tracked edits. The original
worktree, branches, production and databases were not modified by this work.

The unrelated untracked `momo-armada/` project and two local screenshots were
not copied or treated as approved release assets. The root repo has 168 tracked
files at the baseline, including three vendored security backports. Discovery
covered workspace manifests, source/registry, migrations, config, deployment,
workflows, existing engineering docs and available local history. Transitive
dependency and historical-secret conclusions have the limits in the audit report.

## Phases and implementation decisions

1. **Discovery/audit:** retain Rust architecture and active development; inspect
   registry/configuration/deployment, history, licensing and existing assets.
2. **Framework:** free Material for MkDocs, pinned Python requirements, static
   GitHub Pages, beginner-first navigation and independent local build.
3. **Coverage:** export the composed registry offline, generate references and
   document all eleven modules, with source-based configuration inventory.
4. **Hosting:** retain multi-stage/non-root Dockerfile and two-service Compose;
   add secure env forwarding, fail-fast mounts, bounded logs, setup and recovery
   guides. No misleading database-write health check.
5. **Identity:** monochrome theme, responsive cards, focus/reduced-motion rules,
   system typography and social metadata; owner-approved artwork is a blocker.
6. **Automation:** strict build, stale-reference/internal-link/asset checks,
   official Pages actions, read-only PR validation, explicit publication gate.
7. **Community/licensing:** contribution/security/issue templates and notices;
   preserve existing MIT pending ownership review, prepare official PolyForm text.
8. **Validation:** offline exporter/tests, docs build/link and browser checks,
   redacted security review. Report Docker/live Discord/security/rights gaps.

Independent workers own references/features, deployment/setup, and security/
licensing/community; the primary worker owns framework, developer docs, CI,
integration and final validation. Publishing, merging, visibility, licensing,
production operations and history rewriting are outside the current authorization.

## Implemented surface

| Module | Default | Implemented surface |
| --- | --- | --- |
| core | required | Help/about, module and channel activation, permissions, prefixes, guild music configuration |
| general | on | Ping/avatar and user/server/role information |
| utility | on | Choices, native polls, one-off/recurring reminders and management, private bookmarks, timestamps, two message actions |
| admin | off | Announcements, slowmode, nicknames, role add/remove |
| moderation | off | Kick/ban/unban/timeout/untimeout/purge/warn/warnings/cases |
| music | off | Join/play/control, queues, repeat/shuffle, saved playlists, provider diagnostics |
| download | off | Download/reupload, probe, audio/compress/GIF/thumbnail, message action |
| media | off | Anime/movie/TV lookup and interactive search selection |
| image | off | Safe image search and expiring navigation |
| torbox | off | Cached magnet file picker/ZIP and subscription-gated stream |
| social-embeds | off | Automatic webhook embed correction and manual message action |

All composed command nodes, contexts, arguments, declared permissions and
cooldowns are in `docs/reference/commands.json` and its Markdown rendering.
Body-level and shared policy restrictions are explained in the reference and
feature pages. Source metadata alone cannot establish external-service acceptance.

## Configuration, dependencies and integrations

- Configuration: typed TOML sections `discord`, `commands`, `database`, `music`,
  `download`, `social_embeds`, `logging`; `config/example.toml` has no credentials.
  Secrets come from environment; the native app does not load `.env` itself.
- Required service credential: `DISCORD_TOKEN` for normal login. Offline reference
  export and validation do not need a Discord token. All supported fields and
  environment variables are enumerated in the configuration reference.
- Rust dependency versions/features: root and crate Cargo manifests; resolved
  transitive packages: Cargo.lock. Poise/Serenity, Songbird/Symphonia, SQLx SQLite,
  Reqwest/TLS, Tokio and common serialization/runtime utilities are retained.
- Runtime tools: yt-dlp and matching EJS, Deno, private BgUtils PO-token helper;
  FFmpeg/FFprobe for downloads/conversions; SQLite CLI for consistent backups.
- Optional accounts: Easynews (paid authenticated audio retrieval, unofficial
  search), TorBox (shared operator API account, plan limits), TMDB token/key.
- Credential-free APIs/sites: Discord, AniList, YouTube/SoundCloud/Bandcamp and
  approved download sites, DuckDuckGo/Bing unsupported search interfaces, selected
  social embed hosts. GitHub stable-release availability is a bounded startup
  check, not an updater. Pages/build tooling never depends on these bot services.
- Network: outbound DNS/HTTPS, Discord gateway WebSocket, voice UDP; private
  Compose token-helper HTTP. No inbound host port or external SQL server needed.

## Persistence and background work

SQLite uses foreign keys, WAL, busy timeout and startup SQLx migrations. Migration
1 creates guild settings/modules, permission rules, moderation cases, reminders,
music settings/playlists/tracks and command sync. Migration 2 adds recurrence
and bookmarks; supplied migration 3 adds module channel restrictions.
Compose stores SQLite plus backup files in `momo-data` at `/data`, UID 10001.

Reminders run under Supervisor every 15 seconds. Per-guild music actors use a
one-second timer for lifecycle decisions; voice events and UI components route
only to subscribed enabled modules. Interactive sessions/caches and live queues
are bounded in-memory state; saved settings/playlists and user records persist.
No external scheduler or cron deployment is required. Shutdown is bounded.

## Existing infrastructure and known gaps

The baseline already had a multi-stage Debian/Rust Dockerfile, UID 10001 runtime,
SQLite volume, read-only config, 20-second grace, restart policies, pinned media
tools and a private token-helper service. Existing Linux CI runs formatting,
Clippy, workspace tests, offline check, cargo-deny and a Docker build.

Pre-1.0 acceptance remains incomplete: live interactions/permissions/cache
fallbacks, voice provider/seek behavior, active-load shutdown, full replacement
volume recovery and ARM64 execution. Existing notes report a local migration
checksum mismatch and multiple blocked advisory gates; this initiative does not
silently edit migrations or waive vulnerabilities. External interfaces may change.
There is no published GHCR image, approved mascot/logo source, official bot invite,
verified minimum hardware specification, existing accepted roadmap or finalized
noncommercial license transition. Planned work is labeled accordingly.
