# Dependency security backport

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
