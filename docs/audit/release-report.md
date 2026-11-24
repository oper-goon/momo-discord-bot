# Momokun public-release preparation report

Review date: 2026-11-24. Branch: `feature/public-release-docs`.
Worktree: `work/public-release` inside the original repository.

Follow-up: the owner approved the documentation and generated branding. The
candidate was pushed privately as draft PR #10. Compatible dependency updates
subsequently reduced the eight baseline advisory findings to four; see
[dependency remediation](dependency-remediation.md). Counts below describe the
initial audit unless explicitly marked as a follow-up.

Licensing follow-up: the owner confirmed all first-party relicensing and artwork
publication rights on 2026-11-24. The requested PolyForm Noncommercial 1.0.0
transition is applied in this candidate, including Cargo/container metadata;
the prior MIT notice is retained and third-party terms are unchanged. No CLA
was imposed. Complete binary/image notices remain a release requirement.

Final follow-up checks passed: 81 workspace tests (two live tests ignored),
strict Clippy, formatting, offline startup, registry freshness, 13 Python tests,
strict documentation build and 2,883 internal links/assets. The advisory gate
still reports four failures; license, source and ban checks pass.

## Executive summary

The documentation and community-release foundation is implemented and locally
validated. Material for MkDocs builds a standalone static site with generated
references, all module guides, beginner Docker installation, operations and
developer documentation. The production bot, default branch and repository
visibility have not been changed. No Pages site or public container was published.

This candidate is **not cleared for public release**. The supplied dependency
lockfile fails eight advisory checks. Noncommercial relicensing requires rights
review and owner approval. Container startup/recovery and live Discord acceptance
remain incomplete, as do full binary notices and original-art provenance review.
Three generated branding derivatives are integrated and approved by the owner.
The documentation build and 2,883 internal link/asset checks passed afterward.

## Isolation and supplied development

The source worktree was already dirty on `feature/music-providers`, based on
`ac327f3`. Its tracked changes plus the new media crate, channel policy and
migration 3 were copied into the isolated worktree to document the actual code.
Those feature changes are inherited development, not authored by this initiative.
`input-snapshot.json` records original tracked-file hashes; the original source
was preserved. The unrelated `momo-armada/` project and local screenshots were
not copied. The candidate is being committed and pushed to a private review branch
at the owner's request. No merge, public deployment, force-push, branch deletion,
history rewrite, credential rotation, database replacement or production operation
is authorized by this review upload.

Review the worktree changes with that baseline distinction. Existing active
development must be integrated deliberately before this release work is merged.
The Windows toolchain and ignored build cache were reused; source edits remain
in the isolated worktree.

The hashes of all 12 originally modified tracked files were rechecked at the
end of implementation and still match the source snapshot.

## Implemented deliverables

| Area | Status | Result |
| --- | --- | --- |
| Repository discovery | Completed, scoped audit | Source/config/registry/migrations/deployment inventory, 546 resolved package metadata records, 44 local commits reviewed by scanners |
| Documentation framework | Implemented and tested | Pinned direct/transitive Python requirements, Material search, tabs, code copy/highlighting, Mermaid, mobile navigation and table of contents |
| Feature documentation | Completed | Eleven implemented modules, setup, accounts, examples, restrictions, limits and common problems |
| Commands/configuration | Generated and checked | Offline composed-registry exporter; freshness and coverage gates |
| Docker preparation | Implemented, runtime test pending | Existing multi-stage/non-root image retained; Compose security/logging/configuration improvements and setup helper |
| Visual theme | Implemented, owner approved | Monochrome responsive theme, hero/cards, focus and reduced-motion support; three generated branding derivatives integrated from supplied references |
| Pages | Configured, deployment disabled | Official Actions artifact/deploy workflow; PR validation; publication gated by repository variable and default branch |
| Community | Completed, contact setup pending | README, contribution/security/conduct policies, issue and pull-request templates, branding/credits/license summaries |
| Licensing | Prepared for review | Official unmodified PolyForm candidate; existing MIT preserved; canonical Apache text and initial notices included |
| Security | Scoped review completed, blockers open | Redacted current/history/latest-CI-log scans, dependency audit, privacy and infrastructure review |

