# Dependency security backport

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
