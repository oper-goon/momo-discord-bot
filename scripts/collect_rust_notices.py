#!/usr/bin/env python3
"""Collect notices from a locked, target-filtered Cargo metadata graph.

This produces evidence, not automatic license clearance. It never downloads files.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import tomllib


NOTICE = re.compile(r"^(licen[cs]e|copying|notice|copyright|authors)(?:[._-].*)?$", re.I)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def safe_file(root: Path, path: Path) -> bool:
    return path.is_file() and not path.is_symlink() and path.resolve().is_relative_to(root.resolve())


def notice_files(root: Path, declared: str | None = None) -> list[Path]:
    files = set()
    for directory, subdirs, names in os.walk(root, followlinks=False):
        subdirs[:] = sorted(d for d in subdirs if d not in {".git", "target", "__pycache__"}
                            and not (Path(directory) / d).is_symlink())
        for name in names:
            path = Path(directory) / name
            in_license_directory = any(p.lower() in {"licenses", "licences"}
                                       for p in path.relative_to(root).parts[:-1])
            if (NOTICE.match(name) or in_license_directory) and safe_file(root, path):
                files.add(path)
    if declared:
        path = root / declared
        if not safe_file(root, path):
            raise ValueError("declared license file is missing or escapes its package")
        files.add(path)
    return sorted(files, key=lambda p: p.relative_to(root).as_posix())


def reachable(metadata: dict, root_name: str) -> list[dict]:
    packages = {p["id"]: p for p in metadata["packages"]}
    roots = [p["id"] for p in packages.values() if p["name"] == root_name]
    if len(roots) != 1 or not metadata.get("resolve"):
        raise ValueError("expected one root package and a resolved dependency graph")
    nodes = {n["id"]: n for n in metadata["resolve"]["nodes"]}
    seen, pending = set(), roots[:]
    while pending:
        current = pending.pop()
        if current in seen:
            continue
        seen.add(current)
        for dep in nodes[current]["deps"]:
            if any(k["kind"] != "dev" for k in dep["dep_kinds"]):
                pending.append(dep["pkg"])
    return sorted((packages[i] for i in seen), key=lambda p: (p["name"], p["version"]))


def collect(metadata: dict, lockfile: Path, output: Path, cargo_home: Path,
            target: str, revision: str) -> dict:
    if output.is_symlink() or (output.exists() and (not output.is_dir() or any(output.iterdir()))):
        raise ValueError("output must be empty; refusing to overwrite a notice bundle")
    output.mkdir(parents=True, exist_ok=True)
    workspace = Path(metadata["workspace_root"])
    supplement_path = workspace / "licenses" / "rust-supplements.json"
    supplements = json.loads(supplement_path.read_text(encoding="utf-8")) if supplement_path.exists() else {}
    lock = tomllib.loads(lockfile.read_text(encoding="utf-8"))
    checksums = {(p["name"], p["version"], p.get("source")): p.get("checksum")
                 for p in lock["package"]}
    records, unresolved = [], []
    members = set(metadata["workspace_members"])
    for package in reachable(metadata, "bot-app"):
        name, version = package["name"], package["version"]
        label = f"{name}-{version}"
        root = Path(package["manifest_path"]).parent
        destination = output / "rust" / label
        record = {"name": name, "version": version, "license": package.get("license"),
                  "repository": package.get("repository"), "source": package.get("source"),
                  "checksum": checksums.get((name, version, package.get("source"))),
                  "notices": [], "covered_source": None}
        for path in notice_files(root, package.get("license_file")):
            relative = path.relative_to(root)
            target_path = destination / relative
            target_path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, target_path)
            record["notices"].append({"path": target_path.relative_to(output).as_posix(),
                                      "sha256": digest(target_path)})
        if package["id"] in members:
            target_path = destination / "LICENSE.workspace"
            target_path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(workspace / "LICENSE", target_path)
            record["notices"].append({"path": target_path.relative_to(output).as_posix(),
                                      "sha256": digest(target_path)})
        supplement = supplements.get(label)
        if supplement:
            vcs = json.loads((root / ".cargo_vcs_info.json").read_text(encoding="utf-8"))
            if vcs["git"]["sha1"] != supplement["revision"]:
                raise ValueError("supplement revision does not match packaged source provenance")
            for item in supplement["files"]:
                source = workspace / "licenses" / item["file"]
                if not safe_file(workspace / "licenses", source) or digest(source) != item["sha256"]:
                    raise ValueError("upstream supplement hash/path mismatch")
                target_path = destination / "upstream" / item["file"]
                target_path.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(source, target_path)
                record["notices"].append({"path": target_path.relative_to(output).as_posix(),
                                          "sha256": item["sha256"], "url": item["url"],
                                          "revision": supplement["revision"]})
        if "MPL-2.0" in (record["license"] or ""):
            source = workspace / "licenses" / "MPL-2.0.txt"
            target_path = destination / "LICENSE.MPL-2.0.txt"
            target_path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target_path)
            record["notices"].append({"path": target_path.relative_to(output).as_posix(),
                                      "sha256": digest(source),
                                      "url": "https://www.mozilla.org/media/MPL/2.0/index.txt"})
        if not record["notices"]:
            unresolved.append({"package": label, "reason": "No standalone notice found; inspect source headers and exact upstream release."})
        if package.get("source", "") and package["source"].startswith("registry+"):
            # Preserve exact sources and embedded headers, including MPL-covered files.
            candidates = sorted((cargo_home / "registry" / "cache").glob(f"*/{label}.crate"))
            valid = [p for p in candidates if record["checksum"] and digest(p) == record["checksum"]]
            if not valid:
                unresolved.append({"package": label, "reason": "Exact checksum-verified registry source archive unavailable."})
            else:
                target_path = output / "sources" / valid[0].name
                target_path.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(valid[0], target_path)
                record["covered_source"] = {"path": target_path.relative_to(output).as_posix(),
                                             "sha256": digest(target_path), "terms": record["license"]}
        elif package["id"] not in members:
            # Locally adapted third-party code must include the actual modified source.
            source_root = output / "sources" / label
            copied = []
            for directory, subdirs, names in os.walk(root, followlinks=False):
                subdirs[:] = sorted(d for d in subdirs if d not in {".git", "target", "__pycache__"}
                                    and not (Path(directory) / d).is_symlink())
                for filename in sorted(names):
                    path = Path(directory) / filename
                    if not safe_file(root, path):
                        raise ValueError("local third-party source contains an unsafe file")
                    target_path = source_root / path.relative_to(root)
                    target_path.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(path, target_path)
                    copied.append({"path": target_path.relative_to(source_root).as_posix(), "sha256": digest(target_path)})
            record["covered_source"] = {"path": source_root.relative_to(output).as_posix(),
                                         "files": copied, "terms": record["license"]}
        records.append(record)
    report = {"schema": 1, "revision": revision, "target": target,
              "lockfile_sha256": digest(lockfile),
              "scope": "Conservative resolved non-dev graph, including build tools; metadata must be target-filtered. Not proof of linked bytes or legal clearance.",
              "packages": records, "unresolved": unresolved}
    (output / "rust-index.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    (output / "README.txt").write_text(
        "Momokun distribution evidence\n\n"
        "rust-index.json records package licenses, original notices, source checksums and unresolved review items.\n"
        "sources/*.crate are exact upstream Rust source archives under the terms in rust-index.json.\n"
        "Extract with: tar -xzf PACKAGE.crate\n"
        "sources/PACKAGE/ directories retain the actual locally adapted third-party source.\n"
        "MPL-covered source remains under MPL-2.0; Momokun's PolyForm terms do not restrict it.\n"
        "See licenses/MPL-2.0.txt and the copyright headers retained in each source archive.\n"
        "This is not a complete application/source bundle or a redistribution clearance.\n"
        "Image/system/Python/Deno materials and license alternatives also require review.\n", encoding="utf-8")
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--metadata", type=Path, required=True)
    parser.add_argument("--lockfile", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--cargo-home", type=Path, required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--revision", default="unknown")
    parser.add_argument("--require-reviewed", action="store_true")
    args = parser.parse_args()
    report = collect(json.loads(args.metadata.read_text(encoding="utf-8")), args.lockfile,
                     args.output, args.cargo_home, args.target, args.revision)
    print(f"Rust notice evidence: {len(report['packages'])} packages; {len(report['unresolved'])} unresolved items.")
    return int(args.require_reviewed and bool(report["unresolved"]))


if __name__ == "__main__":
    raise SystemExit(main())