## Files created or modified

The complete [changed-file inventory](changed-files.json) lists every path and
marks inherited development separately. Main release-owned groups are:

| Files | Purpose |
| --- | --- |
| `mkdocs.yml`, `requirements-docs.txt`, `docs/overrides/main.html`, `docs/stylesheets/momokun.css` | Site configuration, reproducible Python versions and presentation |
| `docs/index.md`, `docs/getting-started/`, `docs/self-hosting/`, `docs/features/`, `docs/reference/`, `docs/development/`, `docs/project/`, `docs/troubleshooting.md` | Public documentation and generated coverage |
| `crates/bot-app/src/reference.rs`, narrow exporter dispatch in `main.rs` | Offline composed-registry snapshot |
| `scripts/generate_reference.py`, `check_docs.py`, `docs_hooks.py`, `setup.py`, and Python tests | Generation, validation, branding integration and local setup |
| `.github/workflows/docs.yml`, `.github/dependabot.yml` | Documentation CI/deployment gate and dependency upkeep |
| `Dockerfile`, `compose.yaml`, `.dockerignore`, `.env.example`, `examples/compose.prebuilt.yaml.example` | Deployment preparation and initial image notices |
| `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `THIRD_PARTY_NOTICES.md`, `CHANGELOG.md`, `.github/ISSUE_TEMPLATE/`, PR template | Community and release guidance |
| `AGENTS.md`, `.gitignore`, `licenses/`, `docs/audit/` | Maintenance rules, exclusions, license candidates and private review evidence |
| Sanitized legacy `docs/release-status.md` and `docs/music-providers.md` | Remove current private deployment identifiers and align Compose guidance |

`LICENSE`, workspace license metadata, migrations and feature behavior were not
changed by the release initiative. Existing supplied feature changes remain
visible in the overall worktree diff and are identified in the inventory.

## Documentation coverage

The checked-in `docs/reference/coverage.json` is generated alongside references:

- **11 / 11 registered modules** have feature guides: Core, General, Utility,
  Administration, Moderation, Music, Download, Media, Image, TorBox and Social Embeds.
- **112 / 112 command nodes** documented: **92 slash-command leaves**, **4 message
  context actions**, and **16 organizing groups**. There are 108 slash-enabled
  nodes and 95 prefix-enabled nodes, both counts including applicable groups.
- **31 / 31 TOML fields** documented from serialized runtime defaults.
- **18 bot/logging/terminal environment variables** documented and checked against
  source reads; container tool flags and image build arguments are explained in
  the hosting guides/Dockerfile.
- The strict site build and **2,812 internal link, fragment and asset checks** pass.

Each command includes syntax, arguments, declared permissions, cooldowns,
restrictions, examples and source locations. Body-level hierarchy, voice/DJ,
string-parser and target-dependent checks also need curated prose; generation
does not certify those behaviors or external account compatibility. No live bot
or production credentials are needed to export commands.

## Docker installation for the reviewed source checkout

The complete guide, including PowerShell, is `docs/self-hosting/index.md`.
After approved integration/release, new users can clone the repository and run:

```sh
cp .env.example .env
cp config/example.toml config/config.toml
```

Edit `.env` privately to set `DISCORD_TOKEN`. Enable Message Content Intent in
Developer Portal, invite the application with `bot` and `applications.commands`
scopes and feature-appropriate permissions. Edit TOML to select global command
registration, or guild mode with a real private test server ID. The unchanged
example intentionally fails guild-mode validation until that choice is made.

```sh
docker compose build
docker compose run --rm --no-deps bot --config /app/config/config.toml --check
docker compose up -d
docker compose ps
docker compose logs --tail 100 bot
```

Look for `Bot ready`, then verify `/ping` and `/help`. Optional accounts and module
activation remain optional. SQLite initializes/migrates in the existing named
`momo-data` volume. Keep the volume and verified off-host backups; do not use
`down -v`. The check writes/migrates storage and is not a Discord liveness check.

Compose adds init, capability dropping, no-new-privileges, bounded bot logs,
complete private env forwarding and fail-fast read-only configuration mounts.
It retains the private token-helper service, no host ports and 20-second shutdown
grace. The image includes OCI source/version/revision/license metadata and an
initial notice bundle. It remains MIT-labeled until licensing is approved.

**Not newly tested:** Docker is unavailable here. The revised Compose build/start,
volume recovery instructions and container runtime must pass an isolated Linux
x86_64 test before release. ARM64 is unverified. No measured hardware minimum is
claimed. There is no application liveness health endpoint, so no misleading
database-writing health check was added.

## Container images

GHCR publishing is feasible but **not performed or enabled**. Source builds are
the actionable installation method. `examples/compose.prebuilt.yaml.example`
requires an actual approved image reference; it does not promise an existing tag.

A future approved publisher should build/test Linux x86_64, supply VERSION and
REVISION metadata, publish reviewed release-specific/stable tags and record the
digest. Full reproducibility needs immutable base-image and OS-package inputs;
the present version-tagged Dockerfile and Cargo.lock are not a bit-for-bit build
guarantee. Additional architectures require their own runtime/assembly tests.

## Preview and visual checks

The local preview is served at `http://127.0.0.1:8000/` while its preview process
is running. Recreate it independently with the documentation instructions:

