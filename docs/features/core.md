# Core

Core makes the bot manageable without editing its database. It supplies `/help`, `/about`, module activation, channel restrictions, custom permission policy, prefixes and server music defaults. It is required and cannot be disabled.

## Setup and settings

No external account is needed beyond Discord. Server management commands require Manage Server. Module activation, allowed channels, custom rules, prefixes and server settings are stored in SQLite and survive restart.

`[discord]` controls owners, the default prefix, global prefix availability and administrator policy bypass. `[commands]` controls application-command registration. Prefix commands require both operator configuration and server enablement, plus Message Content Intent. `/config` manages supported per-server music settings; it does not edit arbitrary TOML or credentials. See the [configuration reference](../reference/configuration.md).

## Commands and examples

```text
/help
/about
/module list
/module info music
/module enable music
/module channels add music #bot-commands
/prefix show
/config show
```

Use `/permissions list`, `/permissions set`, `/permissions reset` and `/permissions explain` to review and manage custom command/module rules. Use `/prefix set`, `/prefix enable` and `/prefix disable` to manage server message commands. `/config set` accepts `volume`, `queue_limit`, `idle_seconds`, `empty_seconds`, `dj_role` and `announce`; `/config reset` restores configured defaults. Get exact syntax and supported ranges from the [command reference](../reference/commands.md).

## Limits and common problems

Channel lists restrict commands, including context menus, rather than automatic event handlers. Threads inherit their parent channel. Removing the final allowed channel or resetting the list restores access everywhere. Administrators bypass channel limits independently of custom permission bypass; Core cannot be channel-restricted.

If a command fails, first check `/module list` and ask a server manager to use `/permissions explain`. If prefixes do nothing, check the operator's `prefix_commands` setting and Message Content Intent as well as `/prefix show`. If slash commands are absent, check registration mode and installation scopes; enabling a module does not replace command registration.

