# Coordinated voice-backend upgrade

Review date: 2026-11-24. Private branch: `feature/public-release-docs`.

## Result

RUSTSEC-2026-0387 is remediated by upstream crypto-bigint 0.7.11,
ctutils 0.4.8, P-256 0.14 and HPKE 0.8. The production graph no longer contains
crypto-bigint 0.5. No vulnerability exception was added. Cargo-deny passes
advisory, license, source and ban checks. Existing dated maintenance exceptions
remain documented separately.

## Scope and provenance

Three MIT-licensed registry crates are adapted locally because no compatible
upstream release performs the whole upgrade. `vendor/README.md` records exact
source commits, license hashes, upstream API references and removal criteria.
Existing package APIs are preserved so Songbird can use them without bot
architecture changes. Production source changes are small P-256 API adapters;
no cryptographic arithmetic was rewritten. DAVE/OpenMLS core protocol/storage
implementations remain unchanged.

The old libcrux patches and SHA3 exceptions are no longer active; newer upstream
versions resolve through HPKE 0.8. Historical source copies remain for provenance.
Ringbuf's backport remains active and unchanged. A root lockfile guard prevents
reintroducing pre-fix crypto-bigint, and CI tests the adapted packages.
The workspace accurately declares Rust 1.94, already required by SQLx 0.9.1.
The existing pinned toolchain and Docker builder satisfy it.

## Compatibility evidence

- Six prototype tests passed: hash-ratchet regression, DAVE handshake/member-add
  epoch rotation, bidirectional P-256 0.13/0.14 signatures with persisted-key
  reload, RFC6979 signature vector, malformed-input rejection, and HPKE
  encryption/exported-secret matching.
- The integrated graph passes five retained backend tests. The old P-256 dev
  dependency is kept out of the production graph; the standalone mixed-version
  reference supplies the backward-compatibility comparison instead.
- The checked-in `tests/voice-interop` harness passed original-host/new-joiner
  and new-host/original-joiner scenarios. Each uses three peers, serialized key
  packages/proposals/commits/welcomes, matching privacy/authenticator/verification
  codes, epoch 1→2 rotation, six encrypted packet roundtrips, and tamper rejection.
  The scenarios total twelve packet roundtrips. Keys are generated in memory;
  no credentials, sockets or production data are used.
- CI builds both pinned helper workspaces and runs the test. The original
  reference intentionally includes affected dependencies and is strictly
  test-only, excluded from the bot graph and Docker build context.

## Practical limits

Final local validation passed: 83 workspace tests, five integrated backend tests,
strict workspace Clippy, formatting, offline startup, fresh generated registry,
13 Python tests, strict MkDocs and 2,884 links/assets. The root inventory contains
586 packages. A redacted Gitleaks scan covered 293 candidate files and reported
three generic-api-key matches in root/new-reference/legacy-reference Cargo.lock.
Triage verified all three contain only libcrux dependency-name strings, not
credentials. No real credential was identified in this follow-up scope. Linux
CI and Docker build results remain pending the branch push.

The owner reports months of live Discord testing and accepts that as the
running bot's 1.0 acceptance. This new dependency build has not been deployed.
Offline tests do not exercise Discord gateway negotiation, real RTP, decoded
Opus playback or reconnects. A focused voice smoke test of the new build is
appropriate before release; repeating months of acceptance is not required.
No production infrastructure, credentials or repository visibility changed.
Full container/third-party distribution validation remains separate.