```sh
python -m pip install -r requirements-docs.txt
python -m mkdocs serve --dev-addr 127.0.0.1:8000
```

Desktop and 390 × 844 mobile layouts were inspected in the in-app browser.
Search returned 13 matching documents for TorBox; the mobile drawer opened and
navigated to Getting started. No home-page horizontal overflow was observed.
The architecture Mermaid diagram renders. Copy controls and tabbed OS instructions
are enabled by the theme; clipboard behavior and screen-reader acceptance were
not exhaustively tested. Screenshots are local ignored artifacts:

- [Desktop preview](../../outputs/release-preview/desktop.jpg)
- [Mobile preview](../../outputs/release-preview/mobile.jpg)

The supplied character and glitch wordmark are the approved identity. Exact
original files are not yet accessible in the repository. Optional integration
hooks accept approved banner/wordmark/mascot WebP files, favicon PNG and social
PNG under `docs/assets/brand/`. No substitute mascot was created. Recommended
additional assets: a separate wordmark, simple favicon emblem, 1200 × 630 sharing
card, and optional small 404 illustration. Artist credit and distribution rights
still need recording.

## GitHub Pages and manual configuration

Read-only GitHub checks confirmed **private repository**, **main default branch**,
and **Pages disabled**. Settings were not changed. Intended URL after approval:
`https://oper-goon.github.io/momo-discord-bot/`.

The workflow installs pinned documentation dependencies, exports fresh registry
metadata, checks references/tests, builds strictly, checks internal links/assets,
and uploads/deploys only after `PAGES_ENABLED=true` on the default branch.
PR jobs have no deployment permission. Only deployment receives `pages: write`
and `id-token: write`; there is no production credential or VPS dependency.

After explicit publication approval, an administrator must select GitHub Actions
under Settings → Pages, protect the github-pages environment/default branch,
and set the Actions variable `PAGES_ENABLED=true`. Review private-repository plan
availability; do not change visibility automatically. Also establish private
security/conduct/commercial contact routes, enable private vulnerability reporting
where supported, and configure branch protection/required checks as appropriate.
No custom domain or CNAME is configured.

## Security and licensing findings

See `public-release-review.md` for the redacted evidence and limits.

