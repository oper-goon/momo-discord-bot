# Utility

Utility helps members make choices, run polls, schedule messages and save private message bookmarks. It is enabled by default; use `/module enable utility` to restore it. It needs Discord and the persistent SQLite volume, with no external API credentials.

## Commands and examples

```text
/choose Tea | Coffee | Water
/poll create question:Which day? options:Saturday | Sunday
/remind duration:10m message:Check the oven
/reminder repeat interval:1d message:Review the queue
/reminders
/timestamp unix_seconds:1735689600
```

`/choose` accepts 2–25 pipe-separated choices. `/poll create` makes a native Discord poll with 2–10 choices and a 24-hour duration. Reminder durations use a number followed by `s`, `m`, `h` or `d`, from one second to 365 days; messages are limited to 1,500 characters. `/reminder edit`, `snooze` and `cancel` manage your own saved reminders.

To bookmark a message, right-click or long-press it and choose **Apps → Bookmark Message**. `/bookmark list`, `search`, `note` and `delete` manage your private saved excerpts and notes. Bookmark commands are slash-only to keep these results out of channel history. **Apps → Remind Me in 1 Hour** creates a reminder linked to the selected message. See the [command reference](../reference/commands.md) for exact fields.

## Persistence and delivery

A supervised worker checks due reminders every 15 seconds. Delivery goes to the saved channel and may mention the requesting user; the scheduling confirmation being private does not make eventual delivery private. Failed deliveries are retained for retry. Recurring reminders persist their interval. Disabling Utility in a server pauses its server reminder delivery.

No Utility-specific TOML is needed. Preserve `/data` when using Docker, because reminders and bookmarks live in SQLite. See [operations and backups](../self-hosting/operations.md).

## Limits and common problems

Reminders are not guaranteed to arrive at the exact second, especially during downtime or Discord failures. Check channel existence and Send Messages permissions when a reminder is late. Restarting with an empty or different volume loses access to the original stored records; reconnect the correct volume rather than creating duplicate reminders. Poll creation needs channel access and permission to send a native poll. Bookmark excerpts are stored data: include them in your backup/privacy practices and delete bookmarks you no longer need.
