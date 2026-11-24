# Dependency remediation â€” 2026-11-24

The owner approved documentation and branding, then requested the next release
preparation step. This follow-up addresses dependency findings on the isolated
private review branch. It does not authorize production changes or publication.

## Compatible updates

| Package | Before | After | Finding removed |
| --- | --- | --- | --- |
| tokio | 1.53.1 | 1.54.1 | RUSTSEC-2026-0386 |
| rustls-webpki | 0.103.15 | 0.103.17 | RUSTSEC-2026-0382 |
| webpki-roots | 1.0.9 | 1.0.12 | RUSTSEC-2026-0381 |
| url | 2.5.8 | 2.5.9 | RUSTSEC-2026-0592 |
| form_urlencoded | 1.2.2 | 1.2.3 | URL dependency update |
| pin-utils | 0.1.0 | 0.1.1 | Still unmaintained |

Compatible parent updates were also checked and retained: Songbird 0.6.1,
davey 0.1.5, openmls 0.8.3 and SQLx 0.9.1. They do not remove the remaining
four findings. The refreshed dependency/license inventory contains 549 packages.

## Findings that remain

- **chrono / RUSTSEC-2026-0543:** unmaintained, with no patched version.
  Serenity and Songbird's serde-aux chain use it. Removing only Poise's chrono
  feature would not remove those dependencies.
- **pin-utils / RUSTSEC-2026-0441:** unmaintained, with no patched version.
  Songbird â†’ davey â†’ openmls â†’ fluvio-wasm-timer introduces it.
- **crypto-bigint / RUSTSEC-2026-0387:** patched in 0.7.6; the voice encryption
  stack currently resolves 0.5.5 through elliptic-curve 0.13.8 and p256/k256/p384.
  Cargo cannot substitute a semver-incompatible arithmetic API safely. Review
  upstream coordinated voice/cryptography upgrades rather than overriding it.
- **libsqlite3-sys / RUSTSEC-2026-0459:** patched in 0.39.0; SQLx 0.9.1 resolves
  0.37.0. The advisory concerns opening attacker-controlled database/WAL pairs.
  Keep the volume and backups private and do not import untrusted databases.
  This reduces exposure but does not clear the dependency gate. Review an
  upstream SQLx update or a separately reviewed SQLite backport.

`cargo deny --offline check` against the locally refreshed advisory database
confirms four remaining failures; bans, licenses and sources pass. No new
advisory waiver, cryptography patch or database migration was introduced.

Original audit reports record the eight-finding baseline; this record supersedes
that count for the updated lockfile. Docker is still unavailable locally.

## Validation

The final dependency update passed workspace tests (81 passed, two existing live
tests ignored), strict workspace Clippy, formatting, offline startup with an
in-memory SQLite database, strict MkDocs build, 2,883 internal link/asset checks
and 13 Python tests. Live Discord/provider and Docker runtime checks remain
pending. Existing vendored ringbuf emits a nonfatal lifetime-style warning.
