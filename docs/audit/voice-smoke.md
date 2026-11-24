# Focused upgraded-build voice smoke test

Prepared 2026-11-24. **Authorized trial completed; audible playback was not exercised.** This covers the newly adapted P-256/HPKE/DAVE dependency build, not a repeat of the owner's months of acceptance of the running bot. The offline original/upgraded peer handshake, epoch rotation, encrypted packet and tamper tests are recorded in [voice upgrade](voice-upgrade.md). Those tests cannot prove Discord voice negotiation, RTP delivery, audible Opus playback, or active-audio shutdown.

## Chosen route: existing bot during an agreed maintenance window

The owner selected the **existing Discord bot identity**, rather than creating another bot. This is a planned short candidate trial, not a completed test or authorization to start it immediately. The operator must agree the exact window and rollback point before any service interruption. This preparation has not contacted the host, inspected private configuration or started a process.

Only one process may connect with that token at a time. Stop and confirm the original bot is exited before starting the candidate; stop and confirm the candidate is exited before restoring the original. A new Compose project name alone does not prevent simultaneous Discord sessions.

### Prepare before the window

1. Record the deployed source revision if available, the running container's immutable image ID, and the exact existing Compose/configuration inputs privately. Retain the original image locally under a rollback tag; verify it can still be inspected. Do not rely on rebuilding an old tag or pulling an image later. Build the candidate separately without recreating any running service, and record its image ID and tested commit.
2. Make an online SQLite backup using the existing image's SQLite `.backup` command. Copy it off the live volume, check `PRAGMA integrity_check`, and rehearse restoring it into a separate disposable database/volume. Run candidate offline `--check` against that **disposable copy**, never against the live database during preparation. The candidate check may migrate its copy.
3. Compare the deployed migration files and recorded SQLite migration versions/checksums with the candidate's three migration files (`0001_initial.sql` through `0003_module_channels.sql`). For the short trial, require the existing live schema/checksums already to match. A new or edited migration means the proposed switch is broader than a voice smoke: review that upgrade separately before proceeding. Never rewrite applied migration checksums or assume an older binary can read a newly migrated database.
4. Review the candidate's command registry and user-facing/source delta against the **actually deployed build**, rather than assuming it matches the default branch. This release worktree inherited feature changes, including Media search commands and module-channel policy; those may already be deployed, or may be new relative to an older deployment. Record that determination explicitly. Do not describe a default-branch-to-candidate switch as only a crypto update without checking it.
5. To prevent accidental global/guild command replacement during the short trial, prepare a private candidate-only TOML copy with `[commands] mode = "none"`, preserving other settings. Do not pass `--sync-commands`. Existing registered `/join`, `/play`, `/pause`, `/resume`, `/skip`, `/nowplaying`, `/queue show`, `/leave` and `/seek` must have matching candidate paths/arguments. If a required command differs, resolve the scope before the trial instead of silently synchronizing the entire candidate registry. Retain the original TOML unchanged for rollback.
6. Select a quiet test voice/text channel and an authorized listener. Confirm current Music activation, channel permissions and DJ policy beforehand. Preserve production provider settings; start with a known authorized public direct-audio fixture to distinguish voice from external search issues, then test a normal currently used provider. No new subscription/credential is needed for the baseline.
7. Review the exact local service replacement/rollback commands against the real private Compose files. The intended scope is **only `bot`**; preserve the provider service, existing named volume, private environment and restart policy. Candidate override files must select the retained candidate image explicitly and mount the candidate-only TOML read-only. Do not run an unqualified `up --build` that could rebuild/recreate unrelated services. Do not print expanded Compose configuration containing secrets.

The existing database is the preferred runtime data source for this short trial **only after schema equivalence is verified**, so normal reminders and server policy remain authoritative. A cloned production database is useful for offline validation but is not automatically safe to connect live: its pending reminder worker can emit duplicate reminders, and moderation/social handlers can produce real side effects against Discord while writing only the clone. A live clone requires a separate, reviewed suppression/access plan; none is implied here. Do not erase reminders or change production module policy merely to make a clone convenient.

### Execute only in the agreed window

1. Announce/confirm the short interruption with the operator. Capture a fresh verified backup immediately before the switch and note the rollback image/configuration.
2. Stop **only the original `bot` service** gracefully, and verify the container has exited and its Discord session has ended. Do not stop/recreate the provider or delete any volume.
3. Recreate only `bot` using the prepared candidate image and candidate-only TOML, the unchanged private environment, and the verified existing storage. Confirm the effective image and mounts privately before declaring readiness. Wait for `Bot ready`, check restart count, then perform the observation checklist below in the designated channel.
4. If readiness fails, audio fails on both known sources, the process restarts unexpectedly, or the agreed timebox expires, stop the candidate and restore the retained original image/configuration immediately. Do not broaden the live debugging session or force command synchronization. Both source extraction and audible voice observations must be recorded separately.
5. Perform the active-audio stop and one restart only within that window. All stop/start/log commands in this route must target the **actual original project/service**, not the alternative `momokun-voice-smoke` project shown later. The current Compose grace period is 20 seconds; the application budgets 15 seconds for graceful shutdown. Record timing, exit state and any `Graceful shutdown budget exceeded` warning.
6. Finish by returning to the retained original image/configuration unless the owner explicitly approves leaving the candidate running. Verify `Bot ready`, restart count and ordinary command response after rollback. Since this trial requires schema equivalence, preserve the current authoritative database rather than automatically restoring the pretrial backup and discarding events that occurred during the window. Any unexpected schema change or recovery restore requires a separate stopped, reviewed recovery decision.

