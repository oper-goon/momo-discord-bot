# Dependency and media security policy

Review date: 2026-09-20. Next required exception review: 2026-12-20.
Every dependency change must re-evaluate these conclusions. CI blocks new
vulnerabilities, yanked dependencies, unmaintained packages, and unapproved
licenses unless a specific reviewed exception is in `deny.toml`.

## Dependency decisions

- Serenity and Songbird use native TLS, removing Serenity 0.12.5's old
  rustls-webpki 0.102 chain. Linux deployments must keep system OpenSSL updated.
  Direct media HTTP uses the current rustls chain through reqwest.
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

Do not commit bot tokens, local TOML, `.env`, SQLite files, resolver outputs,
or signed media URLs. Default logs omit command arguments and dependency
error payloads. Debug logging of third-party crates may expose extra data.

TorBox uses a fixed HTTPS API origin and reads `TORBOX_API_KEY` only from the
environment. Magnet arguments, API responses, generated file links, and stream
tokens must not be logged or persisted. Generated links are sent only in
ephemeral Discord responses. Keep the module disabled or permission-restricted
outside trusted guild roles because those links consume the operator's TorBox
account and bandwidth.

Image Search uses DuckDuckGo's fixed HTTPS search and image-result origins.
DuckDuckGo does not publish a supported image-search API, so endpoint changes
must fail closed without exposing response bodies. Queries and response
payloads must not be logged or persisted. Strict SafeSearch is always
requested, remote images are referenced in Discord embeds rather than fetched
by the bot, and responses are bounded before token extraction or JSON parsing.
Keep the module disabled or permission-restricted where automated upstream
traffic must be controlled.
