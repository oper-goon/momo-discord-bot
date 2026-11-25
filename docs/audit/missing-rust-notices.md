# Missing Rust notice review

Review date: 2026-11-24. Scope: the 39 non-MPL packages without collected standalone notices in the supplied Linux notice index. This is an evidence review and proposed remediation, not final legal clearance.

Input lockfile SHA-256: `f9747515cfd9da5e9827c661262c77ed7fc345a6fc5a8db6b748ee28fe5a2e22`. The supplied collector graph contains 488 reachable packages and 59 unresolved notice records. The other 20 are MPL packages whose source archives are retained separately; their obligations were not re-reviewed here.

Only the exact package directories named by `.cache/linux-notice-metadata.json` were inspected. Evidence paths below are relative to each named, versioned package directory. Normalized `Cargo.toml`, packaged README files, source/header text, and packaged `.cargo_vcs_info.json` were examined. No network requests, external release retrieval, notice-bundle edits or license substitutions were performed.

## Findings

- **3 collector false positives:** the license terms are already packaged under SPDX filenames in `LICENSES/`. Their names do not match the current collector filename pattern.
- **36 packages still need release-specific notice recovery:** 20 Apache-only libcrux packages, 3 MIT OpenMLS packages, 6 ISC packages, and 7 other permissively licensed packages.
- The missing 36 have manifest license declarations but no standalone software copyright/permission notice located in their packaged README/source/header text. Cargo author metadata is not treated as a copyright notice. No copyright name or year has been inferred or fabricated.
- A package declaring a license does not establish that an arbitrary sibling package notice, generic license template or current upstream notice is the correct notice for this exact release.

## Recover locally: three collector false positives

| Exact package | Packaged evidence | Proposed resolution |
| --- | --- | --- |
| `crc-catalog 2.5.0` | `LICENSES/Apache-2.0.txt` (copyright application at line 189); `LICENSES/MIT.txt:3` explicitly identifies the copyright; `README.md:15â€“18` identifies the alternatives. | Collect both packaged license files verbatim. No network retrieval or invented notice is needed. |
| `enum-map 2.7.3` | `LICENSES/Apache-2.0.txt`, `LICENSES/MIT.txt`, `LICENSES/CC0-1.0.txt`; `README.md:2â€“4`, `CHANGELOG.md:2â€“5`, and `src/*.rs` carry SPDX attribution/license headers. `LICENSES/MIT.txt:3` is an upstream template, with literal placeholders. | Collect the packaged license terms and all distinct SPDX copyright/license header blocks from the README, changelog and source, preserving their file associations. Do not replace the MIT template placeholders with guessed names or years. |
| `enum-map-derive 0.17.0` | Same three `LICENSES/` filenames; `README.md:2â€“4`, `src/lib.rs:1â€“3`, `src/derive_enum.rs:1â€“4` and `src/derive_struct.rs:1â€“4` contain SPDX metadata. The MIT file again contains upstream placeholders. | Collect the license files and distinct SPDX source/README blocks verbatim. Keep their different attribution years and contributors; do not collapse them into a guessed project-wide notice. |

Collector change needed: recognize SPDX-named files inside a package-local `LICENSES/` directory, in addition to existing `LICENSE*`, `COPYING`, `NOTICE`, `COPYRIGHT` and `AUTHORS` filename patterns. Continue to reject symlinks and package-directory escapes. SPDX header extraction is a separate preservation step; simply collecting the MIT template is incomplete evidence.

## Apache-only libcrux family: 20 packages

All 20 normalized manifests declare `Apache-2.0`; their author field identifies Cryspen, but that is not substituted for a copyright statement. Packaged source and README/header text provided no standalone software notice. Most share one exact upstream revision; two utility crates use another, so one unversioned current-tree license copy is insufficient provenance.

