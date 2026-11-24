# Music provider integration

## Audit and implementation plan (2026-11-24)

The existing `module-music/src/source.rs` owns `AudioSourceProvider`, yt-dlp
search/playlist inspection, just-in-time resolution, and bounded HTTP playback.
`Sources` currently selects the first accepting provider and resolves only that
provider. yt-dlp supports YouTube, SoundCloud and Bandcamp; preserve all three.
Diagnostics are bounded and reduced to safe categories (access challenge,
restricted content, missing format, timeout or generic failure).

`player.rs` owns each guild queue in one actor, cancels stale resolution by
generation, advances failed tracks and retains the pending queue after three
consecutive failures. Resolution concurrency is four, with a five-second permit
wait. There is no disk or memory audio cache to consult. Playback uses
Songbird/Symphonia directly, with DNS-pinned public HTTP, no redirects/proxies,
range resumption and a 64 MiB ceiling. FFmpeg is used by other media features,
not by this music path. No new transcoder or full-file download is needed.

Implementation stages, committed separately:

1. Audit/API feasibility and this plan.
2. Extend the existing provider contract, add normalized matching/configuration.
3. Easynews standalone audio search and authenticated HTTP streaming.
4. TorBox account-library discovery, explicit cache check and single-file links.
5. Bounded fallback and shared circuit breakers without queue reinsertion.
6. Provider diagnostics and active playback provider in Discord.
7. Mock tests, documentation and validation.

## API feasibility

