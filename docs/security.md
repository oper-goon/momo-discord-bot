# Dependency and media security policy

Review date: 2026-10-14. Next required exception review: 2026-12-20.
Every dependency change must re-evaluate these conclusions. CI blocks new
vulnerabilities, yanked dependencies, unmaintained packages, and unapproved
licenses unless a specific reviewed exception is in `deny.toml`.

## Dependency decisions

The 2026-11-24 release follow-up updates the lockfile to Tokio 1.54.1,
rustls-webpki 0.103.17, webpki-roots 1.0.12 and url 2.5.9 (with
form_urlencoded 1.2.3). These remove RUSTSEC-2026-0386, -0382, -0381 and
-0592 from the advisory gate. Updating pin-utils to 0.1.1 does not resolve its
unmaintained status. The remaining gate failures are chrono, pin-utils,
crypto-bigint 0.5.5 and bundled libsqlite3-sys 0.37.0. No new exception was
added. See `docs/audit/dependency-remediation.md` in the repository for
dependency chains and remaining work.

- Serenity and Songbird use native TLS, removing Serenity 0.12.5's old
  rustls-webpki 0.102 chain. Linux deployments must keep system OpenSSL updated.
  Direct media HTTP uses the current rustls chain through reqwest. The 0.9.0
  release updates Rustls from 0.23.45 to the advisory-patched 0.23.46; existing
  vendored backports and reviewed exceptions remain unchanged. The yanked
  yoke-derive 0.8.3 is replaced with compatible 0.8.4.
- `libcrux-chacha20poly1305` 0.0.7 is vendored with the upstream 0.0.8
  overlong-output-buffer panic fix for RUSTSEC-2026-0124 because Songbird 0.6's
  HPKE chain pins the older API. See `vendor/README.md`. The public API and
  cryptographic output are unchanged. Cargo-deny cannot associate registry
  advisories with path dependencies, so the regression guard remains required.
- `libcrux-secrets` 0.0.5 is vendored with the upstream four-instruction fix for
  RUSTSEC-2026-0212. See `vendor/README.md`. No affected instruction remains.
  Cargo-deny does not scan this path dependency as a registry package; review
  its provenance and the regression guard separately. Local tests do not
  execute the ARM assembly.
- `ringbuf` 0.4.8 is vendored with the upstream 0.5.2 panic-safe consumer
  removal guard for RUSTSEC-2026-0293 because Songbird 0.6 still requires the
  0.4 API. See `vendor/README.md`. Cargo-deny cannot associate registry
  advisories with path dependencies, so the source regression guard and
  provenance review are mandatory until Songbird supports 0.5.2 or later.
- RUSTSEC-2026-0207 and RUSTSEC-2026-0208 in libcrux-sha3 0.0.8 have narrow
  reachability exceptions: its only active consumer, hpke-rs 0.6.1
  `src/kem.rs` lines 154 and 158, calls the one-shot `shake256::<32>` and
  `shake256::<64>` functions. It does not use repeated incremental squeezes,
  and both fixed output sizes are divisible by eight. The affected APIs and
  output sizes cannot be requested by bot users. A dependency update or new
  consumer invalidates this review; do not broaden these exceptions.
- Unmaintained `derivative` and `proc-macro-error2` are transitive build-time
  macros. `instant` is required through OpenMLS/parking_lot and uses native
  time on supported deployments. These are maintenance risks, not claims
  that the packages are maintained. Track upstream removal and review by the
  date above. No arbitrary future advisory is ignored.
- CDLA-Permissive-2.0 is allowed for Mozilla trust-root data in webpki-roots.

## Media boundaries

Startup update checks use a fixed HTTPS GitHub release API endpoint, with no
authentication, proxies, or redirects. The entire check is limited to three
seconds and 64 KiB of response data. Only canonical numeric stable version tags
are rendered; remote release bodies and error payloads are never logged. Failures
are nonfatal, and the offline `--check` path makes no release request.

Direct HTTP resolves and validates all DNS addresses, pins the checked result,
disables proxies and redirects, rejects credentials and nonstandard ports,
caps each audio stream at 64 MiB, and applies connection/read timeouts. Local
files are disabled. Playback begins after bounded header probing and resumes
interrupted HTTP streams with byte-range requests when the source supports it;
files above 64 MiB remain unsupported.

yt-dlp runs without a shell or operator configuration, with bounded JSON,
diagnostics, playlist size, and time. Supported extractor sites are YouTube,
SoundCloud, and Bandcamp. Resolution of arbitrary website extractors is not
enabled. The resolved media URL passes the same pinned public HTTP checks.
For a hostile multi-tenant deployment, also enforce outbound network policy
around the yt-dlp subprocess; its own website traversal cannot be DNS-pinned
by the application.

The Download module separately invokes yt-dlp without a shell or operator
configuration. It rejects playlists and live media, applies time, duration,
concurrency, site, and byte ceilings, and writes each job into an isolated
temporary directory. Slash invocations use Discord's reported attachment
limit, bounded by the operator ceiling. Prefix invocations fall back to the
documented guild boost-tier limit because message events do not report an
attachment limit. Discord attachment fetches accept only pinned public HTTPS
addresses on Discord CDN hosts. Temporary media, source URLs, resolver output,
and signed attachment URLs are neither persisted nor logged.
The container pins Deno and installs yt-dlp's matching EJS scripts for YouTube
challenge solving. Deno update checks and prompts are disabled, and yt-dlp
invokes the runtime with its restricted default permissions. No browser
cookies or Google account credentials are supplied.
The Compose deployment also pins the BgUtils PO-token plugin and provider to
the same release. The provider is reachable only on the private Compose network;
it publishes no host port. It receives bounded requests from yt-dlp and does
not receive Discord credentials, browser cookies, or Google account credentials.
Its container logging is disabled because upstream diagnostic output can include
generated, short-lived tokens.