Proposed resolution for every row: recover the upstream repository-root Apache license, any `NOTICE` file, and applicable bundled-code attributions from the recorded release commit; associate those exact bytes and provenance with each package. Include the full Apache terms in the distribution. Specifically check HACL-derived generated code and its upstream attribution instead of assuming the manifest covers every bundled component. If the exact release genuinely has no `NOTICE`, record that evidence rather than creating one.

| Exact package | Manifest evidence | Packaged README | Release revision / upstream subdirectory |
| --- | --- | --- | --- |
| `libcrux-aead 0.0.11` | `Cargo.toml:26` | `Readme.md` | A; `crates/primitives/aead` |
| `libcrux-aes 0.0.11` | `Cargo.toml:27` | `README.md` | A; `crates/algorithms/aes` |
| `libcrux-chacha20poly1305 0.0.11` | `Cargo.toml:26` | `Readme.md` | A; `crates/algorithms/chacha20poly1305` |
| `libcrux-curve25519 0.0.10` | `Cargo.toml:26` | `Readme.md` | A; `crates/algorithms/curve25519` |
| `libcrux-ecdh 0.0.10` | `Cargo.toml:27` | `Readme.md` | A; `libcrux-ecdh` |
| `libcrux-hacl-rs 0.0.7` | `Cargo.toml:26` | `Readme.md` | A; `crates/utils/hacl-rs` |
| `libcrux-hkdf 0.0.10` | `Cargo.toml:26` | `Readme.md` | A; `crates/algorithms/hkdf` |
| `libcrux-hmac 0.0.10` | `Cargo.toml:26` | `Readme.md` | A; `crates/algorithms/hmac` |
| `libcrux-hmac-drbg 0.0.3` | `Cargo.toml:26` | `Readme.md` | A; `crates/algorithms/hmac-drbg` |
| `libcrux-intrinsics 0.0.10` | `Cargo.toml:27` | `Readme.md` | A; `crates/utils/intrinsics` |
| `libcrux-kem 0.0.11` | `Cargo.toml:27` | `README.md` | A; `libcrux-kem` |
| `libcrux-macros 0.0.4` | `Cargo.toml:26` | `Readme.md` | B; `crates/utils/macros` |
| `libcrux-ml-kem 0.0.12` | `Cargo.toml:35` | `README.md` | A; `libcrux-ml-kem` |
| `libcrux-p256 0.0.10` | `Cargo.toml:26` | `Readme.md` | A; `crates/algorithms/p256` |
| `libcrux-platform 0.0.4` | `Cargo.toml:26` | `Readme.md` | B; `crates/sys/platform` |
| `libcrux-poly1305 0.0.8` | `Cargo.toml:26` | `Readme.md` | A; `crates/algorithms/poly1305` |
| `libcrux-secrets 0.0.8` | `Cargo.toml:27` | `README.md` | A; `crates/utils/secrets` |
| `libcrux-sha2 0.0.10` | `Cargo.toml:26` | `Readme.md` | A; `crates/algorithms/sha2` |
| `libcrux-sha3 0.0.12` | `Cargo.toml:33` | `README.md` | A; `crates/algorithms/sha3` |
| `libcrux-traits 0.0.10` | `Cargo.toml:26` | `README.md` | A; `traits` |

Recorded libcrux source revisions from each packaged `.cargo_vcs_info.json`:

- A: `24330863b3d1d4179e4bcea8cef6bf6fa313057e` (18 packages).
- B: `42d68bd49b244b09bd02626c6ffa95e54dc64f91` (`libcrux-macros` and `libcrux-platform`).

## MIT OpenMLS family: three packages

Each manifest declares `MIT` and identifies OpenMLS Authors in author metadata. No packaged standalone notice or source/README copyright/permission block was found. Recover the OpenMLS repository license and any applicable notices from the exact commit in each row; do not borrow the vendored provider license merely because it is in the same project. These releases do not all share the same commit.

