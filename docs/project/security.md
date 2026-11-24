# Security and privacy

Keep the Discord token and optional service credentials in the host's private
environment file. Do not put them in TOML, screenshots, issue reports or images.
Protect SQLite data and backups: reminders, moderation cases, user settings and
bookmarks can contain personal information. Restrict backup access and choose a
retention period suitable for your server.

Grant the bot only the Discord permissions needed by enabled features. Limit
administrative and paid-provider commands to trusted roles. Social embed repair
can replace messages and requires explicit permissions. Media and download
features contact external services; those services receive relevant requests.

Use pinned release inputs, keep the host updated, inspect logs locally and
redact them before sharing. Debug-level third-party logs may reveal more than
normal logs. Do not expose provider services or database files to the internet.

For a vulnerability, use GitHub private vulnerability reporting when enabled.
If unavailable, request a private contact without posting exploit details.
SECURITY.md describes what to include. No dedicated private contact or response
SLA has been approved yet.

Release preparation found dependency advisories requiring resolution. Existing
vendored security patches and reviewed exceptions do not make a failed advisory
gate safe to ignore. The repository's docs/security.md records technical trust
boundaries and exceptions; the release audit records unresolved checks.
