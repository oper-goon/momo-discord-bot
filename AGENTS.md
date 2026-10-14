# AGENTS.md

This file is the compact operating guide for coding agents. It applies to the
whole repository. Prefer it over loading the full specification unless a task
needs specification-level detail.

## Project snapshot

- Momo is a self-hosted modular Discord bot written in Rust 2024.
- Discord stack: Poise `0.7` over Serenity `0.12.5`; Songbird handles voice.
- Persistence: SQLite through SQLx with forward-only migrations.
- Runtime entry point: `crates/bot-app/src/main.rs`.
- Workspace version and shared dependency versions live in root `Cargo.toml`.
- `main` is intended to stay releasable; active development may occur on
  `staging`.

## Read only what the task needs

1. Start with this file and the relevant crate.
2. Read `README.md` for setup, commands, operations, and release policy.
3. Read `docs/specification.md` only for behavioral contracts or ambiguous
   product requirements. Use its table of contents/search; do not load it all.
4. Read `docs/security.md` before dependency, network, media, secret, or
   trust-boundary changes.
5. Read `docs/release-status.md` for known validation gaps.

Use `rg`/`rg --files` to locate code. Do not scan `target/`, `.tools/`, or the
crate registry unless checking a dependency API.

## Architecture and dependency direction

```text
bot-app (composition root, startup, Discord command synchronization)
  ├─ bot-core (runtime, config, command policy, shared Discord helpers)
  │    ├─ bot-module-api (small Discord-independent shared data contracts)
  │    └─ bot-database (SQLite repository and migrations)
  └─ module-* (feature crates; commands/events/lifecycle)
```

- `bot-module-api`: shared serializable domain types. Keep it small and free of
  Discord framework types.
- `bot-database`: owns SQLx access. Feature crates should call typed database
  methods rather than embed ad hoc SQL.
- `bot-core`: owns `BotData`, module/service registries, lifecycle/event
  routing, permission decisions, common reply helpers, config, and errors.
- `bot-app`: composition root only. It assembles modules, applies default
  cooldowns, synchronizes application commands, starts services, and performs
  graceful shutdown. Do not move feature behavior here.
- `module-general`: ping/avatar and info commands. Reuse `src/info.rs` for the
  shared Info-command color, footer, Discord dates, and bounded role lists.
- `module-admin`: Discord administration actions and hierarchy checks.
- `module-moderation`: moderation commands plus auditable case persistence.
- `module-utility`: polls, timestamps, and persistent reminder delivery.
- `module-music`: resolver, per-guild player actors, voice events, controls,
  queues, and playlists. Preserve actor ownership and bounded async behavior.

Keep dependencies pointing inward as above. A feature module may depend on
core/database/API, but feature modules should not depend on each other.

## Module and command model

Each feature exports `module()` returning `RegisteredModule`. A module declares:

- stable lowercase ID and display name;
- whether it is required/default-enabled;
- top-level Poise commands;
- optional extensions to a command owned by another module;
- gateway intents, event subscriptions/handler, lifecycle, and services.

`bot-app::compose` merges extensions and rejects duplicate module IDs, service
IDs, and command paths. If adding a crate/module, wire it into root
`Cargo.toml`, `bot-app/Cargo.toml`, and `bot-app::compose`, then update registry
tests and user docs.

All commands pass through `bot_core::command_check`. It already handles the
initial ephemeral defer, global command concurrency, shutdown state, per-guild
module activation, prefix enablement, and custom permission policy. Do not
duplicate or bypass these checks in a command. Keep command names, paths,
permissions, and guild restrictions stable unless the task explicitly changes
them; command-tree changes trigger hash-based Discord synchronization.

New commands should support both `slash_command` and `prefix_command` when the
same behavior is safe and practical in both interaction modes. Keep a command
slash-only when its arguments are sensitive, its response must be ephemeral,
or its interaction design does not translate safely to message commands.
Examples include TorBox magnet inputs, secrets, and similar private material
that should not be posted into channel history. Document intentional slash-only
exceptions in the command's module or user documentation. Prefix support also
depends on prefix configuration and Discord's Message Content intent; do not
assume declaring `prefix_command` alone enables it in production.

