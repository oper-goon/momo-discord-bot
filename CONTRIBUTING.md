# Contributing to Momokun

Start with the README and documentation website. For an issue, describe your
version, installation method, expected behavior and steps to reproduce. Remove
credentials, private identifiers and signed URLs from all examples and logs.
Report vulnerabilities privately using SECURITY.md.

Open a focused branch and keep unrelated changes out of the pull request. Read
AGENTS.md, the relevant crate, and docs/security.md before changing dependencies,
networking, media or secrets. Preserve the module boundaries and bounded async
work. Never edit an applied migration; add a new numbered migration instead.

Update command/configuration references and affected guides in the same change.
Run the documented Rust gates and documentation checks. Explain validation you
could not perform, especially live Discord behavior and voice playback.

Use the pull request template. Describe user-visible behavior, compatibility,
database changes, configuration changes and relevant tests. Maintainers review
before merging; opening a PR does not authorize production deployment.

The existing project is MIT-licensed. A PolyForm Noncommercial proposal is under
review and is not effective. Submit only material you have the right to
contribute, retain third-party notices, and identify copied or generated assets
and their provenance. No Contributor License Agreement is currently imposed.
Any future CLA or relicensing requires a separate maintainer decision and clear
contributor rights.