- [TorBox Main API](https://api-docs.torbox.app/) documents `mylist`,
  `checkcached` and `requestdl`. Cache lookup requires known hashes; it does
  not discover all cached music. Initial discovery is restricted to existing
  operator account torrents. No create/control/torrentinfo endpoint is needed,
  so this music integration cannot start a download or contact peers.
- [AIOStreams Usenet guide](https://github.com/Viren070/AIOStreams/blob/main/packages/docs/content/docs/guides/usenet.mdx)
  distinguishes discovery from playback and confirms Easynews direct HTTP.
  Its maintained Easynews integrations and
  [Easynews+](https://github.com/sleeyax/stremio-easynews-addon) are relevant
  references for the web-search response format. Their movie/TV matching is
  not suitable for music and should not be copied.
- Easynews web search is an unofficial interface, not a stable published API.
  Changes must fail closed; archives and ambiguous matches must be skipped.
  Credentials belong only in environment variables, never track metadata.

Only use accounts and content the operator is authorized to access and stream.
Live service and Discord acceptance remain necessary after mocked checks.

## Configuration and operation

Existing `/play <query>` and all queue/playlist commands remain supported.
Use `Artist - Track Title` for the strongest matching, including `live`,
`remix` or other version words only when wanted. `/music providers` and
`/music status` show enabled providers in priority order, configuration
availability and safe failure categories. They do not contact account-auth
endpoints; "ready" indicates configured/local readiness, not verified account
validity. `/nowplaying` displays the actual active provider in an embed.
No per-play provider selector is added; priority is operator-controlled.

Set `[music]` options in `config/example.toml` or these environment overrides:

| Environment | TOML / purpose | Default |
| --- | --- | --- |
| `EASYNEWS_ENABLED` | `easynews_enabled` | `false` |
| `EASYNEWS_USERNAME` | Environment only | unset |
| `EASYNEWS_PASSWORD` | Environment only | unset |
| `TORBOX_ENABLED` | `torbox_enabled` | `false` |
| `TORBOX_API_KEY` | Environment only; shared with existing TorBox module | unset |
| `TORBOX_CACHED_ONLY` | `torbox_cached_only`; false fails startup | `true` |
| `MUSIC_PROVIDER_PRIORITY` | `provider_priority`; comma-separated unique IDs | `youtube,easynews,torbox` |
| `MUSIC_PROVIDER_TIMEOUT` | `provider_timeout_seconds`; 1–120 seconds | `30` |
| `MUSIC_PROVIDER_COOLDOWN` | `provider_cooldown_seconds`; 1–3600 seconds | `120` |
| `MUSIC_MATCH_MIN_CONFIDENCE` | `match_min_confidence`; 0.5–1.0 | `0.85` |

YouTube retains the stored provider ID `yt-dlp` for existing saved playlists.
SoundCloud, Bandcamp and explicit public HTTP audio remain supported. Provider
priority controls text discovery; resolution tries the queued provider first,
then the remaining providers in configured order. Omitting a provider removes
it from music discovery/resolution. Other feature modules are unaffected.

For Docker, put credentials in the host's private `.env` and use:

```sh
docker compose up -d --build bot
docker compose ps
docker compose logs --tail=50 bot
```

The main Compose file forwards the private `.env` file, including optional provider
credentials and overrides. The older `compose.music-providers.yaml` override is
retained for compatibility but is not required for this setup. Verify `Bot ready` and restart counts; never recreate the
data volume. Provider enablement is global configuration; the Music module
still requires per-guild enablement and existing command permissions.

## Reliability and security decisions

Provider activity is logged at INFO with fixed event names, provider IDs,
operation (`search`, `inspect`, `resolve`), safe failure categories and elapsed
milliseconds. Events cover attempts, metadata-cache hits, no-match results,
fallback transitions, cooldown skips, circuit opening/recovery and resolution.
`Music audio resolved` means audio passed initialization; `Music playback
started` is emitted when Songbird reports playback, with the active provider.
Pause/resume events do not duplicate the playback-start record.

Searches have a `search_id`; each resolution pass has a `resolution_id`. Guild
resolutions also inherit a `playback_id` and `player_id`, and the playback ID is
retained across initialization fallback. The same playback ID appears on the
actual playback-start event. Queries, titles, requester IDs, credentials,
headers, raw dependency errors and private URLs are excluded. Automated log
capture tests check redaction and the fallback/start records. Default `RUST_LOG`
already includes these INFO events; no debug logging is required. Inspect with:

```sh
docker compose logs --since 10m bot | grep 'Music '
```

Each provider is serialized behind a bounded wait, inside the global four-job
resolver limit. Search metadata has a bounded one-entry, 30-second cache per
provider; repeated searches share that cached metadata. No audio cache exists
and none is introduced. Independent guilds require independent audio streams.
An attempt has a configured deadline; an error opens its circuit immediately
for the configured cooldown. Numeric HTTP `Retry-After` extends that cooldown.
After cooldown, serialization permits one recovery probe. Search misses do not
open the circuit, while resolution failures do. Each provider is attempted at
most once per fallback pass. Backend initialization failures exclude the failed
provider without re-enqueuing the track; stale voice events are invalidated.
Failures after playback has started retain the existing queue-advance behavior.

Easynews uses `https://members.easynews.com/3.0/api/search`, Basic auth and
`fty[]=AUDIO`. The unofficial schema and direct file URL layout were checked
against [AIOStreams's client](https://github.com/Viren070/AIOStreams/blob/main/packages/core/src/builtins/easynews-search/api.ts).
Only standalone audio extensions are accepted: MP3, FLAC, M4A, AAC, OGG, Opus,
WAV, AIFF and ALAC. Archives are never requested or unpacked. Download origins
must be HTTPS under `easynews.com`. Easynews alone permits up to three manually
validated redirects to its regional download farms, with DNS pinning and public
address checks at every hop. All other media sources still reject redirects.
Credentials are sent only as a sensitive HTTP
header, retained for range resumption on that origin. Queues/playlists contain
an Easynews search marker, never the private download URL. Playback searches
again to avoid persisting expiring URLs. If the unofficial schema changes,
malformed results fail closed.

TorBox searches eligible individual files in at most 1000 existing account
torrents and 1000 files per torrent. It checks an exact hash through
`checkcached`, requires returned file metadata, then refreshes `mylist` and
requires `cached`, `download_present` and `download_finished`. Infected or
unknown-infection-state files are rejected. Only the chosen file is passed to
`requestdl` with `zip_link=false`. The implementation has no create, control,
metadata-from-peers, NZB import, torrent import or polling operation. A cache
miss or ambiguous response skips the result. This deliberately does not add
cached torrents from an external index to the account. Generated links remain
memory-only and undergo the existing pinned public HTTP validation.

Provider responses are capped at 2 MiB. HTTP error payloads, authentication
headers and private links are never returned or logged. Playback retains the
64 MiB stream ceiling and bounded probing. Invalid/corrupt headers fail during
playback initialization; corruption later in a stream can only be discovered
when the decoder reaches it. No full-file integrity download is performed.

## Matching and limitations

Matching requires all requested title/artist tokens, ignores presentation tags
such as "official audio", rejects unrequested version markers, compares
duration when both sources provide it, and penalizes extra title words and
album differences. Accuracy sorts before bitrate. The matching model accepts
optional album, format and bitrate metadata, but existing persisted `TrackRef`
does not carry album or separate artist fields. These are inferred conservatively
from `Artist - Title`; account filenames with insufficient identity are skipped.
Easynews filenames are normalized before matching: leading track numbers and
trailing eight-character mixed hexadecimal release checksums are removed, and
underscore separators are rendered as spaces. The original filename is retained
only for constructing the private file request. Official music-video labels are
presentation metadata and do not reduce a match score. Regression coverage
includes `Conway the machine sarah` and its numbered/checksummed audio files.
Missing duration/album tags cannot prove recording identity. This is metadata
matching, not audio fingerprint verification.

YouTube URLs whose metadata cannot be inspected cannot be converted to an
artist/title fallback reliably; use a text query in that case. Playlists retain
existing inspection/import behavior and fall back per track at playback time.
The Easynews interface, audio availability and TorBox account restrictions need
live operator-account acceptance. No credentials or live media were used by
automated tests. Voice rendering, permissions and command synchronization need
a live Discord acceptance run. Linux Docker validation remains a CI/deployment
gate.

## Validation record

- Full workspace tests passed (including the pre-existing command registry,
  database and media tests); two existing live image-search tests are ignored.
- Music tests use local mock HTTP servers and fake voice backends. They cover
  YouTube success, both fallback orders, persistent block cooldowns and timeout
  recovery, invalid credentials, expired links, corrupt headers, valid WAV
  probing, cache rejection and single-file TorBox retrieval, and queue
  continuity during initialization fallback.
- Workspace Clippy with all targets/features and warnings denied passed.
- Offline `bot-app --check` passed with `DATABASE_URL=sqlite::memory:`. The
  existing local database check failed because migration 2 has a stored checksum
  mismatch. No migration or local database was edited to work around it.
- Cargo-deny fails against the existing dependency set: chrono
  RUSTSEC-2026-0543, crypto-bigint RUSTSEC-2026-0387, libsqlite3-sys
  RUSTSEC-2026-0459, pin-utils RUSTSEC-2026-0441, rustls-webpki
  RUSTSEC-2026-0382, tokio RUSTSEC-2026-0386, url RUSTSEC-2026-0592 and
  webpki-roots RUSTSEC-2026-0381. Bans, licenses and sources pass. This feature
  adds no dependencies or advisory exceptions. Resolve the dependency gate
  before release.
- On 2026-11-24, built the integration on the deployment host against its existing source
  snapshot, authenticated Easynews and successfully resolved/probed an MP3
  through the Rust fallback path with yt-dlp deliberately unavailable. This
  exposed the regional redirect requirement described above. Workspace tests
  and focused strict Clippy passed after the redirect fix.
- Deployed to the existing the deployment host `bot` service with YouTube first, Easynews
  enabled and TorBox music disabled. The production configuration, migration-3
  database and composed registry passed offline validation, startup reached
  `Bot ready`, and the initial automatic restart count was zero. The other
  Compose service was not recreated. The prior image is retained as
  `<private-rollback-image>`; an online database backup at
  `/data/backups/<private-backup>.db` passed integrity checking. Original
  changed source files and Compose configuration are backed up under
  `<private-source-backup-directory>` on the deployment host.
- This live probe verifies HTTP retrieval and decoder initialization, not
  audible Discord voice playback. Discord listening/controls acceptance and
  live TorBox music checks remain outstanding. Temporary credentials are kept
  only in the deployment host's private host environment file, never in source or this guide.

Modified files are the music module's `source.rs`, `player.rs`, `commands.rs`,
`lib.rs`, new `matching.rs` and `providers/{mod,easynews,torbox}.rs`, core
`config.rs`, `config/example.toml`, `compose.music-providers.yaml`, and this
document. Existing unrelated worktree changes are excluded from feature commits.
