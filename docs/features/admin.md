# Administration

Administration lets trusted staff post announcements, adjust slowmode, change nicknames and assign roles from Discord. Enable it with `/module enable admin`; it is disabled by default. It requires no outside service, subscription or module-specific configuration.

## Commands and permissions

| Command | Purpose | Invoking user's permission |
| --- | --- | --- |
| `/announce` | Post a message to a chosen channel without mass mentions | Manage Server |
| `/slowmode` | Set a channel delay from 0 to 21,600 seconds | Manage Channels |
| `/nick` | Set or clear a member nickname | Manage Nicknames |
| `/role add` / `/role remove` | Assign or remove a role | Manage Roles |

```text
/announce channel:#announcements message:Maintenance begins at 18:00 UTC.
/slowmode channel:#general seconds:10
/nick user:@Member nickname:New nickname
/role add user:@Member role:@Events
```

The bot also needs the relevant Discord permissions. Role and member actions enforce hierarchy: the bot cannot manage roles or members above it, and a caller cannot use the bot to bypass applicable hierarchy checks. Managed/integration roles cannot be freely assigned. Read exact arguments and restrictions in the [command reference](../reference/commands.md).

## Setup and common problems

Position the bot's role above roles it must manage, grant only necessary permissions, and restrict this module with Core's `/permissions` and `/module channels` commands where appropriate. There are no Admin-specific TOML settings; [Core](core.md) explains server policy configuration.

If an action is denied, check the caller's permission, the bot's permission, channel overrides, role hierarchy and custom policy separately. Granting the caller Manage Roles alone does not move the bot's role above the target. Use a zero-second slowmode to remove the delay; omit a nickname to clear it. Announcements intentionally suppress mentions, so they should not be used as a mass-ping system.
