# Maintain the documentation

The website uses free Material for MkDocs features and builds independently of
Discord and the production host. Python 3.12 and the pinned requirements are
enough to preview checked-in pages. Rust is needed only to refresh or verify the
command registry export. No production credentials are needed.

## Preview locally

=== "Linux / macOS"

    ```sh
    python3 -m venv .venv-docs
    . .venv-docs/bin/activate
    python -m pip install -r requirements-docs.txt
    python -m mkdocs serve --dev-addr 127.0.0.1:8000
    ```

=== "PowerShell"

    ```powershell
    py -3.12 -m venv .venv-docs
    ./.venv-docs/Scripts/python.exe -m pip install -r requirements-docs.txt
    ./.venv-docs/Scripts/python.exe -m mkdocs serve --dev-addr 127.0.0.1:8000
    ```

Open the local URL printed by MkDocs. The preview refreshes when files change.
Keep the preview bound to localhost. Generated output in `site/` is ignored by
Git; do not commit it or run `mkdocs gh-deploy` as a substitute for the reviewed
Pages workflow.

## Refresh references

Export commands from the actual composed Poise registry, including extensions
and context-menu actions, without logging in to Discord:

```sh
cargo run --locked -p bot-app -- --export-command-reference docs/reference/commands.json
python scripts/generate_reference.py --registry docs/reference/commands.json
```

On the bundled Windows development environment use `./scripts/dev.ps1 --% run`
in place of `cargo run`. Review the resulting changes along with the code.
Generated metadata does not replace explanation: update practical examples,
conditional restrictions, body-level permissions, default argument behavior and
feature limitations when they change. Never put credentials into fixtures.

## Validate before opening a pull request

```sh
python scripts/generate_reference.py --registry docs/reference/commands.json --check
python -m unittest discover -s scripts -p 'test_*.py'
python -m mkdocs build --strict
python scripts/check_docs.py
```

CI exports a fresh registry before checking references, so code changes cannot
silently leave the saved command snapshot stale. Strict MkDocs catches missing
navigation targets and Markdown links. The HTML checker validates local links,
fragment targets and assets under the repository URL path. Use
`python scripts/check_docs.py --external` for a separate online check; external
sites can reject automated requests, so review failures rather than silently
ignoring them. External availability is not a prerequisite for an offline build.

Search is built by MkDocs and runs in the browser. Check a real search query,
sidebar, tab navigation and readable code blocks in desktop and mobile widths
after theme changes. Keep focus indicators, contrast and reduced-motion behavior.
System fonts avoid an external font dependency.

## Editing rules

- Update documentation in the same change as user-facing behavior, commands,
  configuration, integrations, dependencies or hosting/storage requirements.
- Prefer generated references from typed configuration and command metadata.
  Keep manually written explanations accurate; examples must use placeholders.
- Link to source or provider documentation where useful. Describe implemented,
  experimental, unverified and planned behavior explicitly.
- Add new pages to `mkdocs.yml`. Avoid links to excluded engineering/audit notes.
- Add only approved, licensed branding. See [branding](../project/branding.md).
- Keep credentials, local screenshots, hostnames, private backup paths and audit
  reports out of the Pages artifact. `exclude_docs` excludes known internal notes;
  it does not protect Git history when the repository becomes public.

Direct and transitive documentation dependencies are pinned in
`requirements-docs.txt`. Update them deliberately in a clean Python 3.12 virtual
environment, rebuild, check links, and verify the preview before integration.

The installed Material release announces security/critical-fix support through
May 5, 2027. Review the framework's support status before then; this initiative
retains the requested MkDocs implementation and does not silently migrate it.
