#!/usr/bin/env python3
"""Preserve the installed Rust distribution's standard-library/toolchain notices."""
from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import shutil
import subprocess


def rustc_info(rustc):
    result = subprocess.run([rustc, "-vV"], check=True, capture_output=True, text=True)
    fields = dict(line.split(": ", 1) for line in result.stdout.splitlines() if ": " in line)
    if not fields.get("release") or not fields.get("host"):
        raise ValueError("rustc did not report its release and host")
    return result.stdout, fields


def regular_inside(root, path):
    if not path.is_file() or path.is_symlink() or not path.resolve().is_relative_to(root.resolve()):
        raise ValueError("required toolchain notice is missing, symlinked or outside the sysroot")
    for relative_parent in path.relative_to(root).parents:
        if (root / relative_parent).is_symlink():
            raise ValueError("symlinked toolchain notice directory is unsupported")
    return path


def collect(sysroot, output, version_text, info, target, expected_release):
    sysroot, output = pathlib.Path(sysroot), pathlib.Path(output)
    if info["release"] != expected_release:
        raise ValueError("Rust release does not match the expected pinned release")
    if not target or any(char not in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_.-" for char in target):
        raise ValueError("invalid target triple")
    if output.resolve().is_relative_to(sysroot.resolve()):
        raise ValueError("output must be outside the toolchain sysroot")
    if output.is_symlink():
        raise ValueError("output directory must not be a symlink")
    if output.exists() and any(output.iterdir()):
        raise ValueError("output directory must be absent or empty")
    document_root = sysroot / "share/doc/rust"
    selected = [regular_inside(sysroot, document_root / name)
                for name in ("COPYRIGHT-library.html", "COPYRIGHT.html")]
    licenses = document_root / "licenses"
    if not licenses.is_dir() or licenses.is_symlink():
        raise ValueError("Rust distribution license directory is missing or symlinked")
    license_files = sorted(path for path in licenses.rglob("*") if path.is_file() or path.is_symlink())
    if not license_files:
        raise ValueError("Rust distribution license directory is empty")
    selected.extend(regular_inside(sysroot, path) for path in license_files)
    for required in ("Apache-2.0.txt", "MIT.txt"):
        regular_inside(sysroot, licenses / required)
    rustlib = sysroot / "lib/rustlib"
    selected.extend(regular_inside(sysroot, rustlib / name) for name in
                    (f"manifest-rust-std-{target}", f"manifest-rustc-{info['host']}", "components"))
    for name in ("multirust-channel-manifest.toml", "rust-installer-version"):
        path = rustlib / name
        if path.exists():
            selected.append(regular_inside(sysroot, path))
    # Some platform distributions include additional root runtime license files.
    for name in ("COPYING.RUNTIME", "COPYING3", "LICENSE-APACHE", "LICENSE-MIT", "COPYRIGHT"):
        path = sysroot / "share/doc" / name
        if path.exists():
            selected.append(regular_inside(sysroot, path))
    libraries = sysroot / "lib/rustlib" / target / "lib"
    if not libraries.is_dir() or libraries.is_symlink():
        raise ValueError("target standard library is not installed")
    inventory = sorted(path.name for path in libraries.iterdir() if path.is_file())
    if not inventory:
        raise ValueError("target standard library directory is empty")
    output.mkdir(parents=True, exist_ok=True)
    entries = []
    for source in sorted(set(selected)):
        relative = source.relative_to(sysroot)
        destination = output / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
        entries.append({"path": relative.as_posix(), "sha256": hashlib.sha256(destination.read_bytes()).hexdigest(),
                        "bytes": destination.stat().st_size})
    (output / "rustc-version.txt").write_text(version_text, encoding="utf-8")
    manifest = {"schema_version": 1, "rust_release": info["release"], "host": info["host"],
                "target": target, "commit_hash": info.get("commit-hash"), "files": entries,
                "target_library_files": inventory,
                "scope": "Conservative installed-toolchain notice superset, not an exact linked-component inventory.",
                "source": f"https://github.com/rust-lang/rust/tree/{info.get('commit-hash', expected_release)}",
                "limitations": ["Does not collect Rust sources or assert source-offer compliance.",
                                "Compiler/tools and platform notices may describe code absent from the shipped bot.",
                                "Cargo dependencies and system/image components are collected separately."]}
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rustc", default="rustc")
    parser.add_argument("--sysroot", type=pathlib.Path)
    parser.add_argument("--target")
    parser.add_argument("--expected-release", default="1.98.1")
    parser.add_argument("--output", required=True, type=pathlib.Path)
    args = parser.parse_args()
    try:
        text, info = rustc_info(args.rustc)
        sysroot = args.sysroot or pathlib.Path(subprocess.run([args.rustc, "--print", "sysroot"], check=True,
                                                            capture_output=True, text=True).stdout.strip())
        result = collect(sysroot, args.output, text, info, args.target or info["host"], args.expected_release)
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        parser.exit(1, f"Toolchain notice collection failed: {error}\n")
    print(f"Collected {len(result['files'])} Rust toolchain notice/metadata files for {result['rust_release']}.")


if __name__ == "__main__":
    main()
