# Command reference

Generated from the actual composed command registry, including cross-module extensions and message actions. Do not edit this page directly; edit `scripts/generate_reference.py` or command source and regenerate.

`<argument>` is required; `[argument]` is optional. Slash commands use Discord's labeled argument fields. Prefix examples use the default `!`; prefix commands require both configuration and Message Content intent. Optional modules must be enabled per server. All commands also pass module/channel policy, custom permissions, shutdown and concurrency checks.

Replies normally remain private for slash commands. Prefix replies appear in channel history. Bookmark and TorBox command inputs intentionally remain slash-only. Context actions are under a message's **Apps** menu.

Permission values below are framework metadata; feature pages describe additional runtime target-channel, role hierarchy and voice/DJ checks. Empty metadata means no extra framework flag, not unrestricted access. Command groups organize subcommands. Optional arguments have no universal framework default; their omitted behavior is listed where applicable.

## Core (`core`)

[Feature guide](../features/core.md). Always enabled.

### `/help`

List available commands, or show a command path such as queue move.

**Syntax:** `/help [command]`; `!help [command]`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `command` | No | Command path, for example queue move (text) | List commands when omitted. |

**Example:** `/help`. Replace user, role, channel and record references with values from your server.

**Behavior:** Results are filtered by your effective access; pass a command path for details.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:30` (`help`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/about`

Version and service health.

**Syntax:** `/about`; `!about`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

**Example:** `/about`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:142` (`about`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/module`

Manage server modules.

**Syntax:** `/module`; `!module`.

**Subcommands:** `/module list`, `/module info`, `/module enable`, `/module disable`, `/module channels`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:178` (`modules`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/module list`

List module activation.

