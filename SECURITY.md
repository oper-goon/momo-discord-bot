# Reporting a security issue

Do not post tokens, passwords, signed download links, database files or a
working exploit against a live installation in a public issue.

Use GitHub's private vulnerability reporting button in the repository Security
tab when available. If it is unavailable, ask the maintainer to establish a
private reporting channel without including sensitive details. No private
security email or response-time promise has been established.

Include the affected version, a minimal reproduction with dummy credentials,
expected and actual behavior, and the likely impact. Redact user/server IDs,
hostnames and media links unless needed in the private report.

Operators should run a maintained release, keep Docker and host packages
updated, restrict administrative and paid-service commands, and protect `.env`,
configuration and SQLite backups. Rotate a credential at its provider if it
was exposed; deleting a file or commit does not revoke it.

See `docs/security.md` for dependency exceptions, vendored backport requirements
and network/media trust boundaries. Public release is pending the findings in
`docs/audit/public-release-review.md`; there is no supported-version security
guarantee at this stage.
