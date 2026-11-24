# Moderation

Moderation provides member actions and records auditable cases in SQLite. It helps staff review warnings and previous actions without relying only on channel history. Enable it with `/module enable moderation`; it is disabled by default. No third-party account or module-specific TOML settings are required.

## Commands and examples

```text
/warn user:@Member why:Repeated disruption after a reminder.
/warnings user:@Member
/case number:1
/timeout user:@Member seconds:600 why:Repeated disruption.
```

The module also registers `/kick`, `/ban`, `/unban`, `/untimeout` and `/purge`. Kick and ban require their corresponding Discord permissions; timeout uses Moderate Members, while message deletion uses Manage Messages. Warnings, warning lists and case access require Moderate Members. Both the caller and bot must satisfy applicable permissions and hierarchy restrictions. Consult the [command reference](../reference/commands.md) before enabling actions for a role.

Ban optionally deletes up to seven days of messages. Timeout supports up to 28 days. Purge is bounded to 100 recent messages and optionally filters by user; it is not a full-channel history eraser. Supply clear reasons when available so case records remain useful.

## Storage and limits

Case numbering and records belong to the database layer. Preserve the Docker data volume and secure its backups: moderation records include server/user identifiers and staff-entered reasons. No separate database server is required. See [backup procedures](../self-hosting/operations.md).

Discord actions and database writes cannot form a single transaction across services. If a command reports an uncertain result, inspect the member/channel state and case records before retrying a destructive action. Do not assume every failed response means Discord made no change.

## Common problems

Hierarchy failures require moving the bot role above a manageable target, or choosing a target the caller is allowed to moderate. Timeouts cannot be applied to every protected Discord account/role. Old messages may not be eligible for Discord bulk deletion. Check channel permissions and recent logs privately when purging fails. Custom `/permissions` policy and module channel limits can deny an otherwise valid Discord action; use `/permissions explain` to diagnose that layer.

