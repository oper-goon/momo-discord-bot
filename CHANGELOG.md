# Changelog

All notable changes are recorded here. This project follows Semantic
Versioning and Keep a Changelog conventions.

## [Unreleased]

### Added

- Private test-guild acceptance run and production deployment remain queued.

## [0.1.0] - 2026-09-20

### Added

- Rust workspace with core, database, module API, general, administration,
  moderation, utility, and music crates.
- Runtime per-guild module activation and filtered help output.
- Central command checks, explainable permission rules, cooldowns, bounded
  concurrency, supervised tasks, and graceful shutdown.
- SQLite migrations and repositories for settings, policy, moderation cases,
  reminders, music settings, playlists, and command synchronization hashes.
- Moderation hierarchy checks and durable pending/succeeded/failed audit cases.
- Persistent at-least-once reminders with bounded retry delay.
- One music actor per active guild, bot-owned queues, generation-safe events,
  repeat, shuffle, editing, seek, volume, idle and empty-channel cleanup,
  failure halting, and authorized component controls.
- Bounded yt-dlp and direct HTTP providers with DNS pinning, redirect rejection,
  private-network filtering, output and download limits, and local files off.
- Docker image, Compose service, CI, Dependabot, cargo-deny policy, security
  backport regression guard, and committed lockfile.

[Unreleased]: https://github.com/oper-goon/momo-discord-bot/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/oper-goon/momo-discord-bot/releases/tag/v0.1.0
