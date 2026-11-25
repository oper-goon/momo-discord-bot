# Momokun public-release preparation report

Review date: 2026-11-25. Isolated branch: `feature/public-release-docs`.
Private draft review: [PR #10](https://github.com/oper-goon/momo-discord-bot/pull/10).
The workspace version remains 0.9.0 while preparing the owner-accepted 1.0 release;
no release tag, public publication or permanent production upgrade was made.

## Executive summary

The documentation website, approved branding, beginner-friendly self-hosting,
community files and source-available licensing materials are implemented.
References are generated from the actual offline command registry/configuration.
The owner accepted the running bot after months of use. A separate dependency
candidate passed focused live voice testing and was rolled back as agreed.

The dependency vulnerability fixes pass Linux quality gates. Docker startup,
persistent storage, backup and restoration passed in isolation. Exact source
acquisition succeeded for all 216 Debian source packages and 14 Python sdists;
all 703 retained files were reverified locally with zero issues. Deno collection
retains all 1,042 locked registry archives and 20 exact native source archives.
The final Deno/npm packaging commit `0d2b550` passed Linux CI, including image
build, offline recovery, exact runtime-source acquisition and Deno collection.

Release evidence is not automatic legal clearance. Three bot-dependency
attribution/provenance gaps, conservative Deno inclusion limits and the owner's
historical-path disclosure decision remain recorded review items. The repository
is private, main is the default branch, and nothing was merged or published.

## Implemented changes and coverage

| Area | Result |
| --- | --- |
| Website | Material for MkDocs; pinned documentation dependencies; search, navigation, tables of contents, syntax highlighting, code copy, tabs, admonitions and Mermaid |
| Branding | Owner-approved monochrome logo/favicon/social derivatives, homepage hero/cards, accessible focus styles and reduced-motion support |
| Features | All 11 registered modules documented: Core, General, Utility, Administration, Moderation, Music, Download, Media, Image, TorBox and Social Embeds |
| Commands | 112/112 registry nodes: 92 slash leaves, 4 context actions, 16 groups; 108 slash-enabled and 95 prefix-enabled nodes including groups |
| Configuration | 31/31 configuration fields and 18 supported environment variables, with required/optional and sensitive distinctions |
| Self-hosting | Existing multi-stage Docker infrastructure improved; non-root runtime, persistent SQLite volume, bounded logs, restart/init settings, setup helper and operations/recovery guides |
| Maintenance | Offline reference generation, freshness/coverage/link/asset checks, mandatory AGENTS documentation rules and CI |
| Community | README, CONTRIBUTING, SECURITY, Code of Conduct, changelog, issue/PR templates, credits and branding guidance |
| Licensing | Official unmodified PolyForm Noncommercial text, retained historical MIT grants and third-party notices/source provenance |
| Dependencies | All six reviewed vulnerability/unsoundness findings remediated; dated maintenance exceptions remain documented |

See [changed-files.json](changed-files.json) for the full changed-path inventory,
including classification of inherited development. Major release-owned groups:

- `mkdocs.yml`, `requirements-docs.txt`, `docs/`, approved assets and theme overrides.
- Offline registry exporter, reference generators, setup/link/notice/source scripts
  and their regression tests.
- `Dockerfile`, `compose.yaml`, `.dockerignore`, `.env.example`, example configuration
  and explicitly unpublished prebuilt-image example.
- `.github/workflows/`, community policies, templates, `AGENTS.md`, `licenses/`
  and `THIRD_PARTY_NOTICES.md`.
- Reviewed dependency lockfiles/adapters and the narrowly scoped music search fix.

## Installation and image status

The source-build installation is documented in [Docker quick start](../self-hosting/index.md).
Use the reviewed source checkout and its supplied configuration template:

```sh
git clone https://github.com/oper-goon/momo-discord-bot.git
cd momo-discord-bot
cp .env.example .env
cp config/example.toml config/config.toml
# Set DISCORD_TOKEN in .env.
# In config/config.toml, choose global registration or set your test guild ID.
docker compose up -d --build
docker compose ps
docker compose logs --tail=100 bot
```

These public-facing commands target the repository after the approved release is
merged. The current private candidate must be selected explicitly for testing;
do not mistake default-main contents for this review branch. No production
credentials are required for offline `--check` validation.

Persistent data lives in the `momo-data` volume. Configuration is mounted read-only.
The documentation covers backups, restoration, logs, updates and graceful stops.
The bot has no unnecessary published port. Optional services remain optional.

No public GHCR image exists from this initiative. The prebuilt Compose file is
an example for a future approved image, not an installable published artifact.
Only Linux x86_64 is validated; ARM64 is not advertised. Base tags and apt indexes
are not fully digest/snapshot locked, so bit-for-bit reproducibility is not claimed.

## Validation results

| Check | Result |
| --- | --- |
| Rust formatting, strict workspace Clippy/tests, offline startup and cargo-deny | Passed Linux CI for the dependency/search candidate and sysroot/source integration |
| Voice adapters | Offline original/upgraded DAVE interoperability, epoch changes and encrypted roundtrips passed |
| Python regression tests | 43 passed locally; two Windows symlink tests skipped; Linux documentation validation passed |
| Documentation | Strict MkDocs build and 2,905 internal links/assets passed; generated references checked in CI |
| Docker/offline recovery | `fc3859f` built successfully; Compose configuration, repeated startup, volume persistence, backup, restore and integrity checks passed |
| Rust sysroot notices | Linux Rust 1.98.1; compiler commit 48a229ceaefd4985c50990b14116b6d856af0985; 19 original notice/metadata files including both generated COPYRIGHT reports |
| Runtime inventory | 311 Debian packages, 14 Python distributions, all 17 Debian common-license texts; no common-license collection issues |
| BgUtils equivalence | All three installed Python files match the retained 2.0.2 upstream source archive |
| Exact runtime sources | 216 Debian source packages and 14 Python sdists; 703 retained files reverified; zero acquisition issues |
| Deno acquisition | 1,042/1,042 locked registry archives and 20 native source archives retained; 1,704-file notice/provenance bundle prepared |
| Final Deno/npm image integration | Passed Linux CI 45618519701 for 0d2b550, including both source-collection steps; documentation jobs also passed |

Live acceptance is recorded in [voice-smoke.md](voice-smoke.md). The second
approved candidate window confirmed audible direct-YouTube playback,
pause/resume, skip, clean active shutdown (0.576 seconds), and audio after restart.
The original image/configuration was restored and readiness verified. The later
partial-title search correction passed regressions and Linux CI but is not live.
The owner does not need to repeat the entire historical acceptance program.

The focused trial did not cover every Easynews/TorBox subscription plan or ARM64.
Those limits must not be represented as tested support.

The final image evidence was extracted and inspected: all 311 Debian and 14
Python package/source identities match the locally retained exact-source bundle.
The unpacked Deno index contains all 1,042 crates and 20 native records with no
failed downloads; the embedded TypeScript notice and both npm source archives
are present. Source and notice artifacts are private and retained for review.

## Preview and Pages

The local website preview is `http://127.0.0.1:8000/`. Desktop and 390×844 mobile
layouts, search, drawer navigation and Mermaid rendering were inspected earlier.
Local ignored screenshots: [desktop](../../outputs/release-preview/desktop.jpg)
and [mobile](../../outputs/release-preview/mobile.jpg). Clipboard/screen-reader
acceptance was not exhaustive. The supplied mascot identity is preserved through
three approved generated derivatives; exact original chat uploads are not local files.

Five original theme/icon license files now accompany the website, with pinned-
package freshness checks in CI and links on the credits page.

Pages deployment uses official GitHub Pages artifact/deployment actions. Pull
requests validate without deployment. Publishing is gated to the default branch
and `PAGES_ENABLED=true`; deployment alone has Pages/OIDC write permissions.
There is no production-bot credential, VPS web service or uptime dependency.
Intended URL: `https://oper-goon.github.io/momo-discord-bot/`; not published.

## Security findings, redacted

A fresh Gitleaks 8.30.1 scan covered the 351-file tracked candidate snapshot at
`0d2b550` and all 56 available local commits. Both scans reported three candidates;
all were triaged as the known libcrux Cargo.lock dependency-name false positive,
including the isolated interoperability test lockfiles. No confirmed credential
was found in that scope. Reports are redacted and ignored locally.

Earlier heuristic history review identified three historical provider-guide
versions containing private home-directory/deployment paths. Current documents
are sanitized; no history rewrite occurred. The owner must decide whether those
historical disclosures are acceptable before repository visibility changes.
Excluded original screenshots/unrelated projects must not be added without review.
Deleted remote refs, other clones, older/deleted Actions artifacts and image OCR
were not exhaustively covered. No rotation, force push or infrastructure changes
were performed. See [public-release-review.md](public-release-review.md) for
historical findings; old dependency/rights blockers there are superseded by this report.

All four cargo-deny gates pass for the prepared dependency graph. Chrono and
pin-utils maintenance exceptions require review by 2026-12-20; existing narrowly
scoped backport/maintenance decisions remain in [security policy](../security.md).
A green Rust advisory gate does not claim a complete OS/native vulnerability audit.

## Licensing and source materials

The owner confirmed first-party and artwork rights and approved the documentation
and generated assets. The first-party candidate uses official, unmodified PolyForm
Noncommercial 1.0.0. Earlier MIT grants remain valid. Third-party code and sources
retain their own terms; they are not relicensed under PolyForm. No CLA is imposed.

Original notices, source archives, hashes and version provenance are retained.
The final image integrates Deno's conservative notice bundle, including its embedded
Microsoft TypeScript header, and exact Meriyah/Astring ISC/MIT notices and sources.
See [Deno review](deno-notices.md), [sysroot review](rust-toolchain-notices.md),
[distribution inventory](distribution-notices.md) and
[system source materials](system-source-materials.md).

Three bot-dependency notice/provenance questions remain: convert_case 0.4.0,
realfft 3.5.0 and serenity-voice-model 0.3.0. Their exact original source archives
and declared permissive terms are retained. No copyright names/dates or missing
upstream notice text were invented. Deno's whole-lockfile/repository superset is
not an exact linked-component or reproducible-build attestation; its index records
unresolved Chromium DEPS/inclusion limits and archives without standalone notices.

The exact Debian/Python source artifact (1.06 GB) and Deno source collection
(about 403 MB) are retained privately and locally. Opt-in CI regenerates matching
seven-day private artifacts. Before an approved binary/image release, retain and
provide matching source/notice materials to recipients as applicable. Private
expiring artifacts alone are not recipient source delivery or a written offer.

## Approval and manual configuration checklist

- [x] Implement documentation, branding, source-build installation and community files.
- [x] Generate and check command/configuration coverage.
- [x] Remediate reviewed dependency vulnerabilities and complete focused voice acceptance.
- [x] Test Linux container startup, storage, backup and restoration.
- [x] Collect exact runtime sources and Rust/Deno notice evidence.
- [x] Complete final Deno/npm image CI and retain release-specific evidence.
- [ ] Review the recorded third-party attribution/scope exceptions and historical-path disclosure.
- [ ] Obtain explicit approval for merge, public visibility, Pages, image publication or permanent deployment.
- [ ] After approval, configure GitHub Actions as the Pages source, the Pages environment,
      `PAGES_ENABLED`, branch protection/required checks and private security reporting.
- [ ] Prepare the reviewed 1.0 version/tag and publish matching release artifacts only after approval.

No public website, image, release or production promotion is authorized by this report.
Earlier chronological evidence is preserved in [release-history.md](release-history.md).
