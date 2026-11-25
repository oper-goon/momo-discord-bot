# Anime, films and TV

Media looks up titles and presents artwork, a synopsis, score, studios/production companies, genres, status and an available trailer. It helps a server discuss a title without leaving Discord. Enable it with `/module enable media`; it is disabled by default.

## Providers and setup

Anime uses AniList without an API key. Film and TV use TMDB and require a private `TMDB_READ_ACCESS_TOKEN` or supported v3 `TMDB_API_KEY`. Obtain your own credentials under your TMDB account's API settings and keep them in the environment or private Compose `.env`, never TOML. A read-access token takes precedence when configured. No paid subscription is required by the bot itself; provider terms and account requirements still apply.

There are no Media-specific TOML settings. The bot needs Embed Links and channel access. Queries are sent to the chosen provider. See the [configuration reference](../reference/configuration.md).

## Commands and examples

```text
/anime query:Cowboy Bebop
/movie query:The Matrix
/tv query:Severance
/animesearch query:Fullmetal Alchemist
/moviesearch query:Dune
/tvsearch query:The Office
```

The three lookup commands return the provider-ranked best match. The three `search` commands let you select among matching titles instead, useful for remakes or ambiguous names. Search pickers belong to the requester and expire after five minutes. Commands support slash and enabled prefix forms. Slash responses are private; prefix results are visible in the channel. See the [command reference](../reference/commands.md).

## Limits and common problems

Search/lookup commands have a five-second per-user cooldown and share a two-request concurrency limit. Title queries must be 1–200 characters. AniList uses scores out of 100; TMDB uses scores out of 10. Missing fields or trailers display as unavailable rather than invented values.

AniList excludes adult titles and TMDB search excludes adult results, but provider classifications cannot guarantee suitability for every audience. A wrong match is best handled through the corresponding search picker or a more precise query. Movie/TV failures with working anime lookup often indicate absent/invalid TMDB credentials or provider limits. Expired controls require a fresh search. Upstream throttling and outages should be retried later rather than bypassed.

TMDB attribution appears in results; Momokun is not endorsed or certified by TMDB. This module provides metadata and trailer links, not access to full films, episodes or anime streams.
