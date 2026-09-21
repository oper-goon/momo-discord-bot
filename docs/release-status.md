# Release status

Current revision: `0.1.0` foundation release candidate.

## Verified locally

- Workspace format check
- Strict Clippy across all targets and features
- Workspace unit and integration tests, including migrations, policy
  precedence, module validation, queue transitions, stale generations, paused
  Tokio timers, media classification, and private-address rejection
- Dependency advisory, license, source, and ban policy
- Offline application startup check with migrations and command-tree assembly
- Git secrets/path exclusions

## Verified on the deployment host

- Built the release image on an x86_64 Ubuntu host with rootless Docker
- Started in global-command mode and reached Discord's ready event
- Applied migration 1 to the persistent SQLite volume
- Confirmed the pinned yt-dlp executable is available in the runtime image
- Created an online SQLite backup and passed `PRAGMA integrity_check`
- Performed a graceful container restart, reconnected to Discord, and retained
  the migration state

## Required before v1.0.0

- Create a Discord application and private test guild.
- Run the documented command registration in guild mode.
- Exercise every command family with ordinary, moderator, administrator, DJ,
  and bot-owner accounts.
- Play and seek real supported media, test pause/resume/skip/disconnect, and
  verify yt-dlp's current extractor behavior.
- Confirm role hierarchy, channel overrides, component authorization, restart
  persistence, reconnect behavior, and shutdown while audio is active.
- Restore a backup into a disposable volume and confirm the 15-second shutdown
  budget under active load.
- Run an ARM64 build/test if ARM64 deployment is planned; the security backport
  has a source regression test but its assembly was not executed on this host.

The bot should remain in a private test guild until this checklist is complete.
