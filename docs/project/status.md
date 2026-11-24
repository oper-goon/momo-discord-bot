# Release readiness

Momokun is currently version **0.9.0, pre-1.0**. The source implements the features
in the module guides; some live integrations and deployment procedures still
require acceptance testing. Preparation is not a claim of production readiness.

## Security gate

A refreshed cargo-deny run on 2026-11-24 identified eight findings. Compatible
updates cleared all six vulnerability/unsoundness findings. Two maintenance-only
findings have dated, documented exceptions. The advisory, license, source and
ban gates now pass for the candidate dependency graph.

| Package | Advisory |
| --- | --- |
| chrono | Maintenance exception; review by 2026-12-20 |
| pin-utils | Native-target maintenance exception; review by 2026-12-20 |

The DAVE/OpenMLS adapters now use patched upstream arithmetic and preserve
existing protocol/key formats. Chrono and pin-utils remain unmaintained; their narrowly scoped
maintenance exceptions expire for review on 2026-12-20. See the
[dependency security policy](../security.md) for rationale and removal conditions.
Existing [vendored backports and exceptions](../security.md) remain separate
obligations; a green dependency scan alone would not validate their source patches.

## Acceptance still needed

- A focused existing-bot trial passed operator-observed YouTube playback,
  pause/resume, skip, active-audio shutdown and playback after restart; the
  original image was restored afterward. The partial-title search fix found
  during that trial still needs final CI/live verification. Linux x86_64 CI passed the revised
  image build, Compose configuration, offline startup, persistent storage,
  SQLite backup/restore and integrity checks. These used disposable storage
  without credentials or network access.
- The owner confirmed months of live Discord testing and acceptance for 1.0
  on 2026-11-24. This is operator-reported acceptance of the running bot;
  the new dependency build passed a second operator-observed voice trial
  and was rolled back afterward.
- Account-backed Easynews and TorBox behavior across supported plans. Local
  metadata/configured status is not proof of successful authentication.
- ARM64 build, runtime and vendored assembly tests before advertising that target.
- Full transitive binary/image notice collection and any required artist credit.
  The owner confirmed code and artwork rights; three generated derivatives have
  been integrated and approved.

## Publication status

GitHub Pages is configured in source with an approval gate; no site was published
by this initiative. No public GHCR image or official shared-bot invitation is
provided. Source-build installation instructions describe the prepared release
checkout. First-party code now uses PolyForm Noncommercial 1.0.0 following
the owner's confirmation of rights. Earlier MIT grants remain valid; third-party
components retain their licenses. No CLA has been imposed.

The release checklist is tracked in the repository. No separate accepted feature
roadmap exists; do not infer future support or delivery dates from this list.

## Latest release preparation

The new distribution collectors passed private Linux CI, recording 488 reachable
Rust packages, 311 Debian packages and 14 Python distributions. Exact source
archives and recovered original notices are retained. Three Rust attribution
gaps and final system/tool source obligations remain; this is not license
clearance for a public image.

Two explicitly authorized windows returned the original image/configuration.
The first ended before an audible result was supplied; the second passed
operator-observed YouTube audio, controls, active shutdown and post-restart
audio. Restored backup startup and post-rollback readiness passed. A Windows
archive line-ending issue was caught on the disposable copy and corrected
before any switch. The text-search follow-up is separate from that tested build.