Media Toolbox conversions run FFmpeg and FFprobe without a shell inside the
Download module's isolated temporary workspace. They inherit the same source
allowlist, DNS checks, single-job concurrency, cancellation, timeout, input
ceiling, final upload-size check, and cleanup behavior. FFprobe requests only a
fixed set of technical fields and caps its JSON output. FFmpeg diagnostics are
bounded and never returned or logged verbatim.

Utility bookmarks are an explicit persistence action. They store the invoking
user ID, message location and author IDs, a short text excerpt or attachment
name, and an optional note until the user deletes the bookmark. Bookmark lists
and searches are ephemeral and ownership-scoped. Message reminders similarly
store a bounded excerpt and jump link. They do not copy attachments or embeds.

Do not commit bot tokens, local TOML, `.env`, SQLite files, resolver outputs,
or signed media URLs. Default logs omit command arguments and dependency
error payloads. Debug logging of third-party crates may expose extra data.

TorBox uses a fixed HTTPS API origin and reads `TORBOX_API_KEY` only from the
environment. Magnet arguments, API responses, generated file links, and stream
tokens must not be logged or persisted. Generated links are sent only in
ephemeral Discord responses. Keep the module disabled or permission-restricted
outside trusted guild roles because those links consume the operator's TorBox
account and bandwidth. Interactive file-picker sessions are memory-only,
owner-bound, limited to five minutes and 250 eligible files, and capped at 64
concurrent sessions. The magnet is not retained in picker state, infected files
are excluded, and temporary URLs are generated only after the user selects a
file.
The bulk-download control requests TorBox's on-the-fly ZIP only when every file
in the torrent is uninfected; it is omitted if any file is flagged.

Image Search uses fixed HTTPS DuckDuckGo and Bing search origins. DuckDuckGo
does not publish a supported image-search API; a 403 or 429 falls back to
Bing's strict SafeSearch endpoint, while other endpoint changes fail closed
without exposing response bodies. Queries and response payloads must not be
logged or persisted. Remote images are referenced in Discord embeds rather
than fetched by the bot, and responses are bounded before token extraction,
JSON parsing, or HTML metadata parsing. Keep the module disabled or
permission-restricted where automated upstream traffic must be controlled.

Social Embeds processes guild message content only while its per-guild module
is enabled. It rewrites recognized public HTTPS social-post URLs to a fixed set
of embed providers and never logs or persists message content, source URLs, or
webhook tokens. Provider pages needed for Instagram selection or missing author
metadata are fetched with a Discord crawler user agent, strict time and
response-size bounds, no proxy, and no redirects; only fixed, code-reviewed
hosts or operator-selected Instagram hosts are accepted. Each provider receives
the source post path, so operators should trust every configured provider.
Redirect responses from approved direct-media Instagram providers are not
followed by Momo. They qualify only when the target is HTTPS on an approved
Instagram CDN, has a recognized image or video extension, and matches the
source path's required media kind (for example, reels require video).
The manual Repair Social Embed message action may delete and replace the
selected message only when invoked by its author or a member with Manage
Messages. Normal module permission policy and current Discord permissions are
re-evaluated for every invocation.
Webhook reposts disable allowed mentions, add `via Momo` to the sender's
display name, and reuse the sender's public Discord avatar. The original is
deleted only after a successful repost; if deletion fails, Momo removes the
new webhook message to avoid duplicates. Messages with attachments, stickers,
or polls are not reposted. The module requires Message Content plus Discord's
Manage Messages and Manage Webhooks permissions and is disabled by default.

Media lookup sends bounded title queries only to fixed HTTPS AniList and TMDB
API origins. It disables proxies and redirects, bounds each response to 512 KiB,
limits total lookup time to 25 seconds and concurrent lookups to two, and never
logs or persists queries, provider payloads, or TMDB_READ_ACCESS_TOKEN. Only
numeric provider IDs and validated trailer IDs form external links. Provider
HTML is stripped and Discord markdown is escaped and length-bounded. The bot
does not fetch trailers or artwork. TMDB read tokens use Authorization headers; TMDB_API_KEY uses the provider's
required api_key query parameter. HTTP errors are replaced with safe messages
so credential-bearing URLs are never returned or logged. Missing credentials disable movie/TV lookup without affecting
anime or offline startup validation. No dependencies were added to the workspace.

Media cover artwork is referenced in Discord embeds, never fetched by Momo.
Only HTTPS AniList cover URLs on s4.anilist.co and validated TMDB poster paths
on image.tmdb.org are accepted. Search selection menus retain at most ten
bounded result previews per session, with 64 sessions and a five-minute expiry.
Selections are single-use and bound to the invoking user, guild, and channel;
module activation and the originating search command's current permission policy
are rechecked. Selected details share the normal provider concurrency limits.
