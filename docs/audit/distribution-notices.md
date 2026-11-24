# Distribution notice collection assessment

Reviewed 2026-11-24. The collector design below has now been implemented.
This is not completed license clearance or authorization to publish an image.

## Implementation follow-up

The target-filtered local resolved graph contains 488 reachable packages. The
collector retains 470 checksum-verified registry archives, all 20 MPL source
archives, original nested notices and four actual modified vendor source trees.
Packaged SPDX license directories fixed three omissions. Exact upstream commit
notices were recovered for 33 of the remaining 36 non-MPL packages and pinned
in `licenses/rust-supplements.json`; their hashes and packaged Git revisions
are checked during collection. Three standalone notice gaps remain:
convert_case 0.4.0, realfft 3.5.0 and serenity-voice-model 0.3.0. No upstream
copyright was invented. The first two exact repository trees lack a standalone
notice; the third manifest repository cannot resolve its recorded commit.

The build now bundles these Rust materials and inventories installed Debian and
Python distributions with media-tool build evidence. CI extracts the evidence
privately for seven days. Actual Linux execution of this new collection step
is pending; earlier Linux image/start/recovery checks passed before the collectors.
Final system corresponding-source, Deno/manual executable and optional provider
obligations remain under review. The earlier assessment below records the
baseline evidence and rationale; its 586-package counts are historical.


## Evidence and limits

The local `voice-metadata.json` snapshot contains **586 packages**, including
14 first-party workspace packages. Its package list is unfiltered for platform
and includes optional manifest declarations. It is a conservative inventory,
**not the exact Linux binary inventory**. Following manifest dependency
declarations from `bot-app`, including optional/build declarations, reaches all
586 packages. That does not show they are all activated or linked. Build tools,
procedural macros, platform-specific dependencies and optional integrations can
therefore be overrepresented.

For each metadata package, inspection was restricted to its declared manifest
directory. A top-level filename heuristic looked for LICENSE, LICENCE, COPYING,
NOTICE, COPYRIGHT and AUTHORS, including common suffixes. **503 packages** have
at least one matching file; **83 packages** have none. The latter count includes
the 14 first-party packages whose license is at the repository root.

This heuristic does not determine whether a file supplies every required notice,
whether the declared license expression is correct, or which alternative license
was selected. It misses nested files, notices embedded in source headers or
README files, and unusual filenames. A positive match is not a compliance pass;
a missing match is not proof that no notice is needed. No copyright owner or
copyright year should be invented to fill a missing file.

All **20 MPL-2.0 packages** in the snapshot lack a matching top-level notice text:
four HPKE packages and sixteen Symphonia packages. Exact registry `.crate`
archives are available locally for all twenty. Sample Symphonia source headers
contain copyright and MPL notices. The availability of an archive alone does
not establish that a particular executable's source-distribution obligations
have been fulfilled.

`ring` illustrates why one generic license per package is insufficient: it ships
LICENSE, LICENSE-BoringSSL and LICENSE-other-bits. Preserve applicable component
notices and inspect nested bundled code rather than retaining only LICENSE.

## Minimal collector design

1. Generate fresh locked Cargo metadata **inside the actual Linux image builder**,
   using its target, features and package selection. Record the source revision,
   lockfile digest and target. Avoid adding `--all-features` unless the image is
   actually built that way. Traverse non-development dependencies from `bot-app`;
   follow effective `resolve.nodes` edges rather than every manifest dependency,
   and identify build-time/procedural-macro components as conservative inclusions.
2. Use a small Python standard-library script to inspect only those package
   directories. Copy declared `license_file` entries and all relevant license,
   notice, copyright and author files, including nested component notices.
   Preserve file contents and package-relative paths. Reject paths escaping the
   package directory and do not follow symlinks outside it.
3. Write a deterministic index with package name/version, declared license
   expression, upstream repository, registry source/checksum, notice filenames
   and content digests. Do not emit absolute machine paths. Include local vendor
   provenance and modifications separately from registry packages.
4. Produce an unresolved-items report and fail the publishing gate when required
   notices remain unreviewed. Missing upstream text needs a reviewed override
   pinned to the exact upstream version or commit. Generic MIT text without the
   relevant copyright notice is not an adequate automatic replacement. Preserve
   every obligation in combined `AND` expressions; document any selected `OR`
   alternative rather than guessing from filenames.
5. Include the generated bundle in the image and its release artifacts, with a
   clear index telling recipients where to find notices and covered source.
   Rerun collection whenever the lockfile, feature selection, target, local
   patches or image package set changes.

Metadata traversal is an auditable approximation, not proof of which bytes are
linked. Cargo build artifacts can supplement the record. Extra notices are
preferable to incorrectly claiming a smaller exact inventory, but must remain
accurately labeled.

## MPL source availability

[MPL 2.0 sections 3.1–3.3](https://www.mozilla.org/en-US/MPL/2.0/) require covered
source availability and recipient information when distributing executables,
while allowing a larger work under separate terms. A notice-text bundle alone
does not meet the source-availability requirement. See also
[Mozilla's distribution FAQ](https://www.mozilla.org/en-US/MPL/2.0/FAQ/).

For unchanged reachable MPL registry packages, retaining exact `.crate` archives
with verified Cargo.lock checksums is a practical source-delivery mechanism.
Provide the MPL text, preserve existing headers, and explain archive extraction
and the source's MPL terms. Confirm that these archives contain the actual
preferred source used by the build. If an MPL package is modified locally,
provide its modified source rather than pointing only to the original registry
archive. Retain the source materials with the release so availability does not
depend solely on a mutable external link.

Do not describe this as a complete application source bundle. Do not apply
PolyForm restrictions to MPL-covered source or imply that a checksum comparison
settles every legal requirement. The proposed materials still need review
against the actual distributed executable.

## Separate image obligations

Rust collection does not cover Debian packages, FFmpeg's actual build and codecs,
OpenSSL, Opus, Python and installed wheels/extras, yt-dlp, BgUtils, Deno or the
optional provider image. Collect their exact versions, notices, source offers or
source materials as applicable from the completed image. Preserve upstream
package license directories. Review any additional terms introduced by the
actual build options.

Public binary/image redistribution remains pending completion and review of
these materials. A green dependency license allowlist is useful evidence, not
a substitute for this work.