- Gitleaks 8.30.1 scanned all **44 local commits** and **239 candidate public files**.
  Its sole candidate is a Cargo dependency-name false positive involving libcrux;
  no confirmed credential was found in this scope. The supplemental heuristic
  inspected **371 blobs** and identified three historical provider-guide versions
  with personal directory paths. Current text was sanitized; history remains
  subject to the owner's disclosure decision.
- The latest CI run's logs were downloaded privately and scanned with **zero
  Gitleaks findings**. The five latest run summaries all failed. The current
  artifact list is empty; older logs/deleted artifacts/other clones are not
  exhaustively covered. No secret values or raw log contents were printed.
- Fresh cargo-deny: **advisories failed; bans, licenses, sources passed**.
  Affected packages: chrono, crypto-bigint, libsqlite3-sys, pin-utils,
  rustls-webpki, tokio, url and webpki-roots. Exact advisory IDs are in the security
  review and public release-readiness page. No waivers were added.
- MIT remains effective. The official unmodified PolyForm Noncommercial 1.0.0
  text is a candidate, not a finalized LICENSE change. Copyright/contribution and
  artwork rights are not established by Git author names. Existing MIT grants
  and third-party licenses cannot be retroactively removed. No CLA is imposed.
- Third-party source licenses/provenance are retained; canonical Apache text and
  ringbuf MIT text accompany the initial image notice bundle. Full resolved Rust,
  Python, Debian/FFmpeg/Deno/provider notices and relevant source obligations need
  verification before distributing binaries/images.

## Verification results

| Check | Result |
| --- | --- |
| Rust formatting | Passed |
| Workspace Clippy, all targets/features, warnings denied | Passed; existing vendored ringbuf dependency emits a nonfatal lifetime-style warning |
| Workspace tests | 81 passed, 2 pre-existing live image-search tests ignored |
| Offline application check | Passed with SQLite in memory; no Discord login |
| Offline registry export and fresh snapshot comparison | Passed |
| Python setup/link/reference regression tests | 13 passed |
| Strict MkDocs build | Passed |
| Internal HTML links, fragments and assets | 2,812 checks passed |
| Desktop/mobile/search/Mermaid | Inspected locally; limits above |
| cargo-deny | Failed advisory gate; bans/licenses/sources passed |
| Docker build/start and isolated restore | Not run: Docker unavailable |
| Live Discord/provider acceptance | Not run in this initiative |
| GitHub Actions execution of new workflow | Pending reviewed integration; local checks do not certify hosted execution |

Material's installed release announces critical/security support ending May 5,
2027. Review framework support before that date while maintaining the requested
MkDocs site. External-link availability was not exhaustively swept; optional
online checking is supplied separately from the deterministic internal-link gate.

## Release checklist and next steps

- [x] Isolated worktree and supplied development preserved.
- [x] Documentation framework, eleven module guides and generated references.
- [x] Local docs, Rust and Python checks completed with results recorded.
- [x] Docker/source/prebuilt-plan, storage, updates and backups documented.
- [x] Community files, notices, licensing candidate and maintenance rules prepared.
- [x] Pages deployment configured without publication or VPS changes.
- [ ] Remediate or specifically review the eight dependency advisory failures.
- [ ] Test revised Docker build/start, graceful shutdown and recovery in isolation.
- [ ] Complete live Discord/voice/provider permission and interaction acceptance.
- [ ] Integrate approved original art and record credit/redistribution rights.
- [ ] Complete copyright/contribution review and approve any license transition.
- [ ] Complete binary/image third-party attribution/source obligations.
- [ ] Decide whether historical infrastructure disclosures can become public.
- [ ] Configure private reporting/contact routes and GitHub release protections.
- [ ] Obtain explicit approval before merge, public visibility, Pages/GHCR release
      or production deployment. No such action has been taken.

The accompanying changed-file inventory distinguishes inherited files from release
edits. Review the concrete candidate before approving any publication step.