This plan deliberately contains no runnable host-specific replacement commands: those depend on private deployment paths/override files and must be made concrete after a read-only preflight, before the window is approved. It does not authorize credential rotation, public deployment, data deletion or a database rollback.


## Alternative route: separate bot identity

Repository examples provide Docker Compose, a private environment template, a guild-registration TOML template, optional provider settings and all required music tools in the image. The current `config/example.toml` deliberately has an empty `development_guilds` list and must be edited before a guild-mode startup. No ready-to-use nonproduction Discord application, test-guild identity, test listener or authorized media fixture is supplied by the repository. No private configuration/token was opened to prepare this procedure.

Use a separate checkout/directory, a separate Discord application/token, a private test guild and a fresh Compose project named `momokun-voice-smoke`. Do not use the running production bot's token concurrently, production TOML, production database/volume or production Compose directory. Changing the Compose project name isolates containers and the default named volume, but does not make a reused production token/configuration safe. Obtain operator approval before starting any live test; this document does not authorize deployment or a Discord connection.

## Separate-bot operator preparation (alternative only)

1. Select and record the exact candidate commit and a locally built image ID. Use that checkout; do not move production branches. Confirm the candidate's Linux CI and dependency gates are green.
2. In the separate test checkout, copy `.env.example` to `.env` and `config/example.toml` to `config/config.toml` only when these test files do not already exist. Set a **new test application's** `DISCORD_TOKEN` privately. Do not include Easynews/TorBox/TMDB credentials for the baseline smoke test.
3. Set `[commands] mode = "guild"` and `development_guilds = [YOUR_PRIVATE_TEST_GUILD_ID]`. Enable Message Content Intent in that test application's Developer Portal because the current composed registry requests Social Embeds' intents even when per-guild disabled. Keep prefix commands disabled unless explicitly testing them.
4. Invite the test application with `bot` and `applications.commands` scopes. Give it View Channel, Send Messages and Embed Links in the test text channel, plus Connect and Speak in the chosen ordinary voice channel. Use a test server manager for `/module enable music`; have an ordinary listener account available too. Do not use a Stage channel for this focused baseline.
5. Keep `[music] easynews_enabled = false`, `torbox_enabled = false`, and `provider_priority = ["youtube"]` for the baseline. Leave normal player timeouts/defaults intact. No provider subscription is needed.
6. Choose one short public HTTPS audio file you are authorized to play (under the 64 MiB stream ceiling, no login, redirects or private addresses), and one known playable public YouTube/SoundCloud/Bandcamp item. Do not put signed or private media URLs in the result report. A precise text query is an alternative for the yt-dlp path.

The commands below belong only to the separate-bot alternative, not the chosen existing-bot maintenance window. They are for the operator, from that isolated directory, and have not been run by this preparation task:

```sh
docker compose -p momokun-voice-smoke config --quiet
docker compose -p momokun-voice-smoke build bot
docker compose -p momokun-voice-smoke run --rm --no-deps bot --config /app/config/config.toml --check
docker compose -p momokun-voice-smoke up -d
docker compose -p momokun-voice-smoke ps
docker compose -p momokun-voice-smoke logs --tail 100 bot
```

Confirm `Bot ready`, zero unexpected restarts and `/ping` in the private guild. Offline `--check` migrates only the fresh test volume and does not authenticate the token. Do not run `docker compose config` without `--quiet` when collecting evidence because its expanded output contains private values.

## Focused observation checklist

This observation checklist applies to either route. For the existing bot, retain its already-enabled Music policy and use only the approved channel; do not repeat separate-bot setup or change global/provider configuration. Complete these in order. Write down pass/fail and observable results; a successful command response or `Music playback started` log alone is not proof that a person heard audio.