| Exact package | Evidence | Recorded release commit / subdirectory |
| --- | --- | --- |
| `openmls 0.8.3` | `Cargo.toml:34`; `README.md`; source contains no located notice block | `1f4b2f14da510c2fdada5d1bbf2cdbe20dd2cf73` / `openmls` |
| `openmls_memory_storage 0.5.0` | `Cargo.toml:26`; `README.md`; source contains no located notice block | `6b85f0edc560b4fe0f5b9266092947a774614f3f` / `memory_storage` |
| `openmls_traits 0.5.0` | `Cargo.toml:26`; `README.md`; source contains no located notice block | `6b85f0edc560b4fe0f5b9266092947a774614f3f` / `traits` |

## ISC family: six packages

Each declares `ISC`. No packaged ISC permission/copyright block was found. Recover the precise upstream release license/copyright notice and retain it verbatim, including any years and holders actually supplied by upstream. Do not synthesize it from Cargo authors. Songbird README/source discussion of its logo artwork is not the software license notice.

| Exact package | Evidence | Recorded release commit / subdirectory |
| --- | --- | --- |
| `serenity-voice-model 0.3.0` | `Cargo.toml:36`; no packaged README | `cef32674d5ad6c539b1ee2ff9dcda39b6d71e18f` / `(repository root)` |
| `songbird 0.6.1` | `Cargo.toml:39`; `README.md`; `README.md:69` / `src/lib.rs:52` discuss logo copyright only | `957e17fcbc687cd88979c90b76bdf79017803b09` / `(repository root)` |
| `stream_lib 0.5.2` | `Cargo.toml:21`; `README.md` | `342e78a27dd2fca5da99d07ad083406cdbc2bd52` / `stream_lib` |
| `twilight-gateway 0.17.1` | `Cargo.toml:42`; `README.md` | `b4ce13b727e7731b917576ad977300ab6926bb6b` / `twilight-gateway` |
| `twilight-gateway-queue 0.17.0` | `Cargo.toml:37`; `README.md` | `007889d01731d75a0dadcee106795de21235acb4` / `twilight-gateway-queue` |
| `twilight-model 0.17.1` | `Cargo.toml:37`; `README.md` | `b4ce13b727e7731b917576ad977300ab6926bb6b` / `twilight-model` |

## Other permissive packages: seven packages

| Exact package | Declared terms / packaged evidence | Proposed resolution | Recorded source commit / subdirectory |
| --- | --- | --- | --- |
| `convert_case 0.4.0` | `MIT`; `Cargo.toml:22`; `README.md` has no located notice block | Recover the exact upstream release MIT permission/copyright notice; manifest/README license labels alone are not that notice. | `300d9eef9e8c970e15f324afbc3df8b6a4ebbf71` / `(repository root)` |
| `from_map 0.1.0` | `Unlicense OR MIT OR Apache-2.0`; `Cargo.toml:19`; no packaged README or located source notice block | Recover the exact repository release terms. Preserve the original alternative-license declaration; do not treat the Unlicense alternative as an automatic waiver of provenance review. | `c024f6840d1acd913fb04cf13aea97bc024dfc39` / `(repository root)` |
| `hashmap_derive 0.1.0` | `Unlicense OR MIT OR Apache-2.0`; `Cargo.toml:19`; no packaged README or located source notice block | Recover the exact repository release terms. Preserve the original alternative-license declaration; do not treat the Unlicense alternative as an automatic waiver of provenance review. | `50f44721a4eebeefef085e68135b9545a590bf79` / `(repository root)` |
| `poise_macros 0.7.0` | `MIT`; `Cargo.toml:25`; no packaged README or located source notice block | Recover the Poise repository license at this macros release commit, not an unrelated later main-package release. | `9dee0a4887bb9d6c0bccc60a1bc2e10ac87097e5` / `macros` |
| `realfft 3.5.0` | `MIT`; `Cargo.toml:38`; `README.md:184` says License: MIT, without a copyright/permission block | Recover the exact upstream release MIT permission/copyright notice; manifest/README license labels alone are not that notice. | `d0d4eee0525fd27c96c8a046d6d107acd5ed84a6` / `(repository root)` |
| `streamcatcher 1.0.1` | `MIT/Apache-2.0`; `Cargo.toml:24`; no packaged README or located source notice block | Recover the release MIT and Apache license files/notices. Preserve the raw legacy `MIT/Apache-2.0` metadata in evidence; verify the intended alternatives before normalizing its syntax. | `bfc9825a5b9f39e20a9aa824ba2e4e302e485f90` / `(repository root)` |
| `typesize-derive 0.1.11` | `MIT`; `Cargo.toml:25`; no packaged README or located source notice block | Recover the typesize repository release license/attribution. This normalized manifest has no authors field; do not invent an owner or year. | `87ee70113c5689ca009e38ce663f9905c90a7a7c` / `typesize-derive` |

