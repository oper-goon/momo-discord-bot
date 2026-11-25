# Image search

Image Search returns safe-filtered web image results and lets the requester move through them with Previous and Next buttons. Enable it with `/module enable image`; it is disabled by default.

## Setup and use

No account, API key, subscription or module-specific TOML configuration is required. The bot uses DuckDuckGo with a Bing fallback through unofficial web interfaces. These are implemented integrations with experimental upstream compatibility, not supported public image-search API contracts. The bot needs Embed Links.

```text
/image query:black cat
```

Slash replies are private. Enabled prefix commands also work, with results visible in the channel. Up to ten results are retained per search; controls belong to the person who searched and expire after five minutes. Identical searches can reuse a bounded ten-minute result cache. The command has a five-second user cooldown and searches have a four-request concurrency bound. See the [command reference](../reference/commands.md).

## Limits and common problems

Safe-search filtering reduces exposure but cannot guarantee every result is suitable. The bot sends your query to third parties and displays remote artwork; search results do not grant copyright permission to reuse images. Results and navigation sessions are held in memory and do not survive restart.

If no safe results appear, try a clearer query. An expired picker needs a new search. If DuckDuckGo challenges the request, the implementation can try Bing; both may still be unavailable or change response formats. Repeated failures should be investigated as upstream compatibility problems instead of solved by adding credentials or disabling safety checks. No hidden API-key setting exists; review the [configuration reference](../reference/configuration.md) for actual supported variables.
