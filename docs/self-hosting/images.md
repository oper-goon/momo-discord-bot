# Container images and publishing status

No public Momokun image was published by this initiative. Source builds are the currently actionable installation path. `examples/compose.prebuilt.yaml.example` is an integration-ready template, not a claim that an image exists: it requires the operator to supply `MOMOKUN_IMAGE` after a verified release is published.

For an approved future release, copy that example to `compose.yaml` in an empty directory, copy `.env.example` to `.env`, create `config/config.toml` from the repository example, configure credentials and registration, and set `MOMOKUN_IMAGE` to the actual published version or digest. The same storage, backup, provider and permission rules apply. A hypothetical registry path is deliberately not provided as a working install command.

Publishing through GitHub Container Registry is feasible using the existing multi-stage Dockerfile, but requires an explicit operator approval and a workflow with narrowly scoped package-write permission. No publishing workflow is enabled here. Existing CI builds the image without pushing it.

A release workflow should publish a full semantic version tag and optionally a stable tag, attach OCI source/revision/version/license labels, and record the image digest. Prefer digest-pinned deployments for reproducibility; tags may move. Publish Linux x86_64 only after a clean build and isolated startup check. ARM64 needs its own build, runtime and vendored-assembly validation before being advertised. The current Dockerfile uses pinned version tags and Cargo.lock, but base images and apt packages are not digest/snapshot locked, so builds are not bit-for-bit reproducible.

After an actual image exists, updates use `docker compose pull` followed by `docker compose up -d`; back up first and choose a reviewed version. Release licensing and third-party notices must be resolved before distribution.

The image now accepts VERSION and REVISION build arguments for OCI metadata (defaults: development/unknown). Release builders must supply the reviewed tag and commit. The first-party license label is PolyForm-Noncommercial-1.0.0; application license and third-party notices are included under /usr/share/doc/momokun/. Full transitive binary-license collection remains a release gate.

Other image build arguments are `YT_DLP_VERSION=2026.10.26` and
`BGUTIL_PROVIDER_VERSION=2.0.2`; change these only with compatibility/security
testing. The Deno image stage is pinned to `2.9.4`. Runtime image settings
`DENO_NO_UPDATE_CHECK=1` and `DENO_NO_PROMPT=1` disable Deno update checks and prompts;
they are media-tool settings, not bot configuration fields. `PATH` includes
`/opt/ytdlp/bin`, and Compose fixes `DATABASE_URL` to the persistent volume path.
The standalone example's `MOMOKUN_IMAGE` is a required Compose image reference,
not a secret or an application setting.

## Included distribution evidence

Source builds collect Rust notices and exact registry source archives under
`/usr/share/doc/momokun/bundle/`. The index records the build target, revision,
lockfile digest, declared terms, original notices and unresolved items. Modified
vendored dependencies include their actual source. MPL-covered source remains
under MPL 2.0; extract an included crate with `tar -xzf PACKAGE.crate`.

`/usr/share/doc/momokun/image-notices/inventory.json` records installed
Debian/Python packages and tool build information, with packaged notices beside
it. These are review evidence, not a declaration that every redistribution
obligation is complete. Missing upstream notices, system corresponding source,
Deno and optional-provider materials remain release checks. Personal source
builds do not require a public registry image.