## Pinned upstream recovery map

The repository URLs below are Cargo metadata evidence, not an assumption that a current branch has the same terms as an old release. Combine the repository root with the full commit and `path_in_vcs` recorded in the package tables. URLs containing `tree/main` or `tree/master` identify a monorepo subdirectory only: use the recorded commit instead of fetching the mutable branch. Candidate upstream license/notice filenames have not been verified by network access in this review.

| Packages from the tables above | Declared repository | Repository root to inspect at the recorded commit |
| --- | --- | --- |
| All 20 `libcrux-*` packages | `https://github.com/cryspen/libcrux` | `cryspen/libcrux` at revisions A and B, then the respective package subdirectory. |
| `openmls 0.8.3` | `https://github.com/openmls/openmls/` | `openmls/openmls` at `1f4b2f14da510c2fdada5d1bbf2cdbe20dd2cf73`. |
| `openmls_memory_storage 0.5.0` | `https://github.com/openmls/openmls/tree/main/memory_storage` | `openmls/openmls` at `6b85f0edc560b4fe0f5b9266092947a774614f3f`, including `memory_storage/`. |
| `openmls_traits 0.5.0` | `https://github.com/openmls/openmls/tree/main/traits` | Same OpenMLS revision as memory storage, including `traits/`. |
| `convert_case 0.4.0` | `https://github.com/rutrum/convert-case` | `rutrum/convert-case` at its table's full commit. |
| `from_map 0.1.0`, `hashmap_derive 0.1.0` | `https://github.com/Luro02/shorthand` | `Luro02/shorthand` at each package's different recorded commit. |
| `poise_macros 0.7.0` | `https://github.com/serenity-rs/poise/` | `serenity-rs/poise` at the recorded commit, including `macros/`. |
| `realfft 3.5.0` | `https://github.com/HEnquist/realfft` | `HEnquist/realfft` at the recorded commit. |
| `serenity-voice-model 0.3.0` | `https://github.com/serenity-rs/serenity.git` | `serenity-rs/serenity` at the recorded commit. |
| `songbird 0.6.1` | `https://github.com/serenity-rs/songbird.git` | `serenity-rs/songbird` at the recorded commit. |
| `stream_lib 0.5.2` | `https://github.com/Erk-/rsget/tree/master/stream_lib` | `Erk-/rsget` at the recorded commit, including `stream_lib/`. |
| `streamcatcher 1.0.1` | `https://github.com/FelixMcFelix/streamcatcher.git` | `FelixMcFelix/streamcatcher` at the recorded commit. |
| `twilight-gateway 0.17.1`, `twilight-model 0.17.1` | `https://github.com/twilight-rs/twilight.git` | `twilight-rs/twilight` at `b4ce13b727e7731b917576ad977300ab6926bb6b`, including each package subdirectory. |
| `twilight-gateway-queue 0.17.0` | `https://github.com/twilight-rs/twilight.git` | Same repository at the distinct commit `007889d01731d75a0dadcee106795de21235acb4`, including `twilight-gateway-queue/`. |
| `typesize-derive 0.1.11` | `https://github.com/GnomedDev/typesize` | `GnomedDev/typesize` at the recorded commit, including `typesize-derive/`. |

