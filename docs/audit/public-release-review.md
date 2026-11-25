# Public release security and licensing review

Review date: 2026-11-24. This is a release-preparation report, not authorization
to publish, merge, rotate credentials, rewrite history or alter production.

## Scope and evidence

The isolated `work/public-release` candidate preserves the supplied development
snapshot. Available history has **44 commits**. Root's Gitleaks 8.30.1 run scanned
**213 candidate public files** and all available local Git refs using
`git --log-opts=--all`. The initial file count is a scan snapshot. A final candidate
scan covered **239 public files** and found only the same dependency-name false
positive. Tool reports are fully redacted local
artifacts under `.cache/gitleaks/` and are not website content.

The official Windows Gitleaks archive was verified against its published SHA256:
`D29144DEFF3A68AA93CED33DDDF84B7FDC26070ADD4AA0F4513094C8332AFC4E`.
Both current-tree and history scans reported one generic-api-key finding in
Cargo.lock. Triage identified Cargo dependency-name strings involving libcrux
dependencies, not credential values. The history finding originates in commit
`0ea60524acd35b2eccf2c0cb3980eedeaff5ca1f`. No actual credential was identified by
this scanner in this scope. This does **not** establish that every credential,
personal detail or private artifact is absent.

The coordinating agent also ran `scan_history.py` over 44 commits and 371 Git
blobs. It found three historical versions of `docs/music-providers.md` containing
personal home-directory paths and no credential-pattern matches. These locations
were reported without their private values. The current release copy is sanitized;
history still requires the owner's disclosure decision. Gitleaks does not OCR
screenshots or prove every secret format is
covered. Remote deleted refs, other clones, releases, caches and older Actions
logs have not been exhaustively scanned in this review.

The original workspace includes an unrelated untracked `momo-armada/` project
and untracked screenshots, excluded from the release worktree. Its license and
history are separate and were not audited. One screenshot shows private
repository UI; a banner preview includes creator contact details and sample
counters. Do not include these originals in a public release without review.

## Findings and required action

| Status | Finding | Required action |
| --- | --- | --- |
| Release blocker | Fresh cargo-deny advisory gate fails | Resolve affected dependency chains and rerun; do not waive merely to publish |
| Release blocker | MIT-to-PolyForm rights and approval unresolved | Confirm ownership/contribution rights and approve exact license transition |
| Release blocker | Final image dependency attribution bundle incomplete | Collect exact resolved Rust, Python and OS package notices and source obligations |
| Review required | Legacy documentation contained host names and private backup paths | Current text sanitized by coordinator; historical copies remain and require owner disclosure decision |
| Review required | Original screenshots disclose private UI/contact details | Excluded from candidate; inspect any future replacement manually |
| Review required | Approved art supplied in chat, original distribution files unavailable | Confirm creator rights/credit and integrate optimized originals |
| Manual setup | Pages not enabled; repository private | Configure only after owner approval; do not assume private Pages support on the account plan |
| Manual setup | Private vulnerability reporting not confirmed enabled | Enable an appropriate private reporting channel before launch |
| Partial review | Latest Actions log scanned; older logs not exhaustively inspected | Review remaining accessible runs before public release; never print their secrets |

The fresh advisory failures reported by the coordinating agent are
RUSTSEC-2026-0543 (chrono), RUSTSEC-2026-0387 (crypto-bigint),
RUSTSEC-2026-0459 (libsqlite3-sys), RUSTSEC-2026-0441 (pin-utils),
RUSTSEC-2026-0382 (rustls-webpki), RUSTSEC-2026-0386 (tokio),
RUSTSEC-2026-0592 (url), and RUSTSEC-2026-0381 (webpki-roots).
The same run passed bans, license allowlists and source policy. It did not pass
the overall dependency-security gate. Existing exception/backport obligations
in docs/security.md and vendor/README.md remain in force, including ARM testing
limitations and path-dependency advisory coverage gaps.

No confirmed exposed credential was found, so this report does not identify a
specific credential requiring rotation. If further review finds one, rotate at
its provider before any history-cleanup discussion. No credentials were rotated
and no history was rewritten.

## Current security controls reviewed

