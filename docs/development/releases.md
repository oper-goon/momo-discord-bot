# Release procedures and GitHub Pages

The documentation is a static GitHub Pages website. It requires no VPS changes,
bot uptime, Discord token, service credentials, inbound ports or reverse proxy.
Repository actions build the site and upload the generated Pages artifact.

## Current status

The workflow is prepared but publication is deliberately disabled.
`PAGES_ENABLED` must equal `true` before artifact upload and deployment are allowed.
No deployment is performed merely by building or previewing these files.
The intended URL is
`https://oper-goon.github.io/momo-discord-bot/`; it is not a claim that the site is
already online. Relative navigation, assets and the link checker support this
repository-specific path.

## Configure Pages after approval

An administrator must perform these steps after the owner approves public release:

1. Review the release/security report, contribution rights, licensing candidate
   and approved branding. Resolve blockers before publishing the repository/site.
2. Integrate the reviewed branch into the repository's default branch. Do not
   assume the currently checked-out development branch is the release branch.
3. Under repository **Settings → Pages → Build and deployment**, choose
   **GitHub Actions** as the source. Do not choose a `docs/` branch-folder source.
4. Protect the `github-pages` environment so deployment is restricted to the
   default branch. Add required reviewers if the repository plan supports them.
5. Add the repository Actions variable `PAGES_ENABLED` with value `true`.
6. Run **Documentation** from Actions or merge a documentation change to the
   default branch. Confirm validation passes, then review the deployed URL.

The workflow has read-only repository access by default. Only the deployment
job receives `pages: write` and `id-token: write`. Pull requests build and validate
without deployment permissions. It uses the official
[Pages actions](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages).
No personal access token is needed. Private-repository Pages availability depends
on the repository plan/settings; do not change visibility as a troubleshooting step.

For a future custom domain, configure it in Pages, add the required DNS records,
and update `site_url` and the link-checker's `--site-url`. No domain is required
for the default deployment. Do not commit an unsolicited `CNAME` now.

## Prepare a bot release

1. Review changes and known validation gaps. Keep `main` releasable.
2. Update the workspace version and shared path-dependency versions together,
   regenerate lockfiles/references as needed, and update the changelog.
3. Keep migrations forward-only. Perform an isolated backup/restore drill before
   an upgrade that changes persistent data.
4. Run formatting, strict workspace Clippy, tests, offline startup, cargo-deny,
   Docker build and the full documentation checks. Preserve intentional vendored
   security backports and inspect their notices.
5. Exercise Discord rendering, command sync, permissions and channel limits,
   component ownership/expiry, reminder recovery, music playback and shutdown in
   a private guild. Mock tests cannot substitute for these checks.
6. Review secrets in the current tree and history, redacted dependency/license
   reports, image contents and public artifact contents. Resolve findings.
7. Obtain explicit approval before merging, changing licensing/visibility,
   creating a public release, publishing Pages or GHCR images, or deploying.
8. Use reviewed `vMAJOR.MINOR.PATCH` release tags. Publish images only after the
   separate [image validation requirements](../self-hosting/images.md) are met.

Existing CI retains Rust quality gates, dependency policy and a Linux Docker
build. A previously green pipeline does not resolve newly reported advisories.
Read the release's own results and review exceptions before distribution.

## Distribution evidence

The Docker builder runs `scripts/collect_rust_notices.py` against locked Cargo
metadata filtered for its host target. It follows resolved non-development
edges from `bot-app`, conservatively retaining build tools. It is not an exact
measurement of linked bytes. Original package notices, exact registry archives,
modified vendor source and release-specific notice supplements accompany the
image. Supplement file hashes and packaged Git revisions must match.

The runtime stage runs `scripts/collect_image_notices.py` after installing its
tools. It captures Debian/Python versions, packaged notices and FFmpeg build
configuration. CI extracts `/usr/share/doc/momokun/` as a private seven-day
artifact for review. Neither collection script accesses bot configuration,
credentials or production storage.

Review every unresolved item and system/tool source requirement before
publishing; collection success is not a publishing gate approval. The Rust
collector's `--require-reviewed` flag fails when its unresolved list is nonempty;
even a zero list does not replace final image/distribution review. Keep any
reviewed binary's matching evidence/source materials with that release rather
than relying on the short-lived CI artifact.
