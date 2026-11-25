# General

General provides quick, read-only Discord information: bot responsiveness, avatars, profile dates, server details and role permissions. It is enabled by default; a server manager can restore it with `/module enable general`.

## Setup and use

No API key, subscription, media executable or module-specific configuration is required. It uses Discord data; some details may be unavailable when the bot cannot access the relevant member or cached server data.

```text
/ping
/avatar
/user info
/server info
/role info @Moderators
```

`/avatar` and `/user info` accept an optional user and default to you. Server and role details apply in a server. These commands also support the configured prefix where enabled. Their normal responses are ephemeral when invoked as slash commands. See the [command reference](../reference/commands.md) for restrictions and arguments.

## Limits and common problems

`/ping` confirms that a command can respond; it is not a full external-provider or voice health check. `/about` in Core gives broader runtime information. Information commands display bounded role lists and handle missing data rather than promising a complete server export.

If a field is unavailable, check the bot's server/channel access and try again. If the whole command is denied, check General activation, channel limits and custom permission policy. There are no General-specific entries in the [configuration reference](../reference/configuration.md).
