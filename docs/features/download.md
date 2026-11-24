# Downloads and conversions

Download retrieves supported public media and uploads a bounded result to Discord. Its Media Toolbox can inspect files, extract MP3 audio, compress video, create a short GIF, or take a thumbnail. Enable it with `/module enable download`; it is disabled by default because media work consumes bandwidth, CPU and temporary disk space.

## Setup and commands

Docker includes yt-dlp, Deno, FFmpeg and FFprobe. Native installations must supply them; `YT_DLP_PATH` or `[download].yt_dlp_path` selects yt-dlp. No paid account or browser cookies are required. YouTube requests use the private PO-token provider included in Compose.

```text
/download url:https://www.youtube.com/watch?v=PUBLIC_VIDEO_ID
/media info
/media audio
/media compress
/media gif
/media thumbnail
```

Replace the example ID with a public video you are authorized to download. When URL is omitted, commands inspect a replied message and then recent channel history. **Apps → Download Media** acts on the selected message. Supported sources include configured public site URLs and validated Discord attachments. The bot needs Send Messages and Attach Files; history-based discovery also needs access to the relevant messages. See the [command reference](../reference/commands.md).

## Configuration and safety limits

`[download]` sets the yt-dlp path, history scan size, timeout, maximum duration, absolute byte ceiling and allowed domains. Defaults scan 25 messages, allow 180 seconds per bounded operation, cap duration at 900 seconds and set a 500 MiB absolute ceiling. Actual uploads also respect Discord's reported attachment limit; prefix requests use the server boost-tier fallback. The domain allowlist includes YouTube, TikTok, X/Twitter, Reddit and Instagram. It does not promise every post on those sites is downloadable.

Jobs share a single concurrency permit and use isolated temporary directories. Playlists and live streams are rejected. GIF conversion uses the first ten seconds. Private/local addresses, unsafe source URLs and disallowed sites are rejected rather than fetched. See the [configuration reference](../reference/configuration.md).

## Common problems

A large file can be below the operator ceiling but still exceed Discord's smaller upload limit. Try a shorter source or compression where appropriate. Private, login-only, age-restricted or unavailable media may fail; the bot does not import your browser session. If the service reports yt-dlp unavailable, check the image build or native executable path. If the bot is busy, wait for the current bounded job. Repeated upstream extraction failures may require a reviewed yt-dlp update and rebuilt image. Do not post signed attachment URLs or raw dependency errors in public support requests.