Use existing helpers such as `reply`, `guild_id`, `effective_permissions`,
`require_channel_permissions`, and `require_hierarchy`. Responses are normally
ephemeral and must suppress allowed mentions unless intentional. Optional
Discord/cache data must degrade gracefully; never panic on user input or an
external response.

## Runtime invariants

- Core is required. Module enablement is per guild and persisted.
- Event handlers run only for subscribed event kinds and enabled modules.
- Background work belongs under `Supervisor`; respect its cancellation token
  and graceful-shutdown budget. Avoid detached `tokio::spawn` tasks.
- Long-lived mutable guild state should have one clear owner. Music uses a
  per-guild actor/handle model; do not introduce parallel mutation paths.
- Request only gateway intents actually required by a module.
- Errors shown to users must remain safe. Do not log tokens, signed URLs,
  secrets, or dependency error payloads that may contain them.
- Preserve rate/concurrency bounds, timeouts, queue limits, and message/embed
  limits. Do not hold Serenity cache guards or standard locks across `.await`.

## Database changes

- Add a new monotonically numbered file in `migrations/`; never edit an applied
  migration.
- Migrations run in `Database::open` before Discord connects.
- Store Discord snowflakes as canonical decimal text, matching existing code.
- Put database operations in `bot-database` and use transactions for related
  writes or allocation sequences.
- SQLite runs with foreign keys, WAL, and a busy timeout. Preserve restart and
  retry semantics for reminders and moderation cases.

## Change discipline

- Make the smallest cohesive change; do not refactor unrelated systems.
- Follow existing Rust formatting, `anyhow::Result`, async, and Poise patterns.
- Keep `#![forbid(unsafe_code)]` intact.
- Add no dependency when a small local implementation is sufficient.
- Preserve user changes in a dirty worktree. Never reset or overwrite them.
- User-visible changes belong in `CHANGELOG.md` when preparing a release or
  when the task explicitly includes release documentation.
- Never commit `.env`, `config/config.toml`, databases, tokens, logs, signed
  media URLs, or generated build artifacts.
- `vendor/libcrux-secrets` and `vendor/ringbuf` contain intentional security
  backports. Do not replace or edit them casually; their invariants are tested
  in `bot-app` and documented in `vendor/README.md` and `docs/security.md`.

## Verification

Prefer the workspace wrapper on Windows because it configures the checked-in
Rust/LLVM/CMake/Ninja tool paths:

```powershell
./scripts/dev.ps1 --% fmt --all -- --check
./scripts/dev.ps1 --% clippy --locked --workspace --all-targets --all-features -- -D warnings
./scripts/dev.ps1 --% test --locked --workspace
./scripts/dev.ps1 --% run --locked -p bot-app -- --check
./scripts/dev.ps1 --% deny check
```

The PowerShell stop-parsing token `--%` is intentional: it preserves Cargo's
`--` separator and short flags when forwarding them through `dev.ps1`.

For a focused iteration, run `check`, `clippy`, and tests for the changed crate;
before merge/push, run the full relevant gates. `bot-app --check` validates
configuration, migrations, services, and the composed command registry without
connecting to Discord. A live Discord acceptance test is still required for
interaction rendering, permissions, cache fallbacks, voice, and command sync.

CI (`.github/workflows/ci.yml`) runs formatting, workspace Clippy/tests,
`bot-app --check`, cargo-deny, and a Docker build on Linux.

## Deployment

- Docker Compose service: `bot`; image is built by the multi-stage `Dockerfile`.
- Persistent SQLite data is in the `momo-data` named volume; configuration is a
  read-only bind mount. Rebuild/recreate only the bot service for code changes.
- Keep `DISCORD_TOKEN` in the host's private `.env`, never in TOML or Git.
- After deployment, verify `docker compose ps`, restart count, and recent logs;
  successful startup logs `Bot ready`.
- Do not delete/recreate the data volume or restore a database unless the user
  explicitly requests it and a verified backup exists.
