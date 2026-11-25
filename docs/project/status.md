# Release readiness

Momokun is currently version **0.9.0, pre-1.0**. The owner has accepted the running
bot after months of live use, and the dependency candidate passed a focused
voice trial. Public release and image distribution preparation remain in progress.

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

## Validation and limits

- A focused existing-bot trial passed operator-observed YouTube playback,
  pause/resume, skip, active-audio shutdown and playback after restart; the
  original image was restored afterward. The partial-title search fix found
  during that trial passed regression tests and Linux CI; it has not been deployed
  live. Linux x86_64 CI passed the revised
  image build, Compose configuration, offline startup, persistent storage,
  SQLite backup/restore and integrity checks. These used disposable storage
  without credentials or network access.
- The owner confirmed months of live Discord testing and acceptance for 1.0
  on 2026-11-24. This is operator-reported acceptance of the running bot;
  the new dependency build passed a second operator-observed voice trial
  and was rolled back afterward.
- The focused voice trial did not cover every Easynews or TorBox subscription
  plan. Local configured status alone is not proof of account authentication.
- ARM64 build, runtime and vendored assembly tests before advertising that target.
- The owner confirmed code/artwork rights and approved the three integrated
  branding derivatives. Binary distribution review retains three Rust attribution
  gaps and Deno linked-component scope limits. Source/notice acquisition and
  recipient source delivery are tracked separately from application acceptance.

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


The search correction passed Linux CI and the Docker build at commit `c1c2a33`.
It has not been deployed to the live bot. The exact BgUtils upstream source
archive is now retained with checksum provenance; installed-wheel equivalence
and the remaining runtime distribution obligations still require review.
