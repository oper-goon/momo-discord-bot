# Dependency remediation â€” 2026-11-24

The owner approved documentation and branding, then requested the next release
preparation step. This follow-up addresses dependency findings on the isolated
private review branch. It does not authorize production changes or publication.

## Compatible updates

### Coordinated backend upgrade — current status

The final voice vulnerability is remediated by the backend adapters:
crypto-bigint 0.7.11, P-256 0.14, HPKE 0.8 and ctutils 0.4.8.
All four cargo-deny gates pass. See [the upgrade record](voice-upgrade.md) for
scope, provenance, compatibility evidence and remaining live-test limits.
The sections below preserve the earlier investigation and are historical.

### Latest dependency pass and operator acceptance

The owner confirmed months of live Discord testing and confidence in a 1.0
release. This is accepted as operator-reported acceptance of the running bot;
it does not claim that this new dependency build has been deployed.

- Updated `libsqlite3-sys` 0.37.0 → 0.39.1, bundling SQLite 3.54.1.
  SQLx 0.9.1 already permits `>=0.30.1,<0.41.0`, so this is a supported
  lockfile update, not a vendor patch. The database regression test queries
  the linked engine and requires at least 3.54.0. Migration, persistence and
  snapshot restoration tests pass against the patched engine.
- Reviewed RUSTSEC-2026-0543 and RUSTSEC-2026-0441 as maintenance-only findings.
  Both have explicit, narrow review exceptions due by 2026-12-20 and on
  dependency/target changes; neither is represented as patched or maintained.
  Global vulnerability/unmaintained checking remains enabled.
- Tested removal of Poise's Chrono feature. It removes `Context::id`, which
  existing image/media/TorBox pickers use, so the feature was retained to
  preserve behavior. Chrono also remains through Songbird's serde-aux defaults.
- Reviewed the pin-utils path: OpenMLS 0.8.3 uses `std::time` for native
  lifetimes and imports Fluvio only under `target_arch = "wasm32"`. This
  exception applies to the native bot, not a WebAssembly deployment.
- **RUSTSEC-2026-0387 remains a blocker.** DAVE uses P-256 for actual signatures
  and key exchange. Davey's current upstream 0.2.0 manifest still uses the old
  P-256/OpenMLS chain. The upstream OpenMLS provider migration uses newer
  P-256/HPKE APIs and prerelease provider versions; it is not a compatible
  lockfile replacement. No maintained 0.5 backport was found. The upstream
  constant-time fix spans several arithmetic operations and cross-target
  verification. No partial local crypto patch or vulnerability waiver was added.

Evidence: [SQLx 0.9.1 manifest](https://github.com/transact-rs/sqlx/blob/v0.9.1/sqlx-sqlite/Cargo.toml),
[SQLite advisory](https://rustsec.org/advisories/RUSTSEC-2026-0459.html),
[crypto advisory](https://rustsec.org/advisories/RUSTSEC-2026-0387.html),
[Davey manifest](https://raw.githubusercontent.com/Snazzah/davey/master/davey/Cargo.toml),
[OpenMLS provider migration](https://raw.githubusercontent.com/openmls/openmls/main/openmls_rust_crypto/Cargo.toml).

Latest validation: 82 workspace tests passed; two existing live-search tests
remain ignored. Strict workspace Clippy, formatting, offline in-memory startup,
13 Python tests, strict documentation build and 2,884 internal links/assets pass.
The refreshed inventory remains 549 packages. Cargo-deny reports one unwaived
vulnerability; bans, licenses and sources pass. The existing ringbuf dependency
emits a nonfatal lifetime-style warning. No production changes were made.

### Prior pass baseline

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

## Findings before the latest dependency pass

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
