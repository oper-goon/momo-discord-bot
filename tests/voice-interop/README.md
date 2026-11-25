# Offline DAVE interoperability test

This test checks that the upgraded, vendored voice crypto providers exchange
the same DAVE/MLS wire messages and encrypted audio packets as the original
registry Davey 0.1.5 stack. The peers run in separate processes so Cargo cannot
silently apply upgraded provider patches to the reference peer. Both binaries
compile the same [RPC helper source](shared/main.rs).

**The legacy reference stack contains dependencies with known advisories. It is
test-only.** It runs generated in-memory data, without Discord credentials,
network access or production files. Never deploy the legacy helper, add its
dependencies to the bot workspace, or include it in a production image.
The manifests are separate, unpublished Cargo workspaces. Only `new` applies
the three voice-provider patches from `vendor/`.

## Run on Linux or macOS

From the repository root, with the Rust toolchain and Python 3 installed:

```sh
cargo build --locked --manifest-path tests/voice-interop/new/Cargo.toml
cargo build --locked --manifest-path tests/voice-interop/legacy/Cargo.toml
python3 tests/voice-interop/check_interop.py \
  --new tests/voice-interop/new/target/debug/dave-interop-new \
  --old tests/voice-interop/legacy/target/debug/dave-interop-old
```

For Windows, use `scripts/dev.ps1` to select the configured toolchain. From the
repository root:

```powershell
./scripts/dev.ps1 --% build --locked --manifest-path tests/voice-interop/new/Cargo.toml
./scripts/dev.ps1 --% build --locked --manifest-path tests/voice-interop/legacy/Cargo.toml
python tests/voice-interop/check_interop.py --new tests/voice-interop/new/target/debug/dave-interop-new.exe --old tests/voice-interop/legacy/target/debug/dave-interop-old.exe
```

If you set `CARGO_TARGET_DIR`, use the actual binary paths from that directory
instead. Dependency downloads are needed for the first build. The test itself
uses no network, accounts or API keys. After dependencies are cached, add
`--offline` to the Cargo commands to prohibit build-time network access too.
The Python orchestrator uses only the standard library, limits each RPC to
30 seconds, cleans up subprocesses and exits unsuccessfully on a mismatch.

## What passes

The orchestrator runs both original-host/upgraded-joiner and
upgraded-host/original-joiner scenarios. Each scenario verifies:

- A generated delivery-service signing identity and external sender package.
- Serialized key package validation, Add proposal, commit and welcome exchange.
- Matching group membership, epoch authenticator, privacy code and pairwise
  verification code.
- Bidirectional, non-silence audio encryption/decryption. Ciphertext must differ
  from plaintext; silence packets would bypass this check in Davey.
- A third member joining, remote commit processing, epoch/key rotation and
  additional audio roundtrips across backend versions.
- Rejection of a modified encrypted packet.

A successful run reports two scenarios and twelve audio roundtrips. Output
contains counts and status, without generated keys or packet contents.

## What this does not prove

The input packets exercise DAVE framing and cryptography; the test does not
decode Opus or connect to Discord. A live Discord acceptance test remains
necessary for gateway negotiation, real DAVE transitions, RTP transport,
voice reconnects and audible playback. Do not interpret this test as a security
audit of the entire crypto implementation or a replacement for upstream
cryptographic known-answer tests.

Preserve the original reference lockfile when updating the production backend.
When intentionally updating the reference baseline, review its registry source
provenance and advisories and rerun both scenarios. Do not add provider patches
to the legacy manifest.
