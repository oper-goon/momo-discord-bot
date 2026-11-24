# Troubleshooting

Start with `docker compose ps` and `docker compose logs --tail 100 bot`. Inspect
logs privately. Do not share `.env`, expanded `docker compose config`, debug
output, signed URLs, message excerpts or database files in public issues.

## The bot is offline

- Confirm Docker is running and the container has not exited or entered a restart
  loop. A running container alone does not prove Discord connection.
- Check for `Bot ready`. Verify the token in Developer Portal; reset an invalid
  token there, update `.env`, then recreate only the bot with
  `docker compose up -d --force-recreate bot`.
- If Discord rejects intents, enable Message Content Intent. The current composed
  application requests it even if Social Embeds is disabled in a server.
- Verify outbound DNS, HTTPS/gateway connectivity and host clock. No inbound
  port needs opening. Never paste a token into a command intended for sharing.

## Commands do not appear or are rejected

Check that the application was invited with `bot` and `applications.commands`
scopes. In `guild` mode, your server ID must be in
`commands.development_guilds`; `none` mode does not synchronize application
commands. Global updates can take time to appear. Restart the client and check
registration logs before forcing synchronization. For an intentional forced sync,
run the bot with `--sync-commands`; do not start a second production instance.

Use `/module list`, `/module info music`, `/permissions explain play` and
`/module channels list music` to check feature activation, policy and channel
limits. Prefix commands need both global `discord.prefix_commands = true` and
the server prefix enabled, plus Message Content intent. The default is slash-only.

Check the invoking user's Discord permissions, the bot's permissions and channel
overrides. Role actions cannot target equal/higher roles. Music additionally
checks voice/DJ policy. Administrative bypass of channel restrictions is separate
from the configurable custom-permission bypass. Consult each command's reference.

## Docker build or startup fails

Create `.env` and the regular file `config/config.toml` before startup. A missing
bind-mounted file deliberately fails rather than becoming an empty directory.
Replace the token placeholder; set global registration or a real private guild
ID. Check free disk, memory and connectivity to image/package registries.

```sh
docker compose build
docker compose run --rm --no-deps bot --config /app/config/config.toml --check
```

The check initializes storage/migrations and does not authenticate credentials.
Run it on a test copy when investigating migration failures. A source build can
require much more memory than the runtime; there is no measured minimum resource
guarantee. ARM64 is unverified.

## Database errors or lost settings

SQLite is local, not a network database. Check disk space, the named volume,
UID 10001 write access, and the configured path. Compose uses `/data/bot.db`;
native operation defaults to `data/bot.db` relative to its working directory.
Changing the Compose project directory/name can select a new empty volume.

For a migration checksum mismatch, stop and retain a verified backup. Do not
edit an applied migration, delete `_sqlx_migrations`, or remove the database to
hide the error. Compare the binary/source and migration history and reproduce
against a copy. See [backup and restore](self-hosting/operations.md).

Queues and interactive search/picker sessions are in memory and expire on restart.
Saved playlists, music settings, reminders and bookmarks persist. Reminder delivery
also depends on Discord permissions and reachable target channels.

## External services fail

Check whether the module is enabled, its optional credential is configured, and
the account has the required subscription. TMDB token/key lookup affects movie/TV,
not AniList anime. TorBox cached-file commands share the operator's account;
stream API availability is subscription-gated. Easynews uses an unofficial
interface and can change without notice. Image search is also an unsupported
upstream interface and can be rate-limited.

Do not post provider response bodies, Basic Auth URLs, magnets or generated links.
Report safe error categories, module name, bot version and approximate time.
Provider outages do not justify disabling security bounds or retries indefinitely.

## Music or downloads fail

Check Connect/Speak permissions, the correct voice channel and UDP egress. Run
`/music providers`, `/music status`, `/nowplaying` and `/queue show` to inspect safe
local status. Provider "ready" does not establish that account authentication works.

YouTube extraction can fail when upstream challenges change. Compose includes
the pinned tools and a private PO-token provider; native installs must provision
them separately. Verify executable availability privately (`yt-dlp --version`,
`deno --version`, `ffmpeg -version`) and use the release's pinned versions.
Restricted/live content, unsupported extractors and missing formats can fail.

Direct playback has a 64 MiB ceiling and rejects unsafe addresses. Downloads and
conversions have duration, size, concurrency and timeout limits plus Discord's
upload ceiling. Prefix downloads estimate upload allowance from guild boost tier.
Do not remove limits to make one file work; choose a supported, smaller source.

If a problem persists, open a redacted issue using the repository templates.
Security reports belong in the private [security process](project/security.md).

## SQLite reports a modified migration

Stop the upgrade if SQLx says a previously applied migration was modified. Do
not edit `_sqlx_migrations`, replace its checksums or change applied SQL files.
SQLx hashes exact bytes, so CRLF versus LF line endings can cause this error
even when the visible SQL is identical. The repository requires LF migrations.

On Windows, use a fresh checkout that honors `.gitattributes`. When producing
a source archive for a Linux build, use canonical bytes:

```sh
git -c core.autocrlf=false archive --format=tar --output=candidate.tar HEAD
```

Keep your original working image and a verified backup. Compare the candidate's
canonical migration hashes with the database's applied hashes on a restored
copy before switching services. If an older installation applied different
bytes, retain its matching original build and review recovery separately; do
not force the new build to accept altered history.
