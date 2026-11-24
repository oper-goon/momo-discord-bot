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
