# Project status

Momokun is source-available under PolyForm Noncommercial 1.0.0. The latest bot
source and documentation are public. The workspace version is **0.9.0** while
preparing the versioned 1.0 release; no 1.0 release tag or public prebuilt image
is available yet.

## Deployment and validation

Docker Compose source builds are the supported installation path. Linux
x86_64 CI checks Rust formatting, Clippy, tests, dependency policy, offline
startup, image builds, persistent storage, and SQLite backup/restoration.
Documentation references, internal links and strict builds are checked in CI.
The documentation is [live on GitHub Pages](https://oper-goon.github.io/momo-discord-bot/).

The latest source was deployed to the maintained bot on 2026-11-25. Startup,
configuration retention and database integrity checks passed. A focused voice
trial also verified audible YouTube playback, pause/resume, skip and playback
after restart. Automated checks do not replace live Discord acceptance testing.

## Current limitations

- ARM64 builds and runtime behavior are not validated; do not assume support.
- Optional paid-service behavior depends on account access and upstream service
  availability. Every subscription plan has not been tested.
- Public prebuilt container images require a separately reviewed release,
  matching notices and applicable source delivery.
- Dependency maintenance exceptions and their review deadlines are documented
  in the [security policy](../security.md).
- The owner reviewed and accepted three exact-release attribution exceptions
  for convert_case 0.4.0, realfft 3.5.0 and serenity-voice-model 0.3.0. Original
  declarations, sources and available notices remain preserved. This decision
  does not claim that missing upstream notices were recovered.

See the [changelog](changelog.md) for implemented changes. No separate committed
feature roadmap or delivery schedule is promised.