The existing `.gitignore` excludes root `.env`, private TOML, data, SQLite files,
logs, target and tools. `.dockerignore` excludes private environment/config/data
and Git history; the coordinator is hardening release-context exclusions.
Examples must remain placeholders only. Ignoring a path does not erase a tracked
or historical file. New nested credentials and alternate filenames need review.

Existing CI uses read-only contents permission and validates Rust, dependencies
and Docker. Pages deployment is prepared with a publication gate; container
publishing remains a documented plan. No public deployment was performed. The container has a non-root user,
multi-stage build and persistent data volume. Runtime secrets are supplied by
the host rather than copied into the image. No inbound bot port is necessary;
Discord/media traffic is outbound, and the optional provider stays private.

Media code and security policy document bounded subprocess/network behavior,
public-address checks, suppressed mentions, sensitive-link handling and safe
errors. Paid services need permission restrictions. The SQLite volume and
backups contain user data and require host access control and retention policy.
This documentation initiative is not a new penetration test of every Discord
permission edge or third-party extractor.

Read-only GitHub metadata check confirmed `main` as default branch, repository
visibility private and `has_pages=false`; private reporting status was null.
No GitHub settings were changed. A subsequent read-only Actions check found the
five latest run summaries all failed. The latest CI run, `42307205599` on
`staging`, was downloaded only to ignored `.cache/ci-latest.log` and scanned
locally with Gitleaks: exit 0, zero findings. Contents were not printed. The
repository artifacts endpoint returned `total_count=0`. This establishes no
currently listed artifacts, not absence of historical artifacts or leaks.
Older run logs were not exhaustively scanned; summaries alone cannot certify
their contents.

## Licensing review

The current LICENSE is MIT, copyright 2026 Momo contributors; workspace Cargo
metadata is MIT. Available commit authors are an automation identity and
Dependabot. These identities do not prove the owner holds every copyright or
has relicensing authority. LICENSE first appears in foundation commit
`0ea6052`. Prior MIT grants cannot simply be revoked by changing future files.

The official unmodified candidate was downloaded from
<https://polyformproject.org/licenses/noncommercial/1.0.0.txt> into
`licenses/PolyForm-Noncommercial-1.0.0.txt`. SHA256:
`FFCCA38841ADB694B6F380647E15F17C446A4D1656FED51A1E2041D064C94CC8`.
This is a review artifact only; LICENSE and Cargo metadata remain MIT.
No CLA has been imposed. A future commercial licensing strategy should review
ownership and any contributor agreement with qualified advice before adoption.

Bundled libcrux-chacha20poly1305 and libcrux-secrets declare Apache-2.0. Ringbuf
declares MIT/Apache-2.0 and includes both license files. Upstream headers,
provenance and backport notes are retained. Official Apache text is supplied at
`licenses/Apache-2.0.txt`, SHA256
`CFC7749B96F63BD31C3C42B5C471BF756814053E847C10F3EB003417BC523D30`.
The container notice bundle includes this and ringbuf's MIT text. Exact upstream
NOTICE obligations still need verification. No third-party code was relicensed.

Cargo-deny's license allowlist passed, including MPL-2.0 and trust-root data
under CDLA-Permissive-2.0. An allowlist pass does not settle source disclosure,
notice collection or compatibility for every distribution form. Audit actual
FFmpeg build options, installed Python extras, Deno, provider images and Debian
packages before publishing images. THIRD_PARTY_NOTICES.md explicitly records
these gaps rather than claiming a complete redistribution compliance bundle.

## Validation and release decision

Coordinator reports: formatting passed; strict workspace Clippy passed;
81 workspace tests passed and two ignored; offline startup/configuration check
passed with an in-memory database. Dependency advisories failed as above.
Documentation, container and visual checks are recorded in the final release
report as their results become available. No live Discord acceptance or
production modification is part of this audit.

Before publication, rerun secret scanning on the final candidate, resolve
advisories, review historic infrastructure disclosure, inspect remote artifacts,
confirm rights and asset provenance, finish binary notices, and obtain the
owner's explicit publication/license approval. This candidate is **not yet
cleared for public release**.
