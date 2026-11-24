# Release readiness

Momokun is currently version **0.9.0, pre-1.0**. The source implements the features
in the module guides; some live integrations and deployment procedures still
require acceptance testing. Preparation is not a claim of production readiness.

## Security gate

A refreshed cargo-deny run on 2026-11-24 failed the advisory gate on the supplied
lockfile. Dependency bans, licenses and sources passed. The findings are:

| Package | Advisory |
| --- | --- |
| chrono | RUSTSEC-2026-0543 (unmaintained) |
| crypto-bigint | RUSTSEC-2026-0387 |
| libsqlite3-sys | RUSTSEC-2026-0459 |
| pin-utils | RUSTSEC-2026-0441 |
| rustls-webpki | RUSTSEC-2026-0382 |
| tokio | RUSTSEC-2026-0386 |
| url | RUSTSEC-2026-0592 |
| webpki-roots | RUSTSEC-2026-0381 |

These are release blockers pending dependency remediation or a specific,
documented reachability review. This initiative adds no advisory waivers.
Existing [vendored backports and exceptions](../security.md) remain separate
obligations; a green dependency scan alone would not validate their source patches.

## Acceptance still needed

- An isolated build/start of the revised Docker deployment and backup/restore
  commands. Docker was unavailable in the documentation implementation environment.
- Live Discord command/component rendering, permissions and hierarchy, cache
  fallbacks, command synchronization, voice playback/seek and load-time shutdown.
- Account-backed Easynews and TorBox behavior across supported plans. Local
  metadata/configured status is not proof of successful authentication.
- ARM64 build, runtime and vendored assembly tests before advertising that target.
- Full transitive binary/image notice collection and software/artwork ownership review.
- Original approved artwork files and attribution in the branding asset directory.

## Publication status

GitHub Pages is configured in source with an approval gate; no site was published
by this initiative. No public GHCR image or official shared-bot invitation is
provided. Source-build installation instructions describe the prepared release
checkout. Existing MIT licensing remains in effect while the proposed PolyForm
transition is reviewed. No CLA has been imposed.

The release checklist is tracked in the repository. No separate accepted feature
roadmap exists; do not infer future support or delivery dates from this list.