| Step | Action | Expected observation |
| --- | --- | --- |
| Activate | Server manager runs `/module enable music`, then listener joins the test voice channel and runs `/join` | Bot visibly joins the correct channel without a permission error |
| Direct audio | `/play input:<authorized-public-audio-url>`, then `/nowplaying` | Listener hears intelligible audio for at least 30 seconds; status/provider correspond to the selected source |
| Controls | `/pause`, wait, `/resume`; optionally use the displayed Pause/Resume buttons | Audio stops and resumes without duplicate playback or a dead control |
| Seek response | `/seek seconds:10` | Record actual support: current bounded HTTP streams advertise non-seekable input, so a safe `This track cannot seek to that position` response is acceptable; playback should remain usable. Only claim a successful seek if audibly observed |
| Queue | Queue another item with `/play input:<second-item>`, inspect `/queue show`, then `/skip` | Queue advances once and the next item is audible |
| Website resolver | Play the authorized supported-site item/text query | yt-dlp resolution succeeds and audio is heard; distinguish upstream challenge/format errors from negotiated voice failing |
| Membership change | While playing, a second normal Discord client joins and then leaves the same channel | Both present listeners hear audio; the remaining listener continues hearing it after the membership change |
| Reconnect | `/leave`, confirm disconnect, then `/join` and play again | Bot can establish a fresh voice session and audio is heard again |
| Active shutdown | While audio is heard, run the isolated stop command below | Audio stops, bot leaves voice, container exits within the 20-second grace interval, and logs do not show `Graceful shutdown budget exceeded` |
| Restart | Start only the stopped test bot, wait for readiness, join/play again | Startup is clean and a new voice session is audible; old queue/audio is not expected to resume automatically |

These stop/start examples apply exclusively to the separate-bot alternative. The chosen existing-bot route uses the reviewed original project/service commands from its maintenance-window plan.

```sh
docker compose -p momokun-voice-smoke stop -t 20 bot
docker compose -p momokun-voice-smoke ps -a
docker compose -p momokun-voice-smoke logs --since 10m bot
docker compose -p momokun-voice-smoke start bot
```

Use `/stop` to clear playback while remaining connected, or `/leave` to clear and disconnect after the test. Stop the isolated stack with `docker compose -p momokun-voice-smoke stop` when finished. Retain the test volume until results are reviewed; no volume deletion or production cleanup is part of this checklist.

The second client's membership change exercises real-session group changes; without protocol instrumentation it does **not** independently prove which DAVE epoch/protocol was negotiated. Record the observed behavior and rely on the existing offline harness for its precise cryptographic assertions. Optional Easynews/TorBox playback can be tested separately by the account owner, but is unnecessary to establish the baseline voice dependency smoke and must not be silently reported as tested.

## Failure triage and evidence

Record candidate commit, image ID, host OS/architecture, Discord client type/version, date/time, pass/fail per step, whether direct audio and website resolution differ, readiness/restart/shutdown observations and any safe fixed error category. Do not record token values, user/server identifiers, raw auth errors, credentials, signed media links, raw voice packets or private configuration. Review logs privately before sharing a redacted excerpt.

If direct audio works but a website fails before `Music audio resolved`, investigate yt-dlp/provider availability first. If both resolve but no listener hears anything, check voice permissions, listener mute/deafen state, outbound UDP and voice negotiation. Do not declare DAVE compatibility from queue metadata alone. During the chosen window, do not broaden the approved candidate trial to diagnose a failure; use its rollback plan. A pass requires actual audible playback and a clean active shutdown/restart; incomplete steps remain explicitly untested.

## Unavoidable operator actions

For the chosen existing-bot route, the operator must agree the maintenance window, verify image/data rollback readiness and the candidate registry/schema scope, supply access to the existing private configuration without disclosing secrets, and authorize the concrete bot-only switch. No new application or token is required. The separate-bot alternative instead needs a new application/token and invite. At least one human listener must verify audible audio; a second listener/client is needed for the membership-change observation. These cannot be replaced by the credential-free offline CI checks, and were not performed during documentation preparation.

## 2026-11-24 trial evidence

The owner explicitly authorized separate server preparation and a 20-minute
existing-bot trial with return to the original image. Read-only preflight found
all three deployed migration checksums match the candidate's canonical Git
bytes; all inspected feature-module source files match. The running image was
retained under a private rollback tag, and fresh online backups passed SQLite
integrity/foreign-key checks. No credentials or private paths are included here.

The first Windows-created Git archive used CRLF SQL files; offline startup on
the restored copy correctly rejected their checksums. No live switch occurred.
Re-creating the archive with `git -c core.autocrlf=false archive` restored the
canonical LF bytes; all three checksums were checked before transfer. The rebuilt
candidate then passed restored-copy integrity and offline application startup.
An explicit migration LF rule prevents this checkout/build inconsistency.

A raw Compose JSON comparison initially suggested credential drift. Docker's
escaped-dollar serialization explained it; normalized values matched. Temporary
credential overrides were removed. The final switch verified that all sensitive
runtime environment values and DATABASE_URL match the original process, without
printing them or editing the original environment file.

The authorized candidate reached readiness with command synchronization disabled,
unchanged provider service and authoritative storage. A private watchdog will
restore the retained original image/configuration after 20 minutes if manual
rollback has not completed. It did so successfully when the window expired.
Original image identity, readiness and zero restart count were verified after
rollback; authoritative data and the provider service were preserved. No
playback-start or audio-resolution events were recorded in the window, and no
human audible result was supplied. Audible playback, controls, membership
changes, reconnects and active-audio shutdown remain untested for this build.
The trial proves readiness, unchanged runtime credentials, schema-compatible
startup and timed image/configuration rollback; it does not prove voice output.