For each upstream recovery, retain the selected file's exact repository-relative path, commit, original bytes and hash. Check both repository-level and package-local terms and any bundled-code notices. A missing upstream file should be recorded as missing rather than replaced with a fabricated release notice.

## Collector follow-up status

The coordinating agent reports that the collector now recognizes the three local `LICENSES/` cases and retains checksum-verified `.crate` archives for all 470 registry packages plus actual modified source archives for four vendored packages. This addresses source-header preservation and reduces standalone-missing records from 59 to 56. Of those, 20 are MPL packages with retained source and **36 are the non-MPL exact-release notice recoveries listed above**. This follow-up does not change the local evidence or turn the 36 pending upstream recoveries into verified notices.

## Release checkpoint

Completed: exact-version local inspection of all 39 requested package directories and identification of three locally recoverable collector omissions. No copyright text was invented and no package source was modified.

Pending: verify the follow-up collector output and distinct SPDX attribution preservation; retrieve and verify the exact-release notices for the remaining 36; preserve hashes and upstream revision/path provenance in the final notice index; perform final distribution review. The retained MPL source archives and their notice/source-access requirements remain a separate checklist item.

This report does not declare the notice bundle complete or authorize public image publication.

## Retrieval follow-up

33 of the 36 missing non-MPL package notices were subsequently recovered from
the exact upstream Git revisions using GitHub tree/blob APIs. Original bytes,
URLs, revision bindings and SHA-256 digests are retained in
`licenses/rust-supplements.json` and `licenses/upstream/`. The three packaged
SPDX-directory omissions are fixed, and full original archives preserve headers.
The collector now reports three standalone notice gaps: convert_case 0.4.0 and
realfft 3.5.0 exact upstream trees contain no matching standalone notice; the
serenity-voice-model 0.3.0 recorded commit could not be resolved in its declared
repository. These are not silently filled using unrelated later notices.
No final distribution clearance is asserted.

## Final three gaps: second packaged-source review

A second, read-only review examined every packaged file in the metadata-declared directories for `convert_case 0.4.0`, `realfft 3.5.0`, and `serenity-voice-model 0.3.0`. This included original and normalized manifests, VCS metadata, all Rust source/tests/examples/benchmarks, README text where present, dotfiles and packaged workflows. No additional software copyright/permission block was found. No external repository was queried in this review; the earlier upstream retrieval results above remain the coordinating agent's recorded evidence.

### `convert_case 0.4.0`

- `Cargo.toml.orig:7` explicitly declares `MIT`; `Cargo.toml.orig:11` declares `https://github.com/rutrum/convert-case`.
- `README.md` and `src/lib.rs:3–7` document the move of the `ccase` command-line application into a separate crate. The packaged source here is the library, not that separate application. A notice for `ccase` alone cannot be assumed to be this release's notice.
- The package contains `src/case.rs`, `src/lib.rs`, `src/words.rs`, the README and manifest/VCS metadata, but no license file or copyright/permission header. The author entry in `Cargo.toml.orig:4` supplies author metadata, not a copyright year or an entire MIT permission notice.
- `.cargo_vcs_info.json` records commit `300d9eef9e8c970e15f324afbc3df8b6a4ebbf71` but **does not contain a `path_in_vcs` field**. Earlier table wording that treated this omitted field as a proved repository-root path should not be relied upon. The source subdirectory is not recorded by this old package metadata.
- The supplied resolved dependency graph identifies the direct consumer as `derive_more 0.99.20`.

Safe resolution: retain the checksum-verified original archive and the manifest MIT declaration as evidence; request an exact-release licensing/attribution clarification from upstream, or obtain authoritative evidence explicitly covering this release. A later library release's or the CLI crate's copyright notice is not automatically proof for 0.4.0. Replacing/upgrading the parent dependency is another possible engineering route, requiring separate compatibility checks rather than an unreviewed notice substitution.

