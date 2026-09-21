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

## Required before v1.0.0

- Create a Discord application and private test guild.
- Run the documented command registration in guild mode.
- Exercise every command family with ordinary, moderator, administrator, DJ,
  and bot-owner accounts.
- Play and seek real supported media, test pause/resume/skip/disconnect, and
  verify yt-dlp's current extractor behavior.
- Confirm role hierarchy, channel overrides, component authorization, restart
  persistence, reconnect behavior, and shutdown while audio is active.
- Build and run the container on the deployment host, test an online SQLite
  backup and restore, and confirm the 15-second shutdown budget.
- Run an ARM64 build/test if ARM64 deployment is planned; the security backport
  has a source regression test but its assembly was not executed on this host.

The bot should remain in a private test guild until this checklist is complete.
