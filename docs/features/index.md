# Features

Momokun contains eleven registered modules. Core is required. General and Utility start enabled; the others need a server manager to run `/module enable <module-id>`. Configuration and API credentials do not automatically enable a module in a server.

| Module | Use it for | Default |
| --- | --- | --- |
| [Core](core.md) | Help, activation, permissions and server settings | Required |
| [General](general.md) | Avatars and user, server and role information | Enabled |
| [Utility](utility.md) | Choices, polls, persistent reminders and private bookmarks | Enabled |
| [Administration](admin.md) | Announcements, slowmode, nicknames and roles | Disabled |
| [Moderation](moderation.md) | Moderation actions and stored case records | Disabled |
| [Music](music.md) | Voice playback, queues, playlists and provider fallback | Disabled |
| [Downloads](download.md) | Media downloads, inspection and conversions | Disabled |
| [Media](media.md) | Anime, film and TV metadata | Disabled |
| [Image search](image.md) | Safe image results with navigation buttons | Disabled |
| [TorBox](torbox.md) | Cached-magnet downloads and subscription-gated streaming | Disabled |
| [Social embeds](social-embeds.md) | Repair social post previews using webhooks | Disabled |

The [command reference](../reference/commands.md) lists exact arguments, permissions and cooldowns from the registered command tree. The [configuration reference](../reference/configuration.md) distinguishes file settings from private environment variables.

These features are implemented, but pre-1.0 live acceptance remains incomplete. Credential-free services still depend on upstream availability. Easynews web search and DuckDuckGo image search use unofficial interfaces, so operators should treat their upstream compatibility as experimental. See [release status](../project/status.md) before deployment.

