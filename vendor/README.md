# Dependency security backport

## Active DAVE/OpenMLS backend adapters

The following registry packages retain their existing API versions while using
patched, stable upstream cryptographic implementations:

| Directory | Registry base | Upstream commit |
| --- | --- | --- |
| `davey` | 0.1.5 | `73bcec27b7190ce47088c831e0aa72600cc657e7` |
| `openmls_basic_credential` | 0.5.0 | `6b85f0edc560b4fe0f5b9266092947a774614f3f` |
| `openmls_rust_crypto` | 0.5.1 | `47dbedecad0c1fd8eb5368d582250ebfcc1e1ce6` |

Their `.cargo_vcs_info.json` and `Cargo.toml.orig` retain upstream provenance.
The original MIT LICENSE texts were retrieved at those exact commits. Davey's
license SHA-256 is `90760006B6E6C76A67476DE39BEE25E42E584679EAC5D608765355D5D54E8AFA`;
both OpenMLS notices have SHA-256
`43E5E3C4B5CCA67F9EA912F7E1929702A848AA765D3B3E14F25D3838A5A5565D`.

The only production source changes adapt P-256 0.14 key generation, SEC1 public
key conversion, checked scalar-byte conversion and signer-trait imports. The
provider's HPKE dependency family moves to 0.8. Existing OpenMLS 0.8.3,
traits/storage 0.5, DAVE protocol 1, raw 32-byte private keys, uncompressed
SEC1 public keys, DER signatures and ciphersuites are preserved. Ed25519 and
the existing OpenMLS random-provider interface retain their older RNG API;
P-256 uses the OS RNG via the newer upstream API. No arithmetic or protocol
implementation is locally rewritten. API conversions follow the corresponding
[upstream provider](https://github.com/openmls/openmls/blob/main/openmls_rust_crypto/src/provider.rs)
and [credential implementation](https://github.com/openmls/openmls/blob/main/basic_credential/src/lib.rs).

The root lockfile now resolves crypto-bigint 0.7.11 and ctutils 0.4.8, fixing
RUSTSEC-2026-0387. Keep the bot-app lockfile regression guard, backend package
tests, RFC6979 vector, HPKE tests and offline DAVE handshake/epoch tests.
`tests/voice-interop` additionally compares original registry and adapted peers
in separate processes. Its legacy reference is test-only and deliberately
retains the original affected dependencies; it must never enter the bot graph
or runtime image. Do not claim offline tests validate live RTP/Opus playback.

Remove each adapter and its Cargo patch together once compatible upstream
Davey/OpenMLS releases use patched arithmetic and pass the same tests. Do not
replace these adapters with a vulnerability exception or enable crypto-debug
features. Broader binary/image attribution remains a separate release check.

## Historical libcrux backports

The two libcrux copies below are preserved as historical source/provenance.
Their Cargo patches are no longer active after HPKE 0.8 resolves newer upstream
versions. The prior SHA3 exceptions were removed. Their old source regression
guards remain as provenance protection; the active ringbuf backport below still
requires its own guard. Historical descriptions below explain why they existed.

`libcrux-chacha20poly1305/` is crates.io `libcrux-chacha20poly1305` 0.0.7
(Apache-2.0), retained because Songbird 0.6's HPKE dependency chain pins
`libcrux-aead` 0.0.7, which pins this exact version. The package's
`.cargo_vcs_info.json` records its upstream commit.

The `encrypt` implementation in `src/impl_hacl.rs` backports the upstream
0.0.8 fix for
[RUSTSEC-2026-0124](https://rustsec.org/advisories/RUSTSEC-2026-0124). It
separates the fixed-size authentication tag from any excess output-buffer
capacity instead of converting the entire remainder into a tag and panicking.
The upstream regression case is included in `tests/chachapoly.rs`. No public API
or cryptographic operation was changed.

Cargo-deny does not match registry advisories against path dependencies. Keep
the source regression guard in `bot-app`; a green advisory scan alone is not
evidence that the patch remains present. Remove this vendor directory, patch
entry, regression guard, and security-policy note together once Songbird's
dependency chain supports `libcrux-chacha20poly1305` 0.0.8 or later.

`libcrux-secrets/` is crates.io `libcrux-secrets` 0.0.5 (Apache-2.0), retained
because Songbird 0.6's OpenMLS dependency chain pins this API and hax 0.3.6.
The package's `.cargo_vcs_info.json` records its upstream commit.

Exactly four assembly instructions in `src/int.rs` were changed:

```diff
- "cmp {cond:w}, 0",
+ "tst {cond:w}, 0xff",
```

This is the selector-mask fix shipped by upstream libcrux-secrets 0.0.6 for
[RUSTSEC-2026-0212](https://rustsec.org/advisories/RUSTSEC-2026-0212).
The four AArch64 select32/select64/swap32/swap64 operations now test only the
defined low byte of the selector, matching the upstream fix. No x86 assembly
or public API was changed. Upgrading the whole crate to 0.0.6 is currently
incompatible with the upstream hax version pins.

Cargo-deny does not match registry advisories against path dependencies. The
vendored patch therefore requires explicit review and a regression guard;
a green advisory scan alone is insufficient. Keep the patch until Songbird's dependency chain supports
the corrected upstream crate, then remove the vendor directory, patch entry,
and regression guard together. ARM execution remains a release acceptance
check; local validation was on Windows x86_64.

## ringbuf

`ringbuf/` is crates.io `ringbuf` 0.4.8 (MIT OR Apache-2.0), retained because
Songbird 0.6 requires the `0.4` API. The package's `.cargo_vcs_info.json`
records its upstream commit.

`src/traits/consumer.rs` backports the `Consumer::skip`/`clear` drop guard from
upstream `ringbuf` 0.5.2. The guard advances the read index during unwinding if
an element destructor panics, preventing already-dropped elements from being
dropped again. This fixes
[RUSTSEC-2026-0293](https://rustsec.org/advisories/RUSTSEC-2026-0293).
No public API or other implementation was changed.

Cargo-deny does not match registry advisories against path dependencies. Keep
the source regression guard in `bot-app`; a green advisory scan alone is not
evidence that the patch remains present. Remove this vendor directory, patch
entry, regression guard, and security-policy note together once Songbird uses
`ringbuf` 0.5.2 or later.
