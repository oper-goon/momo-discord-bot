# Third-party notices

Momokun's first-party release candidate uses PolyForm Noncommercial 1.0.0 in
`LICENSE`, following the owner's confirmation of code and artwork rights.
Earlier MIT grants remain valid; `licenses/MIT-legacy.txt` preserves the prior
notice. Third-party components retain their own licenses.

## Bundled source

| Component | Version | Declared license | Local evidence |
| --- | --- | --- | --- |
| Davey | 0.1.5 with backend API adapter | MIT | vendor/davey/LICENSE, .cargo_vcs_info.json and vendor/README.md |
| OpenMLS basic credential | 0.5.0 with P-256 API adapter | MIT | vendor/openmls_basic_credential/LICENSE and vendor/README.md |
| OpenMLS RustCrypto provider | 0.5.1 with P-256/HPKE API adapter | MIT | vendor/openmls_rust_crypto/LICENSE and vendor/README.md |
| libcrux-chacha20poly1305 | 0.0.7 with security backport | Apache-2.0 | vendor/libcrux-chacha20poly1305/Cargo.toml and vendor/README.md |
| libcrux-secrets | 0.0.5 with security backport | Apache-2.0 | vendor/libcrux-secrets/Cargo.toml and vendor/README.md |
| ringbuf | 0.4.8 with security backport | MIT OR Apache-2.0 | vendor/ringbuf/LICENSE-MIT and LICENSE-APACHE |

The two historical libcrux copies above are retained for provenance but no longer
patch the active graph after the HPKE upgrade. The three voice adapters preserve
their original MIT terms; they are not relicensed under the project's PolyForm
license. Their upstream notices are copied into the runtime image notice bundle.
`tests/voice-interop/legacy` is a separate, unpublished test reference with known
advisories. It is excluded from the container build context and is not a runtime
dependency or a distributable bot executable.

Preserve upstream source copyright headers, provenance files, license texts,
and the backport descriptions. The two libcrux directories do not include their
own standalone license texts; the official, unmodified Apache 2.0 text is now
provided in `licenses/Apache-2.0.txt` and included in the container notice bundle.
Verify exact upstream NOTICE requirements and copyright notices before
redistribution. Do not relabel these crates as PolyForm code.

## Dependencies and container tools

Cargo.lock records resolved Rust dependencies. `deny.toml` permits MIT,
Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, Unicode-3.0, MPL-2.0 and
CDLA-Permissive-2.0. A successful allowlist check is not a complete attribution
bundle. Before publishing binaries/images, collect exact resolved dependency
licenses, copyright notices and any source-disclosure obligations, including
MPL-covered files and Mozilla trust-root data.

The image build now creates `/usr/share/doc/momokun/bundle/rust-index.json`,
original notices, checksum-verified registry source archives and the actual
modified source of active vendored packages. MPL-covered source retains its
MPL terms; `licenses/MPL-2.0.txt` is Mozilla's official text. Archive extraction
instructions accompany the bundle. `licenses/rust-supplements.json` pins
recovered upstream notices to the exact packaged release commit and file hashes.
These materials do not apply Momokun's noncommercial restriction to third-party
source. License alternatives are preserved rather than automatically selected.

The runtime build also creates `/usr/share/doc/momokun/image-notices/inventory.json`
with installed Debian/Python versions, original notices and media-tool build
information. Private CI retains the generated evidence for seven days, without
publishing an image. Three Rust notice gaps remain under review: convert_case
0.4.0, realfft 3.5.0 and serenity-voice-model 0.3.0. Exact sources are retained,
but a manifest license declaration is not substituted for missing attribution.
System-package corresponding source, Deno/manual-tool notices, optional-provider
image obligations and applicable combined-license requirements need final review.

Exact-release supplements also retain Deno 2.9.4's MIT notice at upstream commit
`14eea3160ae5834476aa3b9d317b8d41d991b982` and BgUtils 2.0.2's GPL 3.0 text at
`26475e9d3665b972c4aeb83b4dbaafcf64b88438`. Their original files are under
`licenses/upstream/` and copied into the image notice directory. BgUtils source
distribution remains outstanding. The exact upstream source archive is now
retained beside its license with SHA-256 provenance in
`licenses/runtime-supplements.json`. All three plugin Python files in the
PyPI 2.0.2 wheel match the retained upstream source byte for byte. The Docker
build now verifies this comparison and retains the archive and its result.
These build changes still require Linux CI validation. Do not assume a GPL license-text copy alone completes its obligations.
Deno's root MIT notice likewise does not establish that all its embedded
third-party notices have been collected. Debian's installed FFmpeg build has
GPL enabled; preserve its package notices and address corresponding source
before redistribution. These separate tools retain their own terms.

The container also includes Debian packages (including FFmpeg, OpenSSL, Opus,
SQLite and Python), yt-dlp and its installed extras, the BgUtils provider
plugin, and Deno. Their licenses and required notices must accompany the
distribution as applicable. FFmpeg obligations depend on the actual packaged
build and linked codecs. Inspect the built image's package metadata and license
directories; this document does not certify a complete binary redistribution
audit. The optional provider container has separate upstream obligations.

Discord, YouTube, SoundCloud, Bandcamp, Easynews, TorBox, AniList, TMDB and
social embed providers are external services. Use remains subject to their
terms and content rights. A software license does not grant media rights or
authorize bypassing service restrictions.

## Artwork

Approved Momokun reference artwork was supplied by the owner, who confirmed
permission to publish it and its derivatives on 2026-11-24. Three AI-generated
derivatives are integrated and owner-approved. Creator attribution, if required
by the original agreement, should be supplied before public publication. No
third-party mascot has been substituted.