### `realfft 3.5.0`

- `Cargo.toml.orig:7` declares `MIT`; line 8 identifies `https://github.com/HEnquist/realfft`.
- `README.md:184` explicitly states `License: MIT`. `src/lib.rs:1` embeds this README with `include_str!`, so the MIT declaration is also incorporated into the crate's documentation. This is additional affirmative license-label evidence, but it is not a full copyright/permission notice.
- No additional notice appears in the library, `examples/concurrency.rs`, `benches/realfft.rs`, or the two packaged GitHub workflows. No standalone license is packaged. The manifest author identity does not establish a copyright year.
- `.cargo_vcs_info.json` records commit `d0d4eee0525fd27c96c8a046d6d107acd5ed84a6` and explicitly records `path_in_vcs` as the empty string. Unlike the convert_case metadata, this is a recorded repository-root package path.
- The supplied resolved graph identifies the direct consumer as `rubato 0.16.2`.

Safe resolution: preserve the original README declaration and source archive, and seek exact-release notice clarification from upstream. Do not copy RustFFT's notice just because RealFFT wraps RustFFT: they are separate packages. Do not manufacture a `Copyright` line from the manifest author and release date. A separately supplied full MIT license text can be identified as standard license terms, but must not be mislabeled as a recovered upstream release notice or used to conceal the attribution uncertainty.

### `serenity-voice-model 0.3.0`

- `Cargo.toml.orig:13` declares `ISC`. Lines 9–11 explicitly use Serenity's documentation, homepage and `https://github.com/serenity-rs/serenity.git` repository addresses; these fields are present directly in the published original manifest, not merely synthesized by this project's dependency metadata.
- `Cargo.toml.orig:7` restricts the package include list to `src/**/*.rs` and `Cargo.toml`; the README declaration is commented out at line 6. Thus an upstream root-level license/notice would not be included by that stated source include list. This explains a packaging omission mechanism, but does **not** prove which upstream license file existed or its exact contents.
- All packaged Rust files, including `src/event/tests.rs`, lack a located software copyright/permission notice. The author entry at `Cargo.toml.orig:4` is not substituted for an ISC copyright line.
- `.cargo_vcs_info.json` explicitly records a root package path (`path_in_vcs: ""`) at commit `cef32674d5ad6c539b1ee2ff9dcda39b6d71e18f`. The earlier retrieval could not resolve that commit in the declared Serenity repository. Local package evidence contains no alternative repository address identifying the actual origin of this commit. A similarly named separate repository, a different Serenity branch, or a later Serenity license is therefore not established provenance for this release by the evidence available here.
- The supplied resolved graph identifies the direct consumer as `songbird 0.6.1`.

Safe resolution: have the upstream maintainer identify the repository/ref corresponding to this exact published crate and confirm which license/notice covers it, then retain the response and any retrieved file's immutable revision/path/hash. Alternatively, select a compatible release with verified provenance after the necessary voice integration checks. Do not describe the unreachable recorded commit as proof that the package is unlicensed or malicious; it is a source-provenance gap. Do not relabel a different Serenity release's ISC notice as the missing exact-release notice.

### Remaining uncertainty and release decision

All three packages affirmatively declare permissive license identifiers, and RealFFT repeats its identifier in embedded documentation. The outstanding question is the authoritative exact-release copyright/permission notice or confirmation that no additional upstream notice exists; for serenity-voice-model there is also an unresolved repository/commit binding. Packaged author metadata, a guessed year, a current branch's unrelated notice, and a sibling project's license do not resolve those questions. No new notice recovery or distribution clearance resulted from this second review. Keep these three exceptions visible until the release owner has reviewed authoritative clarification or a validated dependency replacement.


## Owner review decision — 2026-11-25

The owner reported independently reviewing these three dependencies and accepted
the recorded exceptions for publication. Preserve the evidence above and the
original notices/source archives. This decision does not assert that missing
upstream notices were recovered or authorize inventing attribution text.
