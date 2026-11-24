# Changelog

All notable changes are recorded here. This project follows Semantic
Versioning and Keep a Changelog conventions.

## [Unreleased]

### Fixed

- Accept natural music discovery queries containing the full artist and a partial
  title, such as `tyler the creator sugar`, without weakening explicit
  `Artist - Title` or playback fallback matching. Keep discovery cache entries
  distinct from explicit recording requests and reject unrequested versions.
  Check up to five YouTube discovery candidates instead of rejecting a query
  solely because its first upstream result is unrelated. Preserve matching
  YouTube creator context for title-only results only when that complete
  channel name was explicitly requested; never infer an artist from an uploader.

### Added

- Collect exact-image Debian/Python inventories and Rust dependency notices with
  checksum-verified upstream source archives. Retain unresolved attribution and
  source requirements for review before binary/image publication.

- Material for MkDocs documentation with beginner Docker setup, module guides,
  generated command/configuration references, operations and developer guides.
- Strict documentation CI, offline registry export, stale-reference and built-site
  link/asset checks, with approval-gated GitHub Pages deployment.
- Private configuration setup helper, contribution/security policies and issue
  templates, third-party notices and a proposed noncommercial licensing review.

### Changed

- Enforce canonical LF migration files across checkouts and exclude local audit
  caches from Docker build contexts. A restored-copy trial caught checksum
  mismatches in a Windows-created source archive before any live switch.

- Pin tested Rust 1.98.1 consistently across tooling, CI and Docker; avoid
  compiler drift triggering Poise's generated rest-argument lint on Rust 1.100.

- Upgrade the DAVE P-256/HPKE backends through narrowly scoped, MIT-licensed
  adapters, preserving OpenMLS protocol/storage APIs. Replace vulnerable
  crypto-bigint with patched upstream 0.7.11 and remove obsolete SHA3 exceptions.
  Add offline handshake, epoch rotation, signature and mixed-version wire tests.
- Correct the workspace's minimum Rust requirement to 1.94, matching SQLx 0.9.1;
  the existing pinned toolchain and Docker builder already satisfy it.

- Update bundled SQLite to 3.54.1 through compatible libsqlite3-sys 0.39.1,
  with a linked-runtime security-version regression check. Record two dated
  maintenance exceptions while preserving picker IDs. The subsequent voice
  backend upgrade clears the remaining cryptography advisory.
- Record the owner's months of live Discord testing as operator acceptance
  for the planned 1.0 release.

- Apply PolyForm Noncommercial 1.0.0 to first-party release-candidate code after
  owner confirmation of rights; preserve earlier MIT grants and third-party terms.

- Update Tokio, TLS certificate verification/trust roots and URL parsing to
  patched compatible releases, clearing four release-audit findings. Subsequent
  SQLite and voice updates clear the other vulnerability/unsoundness findings;
  two maintenance findings have dated exceptions.

- Docker Compose forwards optional integration settings from the private env
  file, uses safer container defaults, bounds logs and rejects missing config
  mounts. Existing database volume and non-root image behavior are retained.
- README directs new users to installation and reference guides. Public-site
  publication, approved artwork and licensing transition remain pending review.

## [0.9.0] - 2026-10-14

### Security

- Backported the upstream `libcrux-chacha20poly1305` overlong-output-buffer
  panic fix required by Songbird's pinned HPKE dependency chain.
- Updated Rustls to `0.23.46` to resolve the newly reported TLS advisories.
- Replaced the yanked `yoke-derive` release with compatible version `0.8.4`.

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
  filtering, safe whole-torrent ZIP downloads, and direct file-number fallback
  for cached downloads and streams.
- Ghost Silver startup banner with bot identity, version, server/module/command
  counts, startup duration, and bounded GitHub release availability checks.
- On-disk persistence and snapshot restoration tests, plus a live persistence
  acceptance guide. Deployment bookmark restart, overdue-reminder recovery, and
  isolated backup restoration have been verified.

### Changed

- TorBox ZIP downloads now show preparation progress and an explicit button to
  open the generated archive link.
- TorBox streams expose temporary HLS media links with clearer player guidance.

### Fixed

- Media uploads now complete the original deferred slash response instead of
  sending a follow-up and deleting the original response.
- TorBox stream selection prefers supported media fields rather than unrelated
  metadata URLs, and generated links normalize spaces for Discord buttons.
- Instagram download and social-embed fallback handling, including validation
  of redirected media types.

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

[Unreleased]: https://github.com/oper-goon/momo-discord-bot/compare/v0.9.0...HEAD
[0.9.0]: https://github.com/oper-goon/momo-discord-bot/compare/v0.2.0...v0.9.0
[0.2.0]: https://github.com/oper-goon/momo-discord-bot/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/oper-goon/momo-discord-bot/releases/tag/v0.1.0