**Syntax:** `/module list`; `!module list`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/module list`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:322` (`module_list`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/module info`

Inspect a compiled module.

**Syntax:** `/module info <module>`; `!module info <module>`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `module` | Yes | Stable module ID, for example music (text) | Not applicable |

**Example:** `/module info music`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:351` (`module_info`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/module enable`

Enable a module for this server.

**Syntax:** `/module enable <module>`; `!module enable <module>`.

**User permissions:** Manage Guild. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `module` | Yes | Stable module ID, for example music (text) | Not applicable |

**Example:** `/module enable music`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:372` (`module_enable`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/module disable`

Disable a module and stop its server services.

**Syntax:** `/module disable <module>`; `!module disable <module>`.

**User permissions:** Manage Guild. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `module` | Yes | Stable module ID, for example music (text) | Not applicable |

**Example:** `/module disable music`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:394` (`module_disable`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/module channels`

Manage module channel allowlists (slash-only configuration).

**Syntax:** `/module channels`.

**Subcommands:** `/module channels add`, `/module channels remove`, `/module channels list`, `/module channels reset`.

**User permissions:** Manage Guild. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:195` (`module_channels`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/module channels add`

Allow a module in a channel; the first entry restricts use elsewhere.

**Syntax:** `/module channels add <module> <channel>`.

**User permissions:** Manage Guild. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `module` | Yes | Stable module ID, for example music (text) | Not applicable |
| `channel` | Yes | Discord channel (Discord channel) | Not applicable |

**Example:** `/module channels add music #general`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:215` (`module_channels_add`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/module channels remove`

Remove an allowed channel; the last removal restores access everywhere.

**Syntax:** `/module channels remove <module> <channel>`.

**User permissions:** Manage Guild. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `module` | Yes | Stable module ID, for example music (text) | Not applicable |
| `channel` | Yes | Discord channel (Discord channel) | Not applicable |

**Example:** `/module channels remove music #general`. Replace user, role, channel and record references with values from your server.

**Behavior:** Removing the last allowed channel restores access in all channels.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:250` (`module_channels_remove`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/module channels list`

Show a module's allowed channels.

**Syntax:** `/module channels list <module>`.

**User permissions:** Manage Guild. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `module` | Yes | Stable module ID, for example music (text) | Not applicable |

**Example:** `/module channels list music`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:297` (`module_channels_list`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/module channels reset`

Restore a module's access to all channels.

**Syntax:** `/module channels reset <module>`.

**User permissions:** Manage Guild. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `module` | Yes | Stable module ID, for example music (text) | Not applicable |

**Example:** `/module channels reset music`. Replace user, role, channel and record references with values from your server.

**Behavior:** Clear the module's channel allowlist, restoring access in all channels.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:308` (`module_channels_reset`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/permissions`

Manage command authorization rules.

**Syntax:** `/permissions`; `!permissions`.

**Subcommands:** `/permissions list`, `/permissions set`, `/permissions reset`, `/permissions explain`.

**User permissions:** Manage Guild. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:422` (`permissions`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/permissions list`

List server authorization rules.

**Syntax:** `/permissions list [target]`; `!permissions list [target]`.

**User permissions:** Manage Guild. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `target` | No | Command or module policy target (text) | List all targets. |

**Example:** `/permissions list`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:433` (`permission_list`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/permissions set`

Set a rule; subject is everyone, user:ID, role:ID, or channel:ID.

**Syntax:** `/permissions set <target> <effect> <subject> [channel]`; `!permissions set <target> <effect> <subject> [channel]`.

**User permissions:** Manage Guild. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `target` | Yes | Command or module policy target (text) | Not applicable |
| `effect` | Yes | Permission effect (integer); choices: Allow, Deny | Not applicable |
| `subject` | Yes | Permission subject (text) | Not applicable |
| `channel` | No | Discord channel (Discord channel) | Rule applies across channels. |

**Example:** `/permissions set music Deny everyone`. Replace user, role, channel and record references with values from your server.

**Behavior:** Target is a stable module ID or dot-separated command path (for example queue.move). Subject accepts everyone, user:ID, role:ID or channel:ID. A channel argument limits the rule's scope.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:495` (`permission_set`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/permissions reset`

Remove a matching rule.

**Syntax:** `/permissions reset <target> <subject> [channel]`; `!permissions reset <target> <subject> [channel]`.

**User permissions:** Manage Guild. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `target` | Yes | Command or module policy target (text) | Not applicable |
| `subject` | Yes | Permission subject (text) | Not applicable |
| `channel` | No | Discord channel (Discord channel) | Remove matching rule without channel restriction. |

**Example:** `/permissions reset music everyone`. Replace user, role, channel and record references with values from your server.

**Behavior:** Remove the exact matching target, subject and optional channel rule.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:533` (`permission_reset`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/permissions explain`

Explain the winning custom rule for a command.

**Syntax:** `/permissions explain <command> [user]`; `!permissions explain <command> [user]`.

**User permissions:** Manage Guild. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `command` | Yes | Command path, for example queue move (text) | Not applicable |
| `user` | No | Discord user/member (unban accepts a decimal user ID) (Discord user) | You. |

**Example:** `/permissions explain ping`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:566` (`permission_explain`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/prefix`

Configure optional prefix commands.

**Syntax:** `/prefix`; `!prefix`.

**Subcommands:** `/prefix show`, `/prefix set`, `/prefix enable`, `/prefix disable`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:606` (`prefix`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/prefix show`

Show the server prefix.

**Syntax:** `/prefix show`; `!prefix show`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/prefix show`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:611` (`prefix_show`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/prefix set`

Set the server prefix, up to 16 characters.

**Syntax:** `/prefix set <value>`; `!prefix set <value>`.

**User permissions:** Manage Guild. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `value` | Yes | Setting value (text) | Not applicable |

**Example:** `/prefix set !`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:638` (`prefix_set`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/prefix enable`

Enable server prefix commands.

**Syntax:** `/prefix enable`; `!prefix enable`.

**User permissions:** Manage Guild. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/prefix enable`. Replace user, role, channel and record references with values from your server.

**Behavior:** Enables the server preference; the process must also have prefix_commands enabled and Message Content intent available.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:677` (`prefix_enable`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/prefix disable`

Disable server prefix commands.

**Syntax:** `/prefix disable`; `!prefix disable`.

**User permissions:** Manage Guild. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/prefix disable`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:688` (`prefix_disable`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/config`

Configure typed server music settings.

**Syntax:** `/config`; `!config`.

**Subcommands:** `/config show`, `/config set`, `/config reset`.

**User permissions:** Manage Guild. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:700` (`config`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/config show`

Show effective music settings.

**Syntax:** `/config show`; `!config show`.

**User permissions:** Manage Guild. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/config show`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:711` (`config_show`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/config set`

Set a music key: volume, queue_limit, idle_seconds, empty_seconds, dj_role, announce.

**Syntax:** `/config set <key> <value>`; `!config set <key> <value>`.

**User permissions:** Manage Guild. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `key` | Yes | Supported setting key (text) | Not applicable |
| `value` | Yes | Setting value (text) | Not applicable |

**Example:** `/config set volume 80`. Replace user, role, channel and record references with values from your server.

**Behavior:** Keys are volume, queue_limit, idle_seconds, empty_seconds, dj_role and announce. Accepted values are in the configuration reference.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:732` (`config_set`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/config reset`

Reset music settings to process defaults.

**Syntax:** `/config reset`; `!config reset`.

**User permissions:** Manage Guild. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/config reset`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:775` (`config_reset`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

### `/role`

Inspect or manage roles.

**Syntax:** `/role`; `!role`.

**Subcommands:** `/role info`, `/role add`, `/role remove`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Implementation and detailed behavior:** `crates/bot-core/src/commands.rs:24` (`role`). Related commands are grouped above; read the [feature guide](../features/core.md) for limits and practical workflows.

## General (`general`)

[Feature guide](../features/general.md). Enabled by default.

### `/role info`

Inspect a role.

**Syntax:** `/role info <role>`; `!role info <role>`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `role` | Yes | Discord role (Discord role) | Not applicable |

**Example:** `/role info @Members`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-general/src/lib.rs:222` (`role_info`). Related commands are grouped above; read the [feature guide](../features/general.md) for limits and practical workflows.

### `/ping`

Check whether the bot is responding.

**Syntax:** `/ping`; `!ping`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

**Example:** `/ping`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-general/src/lib.rs:31` (`ping`). Related commands are grouped above; read the [feature guide](../features/general.md) for limits and practical workflows.

### `/avatar`

Show a user's avatar.

**Syntax:** `/avatar [user]`; `!avatar [user]`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `user` | No | Discord user/member (unban accepts a decimal user ID) (Discord user) | You. |

**Example:** `/avatar`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-general/src/lib.rs:36` (`avatar`). Related commands are grouped above; read the [feature guide](../features/general.md) for limits and practical workflows.

### `/user`

User information.

**Syntax:** `/user`; `!user`.

**Subcommands:** `/user info`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

**Implementation and detailed behavior:** `crates/module-general/src/lib.rs:42` (`user`). Related commands are grouped above; read the [feature guide](../features/general.md) for limits and practical workflows.

### `/user info`

Show a user's profile and account creation time.

**Syntax:** `/user info [user]`; `!user info [user]`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `user` | No | Discord user/member (unban accepts a decimal user ID) (Discord user) | You. |

**Example:** `/user info`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-general/src/lib.rs:47` (`user_info`). Related commands are grouped above; read the [feature guide](../features/general.md) for limits and practical workflows.

### `/server`

Server information.

**Syntax:** `/server`; `!server`.

**Subcommands:** `/server info`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Implementation and detailed behavior:** `crates/module-general/src/lib.rs:137` (`server`). Related commands are grouped above; read the [feature guide](../features/general.md) for limits and practical workflows.

### `/server info`

Show the server name and owner.

**Syntax:** `/server info`; `!server info`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/server info`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-general/src/lib.rs:142` (`server_info`). Related commands are grouped above; read the [feature guide](../features/general.md) for limits and practical workflows.

## Media (`media`)

[Feature guide](../features/media.md). Enable with `/module enable media` (Manage Server).

### `/anime`

Look up an anime's synopsis, score, studio, genres, status, and trailer.

**Syntax:** `/anime <query>`; `!anime <query>`.

**User permissions:** No additional framework permission. **Bot permissions:** Embed Links.

**Restrictions:** Server only. user: 5s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `query` | Yes | Anime title (text) | Not applicable |

**Example:** `/anime Cowboy Bebop`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-media/src/lib.rs:410` (`anime`). Related commands are grouped above; read the [feature guide](../features/media.md) for limits and practical workflows.

### `/movie`

Look up a movie's synopsis, score, production company, genres, status, and trailer.

**Syntax:** `/movie <query>`; `!movie <query>`.

**User permissions:** No additional framework permission. **Bot permissions:** Embed Links.

**Restrictions:** Server only. user: 5s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `query` | Yes | Movie title (text) | Not applicable |

**Example:** `/movie The Matrix`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-media/src/lib.rs:427` (`movie`). Related commands are grouped above; read the [feature guide](../features/media.md) for limits and practical workflows.

### `/tv`

Look up a TV show's synopsis, score, production company, genres, status, and trailer.

**Syntax:** `/tv <query>`; `!tv <query>`.

**User permissions:** No additional framework permission. **Bot permissions:** Embed Links.

**Restrictions:** Server only. user: 5s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `query` | Yes | TV show title (text) | Not applicable |

**Example:** `/tv Breaking Bad`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-media/src/lib.rs:444` (`tv`). Related commands are grouped above; read the [feature guide](../features/media.md) for limits and practical workflows.

### `/animesearch`

Search anime titles and choose from up to ten matches.

**Syntax:** `/animesearch <query>`; `!animesearch <query>`.

**User permissions:** No additional framework permission. **Bot permissions:** Embed Links.

**Restrictions:** Server only. user: 5s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `query` | Yes | Anime title (text) | Not applicable |

**Example:** `/animesearch Naruto`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-media/src/search.rs:358` (`animesearch`). Related commands are grouped above; read the [feature guide](../features/media.md) for limits and practical workflows.

### `/moviesearch`

Search movie titles and choose from up to ten matches.

**Syntax:** `/moviesearch <query>`; `!moviesearch <query>`.

**User permissions:** No additional framework permission. **Bot permissions:** Embed Links.

**Restrictions:** Server only. user: 5s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `query` | Yes | Movie title (text) | Not applicable |

**Example:** `/moviesearch Dune`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-media/src/search.rs:375` (`moviesearch`). Related commands are grouped above; read the [feature guide](../features/media.md) for limits and practical workflows.

### `/tvsearch`

Search TV titles and choose from up to ten matches.

**Syntax:** `/tvsearch <query>`; `!tvsearch <query>`.

**User permissions:** No additional framework permission. **Bot permissions:** Embed Links.

**Restrictions:** Server only. user: 5s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `query` | Yes | TV title (text) | Not applicable |

**Example:** `/tvsearch The Office`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-media/src/search.rs:392` (`tvsearch`). Related commands are grouped above; read the [feature guide](../features/media.md) for limits and practical workflows.

## Administration (`admin`)

[Feature guide](../features/admin.md). Enable with `/module enable admin` (Manage Server).

### `/role add`

Add a role to a member.

**Syntax:** `/role add <user> <role>`; `!role add <user> <role>`.

**User permissions:** Manage Roles. **Bot permissions:** Manage Roles.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `user` | Yes | Discord user/member (unban accepts a decimal user ID) (Discord user) | Not applicable |
| `role` | Yes | Discord role (Discord role) | Not applicable |

**Example:** `/role add @member @Members`. Replace user, role, channel and record references with values from your server.

**Behavior:** Actor and bot must outrank the target member and role; managed roles cannot be assigned. This changes the member's roles.

**Implementation and detailed behavior:** `crates/module-admin/src/lib.rs:128` (`role_add`). Related commands are grouped above; read the [feature guide](../features/admin.md) for limits and practical workflows.

### `/role remove`

Remove a role from a member.

**Syntax:** `/role remove <user> <role>`; `!role remove <user> <role>`.

**User permissions:** Manage Roles. **Bot permissions:** Manage Roles.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `user` | Yes | Discord user/member (unban accepts a decimal user ID) (Discord user) | Not applicable |
| `role` | Yes | Discord role (Discord role) | Not applicable |

**Example:** `/role remove @member @Members`. Replace user, role, channel and record references with values from your server.

**Behavior:** Actor and bot must outrank the target member and role; managed roles cannot be removed.

**Implementation and detailed behavior:** `crates/module-admin/src/lib.rs:148` (`role_remove`). Related commands are grouped above; read the [feature guide](../features/admin.md) for limits and practical workflows.

### `/announce`

Send an announcement without mass mentions.

**Syntax:** `/announce <channel> <message>`; `!announce <channel> <message>`.

**User permissions:** Manage Guild. **Bot permissions:** Send Messages.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `channel` | Yes | Discord channel (Discord channel) | Not applicable |
| `message` | Yes | Message text (text) | Not applicable |

**Example:** `/announce #general Take a break`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-admin/src/lib.rs:25` (`announce`). Related commands are grouped above; read the [feature guide](../features/admin.md) for limits and practical workflows.

### `/slowmode`

Set a channel's slowmode in seconds.

**Syntax:** `/slowmode <channel> <seconds>`; `!slowmode <channel> <seconds>`.

**User permissions:** Manage Channels. **Bot permissions:** Manage Channels.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `channel` | Yes | Discord channel (Discord channel) | Not applicable |
| `seconds` | Yes | Duration or playback position in seconds (integer); min_value: 0.0; max_value: 21600.0 | Not applicable |

**Example:** `/slowmode #general 10`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-admin/src/lib.rs:65` (`slowmode`). Related commands are grouped above; read the [feature guide](../features/admin.md) for limits and practical workflows.

### `/nick`

Set or clear a member nickname.

**Syntax:** `/nick <user> [nickname]`; `!nick <user> [nickname]`.

**User permissions:** Manage Nicknames. **Bot permissions:** Manage Nicknames.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `user` | Yes | Discord user/member (unban accepts a decimal user ID) (Discord user) | Not applicable |
| `nickname` | No | Replacement nickname (text) | Clear nickname. |

**Example:** `/nick @member`. Replace user, role, channel and record references with values from your server.

**Behavior:** Actor and bot hierarchy checks apply. Omit nickname to clear it.

**Implementation and detailed behavior:** `crates/module-admin/src/lib.rs:103` (`nick`). Related commands are grouped above; read the [feature guide](../features/admin.md) for limits and practical workflows.

## Moderation (`moderation`)

[Feature guide](../features/moderation.md). Enable with `/module enable moderation` (Manage Server).

### `/kick`

Kick a member with an audit reason.

**Syntax:** `/kick <user> [why]`; `!kick <user> [why]`.

**User permissions:** Kick Members. **Bot permissions:** Kick Members.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `user` | Yes | Discord user/member (unban accepts a decimal user ID) (Discord user) | Not applicable |
| `why` | No | Audit reason, 1–400 characters (text) | No reason provided. |

**Example:** `/kick @member`. Replace user, role, channel and record references with values from your server.

**Behavior:** Checks hierarchy, allocates an auditable case, then attempts the Discord action. Only use on a member you intend to remove.

**Implementation and detailed behavior:** `crates/module-moderation/src/lib.rs:65` (`kick`). Related commands are grouped above; read the [feature guide](../features/moderation.md) for limits and practical workflows.

### `/ban`

Ban a server member with a bounded message deletion window.

**Syntax:** `/ban <user> [delete_days] [why]`; `!ban <user> [delete_days] [why]`.

**User permissions:** Ban Members. **Bot permissions:** Ban Members.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `user` | Yes | Discord user/member (unban accepts a decimal user ID) (Discord user) | Not applicable |
| `delete_days` | No | Days of message history to delete, 0–7 (integer); min_value: 0.0; max_value: 7.0 | 0. |
| `why` | No | Audit reason, 1–400 characters (text) | No reason provided. |

**Example:** `/ban @member`. Replace user, role, channel and record references with values from your server.

**Behavior:** Checks hierarchy and records an auditable case. Optional delete_days is 0–7; omitting it preserves message history.

**Implementation and detailed behavior:** `crates/module-moderation/src/lib.rs:99` (`ban`). Related commands are grouped above; read the [feature guide](../features/moderation.md) for limits and practical workflows.

### `/unban`

Remove a ban by user ID.

**Syntax:** `/unban <user> [why]`; `!unban <user> [why]`.

**User permissions:** Ban Members. **Bot permissions:** Ban Members.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `user` | Yes | Discord user/member (unban accepts a decimal user ID) (Discord user) | Not applicable |
| `why` | No | Audit reason, 1–400 characters (text) | No reason provided. |

**Example:** `/unban @member`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-moderation/src/lib.rs:140` (`unban`). Related commands are grouped above; read the [feature guide](../features/moderation.md) for limits and practical workflows.

### `/timeout`

Timeout a member for up to 28 days.

**Syntax:** `/timeout <user> <seconds> [why]`; `!timeout <user> <seconds> [why]`.

**User permissions:** Moderate Members. **Bot permissions:** Moderate Members.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `user` | Yes | Discord user/member (unban accepts a decimal user ID) (Discord user) | Not applicable |
| `seconds` | Yes | Duration or playback position in seconds (integer); min_value: 1.0; max_value: 2419200.0 | Not applicable |
| `why` | No | Audit reason, 1–400 characters (text) | No reason provided. |

**Example:** `/timeout @member 10`. Replace user, role, channel and record references with values from your server.

**Behavior:** Checks hierarchy; maximum duration is 28 days. A case records the outcome.

**Implementation and detailed behavior:** `crates/module-moderation/src/lib.rs:177` (`timeout`). Related commands are grouped above; read the [feature guide](../features/moderation.md) for limits and practical workflows.

### `/untimeout`

Remove a member timeout.

**Syntax:** `/untimeout <user> [why]`; `!untimeout <user> [why]`.

**User permissions:** Moderate Members. **Bot permissions:** Moderate Members.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `user` | Yes | Discord user/member (unban accepts a decimal user ID) (Discord user) | Not applicable |
| `why` | No | Audit reason, 1–400 characters (text) | No reason provided. |

**Example:** `/untimeout @member`. Replace user, role, channel and record references with values from your server.

**Behavior:** Checks hierarchy and records removal of the timeout as a moderation case.

**Implementation and detailed behavior:** `crates/module-moderation/src/lib.rs:226` (`untimeout`). Related commands are grouped above; read the [feature guide](../features/moderation.md) for limits and practical workflows.

### `/purge`

Delete up to 100 recent messages, optionally by author.

**Syntax:** `/purge <count> [user]`; `!purge <count> [user]`.

**User permissions:** Manage Messages. **Bot permissions:** Manage Messages and Read Message History.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `count` | Yes | Number of messages (integer); min_value: 1.0; max_value: 100.0 | Not applicable |
| `user` | No | Discord user/member (unban accepts a decimal user ID) (Discord user) | All authors. |

**Example:** `/purge 5`. Replace user, role, channel and record references with values from your server.

**Behavior:** Deletes up to 100 eligible recent messages. Discord cannot bulk-delete messages older than 14 days; optional user filters by author.

**Implementation and detailed behavior:** `crates/module-moderation/src/lib.rs:271` (`purge`). Related commands are grouped above; read the [feature guide](../features/moderation.md) for limits and practical workflows.

### `/warn`

Record a warning in the audit history.

**Syntax:** `/warn <user> <why>`; `!warn <user> <why>`.

**User permissions:** Moderate Members. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `user` | Yes | Discord user/member (unban accepts a decimal user ID) (Discord user) | Not applicable |
| `why` | Yes | Audit reason, 1–400 characters (text) | Not applicable |

**Example:** `/warn @member Please follow the server rules`. Replace user, role, channel and record references with values from your server.

**Behavior:** Records a warning in SQLite after hierarchy checks; does not kick, ban or timeout the user.

**Implementation and detailed behavior:** `crates/module-moderation/src/lib.rs:319` (`warn`). Related commands are grouped above; read the [feature guide](../features/moderation.md) for limits and practical workflows.

### `/warnings`

List recent warnings for a member.

**Syntax:** `/warnings <user>`; `!warnings <user>`.

**User permissions:** Moderate Members. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `user` | Yes | Discord user/member (unban accepts a decimal user ID) (Discord user) | Not applicable |

**Example:** `/warnings @member`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-moderation/src/lib.rs:342` (`warnings`). Related commands are grouped above; read the [feature guide](../features/moderation.md) for limits and practical workflows.

### `/case`

Inspect an audit case.

**Syntax:** `/case <number>`; `!case <number>`.

**User permissions:** Moderate Members. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `number` | Yes | Moderation case number (integer); min_value: -9007199254740991.0; max_value: 9007199254740991.0 | Not applicable |

**Example:** `/case 1`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-moderation/src/lib.rs:377` (`case`). Related commands are grouped above; read the [feature guide](../features/moderation.md) for limits and practical workflows.

## Utility (`utility`)

[Feature guide](../features/utility.md). Enabled by default.

### `/choose`

Choose one option separated by a vertical bar.

**Syntax:** `/choose <options>`; `!choose <options>`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `options` | Yes | Choices separated by \| (text) | Not applicable |

**Example:** `/choose tea \| coffee`. Replace user, role, channel and record references with values from your server.

**Behavior:** Accepts 2–25 pipe-separated options, at most 1800 bytes overall.

**Implementation and detailed behavior:** `crates/module-utility/src/lib.rs:27` (`choose`). Related commands are grouped above; read the [feature guide](../features/utility.md) for limits and practical workflows.

### `/poll`

Create a poll.

**Syntax:** `/poll`; `!poll`.

**Subcommands:** `/poll create`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Implementation and detailed behavior:** `crates/module-utility/src/lib.rs:47` (`poll`). Related commands are grouped above; read the [feature guide](../features/utility.md) for limits and practical workflows.

### `/poll create`

Create a native Discord poll with options separated by a vertical bar.

**Syntax:** `/poll create <question> <options>`; `!poll create <question> <options>`.

**User permissions:** No additional framework permission. **Bot permissions:** Send Messages.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `question` | Yes | Poll question (text) | Not applicable |
| `options` | Yes | Choices separated by \| (text) | Not applicable |

**Example:** `/poll create Lunch? pizza \| sushi`. Replace user, role, channel and record references with values from your server.

**Behavior:** Creates a native Discord poll lasting 24 hours. Accepts 2–10 nonempty choices and a question up to 300 characters.

**Implementation and detailed behavior:** `crates/module-utility/src/lib.rs:58` (`poll_create`). Related commands are grouped above; read the [feature guide](../features/utility.md) for limits and practical workflows.

### `/remind`

Save a reminder; duration examples: 10m, 2h, 1d.

**Syntax:** `/remind <duration> <message>`; `!remind <duration> <message>`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 5s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `duration` | Yes | Human duration, for example 10m (text) | Not applicable |
| `message` | Yes | Message text (text) | Not applicable |

**Example:** `/remind 10m Take a break`. Replace user, role, channel and record references with values from your server.

**Behavior:** Durations use a number and s, m, h or d (for example 10m); maximum 365 days. Delivery is to the saved channel and can mention you, even when confirmation is private.

**Implementation and detailed behavior:** `crates/module-utility/src/lib.rs:107` (`remind`). Related commands are grouped above; read the [feature guide](../features/utility.md) for limits and practical workflows.

### `/reminders`

List your pending reminders.

**Syntax:** `/reminders`; `!reminders`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

**Example:** `/reminders`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-utility/src/lib.rs:133` (`reminders`). Related commands are grouped above; read the [feature guide](../features/utility.md) for limits and practical workflows.

### `/reminder`

Edit, repeat, snooze, or cancel reminders.

**Syntax:** `/reminder`; `!reminder`.

**Subcommands:** `/reminder repeat`, `/reminder edit`, `/reminder snooze`, `/reminder cancel`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

**Implementation and detailed behavior:** `crates/module-utility/src/lib.rs:175` (`reminder`). Related commands are grouped above; read the [feature guide](../features/utility.md) for limits and practical workflows.

### `/reminder repeat`

Create a recurring reminder.

**Syntax:** `/reminder repeat <interval> <message>`; `!reminder repeat <interval> <message>`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 5s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `interval` | Yes | Repeat duration, for example 1h (text) | Not applicable |
| `message` | Yes | Message text (text) | Not applicable |

**Example:** `/reminder repeat 1h Stretch`. Replace user, role, channel and record references with values from your server.

**Behavior:** Recurring reminder interval is persisted. Disabling Utility pauses server delivery; the worker checks due reminders every 15 seconds.

**Implementation and detailed behavior:** `crates/module-utility/src/lib.rs:181` (`reminder_repeat`). Related commands are grouped above; read the [feature guide](../features/utility.md) for limits and practical workflows.

### `/reminder edit`

Change when a reminder fires and optionally replace its message.

**Syntax:** `/reminder edit <id> <duration> [message]`; `!reminder edit <id> <duration> [message]`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `id` | Yes | Record ID shown by its list command (integer); min_value: -9007199254740991.0; max_value: 9007199254740991.0 | Not applicable |
| `duration` | Yes | Human duration, for example 10m (text) | Not applicable |
| `message` | No | Message text (text) | Keep existing message. |

**Example:** `/reminder edit 1 10m`. Replace user, role, channel and record references with values from your server.

**Behavior:** Only your own reminder can be edited; omitted message preserves its current text.

**Implementation and detailed behavior:** `crates/module-utility/src/lib.rs:217` (`reminder_edit`). Related commands are grouped above; read the [feature guide](../features/utility.md) for limits and practical workflows.

### `/reminder snooze`

Move a reminder later without changing its message.

**Syntax:** `/reminder snooze <id> <duration>`; `!reminder snooze <id> <duration>`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `id` | Yes | Record ID shown by its list command (integer); min_value: -9007199254740991.0; max_value: 9007199254740991.0 | Not applicable |
| `duration` | Yes | Human duration, for example 10m (text) | Not applicable |

**Example:** `/reminder snooze 1 10m`. Replace user, role, channel and record references with values from your server.

**Behavior:** Only your own reminder can be rescheduled; the message text is preserved.

**Implementation and detailed behavior:** `crates/module-utility/src/lib.rs:255` (`reminder_snooze`). Related commands are grouped above; read the [feature guide](../features/utility.md) for limits and practical workflows.

### `/reminder cancel`

Delete one of your reminders.

**Syntax:** `/reminder cancel <id>`; `!reminder cancel <id>`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `id` | Yes | Record ID shown by its list command (integer); min_value: -9007199254740991.0; max_value: 9007199254740991.0 | Not applicable |

**Example:** `/reminder cancel 1`. Replace user, role, channel and record references with values from your server.

**Behavior:** Deletes only a reminder owned by you.

**Implementation and detailed behavior:** `crates/module-utility/src/lib.rs:261` (`reminder_cancel`). Related commands are grouped above; read the [feature guide](../features/utility.md) for limits and practical workflows.

### `/bookmark`

Manage private message bookmarks.

**Syntax:** `/bookmark`.

**Subcommands:** `/bookmark list`, `/bookmark search`, `/bookmark note`, `/bookmark delete`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

**Implementation and detailed behavior:** `crates/module-utility/src/lib.rs:278` (`bookmark`). Related commands are grouped above; read the [feature guide](../features/utility.md) for limits and practical workflows.

### `/bookmark list`

List your most recent bookmarks.

**Syntax:** `/bookmark list`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

**Example:** `/bookmark list`. Replace user, role, channel and record references with values from your server.

**Behavior:** Returns your ten most recent private bookmarks.

**Implementation and detailed behavior:** `crates/module-utility/src/lib.rs:284` (`bookmark_list`). Related commands are grouped above; read the [feature guide](../features/utility.md) for limits and practical workflows.

### `/bookmark search`

Search your saved message excerpts and notes.

**Syntax:** `/bookmark search <query>`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `query` | Yes | Search words or title (text) | Not applicable |

**Example:** `/bookmark search black cat`. Replace user, role, channel and record references with values from your server.

**Behavior:** Searches your own saved excerpts and notes; results remain private.

**Implementation and detailed behavior:** `crates/module-utility/src/lib.rs:290` (`bookmark_search`). Related commands are grouped above; read the [feature guide](../features/utility.md) for limits and practical workflows.

### `/bookmark note`

Add or replace a private note on a bookmark.

**Syntax:** `/bookmark note <id> <note>`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `id` | Yes | Record ID shown by its list command (integer); min_value: -9007199254740991.0; max_value: 9007199254740991.0 | Not applicable |
| `note` | Yes | Private bookmark note (text) | Not applicable |

**Example:** `/bookmark note 1 Read later`. Replace user, role, channel and record references with values from your server.

**Behavior:** Updates only a bookmark you own.

**Implementation and detailed behavior:** `crates/module-utility/src/lib.rs:300` (`bookmark_note`). Related commands are grouped above; read the [feature guide](../features/utility.md) for limits and practical workflows.

### `/bookmark delete`

Delete one of your bookmarks.

**Syntax:** `/bookmark delete <id>`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `id` | Yes | Record ID shown by its list command (integer); min_value: -9007199254740991.0; max_value: 9007199254740991.0 | Not applicable |

**Example:** `/bookmark delete 1`. Replace user, role, channel and record references with values from your server.

**Behavior:** Deletes only a bookmark you own.

**Implementation and detailed behavior:** `crates/module-utility/src/lib.rs:317` (`bookmark_delete`). Related commands are grouped above; read the [feature guide](../features/utility.md) for limits and practical workflows.

### `Bookmark Message`

Save this message to your private bookmarks.

**Syntax:** Message → Apps → **Bookmark Message**.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `message` | Yes | The message you selected in Discord; no typed argument | Not applicable |

**Implementation and detailed behavior:** `crates/module-utility/src/lib.rs:330` (`bookmark_message`). Related commands are grouped above; read the [feature guide](../features/utility.md) for limits and practical workflows.

### `Remind Me in 1 Hour`

Create a one-hour reminder for this message.

**Syntax:** Message → Apps → **Remind Me in 1 Hour**.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `message` | Yes | The message you selected in Discord; no typed argument | Not applicable |

**Implementation and detailed behavior:** `crates/module-utility/src/lib.rs:356` (`remind_message`). Related commands are grouped above; read the [feature guide](../features/utility.md) for limits and practical workflows.

### `/timestamp`

Format a Unix timestamp in Discord's local time display.

**Syntax:** `/timestamp <unix_seconds>`; `!timestamp <unix_seconds>`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** No extra location restriction in command metadata. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `unix_seconds` | Yes | Unix timestamp in seconds (integer); min_value: -9007199254740991.0; max_value: 9007199254740991.0 | Not applicable |

**Example:** `/timestamp 1700000000`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-utility/src/lib.rs:464` (`timestamp`). Related commands are grouped above; read the [feature guide](../features/utility.md) for limits and practical workflows.

## Music (`music`)

[Feature guide](../features/music.md). Enable with `/module enable music` (Manage Server).

### `/play`

Queue a supported website, public audio URL, or search query.

**Syntax:** `/play <input>`; `!play <input>`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 2s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `input` | Yes | Supported website URL, public direct audio URL, or search words (text) | Not applicable |

**Example:** `/play lofi hip hop`. Replace user, role, channel and record references with values from your server.

**Behavior:** Joins your voice channel and queues resolved media. Queue, resolver timeout, provider concurrency and configured voice/DJ policy apply. Service availability and website restrictions can prevent playback.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:243` (`play`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/join`

Join your voice channel; DJ or Manage Guild is required to move the bot.

**Syntax:** `/join`; `!join`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/join`. Replace user, role, channel and record references with values from your server.

**Behavior:** Join your voice channel; the bot needs View Channel, Connect and Speak.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:268` (`join`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/leave`

Stop, clear the queue, and disconnect.

**Syntax:** `/leave`; `!leave`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/leave`. Replace user, role, channel and record references with values from your server.

**Behavior:** Stops playback, clears pending tracks and disconnects the server player.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:281` (`leave`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/pause`

Pause current playback.

**Syntax:** `/pause`; `!pause`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/pause`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:286` (`pause`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/resume`

Resume current playback.

**Syntax:** `/resume`; `!resume`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/resume`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:291` (`resume`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/skip`

Skip the current track or retry the retained queue.

**Syntax:** `/skip`; `!skip`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/skip`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:296` (`skip`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/stop`

Clear all playback while staying connected.

**Syntax:** `/stop`; `!stop`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/stop`. Replace user, role, channel and record references with values from your server.

**Behavior:** Stops playback and clears the queue while remaining in voice.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:301` (`stop`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/seek`

Seek to a position in seconds if supported by the source.

**Syntax:** `/seek <seconds>`; `!seek <seconds>`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `seconds` | Yes | Duration or playback position in seconds (integer); min_value: 0.0; max_value: 86400.0 | Not applicable |

**Example:** `/seek 10`. Replace user, role, channel and record references with values from your server.

**Behavior:** Only works for a seekable current track.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:306` (`seek`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/volume`

Set playback volume from 0 to 200 percent.

**Syntax:** `/volume <percent>`; `!volume <percent>`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `percent` | Yes | Volume percentage (integer); min_value: 0.0; max_value: 200.0 | Not applicable |

**Example:** `/volume 80`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:316` (`volume`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/nowplaying`

Show current playback and queue status.

**Syntax:** `/nowplaying`; `!nowplaying`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/nowplaying`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:326` (`nowplaying`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/queue`

View or edit the pending queue.

**Syntax:** `/queue`; `!queue`.

**Subcommands:** `/queue show`, `/queue remove`, `/queue move`, `/queue clear`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:386` (`queue`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/queue show`

Show the current track and up to fifteen upcoming tracks.

**Syntax:** `/queue show`; `!queue show`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/queue show`. Replace user, role, channel and record references with values from your server.

**Behavior:** Shows the current track and at most 15 pending tracks.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:391` (`queue_show`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/queue remove`

Remove a pending track by one-based position.

**Syntax:** `/queue remove <position>`; `!queue remove <position>`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `position` | Yes | One-based queue/playlist position (integer); min_value: 1.0; max_value: 4294967295.0 | Not applicable |

**Example:** `/queue remove 1`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:398` (`queue_remove`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/queue move`

Move a pending track; positions start at one.

**Syntax:** `/queue move <from> <to>`; `!queue move <from> <to>`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `from` | Yes | One-based source position (integer); min_value: 1.0; max_value: 4294967295.0 | Not applicable |
| `to` | Yes | One-based destination position (integer); min_value: 1.0; max_value: 4294967295.0 | Not applicable |

**Example:** `/queue move 1 2`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:404` (`queue_move`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/queue clear`

Clear pending tracks without stopping the current track.

**Syntax:** `/queue clear`; `!queue clear`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/queue clear`. Replace user, role, channel and record references with values from your server.

**Behavior:** Clears pending tracks without stopping the current track.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:415` (`queue_clear`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/repeat`

Set repeat mode to off, track, or queue.

**Syntax:** `/repeat <mode>`; `!repeat <mode>`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `mode` | Yes | Repeat mode (integer); choices: Off, Track, Queue | Not applicable |

**Example:** `/repeat off`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:420` (`repeat`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/shuffle`

Shuffle pending tracks only.

**Syntax:** `/shuffle`; `!shuffle`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/shuffle`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:425` (`shuffle`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/playlist`

Manage server-local, user-owned playlists.

**Syntax:** `/playlist`; `!playlist`.

**Subcommands:** `/playlist save`, `/playlist list`, `/playlist play`, `/playlist delete`, `/playlist move`, `/playlist remove`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:442` (`playlist`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/playlist save`

Save current and pending tracks under your playlist name.

**Syntax:** `/playlist save <name>`; `!playlist save <name>`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `name` | Yes | Playlist name (text) | Not applicable |

**Example:** `/playlist save favorites`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:451` (`playlist_save`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/playlist list`

List saved playlists and their owners.

**Syntax:** `/playlist list`; `!playlist list`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/playlist list`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:470` (`playlist_list`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/playlist play`

Queue a saved playlist, optionally from another owner.

**Syntax:** `/playlist play <name> [owner]`; `!playlist play <name> [owner]`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `name` | Yes | Playlist name (text) | Not applicable |
| `owner` | No | Playlist owner (Discord user) (Discord user) | You. |

**Example:** `/playlist play favorites`. Replace user, role, channel and record references with values from your server.

**Behavior:** Imports the saved playlist subject to playlist import and queue limits. Owner defaults to you.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:488` (`playlist_play`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/playlist delete`

Delete a playlist you own, or administer.

**Syntax:** `/playlist delete <name> [owner]`; `!playlist delete <name> [owner]`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `name` | Yes | Playlist name (text) | Not applicable |
| `owner` | No | Playlist owner (Discord user) (Discord user) | You. |

**Example:** `/playlist delete favorites`. Replace user, role, channel and record references with values from your server.

**Behavior:** Deleting another owner's playlist requires the implementation's server administration check.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:538` (`playlist_delete`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/playlist move`

Reorder a saved playlist transactionally.

**Syntax:** `/playlist move <name> <from> <to> [owner]`; `!playlist move <name> <from> <to> [owner]`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `name` | Yes | Playlist name (text) | Not applicable |
| `from` | Yes | One-based source position (integer); min_value: 1.0; max_value: 4294967295.0 | Not applicable |
| `to` | Yes | One-based destination position (integer); min_value: 1.0; max_value: 4294967295.0 | Not applicable |
| `owner` | No | Playlist owner (Discord user) (Discord user) | You. |

**Example:** `/playlist move favorites 1 2`. Replace user, role, channel and record references with values from your server.

**Behavior:** Reordering another owner's playlist requires the server administration check.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:555` (`playlist_move`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/playlist remove`

Remove a saved playlist entry by one-based position.

**Syntax:** `/playlist remove <name> <position> [owner]`; `!playlist remove <name> <position> [owner]`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `name` | Yes | Playlist name (text) | Not applicable |
| `position` | Yes | One-based queue/playlist position (integer); min_value: 1.0; max_value: 4294967295.0 | Not applicable |
| `owner` | No | Playlist owner (Discord user) (Discord user) | You. |

**Example:** `/playlist remove favorites 1`. Replace user, role, channel and record references with values from your server.

**Behavior:** Editing another owner's playlist requires the server administration check.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:579` (`playlist_remove`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/music`

Inspect enabled music providers and their circuit breakers.

**Syntax:** `/music`; `!music`.

**Subcommands:** `/music providers`, `/music status`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:365` (`music`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/music providers`

Show enabled providers in configured priority order.

**Syntax:** `/music providers`; `!music providers`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/music providers`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:370` (`music_providers`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

### `/music status`

Show provider availability and safe failure categories.

**Syntax:** `/music status`; `!music status`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Example:** `/music status`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-music/src/commands.rs:375` (`music_status`). Related commands are grouped above; read the [feature guide](../features/music.md) for limits and practical workflows.

## TorBox (`torbox`)

[Feature guide](../features/torbox.md). Enable with `/module enable torbox` (Manage Server).

### `/torbox`

Resolve cached TorBox downloads and video streams.

**Syntax:** `/torbox`.

**Subcommands:** `/torbox download`, `/torbox stream`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Implementation and detailed behavior:** `crates/module-torbox/src/commands.rs:25` (`torbox`). Related commands are grouped above; read the [feature guide](../features/torbox.md) for limits and practical workflows.

### `/torbox download`

Generate temporary download links for files in a cached magnet.

**Syntax:** `/torbox download <magnet> [file_number]`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 10s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `magnet` | Yes | A BitTorrent magnet link (text) | Not applicable |
| `file_number` | No | One-based file number (optional picker bypass) (integer); min_value: 0.0; max_value: 4294967295.0 | Open private file picker. |

**Example:** `/torbox download magnet:?xt=urn:btih:<your-content-hash>`. Replace user, role, channel and record references with values from your server.

**Behavior:** Requires TORBOX_API_KEY and cached content. Omit file_number for a private picker (up to 250 safe files, 25 per page, five-minute expiry). ZIP preparation requires all files to be safe; generated links are temporary and sensitive.

**Implementation and detailed behavior:** `crates/module-torbox/src/commands.rs:31` (`download`). Related commands are grouped above; read the [feature guide](../features/torbox.md) for limits and practical workflows.

### `/torbox stream`

Create a TorBox stream for a video in a cached magnet.

**Syntax:** `/torbox stream <magnet> [file_number]`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 10s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `magnet` | Yes | A BitTorrent magnet link (text) | Not applicable |
| `file_number` | No | One-based video file number (optional picker bypass) (integer); min_value: 0.0; max_value: 4294967295.0 | Open private file picker. |

**Example:** `/torbox stream magnet:?xt=urn:btih:<your-content-hash>`. Replace user, role, channel and record references with values from your server.

**Behavior:** Requires cached safe video and operator credentials. Returns a temporary HLS link; stream creation can require a TorBox Pro subscription. The picker belongs to you and expires after five minutes.

**Implementation and detailed behavior:** `crates/module-torbox/src/commands.rs:91` (`stream`). Related commands are grouped above; read the [feature guide](../features/torbox.md) for limits and practical workflows.

## Download (`download`)

[Feature guide](../features/download.md). Enable with `/module enable download` (Manage Server).

### `/download`

Download and reupload media from a URL, replied message, or recent chat.

**Syntax:** `/download [url]`; `!download [url]`.

**User permissions:** No additional framework permission. **Bot permissions:** Send Messages and Attach Files.

**Restrictions:** Server only. user: 15s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `url` | No | Public video URL; omit to inspect a reply or recent messages (text) | Inspect replied message, then recent history. |

**Example:** `/download`. Replace user, role, channel and record references with values from your server.

**Behavior:** When URL is omitted, inspect a replied message and then recent channel history. Download size, duration, supported-host and Discord upload limits apply.

**Implementation and detailed behavior:** `crates/module-download/src/commands.rs:29` (`download`). Related commands are grouped above; read the [feature guide](../features/download.md) for limits and practical workflows.

### `Download Media`

Download media from this message.

**Syntax:** Message → Apps → **Download Media**.

**User permissions:** No additional framework permission. **Bot permissions:** Send Messages and Attach Files.

**Restrictions:** Server only. user: 15s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `message` | Yes | The message you selected in Discord; no typed argument | Not applicable |

**Implementation and detailed behavior:** `crates/module-download/src/commands.rs:47` (`download_message`). Related commands are grouped above; read the [feature guide](../features/download.md) for limits and practical workflows.

### `/media`

Inspect or convert bounded media.

**Syntax:** `/media`; `!media`.

**Subcommands:** `/media info`, `/media audio`, `/media compress`, `/media gif`, `/media thumbnail`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

**Implementation and detailed behavior:** `crates/module-download/src/commands.rs:84` (`media`). Related commands are grouped above; read the [feature guide](../features/download.md) for limits and practical workflows.

### `/media info`

Show container, duration, size, and stream information.

**Syntax:** `/media info [url]`; `!media info [url]`.

**User permissions:** No additional framework permission. **Bot permissions:** No additional framework permission.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `url` | No | Public supported media URL (text) | Inspect replied message, then recent history. |

**Example:** `/media info`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-download/src/commands.rs:90` (`media_info`). Related commands are grouped above; read the [feature guide](../features/download.md) for limits and practical workflows.

### `/media audio`

Extract MP3 audio from a video.

**Syntax:** `/media audio [url]`; `!media audio [url]`.

**User permissions:** No additional framework permission. **Bot permissions:** Send Messages and Attach Files.

**Restrictions:** Server only. user: 15s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `url` | No | Public supported media URL (text) | Inspect replied message, then recent history. |

**Example:** `/media audio`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-download/src/commands.rs:105` (`media_audio`). Related commands are grouped above; read the [feature guide](../features/download.md) for limits and practical workflows.

### `/media compress`

Re-encode a video to a smaller MP4.

**Syntax:** `/media compress [url]`; `!media compress [url]`.

**User permissions:** No additional framework permission. **Bot permissions:** Send Messages and Attach Files.

**Restrictions:** Server only. user: 15s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `url` | No | Public supported media URL (text) | Inspect replied message, then recent history. |

**Example:** `/media compress`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-download/src/commands.rs:118` (`media_compress`). Related commands are grouped above; read the [feature guide](../features/download.md) for limits and practical workflows.

### `/media gif`

Convert the first ten seconds of a video to a GIF.

**Syntax:** `/media gif [url]`; `!media gif [url]`.

**User permissions:** No additional framework permission. **Bot permissions:** Send Messages and Attach Files.

**Restrictions:** Server only. user: 15s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `url` | No | Public supported media URL (text) | Inspect replied message, then recent history. |

**Example:** `/media gif`. Replace user, role, channel and record references with values from your server.

**Behavior:** Converts the first ten seconds to GIF, subject to the shared download/upload limits.

**Implementation and detailed behavior:** `crates/module-download/src/commands.rs:131` (`media_gif`). Related commands are grouped above; read the [feature guide](../features/download.md) for limits and practical workflows.

### `/media thumbnail`

Extract a JPEG thumbnail from a video.

**Syntax:** `/media thumbnail [url]`; `!media thumbnail [url]`.

**User permissions:** No additional framework permission. **Bot permissions:** Send Messages and Attach Files.

**Restrictions:** Server only. user: 15s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `url` | No | Public supported media URL (text) | Inspect replied message, then recent history. |

**Example:** `/media thumbnail`. Replace user, role, channel and record references with values from your server.

**Implementation and detailed behavior:** `crates/module-download/src/commands.rs:144` (`media_thumbnail`). Related commands are grouped above; read the [feature guide](../features/download.md) for limits and practical workflows.

## Image Search (`image`)

[Feature guide](../features/image.md). Enable with `/module enable image` (Manage Server).

### `/image`

Search the web for images using DuckDuckGo with a Bing fallback.

**Syntax:** `/image <query>`; `!image <query>`.

**User permissions:** No additional framework permission. **Bot permissions:** Embed Links.

**Restrictions:** Server only. user: 5s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `query` | Yes | What to search for (text) | Not applicable |

**Example:** `/image black cat`. Replace user, role, channel and record references with values from your server.

**Behavior:** Uses safe image search and cached Previous/Next buttons. Provider results and availability are external; safe search is not a guarantee of suitable content.

**Implementation and detailed behavior:** `crates/module-image/src/commands.rs:25` (`image`). Related commands are grouped above; read the [feature guide](../features/image.md) for limits and practical workflows.

## Social Embeds (`social-embeds`)

[Feature guide](../features/social-embeds.md). Enable with `/module enable social-embeds` (Manage Server).

### `Repair Social Embed`

Repair supported social links in this message.

**Syntax:** Message → Apps → **Repair Social Embed**.

**User permissions:** No additional framework permission. **Bot permissions:** Manage Messages and Manage Webhooks.

**Restrictions:** Server only. user: 1s cooldown.

| Argument | Required | Meaning / accepted values | Omitted behavior |
| --- | --- | --- | --- |
| `message` | Yes | The message you selected in Discord; no typed argument | Not applicable |

**Behavior:** Repairs supported social URLs on the selected message. Repairing another user's message requires Manage Messages.

**Implementation and detailed behavior:** `crates/module-social-embeds/src/lib.rs:432` (`repair_embed`). Related commands are grouped above; read the [feature guide](../features/social-embeds.md) for limits and practical workflows.

