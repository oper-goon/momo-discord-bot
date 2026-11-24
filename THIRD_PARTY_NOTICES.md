# Third-party notices

Momokun's first-party release candidate uses PolyForm Noncommercial 1.0.0 in
`LICENSE`, following the owner's confirmation of code and artwork rights.
Earlier MIT grants remain valid; `licenses/MIT-legacy.txt` preserves the prior
notice. Third-party components retain their own licenses.

## Bundled source

| Component | Version | Declared license | Local evidence |
| --- | --- | --- | --- |
| libcrux-chacha20poly1305 | 0.0.7 with security backport | Apache-2.0 | vendor/libcrux-chacha20poly1305/Cargo.toml and vendor/README.md |
| libcrux-secrets | 0.0.5 with security backport | Apache-2.0 | vendor/libcrux-secrets/Cargo.toml and vendor/README.md |
| ringbuf | 0.4.8 with security backport | MIT OR Apache-2.0 | vendor/ringbuf/LICENSE-MIT and LICENSE-APACHE |

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
