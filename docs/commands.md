# Command reference

Momo commands are available as slash commands. Most also support the configured
server prefix when prefix commands and Discord's Message Content intent are
enabled. TorBox is intentionally slash-only because magnet links are sensitive.
Commands marked as server-only cannot be used in direct messages.

Optional modules must first be enabled with `/module enable <module-id>`.
Permission rules configured through `/permissions` can further restrict any
command. Arguments in angle brackets are required; arguments in square brackets
are optional.

## Core (`core`)

Core is required and always enabled.

| Command | Description |
| --- | --- |
| `/help [command]` | List commands available to you or explain one command path. |
| `/about` | Show the bot version, database migration, task health, and modules. |
| `/module list` | List every module and its activation state in this server. |
| `/module info <module>` | Show a module's version and health. |
| `/module enable <module>` | Enable an optional module; requires Manage Server. |
| `/module disable <module>` | Disable an optional module; requires Manage Server. |
| `/permissions list [target]` | List custom command or module permission rules. |
| `/permissions set <target> <effect> <subject>` | Add an allow or deny rule; requires Manage Server. |
| `/permissions reset <target> <subject>` | Remove a custom rule; requires Manage Server. |
| `/permissions explain <command> [user]` | Explain the effective policy decision; requires Manage Server. |
| `/prefix show` | Show the server's current prefix and whether it is enabled. |
| `/prefix set <value>` | Change the server prefix; requires Manage Server. |
| `/prefix enable` | Enable prefix commands for the server; requires Manage Server. |
| `/prefix disable` | Disable prefix commands for the server; requires Manage Server. |
| `/config show` | Show effective server music settings; requires Manage Server. |
| `/config set <key> <value>` | Change a supported server setting; requires Manage Server. |
| `/config reset` | Restore server settings to configured defaults; requires Manage Server. |

## General (`general`)

Enabled by default.

| Command | Description |
| --- | --- |
| `/ping` | Check whether Momo is responding. |
| `/avatar [user]` | Show a user's avatar, defaulting to yourself. |
| `/user info [user]` | Show profile, membership, roles, and account dates. |
| `/server info` | Show server details and owner information. |
| `/role info <role>` | Show a role's ID, position, and permissions. |

## Utility (`utility`)

Enabled by default.

| Command | Description |
| --- | --- |
| `/choose <options>` | Randomly choose from 2–25 options separated by `|`. |
| `/poll create <question> <options>` | Create a native Discord poll from 2–10 `|`-separated options. |
| `/remind <duration> <message>` | Create a persistent reminder. |
| `/reminders` | List your pending reminders. |
| `/reminder repeat <interval> <message>` | Create a recurring reminder. |
| `/reminder edit <id> <duration> [message]` | Reschedule a reminder and optionally replace its message. |
| `/reminder snooze <id> <duration>` | Move a reminder later without changing its message. |
| `/reminder cancel <id>` | Delete one of your reminders. |
| `/bookmark list` | List your ten most recent private bookmarks. |
| `/bookmark search <query>` | Search your saved message excerpts and notes. |
| `/bookmark note <id> <note>` | Add or replace a private note on a bookmark. |
| `/bookmark delete <id>` | Delete one of your bookmarks. |
| `/timestamp <unix-seconds>` | Render a Discord timestamp and its reusable markup. |

