# Deno 2.9.4 runtime notice review

Review date: 2026-11-25. This review covers the Deno runtime copied into the
Momokun image, not the bot's own Rust dependency graph. Evidence collection
does not automatically establish complete distribution clearance.

## Exact release and CLI behavior

Deno is pinned to `2.9.4`, source revision
`14eea3160ae5834476aa3b9d317b8d41d991b982`. The exact source's CLI parser and
subcommand dispatcher do **not** provide `deno --license` or a `deno license`
subcommand. Do not use that command as a supposed complete notice collector.
The inspected files are the pinned
[CLI argument definitions](https://github.com/denoland/deno/blob/14eea3160ae5834476aa3b9d317b8d41d991b982/cli/args/flags.rs),
[argument module](https://github.com/denoland/deno/blob/14eea3160ae5834476aa3b9d317b8d41d991b982/cli/args/mod.rs),
and [subcommand dispatcher](https://github.com/denoland/deno/blob/14eea3160ae5834476aa3b9d317b8d41d991b982/cli/lib.rs).
The binary entry point simply calls `deno::main()`.

No central, complete third-party notice file was found in that exact Deno tree.
The root `LICENSE.md` is Deno's MIT license, not a license inventory for the
complete executable. Deno's test fixtures containing files called `LICENSE`
are not evidence of runtime component coverage.

## Retained Deno-tree notices

The following files were recovered from exact Git blobs, preserving upstream
bytes. Their per-file SHA-256, Git blob identity, source URL and revision are in
`licenses/upstream/denoland--deno/14eea3160ae5834476aa3b9d317b8d41d991b982/notice-provenance.json`.

| Exact Deno source path | Notice scope |
| --- | --- |
| `LICENSE.md` | Deno MIT terms and its supplied copyright notice |
| `cli/tsc/dts/node/LICENSE` | Packaged Node ambient-type definitions; supplied Microsoft MIT notice |
| `cli/tsc/dts/node/undici/LICENSE` | Packaged Undici definitions; supplied Undici MIT notice |
| `ext/webgpu/LICENSE.md` | WebGPU extension license; identical bytes to this release's Deno MIT license |

These copies are evidence for their stated scope. They do not replace V8,
native dependency, Rust dependency or other embedded-source notices.

## Exact V8 provenance

The verified upstream Deno `Cargo.lock` pins the following dependency chain:

| Component | Exact identity and source binding |
| --- | --- |
| Rust `v8` package | Version `150.2.0`; registry `.crate` SHA-256 `c7f4e905df70d6c00b95e69c5f0831fd5eb5889b0116ae2b30293578c19cd1bc` |
| Rusty V8 source | The verified crate's `.cargo_vcs_info.json` records `d305e6afa7736f6e298c30ae6646f7709ee9382b` in `denoland/rusty_v8` |
| Native V8 source | Rusty V8's `v8` Gitlink is `ac1e23989121713ca642f6650b34deff7b686896` |
| Native V8 repository | The exact Rusty V8 `.gitmodules` declares `https://github.com/denoland/v8.git` |

The native V8 source must be fetched from the recorded **Deno fork** at the
recorded Gitlink. Substituting the same revision against a guessed Google mirror
failed to resolve and does not establish different-source equivalence. The
collector derives the actual repository from the verified `.gitmodules` file.

The pinned native V8 tree includes `LICENSE`, `LICENSE.v8`,
`LICENSE.strongtalk`, `LICENSE.fdlibm`, `AUTHORS`, and component-specific
`third_party/` notices. V8's own root license explicitly distinguishes
externally maintained libraries from the code covered by its main terms.
Retaining only the Deno or Rusty V8 MIT license therefore omits relevant
native component notices.

Rusty V8's exact tree also pins 19 other submodules, including ICU, Abseil,
simdutf, libc++/libc++abi/libunwind, LLVM libc, Highway, FP16, fast_float,
Dragonbox, partition_alloc, and build/tool dependencies. Their declared
repository URLs and exact Gitlink revisions are the collection authority;
neither `DEPS` nor any downloaded upstream program is executed.

## Reproduce evidence collection

From a trusted repository checkout with Python 3.11 or later and network access:

```sh
python scripts/collect_deno_notices.py \
  --cache .cache/deno-notices \
  --output outputs/deno-runtime-evidence \
  --all-rust \
  --native-dependencies
```

The script can bootstrap its input cache from the exact Deno source tree.
It verifies source files against Git blob identities and every registry crate
against the checksum in that verified `Cargo.lock`. It collects the lockfile's
1,042 registry package identities conservatively, including packages that may
not be linked into a particular release target. It retains the original crate
archives as source/header evidence, plus discovered standalone notice files.
Collection uses only Python's standard library and does not invoke Cargo,
compile code, or execute fetched source.

`--native-dependencies` adds bounded repository-root and REUSE `LICENSES/`
notice collection for the 20 explicitly pinned Rusty V8 submodules. GitHub and
Chromium Gitiles responses are bound to their recorded revisions and notice
blobs. `--native-sources` additionally requests original source archives for
those submodules and implies native notice collection. Source archive failures
are recorded without discarding recovered notices.

The output `deno-notice-index.json` records files, archive hashes, provenance,
per-component failures and remaining gaps. Downloads are cached, modified
evidence is not silently overwritten, and a failed dependency collection exits
unsuccessfully. The bundle is a source/notice evidence artifact rather than a
claim that every listed package occurs in the final executable.

## Required review before public image distribution

- Confirm the actual image's Deno binary/version/source binding and retain its
  artifact identity. Source inspection alone is not a reproducible-build proof.
- Review the generated index for failed downloads and crate archives without
  standalone notices. Such archives preserve headers, but a missing notice is
  not automatically equivalent to a complete attribution review.
- Check the native dependency records: repository-root notices and an exact
  source archive are not a complete recursive license inventory for every
  nested component. Retained `DEPS` and Gitlinks expose additional review inputs;
  the collector does not execute or fully resolve Chromium's `DEPS` language.
- Keep release-specific notice/source artifacts accessible with the image and
  follow any applicable source-access terms. Do not silently replace unresolved
  upstream attribution with invented names, dates or license text.
- Review coverage for the actual compiled target and build flags. The
  conservative repository/lockfile superset deliberately includes test/build
  material; it must not be represented as an exact list of linked runtime code.

Five offline regression tests verify crate checksum enforcement, Git object
identity, source-cache tamper rejection, safe notice paths, byte preservation
and use of the explicitly declared V8 fork. A completed download run is an
evidence milestone; outstanding index gaps still require review.

## Local collection results

The initial complete locked-crate run retained all **1,042 of 1,042** registry
archives. Two native source-archive requests failed after their exact Gitiles
root trees had already downloaded:

- `third_party/libunwind/src`, revision
  `d6c7a21e978f0adaa43accaad53bc64f0b64f6ec`: its authoritative `LICENSE.TXT`
  was recovered and verified against the Git tree before the archive failure.
- `third_party/rust`, revision
  `26e8ff47f18a8d28d6187a04b6a16cb7332356f8`: the repository root contains no
  standalone license notice. An archive failure does not establish that its
  nested components have no notice requirements.

The collector now retains successful notice records when an optional source
archive fails, and records the archive URL and HTTP status separately. A rerun
uses the downloaded evidence cache. The final artifact's index, rather than
these initial observations, is authoritative for its download status.

Neither component has been dismissed as build-only. The checksum-verified
`v8-150.2.0.crate` includes `build/config/c++/c++.gni`, where
`use_custom_libcxx` defaults to true, and `build/config/rust.gni`, where
`enable_rust` defaults to `build_with_chromium`. These conditional settings
require an actual target/build-flag review to establish linked components.


## Completed cached collection and runtime bundle

The final cached rerun retained **1,042 of 1,042** exact registry archives,
**20 native dependency records with their exact source archives**, and no
failed download records. The two earlier native archive failures were transient
and are superseded by this result. Twenty-five native V8 notice/provenance files
remain after removing two falsely matched C++ source filenames. Several native
subtrees have no root notice; their source headers remain in retained archives.
Across the conservative Cargo.lock superset, 118 packages lack standalone notice
files. That count is not an assertion that all 118 are linked or unlicensed.

`scripts/package_deno_notices.py` verifies every indexed notice's SHA-256 and
creates a deterministic archive of **1,704 notice/provenance files**, excluding
the larger source archives. The tracked bundle is 688,624 bytes, with SHA-256
`485666a48b188f670f04b630d4f8c84e3a3cb65190bd3258c8e06243b5b4c35f`.
The Dockerfile verifies the archive and unpacks it under
`/usr/share/doc/momokun/licenses/deno-2.9.4`. Regression tests verify its paths,
file hashes and component version. The complete separate local source/evidence
collection is about 403 MB. The opt-in CI source run regenerates that separate
private artifact for retention with an approved release.

This completes the implemented acquisition/packaging workflow. It does not
resolve every Chromium DEPS inclusion question or make a reproducible-build
claim for the upstream Deno binary. The generated index retains those limits.

The embedded TypeScript compiler has a Microsoft Apache-2.0 copyright/license
comment at the start of `cli/tsc/00_typescript.js`. The full source Git blob was
verified against the pinned Deno tree; its leading comment is retained verbatim
as `cli/tsc/TypeScript-header.txt` in the bundle. The index records both full-source
and excerpt hashes and the exact byte-range rule. The image also includes the
full Apache-2.0 license text. This header is intentionally included even though
it is not a separate upstream LICENSE file.
