# Changelog

All notable changes are recorded here. This project follows Semantic
Versioning and Keep a Changelog conventions.

## [Unreleased]

### Security

- Backported the upstream `libcrux-chacha20poly1305` overlong-output-buffer
  panic fix required by Songbird's pinned HPKE dependency chain.

### Added

- Optional Social Embeds module for webhook-based X/Twitter, Instagram,
  TikTok, Reddit, Threads, and Bluesky embeds, including Instagram provider
  failover and linked original, provider, and creator attribution.
- A dedicated command reference grouped by module and linked from the README.
- Native message actions for bookmarking, one-hour reminders, media downloads,
  and manual social-embed repair.
- Recurring, editable, snoozable, and cancellable reminders plus private,
  searchable message bookmarks.
- A bounded media toolbox for inspection, audio extraction, video compression,
  GIF conversion, and thumbnail extraction.
- Private, owner-bound TorBox file pickers with pagination, expiry, infected-file
  filtering, and direct file-number fallback for cached downloads and streams.

## [0.2.0] - 2026-09-21

### Added

- Optional TorBox module for resolving cached magnet files into temporary
  download links, with subscription-gated video streaming support and safe
  handling for unavailable plans.
- Root-level agent guidance covering architecture, module boundaries, runtime
  invariants, focused context loading, verification, and deployment safety.
- Initial Docker deployment with persistent SQLite storage, an integrity-checked
  online backup, and verified Discord reconnection after a graceful restart.
- Structured, privacy-safe audio failure diagnostics.
- Private test-guild command and permission acceptance remains queued.

### Changed

- Server Info and User Info now share a polished embed design with readable
  identities, images, Discord timestamps, grouped details, and safe fallbacks.
- Audio playback now streams through a bounded, resumable HTTP source instead
  of downloading the complete track before playback begins.

### Fixed

- Backported ringbuf's panic-safe consumer removal fix for RUSTSEC-2026-0293
  while Songbird remains on the incompatible ringbuf 0.4 API.

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

[Unreleased]: https://github.com/oper-goon/momo-discord-bot/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/oper-goon/momo-discord-bot/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/oper-goon/momo-discord-bot/releases/tag/v0.1.0
