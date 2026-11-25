# Social embeds

Social Embeds repairs supported social-post previews by reposting a message through a Discord webhook with an appropriate embed-provider link. It supports X/Twitter, Instagram, TikTok, Reddit, Threads and Bluesky. Enable it with `/module enable social-embeds`; it is disabled by default.

## Setup and behavior

Enable Message Content Intent in Developer Portal. Give the bot View Channel, Send Messages, Manage Messages and Manage Webhooks in channels where repair should work. No API credentials, subscription or media executable is required; the bot contacts third-party embed providers over HTTPS.

When a supported plain URL is posted, the bot prepares a webhook post labeled `via Momo`, suppresses mentions, and deletes the original only after the replacement succeeds. The replacement includes links to the untouched original, the corrected provider URL and the platform creator when available. It does not impersonate an unmarked account.

To repair a message manually, right-click/long-press it and choose **Apps → Repair Social Embed**. Repairing someone else's message requires Manage Messages. Automatic processing is an event handler rather than a slash command; Core module channel allowlists restrict commands, not automatic repost events. Use module disablement or Discord channel permissions when controlling that behavior.

## Configuration and examples

`[social_embeds].probe_timeout_seconds` defaults to six seconds. `instagram_providers` gives the ordered providers to test; defaults are `kkinstagram.com`, `oginstagram.com`, `d.oginstagram.com` and `vxinstagram.com`. A provider must return usable metadata before the bot selects it. See the [configuration reference](../reference/configuration.md) and [command reference](../reference/commands.md).

To leave a post untouched, include the standalone word `fxignore`, wrap the URL in angle brackets, or place it inside inline/fenced code:

```text
fxignore https://www.instagram.com/p/EXAMPLE/
<https://www.instagram.com/p/EXAMPLE/>
```

## Limits and common problems

Messages with uploads, stickers or polls are left untouched, as are bot/webhook messages and unsupported message types. Prefix invocations are also avoided when recognized. Unsupported/private/deleted posts or providers without usable metadata may remain unchanged. This feature depends on third-party rendering and is not an archive or guarantee of permanent previews.

If nothing happens, check module activation, Message Content Intent, channel permissions and whether the message used an opt-out or attachment. If webhook posting fails, the original should remain. If metadata providers fail, retry later rather than giving the bot extra account credentials. Forwarding a public post URL to a third-party provider is part of this feature's behavior; do not use it for confidential content.
