# Rust standard-library and toolchain notices

Review date: 2026-11-25. Scope: the Rust 1.98.1 toolchain selected by the Docker builder and repository pin. No production host, token or application source was executed for this collection work.

## Why Cargo notices are insufficient

Cargo metadata covers application crates but not the standard library linked from the compiler sysroot. Rust also carries third-party code in its compiler and library distribution. The official [Rust 1.98.1 COPYRIGHT file](https://github.com/rust-lang/rust/blob/1.98.1/COPYRIGHT) explains that binary distributions ship two generated notices: `COPYRIGHT.html` for the toolchain and `COPYRIGHT-library.html` for its standard-library subset. These generated documents include the upstream component/copyright detail; reducing them to an invented single attribution would lose information.

Rust's [official image build source](https://github.com/rust-lang/docker-rust/blob/master/stable/bookworm/Dockerfile) installs the selected release with rustup's minimal profile. That moving source is implementation context, not proof of the pinned image's contents. The collector reads the **actual installed sysroot inside the pinned builder** and fails if its required notice payload is absent; it does not fetch contemporary replacement notices from the network.

## Collector and integration

`scripts/collect_toolchain_notices.py` runs only the installed `rustc -vV` and `rustc --print sysroot`, then copies ordinary files. It never builds or runs an application, downloads a license, follows notice symlinks, or modifies the sysroot. The default expected release is `1.98.1`; an unexpected compiler version fails rather than silently labeling another toolchain as the pin.

Proposed builder invocation (Docker integration is owned separately):

```dockerfile
COPY scripts/collect_toolchain_notices.py ./scripts/collect_toolchain_notices.py
RUN python3 scripts/collect_toolchain_notices.py \
    --expected-release 1.98.1 --output /build-toolchain-notices
```

Copy the resulting evidence into the final image:

```dockerfile
COPY --from=builder /build-toolchain-notices /usr/share/doc/momokun/toolchain-notices
```

The source-relative hierarchy is preserved under that directory so HTML references to the sibling `licenses` directory continue to work. `manifest.json` records the compiler release/commit, host and target, every copied path/size/SHA-256, and target library filenames. `rustc-version.txt` retains compiler version output. The tool records no credentials or private runtime configuration.

Required payload:

- `share/doc/rust/COPYRIGHT-library.html` and `COPYRIGHT.html`, copied verbatim.
- Every ordinary file under `share/doc/rust/licenses`, including nested notices; Apache-2.0 and MIT texts must exist.
- `lib/rustlib/manifest-rust-std-<target>`, `manifest-rustc-<host>`, and `components` as installed-distribution evidence.
- Available channel manifest/installer version metadata and platform root notice files such as `COPYING.RUNTIME` and `COPYING3`.

The target must have an installed, nonempty standard-library directory. An output directory must be absent or empty; the collector does not delete previous evidence. Cross-compilation can use `--target <installed-target>`; the current native Docker build defaults to the reported host. Changing the pin or image should rerun collection and inspect the new manifest, rather than keeping old copied notices.

## Scope and limits

This is a **conservative installed-toolchain notice superset**, not a linker map or assertion that every listed compiler component, library, license or exception appears in `/app/bot`. It deliberately keeps the full library notice as well as the wider toolchain notice. Keeping a GPL-family/exception text from the compiler distribution does not by itself establish that the bot contains the component to which that text applies; determine actual inclusion and terms from the corresponding upstream component entries and binary evidence.

The collector does not copy `.rlib`, compiler binaries, Rust source trees or source archives. It does not create a source offer, settle source-distribution obligations, replace Cargo dependency notices, or replace final-image Debian/Python/FFmpeg/Deno notices. Those remain separate inventories and distribution decisions. It does not relicense Rust under the project's PolyForm terms.

## Validation evidence

The actual installed local `1.98.1` Windows GNULLVM sysroot reported compiler commit `48a229ceaefd4985c50990b14116b6d856af0985`. Collection preserved 21 notice/metadata files and both generated COPYRIGHT documents. Its license directory contained Apache-2.0, BSD-2-Clause, CC-BY-SA-4.0, GCC-exception-3.1, GPL-2.0-only, GPL-3.0-or-later, ISC, LLVM-exception, MIT, NCSA, OFL-1.1 and Unicode-3.0 texts. This validates local payload handling, **not** Linux image contents.

The isolated standard-library tests use synthetic sysroots to verify byte-for-byte preservation and SHA-256, nested bundled notices, target inventory, required-file failures, version/target mismatch rejection and preservation of existing output. The symbolic-link test runs where host privileges allow symlink creation. Run:

```sh
python3 -B -m unittest discover -s scripts -p test_collect_toolchain_notices.py -v
```

Linux image acceptance must build the integrated Dockerfile and inspect the final-image toolchain manifest/COPYRIGHT/license files. No Linux container was started by this task. A successful future image build should record its actual Linux target and evidence counts rather than reuse the Windows values above.
