# Release status

Current revision: `0.9.0` pre-1.0 feature release.

## Operator acceptance for 1.0

On 2026-11-24 the owner confirmed months of live Discord testing and confidence
in a 1.0 release. This is accepted as operator-reported live acceptance of the
running bot; repeating the whole historical live-test program is not a release
requirement. It does not mean every individual scenario below was separately
observed or that the newly updated dependency build has been deployed.
Dependency/security and packaging checks remain separate release gates. No
production deployment was performed as part of this dependency follow-up.

## Verified locally

- Workspace format check
- Strict Clippy across all targets and features
- Workspace unit and integration tests, including migrations, policy
  precedence, module validation, queue transitions, stale generations, paused
  Tokio timers, media classification, and private-address rejection
- Dependency advisory, license, source, and ban policy
- Offline application startup check with migrations and command-tree assembly
- Git secrets/path exclusions
- Disposable on-disk close/reopen tests for modules, prefixes, permission
  rules, music settings, playlist order/metadata, reminders, bookmark notes,
  moderation cases and numbering, and command synchronization hashes.
- SQLite snapshot reopen and integrity validation using a separate test file;
  reminder delivered/retry/recurring state survives subsequent reopen cycles.
  Live bookmark restart, overdue-reminder recovery, and isolated host backup
  restoration have also passed; see the deployment results below and the
  [persistence acceptance procedure](persistence-testing.md) for remaining checks.
- The TorBox interactive picker is covered by unit and registry tests; its
  Discord select-menu rendering still requires live acceptance testing.

## Release-candidate Linux validation

Private CI passed on 2026-11-24 for commit
`07e3e558edbd52dfa7b57a3666c287a3ca6a2920`: strict code gates, backend and
mixed-version DAVE tests, image build, Compose configuration, offline startup,
persistent-volume reopening, SQLite backup/restore and integrity. The container
checks used no credentials or network and touched only a disposable CI volume.
They do not establish live gateway/voice behavior or active-audio shutdown.

## Verified for v0.2.0

- Exercised the TorBox cached-download flow against the live service, including
  generated file links and the three-hour expiry warning.
- Confirmed uncached magnets are rejected without being queued or downloaded.
- Confirmed the TorBox module's role allow rule and everyone deny rule in the
  private Discord test guild.
- Live TorBox Web Player streaming remains unverified because the configured
  Essentials subscription does not include that feature; this is a documented
  validation gap rather than a blocker for the cached-download release.
## Verified on the deployment host

- Built the release image on an x86_64 Ubuntu host with rootless Docker
- Started in global-command mode and reached Discord's ready event
- Applied migration 1 to the persistent SQLite volume
- Confirmed the pinned yt-dlp executable is available in the runtime image
- Created an online SQLite backup and passed `PRAGMA integrity_check`
- Performed a graceful container restart, reconnected to Discord, and retained
  the migration state
- On 2026-10-14, restarted only the bot container on the deployment host, verified
  `Bot ready` and zero automatic restarts, and received operator confirmation
  that a bookmark created before the restart remained in Discord afterward.
  Live playlist/settings persistence and other record types still require
  separate acceptance checks.
- On 2026-10-14, the operator reported testing a reminder that became due while
  the bot was stopped and confirming delivery after startup. This verifies
  live overdue-reminder persistence and recovery; it does not establish
  recurring-reminder or failed-delivery retry acceptance.
- On 2026-10-14, created an online SQLite backup on the deployment host and restored
  it into a separate temporary database while the live bot remained running.
  SQLite integrity and foreign-key checks passed; all snapshot contents matched
  the restored copy, including bookmark #1's operator-created restore-test note.
  The deployed Momo binary's offline `--check` passed against the restored copy,
  and its contents still matched afterward. The backup is retained in the data
  volume at `/data/backups/<private-backup>.db`.

## Historical detailed live-test checklist

- The operator reports successful live music lifecycle testing. Detailed
  supported-provider playback/seek coverage still needs to be recorded.
- Run the documented command registration in guild mode.
- Exercise every command family with ordinary, moderator, administrator, DJ,
  and bot-owner accounts.
- Record supported-provider playback and seek results and verify yt-dlp's
  current extractor behavior.
- Confirm role hierarchy, channel overrides, component authorization, restart
  persistence, reconnect behavior, and shutdown while audio is active.
- Confirm the 15-second shutdown budget under active load. Isolated database
  backup restoration has passed; a full replacement-container/volume recovery
  drill has not been performed.
- Run an ARM64 build/test if ARM64 deployment is planned; the security backport
  has a source regression test but its assembly was not executed on this host.

The owner's later months-long acceptance above supersedes requiring a repeat of
this entire checklist. For the new dependency build, record a focused live voice
smoke test before production release. ARM64 remains unsupported until tested.
