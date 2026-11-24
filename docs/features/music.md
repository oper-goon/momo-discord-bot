# Music

Music plays audio in Discord voice with an independent queue per server, interactive controls, repeat/shuffle and saved playlists. Enable it with `/module enable music`, join a voice channel, and give the bot Connect and Speak there.

## Play and manage a queue

```text
/play input:Artist - Track Title
/play input:tyler the creator sugar
/nowplaying
/queue show
/pause
/resume
/skip
/playlist save name:Evening
/playlist play name:Evening
/music providers
/music status
```

Other controls include `/join`, `/leave`, `/stop`, `/seek`, `/volume`, `/repeat`, `/shuffle`, queue remove/move/clear, and playlist list/delete/move/remove. Stop clears playback without disconnecting; leave also disconnects. Controls enforce voice-channel and configured DJ policy. Saved playlists persist metadata, not audio files or expiring private URLs. See the [command reference](../reference/commands.md).

## Sources and optional accounts

| Source | Setup | Important limit |
| --- | --- | --- |
| YouTube, SoundCloud, Bandcamp | yt-dlp; included in Docker | Upstream extraction can change or reject a request |
| Public HTTP audio | Public validated media URL | Private/local addresses and unsafe redirects are rejected |
| Easynews | Subscription, `EASYNEWS_ENABLED=true`, private `EASYNEWS_USERNAME` and `EASYNEWS_PASSWORD` | Unofficial web-search interface; standalone audio only |
| TorBox music | Account/API key, `TORBOX_ENABLED=true` | Existing operator-account library and confirmed cached safe files only |

Easynews and TorBox are optional and disabled by default. Configure credentials only in private environment variables; Compose forwards the private `.env` file. `TORBOX_CACHED_ONLY` must remain true. The separate TorBox command module has its own enablement and does not need enabling for music fallback.

`[music]` configures provider priority, timeouts, circuit cooldown and match confidence, yt-dlp path, queue/import bounds, default volume and disconnect timers. Environment overrides include `MUSIC_PROVIDER_PRIORITY`, `MUSIC_PROVIDER_TIMEOUT`, `MUSIC_PROVIDER_COOLDOWN` and `MUSIC_MATCH_MIN_CONFIDENCE`. Per-server `/config` adjusts supported player settings and DJ role. Defaults include a 500-track queue limit, 100-track playlist import limit, 100% volume, 300-second idle disconnect and 120-second empty-channel disconnect. See the [configuration reference](../reference/configuration.md).

## Fallback and reliability

Text discovery follows configured provider priority. At playback, the queued provider is tried first and remaining configured providers can supply a conservative match. `Artist - Title` gives the strongest identity; include live/remix words only when you want that version. Matching is based on metadata rather than audio fingerprints. A blocked provider enters a cooldown so repeated requests do not hammer it. `/music providers` and `/music status` report local configuration and safe failure categories, not authenticated account tests. `/nowplaying` shows the actual active provider.

For initial text discovery, you can supply the artist's full name and part of the title without a separator. For example, `tyler the creator sugar` can select `Tyler, The Creator - Sugar On My Tongue` when the result identifies that artist separately. A title-only YouTube result can also retain its channel name as creator attribution when your natural query explicitly includes every word of that channel name and matching title words. This is creator context, not a verified recording-artist claim; unrelated channels and uploader-only metadata do not qualify. YouTube discovery checks at most five provider-ranked results so an unrelated first hit does not hide a later matching song. Every requested title word must match, and unrequested live, cover, remix or mix versions are still rejected. An explicit `Artist - Title` request and playback fallback keep strict full-recording matching, including available duration metadata. Optional providers can apply additional matching requirements; incomplete or unidentified artist metadata can still require a more precise query. Website playlist imports keep their separately configured limit.

Playback uses Songbird directly with bounded public HTTP streams; there is no persistent audio cache. Streams have a 64 MiB ceiling. Easynews accepts supported individual audio formats, never archives; TorBox music does not import torrents or start peer downloads. Missing metadata, ambiguous matches, unavailable cached files and upstream restrictions can prevent fallback. Review the outstanding live acceptance checks in [release readiness](../project/status.md).

## Common problems

If nothing plays, check voice permissions, caller voice-channel membership, the bot's queue and `/music status`. Check outbound voice UDP and provider HTTPS access. Docker supplies yt-dlp and Deno; native installations must install those tools themselves. YouTube challenges or unavailable formats can fail despite a working bot. Account credentials being configured does not prove subscription validity. Try a precise authorized text query instead of an uninspectable video URL. Audible voice playback, seek behavior and live TorBox music remain release acceptance checks; do not treat mocked provider tests as complete voice validation.