Bookmark management is slash-only so saved excerpts, notes, and results remain
private. Create bookmarks from a message's **Apps** menu with `Bookmark
Message`; reminders and their management commands also support the configured
server prefix.

## Administration (`admin`)

Disabled by default. Commands enforce both Discord permissions and role
hierarchy.

| Command | Description |
| --- | --- |
| `/announce <channel> <message>` | Send an announcement without mass mentions; requires Manage Server. |
| `/slowmode <channel> <seconds>` | Set channel slowmode from 0–21,600 seconds; requires Manage Channels. |
| `/nick <user> [nickname]` | Set or clear a nickname; requires Manage Nicknames. |
| `/role add <user> <role>` | Assign a role; requires Manage Roles. |
| `/role remove <user> <role>` | Remove a role; requires Manage Roles. |

## Moderation (`moderation`)

Disabled by default. Actions are recorded as auditable cases.

| Command | Description |
| --- | --- |
| `/kick <user> [reason]` | Kick a member. |
| `/ban <user> [delete-days] [reason]` | Ban a member and optionally delete up to seven days of messages. |
| `/unban <user-id> [reason]` | Remove a ban by user ID. |
| `/timeout <user> <seconds> [reason]` | Timeout a member for up to 28 days. |
| `/untimeout <user> [reason]` | Remove a member timeout. |
| `/purge <count> [user]` | Delete up to 100 recent messages, optionally filtering by author. |
| `/warn <user> <reason>` | Record a warning case. |
| `/warnings <user>` | List warning cases for a user. |
| `/case <number>` | Show one moderation case. |

## Music (`music`)

Disabled by default. Playback is isolated per server, and control commands
apply the configured DJ and voice-channel policy.

| Command | Description |
| --- | --- |
| `/play <URL or search>` | Queue a supported website, public audio URL, or search result. |
| `/join` | Join your voice channel. |
| `/leave` | Stop playback, clear the queue, and disconnect. |
| `/pause` | Pause playback. |
| `/resume` | Resume playback. |
| `/skip` | Skip the current track. |
| `/stop` | Stop playback and clear the queue without disconnecting. |
| `/seek <seconds>` | Seek within the current track when supported. |
| `/volume <percent>` | Set playback volume from 0–200 percent. |
| `/nowplaying` | Show playback state with interactive controls. |
| `/queue show` | Show the current track and up to 15 queued tracks. |
| `/queue remove <position>` | Remove a pending track. |
| `/queue move <from> <to>` | Reorder a pending track. |
| `/queue clear` | Clear pending tracks without stopping the current track. |
| `/repeat <off|track|queue>` | Set the repeat mode. |
| `/shuffle` | Shuffle pending tracks. |
| `/playlist save <name>` | Save the current and pending tracks. |
| `/playlist list` | List server playlists and their owners. |
| `/playlist play <name> [owner]` | Queue a saved playlist. |
| `/playlist delete <name> [owner]` | Delete a playlist you own or administer. |
| `/playlist move <name> <from> <to> [owner]` | Reorder a saved playlist. |
| `/playlist remove <name> <position> [owner]` | Remove one saved playlist entry. |

## Download (`download`)

Disabled by default.

| Command | Description |
| --- | --- |
| `/download [URL]` | Download and reupload media from a public URL; when omitted, inspect a replied message or recent channel history. |
| `/media info [URL]` | Show container, duration, size, video, and audio stream information. |
| `/media audio [URL]` | Extract MP3 audio from a video. |
| `/media compress [URL]` | Re-encode a video as a smaller MP4. |
| `/media gif [URL]` | Convert the first ten seconds of a video into a GIF. |
| `/media thumbnail [URL]` | Extract a JPEG thumbnail from a video. |

When a media URL is omitted, these commands inspect a replied message and then
recent channel history. Downloads and conversions share the same supported-site,
size, duration, concurrency, and Discord upload limits.

## Image Search (`image`)

Disabled by default.

| Command | Description |
| --- | --- |
| `/image <query>` | Search safe images and browse cached results with Previous and Next buttons. |

## TorBox (`torbox`)

Disabled by default and slash-only. It requires the operator's
`TORBOX_API_KEY`; generated links are ephemeral.

| Command | Description |
| --- | --- |
| `/torbox download <magnet> [file-number]` | Choose a safe cached file from a private, paginated picker and generate its temporary download link. Supply a one-based file number to bypass the picker. |
| `/torbox stream <magnet> [file-number]` | Choose a safe video from a private, paginated picker and generate its stream. Supply a one-based file number to bypass the picker. |

Pickers accept up to 250 eligible files, show 25 per page, work only for the
person who ran the command, and expire after five minutes. Files TorBox marks
as potentially infected are never offered.

## Social Embeds (`social-embeds`)

Disabled by default. When it sees a supported X/Twitter, Instagram, TikTok,
Reddit, Threads, or Bluesky post URL, it reposts the message through a webhook
with links to the original, the selected embed provider, and the platform
creator when available.

Use `/module enable social-embeds` to activate it. Add the standalone word
`fxignore`, wrap a URL in angle brackets, or place it in inline or fenced code
to leave a message untouched.

## Message actions

Right-click or long-press a message and open **Apps** to use these native
context-menu actions when their module is enabled:

| Action | Module | Description |
| --- | --- | --- |
| `Bookmark Message` | Utility | Save a private excerpt and jump link. |
| `Remind Me in 1 Hour` | Utility | Create a one-hour reminder pointing back to the message. |
| `Download Media` | Download | Download and reupload supported media from the selected message. |
| `Repair Social Embed` | Social Embeds | Manually retry supported social links; other users' messages require Manage Messages. |
