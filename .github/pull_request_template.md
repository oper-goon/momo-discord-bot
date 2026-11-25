## Change

Describe the problem and resulting behavior.

## Validation

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --locked --workspace`
- [ ] `cargo run --locked -p bot-app -- --check`
- [ ] `cargo deny check`
- [ ] Documentation generation/checks and strict MkDocs build pass
- [ ] Commands, configuration, features and affected deployment guides updated
- [ ] Update CHANGELOG.md for user-visible changes when preparing a release
- [ ] Describe database migrations and any manual Discord acceptance tests
- [ ] No credentials, private configuration, databases, signed URLs or unlicensed assets included

## Compatibility and risks

Describe configuration, permissions, dependencies, persistence or deployment
changes. List checks that could not be run and relevant live Discord behavior.
Keep publication and production deployment separate from this review.
