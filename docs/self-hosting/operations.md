# Operations and recovery

Run commands from the directory containing `compose.yaml`. Back up before updating: database migrations are forward-only and an older binary may not understand a newer schema.

## Logs, restart, stop

```sh
docker compose ps
docker compose logs --tail 100 bot
docker compose restart bot
docker compose stop bot
```

`restart` does not rebuild images or reload changed Compose environment settings. After changing `.env` or TOML, use `docker compose up -d --force-recreate bot`. Do not use `down -v`: it deletes named volumes and your database. The project directory name determines the default volume prefix; changing it can make an existing database appear missing.

## Create a consistent backup

Do not copy only a live `bot.db`: committed changes may still be in its WAL file. The image includes SQLite's online backup tool:

```sh
docker compose exec bot sqlite3 /data/bot.db ".backup '/data/backups/manual.db'"
docker compose exec bot sqlite3 /data/backups/manual.db "PRAGMA integrity_check;"
mkdir -p backups
docker compose cp bot:/data/backups/manual.db ./backups/manual.db
```

Expect `ok`. Use a different backup filename each time and keep dated copies off the host, with access limited to trusted operators. Back up your private `.env` and `config/config.toml` separately and securely. Database backups contain user/server data and are not public release artifacts.

## Update from source

Read the release notes first and make a backup. Use a clean checkout of the approved release; do not discard local edits.

```sh
git fetch --tags
# Select an existing, reviewed release tag before rebuilding.
git switch --detach <release-tag>
docker compose build --pull
docker compose up -d
docker compose ps
docker compose logs --tail 100 bot
```

Replace `<release-tag>` with a real tag. Rebuild/recreate the bot service for bot-only updates with `docker compose up -d --no-deps --build bot`. Confirm `Bot ready`, `/ping`, restart count, and persistence of a known reminder/bookmark. Keep backups until the new version is verified. A rollback after migrations may require a separately tested restore rather than merely switching images.

## Restore into a separate volume first

A restore is an operator decision, never an automatic update step. Stop the bot, retain its original volume, and validate a backup in a disposable volume before replacing production data. This example creates an isolated volume and copies a backup using a temporary container built from the local image:

```sh
docker volume create momo-restore-test
docker create --name momo-restore-copy --volume momo-restore-test:/data --entrypoint /bin/true "$(docker compose images -q bot)"
docker cp ./backups/manual.db momo-restore-copy:/data/bot.db
docker rm momo-restore-copy
docker compose run --rm --no-deps --user root --cap-add CHOWN --volume momo-restore-test:/restore --entrypoint sh bot -c 'chown -R 10001:10001 /restore'
docker compose run --rm --no-deps --volume momo-restore-test:/restore --entrypoint sqlite3 bot /restore/bot.db 'PRAGMA integrity_check;'
docker compose run --rm --no-deps --volume momo-restore-test:/data bot --config /app/config/config.toml --check
```

These are POSIX-shell commands and have not been executed in this environment. The final check may migrate the test copy. Do not attach an unverified backup to a running production instance. After validation, plan a stopped maintenance window and explicitly select the restored volume in a private Compose override. Retain the original volume and a verified backup; replacing volumes is deliberately not automated here.

## Diagnose startup problems

| Symptom | Action |
| --- | --- |
| Compose says token is unset | Create `.env` beside `compose.yaml` and replace the placeholder. |
| Mount error | Create the regular file `config/config.toml`; the bind mount deliberately refuses to create a missing directory. |
| Guild command mode error | Fill `commands.development_guilds` with your real server ID or use global mode. |
| Bot offline / invalid token | Check recent logs privately; reset an invalid token in Developer Portal, update `.env`, and recreate the bot. |
| Disallowed gateway intents | Enable Message Content Intent for this application in Developer Portal. |
| SQLite cannot open file | Check volume permissions for UID 10001 and free disk space; do not delete the database. |
| Commands missing | Check installation scopes and registration mode, then wait for global registration. |
| Permission errors | Check bot role ordering and channel overrides as well as the command user's permissions. |
| Media/API errors | Check DNS/HTTPS access, provider credentials/subscription, provider availability, and whether the feature is enabled. Do not post signed URLs. |
| Music silent | Check Connect/Speak, voice UDP access, queue state, and current yt-dlp/provider health. |
| Build fails | Check Docker disk/memory, registry access, and the pinned Rust/tool images. Preserve the first useful error without secrets. |

No application liveness health check is implemented. Container running state alone does not prove Discord readiness. Avoid repeatedly running `--check` against a live database as a substitute for monitoring.



