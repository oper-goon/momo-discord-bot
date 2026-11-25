"""Retain immutable Deno 2.9.4 dependency notice/source evidence; execute no code.

Run with network permission, after supplying the verified Deno tree/source cache:
  python scripts/collect_deno_notices.py --cache .cache/deno-notices --output .cache/deno-collected
Add --all-rust to retain every registry crate from the exact upstream Cargo.lock.
This is a conservative evidence bundle, never an automatic clearance decision.
"""
from __future__ import annotations

import argparse
import concurrent.futures
import base64
import configparser
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import tarfile
import tomllib
import urllib.request
import urllib.parse

DENO_REVISION = "14eea3160ae5834476aa3b9d317b8d41d991b982"
DENO_VERSION = "2.9.4"
NOTICE = re.compile(r"^(?:licen[cs]e|copying|notice|copyright|authors)(?:[._-].*)?$", re.I)
DENO_LICENSES = ["LICENSE.md", "cli/tsc/dts/node/LICENSE", "cli/tsc/dts/node/undici/LICENSE", "ext/webgpu/LICENSE.md"]
DENO_INLINE_NOTICES = ["cli/tsc/00_typescript.js"]


def sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def git_blob(raw: bytes) -> str:
    return hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest()


def write_identical(path: Path, raw: bytes) -> None:
    if path.is_symlink() or (path.exists() and path.read_bytes() != raw):
        raise ValueError(f"Refusing to overwrite changed evidence: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(raw)


def download(url: str, destination: Path, checksum: str | None = None, limit: int = 256 * 1024 * 1024) -> bytes:
    if destination.is_file():
        raw = destination.read_bytes()
    else:
        request = urllib.request.Request(url, headers={"User-Agent": "Momokun-Deno-license-evidence"})
        with urllib.request.urlopen(request, timeout=90) as response:
            raw = response.read(limit + 1)
        if len(raw) > limit:
            raise ValueError("Oversized upstream evidence response")
    if checksum and sha256(raw) != checksum:
        raise ValueError("Upstream archive checksum mismatch")
    write_identical(destination, raw)
    return raw


def safe_relative(name: str) -> PurePosixPath:
    path = PurePosixPath(name)
    if path.is_absolute() or ".." in path.parts or "\\" in name or ":" in name:
        raise ValueError("Unsafe archive/tree path")
    return path


def is_notice(name: str) -> bool:
    path = safe_relative(name)
    if path.suffix.lower() in {".rs", ".cc", ".h", ".c", ".cpp", ".hpp", ".py", ".js", ".ts", ".go"}:
        return False
    return bool(NOTICE.match(path.name) or "LICENSES" in path.parts)


def crate_evidence(package: dict, output: Path) -> dict:
    name, version = package["name"], package["version"]
    if not re.fullmatch(r"[A-Za-z0-9_-]+", name) or not re.fullmatch(r"[A-Za-z0-9_.+-]+", version):
        raise ValueError("Invalid locked crate identity")
    label = f"{name}-{version}"
    url = f"https://static.crates.io/crates/{name}/{label}.crate"
    archive = output / "sources" / f"{label}.crate"
    raw = download(url, archive, package["checksum"])
    notices, vcs, manifest = [], None, None
    with tarfile.open(archive, "r:gz") as tar:
        for member in tar:
            path = safe_relative(member.name)
            if not member.isfile() or not path.parts or path.parts[0] != label:
                continue
            relative = PurePosixPath(*path.parts[1:]).as_posix()
            if not is_notice(relative) and relative not in {".cargo_vcs_info.json", "Cargo.toml"}:
                continue
            if member.size > 16 * 1024 * 1024:
                raise ValueError("Oversized individual crate notice")
            content = tar.extractfile(member).read()
            if relative == ".cargo_vcs_info.json":
                vcs = json.loads(content)
            elif relative == "Cargo.toml":
                manifest = tomllib.loads(content.decode("utf-8"))
            else:
                destination = output / "rust" / label / relative
                write_identical(destination, content)
                notices.append({"file": destination.relative_to(output).as_posix(), "archive_path": member.name, "sha256": sha256(content)})
    return {"name": name, "version": version, "source_archive": archive.relative_to(output).as_posix(),
            "archive_url": url, "archive_sha256": sha256(raw), "notices": notices,
            "vcs": vcs, "license": (manifest or {}).get("package", {}).get("license"),
            "repository": (manifest or {}).get("package", {}).get("repository")}


def repo_tree(repo: str, revision: str, output: Path) -> dict:
    if not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise ValueError("An immutable full Git revision is required")
    url = f"https://api.github.com/repos/{repo}/git/trees/{revision}?recursive=1"
    path = output / "provenance" / repo.replace("/", "--") / revision / "tree.json"
    tree = json.loads(download(url, path, limit=32 * 1024 * 1024))
    if tree.get("truncated"):
        raise ValueError("Incomplete upstream Git tree")
    return tree


def repo_notices(repo: str, revision: str, tree: dict, output: Path) -> dict:
    records = []
    for entry in tree["tree"]:
        if entry["type"] != "blob":
            continue
        relative = safe_relative(entry["path"]).as_posix()
        if not is_notice(relative) and relative not in {"DEPS", ".gitmodules"}:
            continue
        encoded = urllib.parse.quote(relative, safe="/")
        url = f"https://raw.githubusercontent.com/{repo}/{revision}/{encoded}"
        destination = output / "upstream" / repo.replace("/", "--") / revision / relative
        raw = download(url, destination, limit=16 * 1024 * 1024)
        if git_blob(raw) != entry["sha"]:
            raise ValueError("Git blob mismatch for upstream notice")
        records.append({"file": destination.relative_to(output).as_posix(), "repository_path": relative,
                        "url": url, "revision": revision, "git_blob": entry["sha"],
                        "sha256": sha256(raw), "kind": "notice" if is_notice(relative) else "source-provenance"})
    return {"repository": repo, "revision": revision, "files": records,
            "gitlinks": [{"path": e["path"], "revision": e["sha"]} for e in tree["tree"] if e["type"] == "commit"]}


def gitmodules(raw: str) -> dict[str, str]:
    config = configparser.ConfigParser(interpolation=None)
    config.read_string(raw)
    return {config[section]["path"]: config[section]["url"] for section in config.sections()}


def github_repository(url: str) -> str:
    parsed = urllib.parse.urlparse(url)
    repo = parsed.path.strip("/").removesuffix(".git")
    if parsed.scheme != "https" or parsed.hostname != "github.com" or parsed.username or parsed.password or parsed.query or parsed.fragment or not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repo):
        raise ValueError("Not an explicit HTTPS GitHub repository URL")
    return repo


def module_root_notices(url: str, revision: str, label: str, output: Path, archive_source: bool) -> dict:
    """Bounded root/REUSE notices for one explicitly pinned gitlink; no DEPS execution."""
    parsed = urllib.parse.urlparse(url)
    records = []
    if parsed.scheme != "https" or parsed.username or parsed.password:
        raise ValueError("Unsafe declared Git submodule URL")
    destination = output / "native-dependencies" / safe_relative(label) / revision
    if parsed.hostname == "github.com":
        repo = github_repository(url)
        tree = repo_tree(repo, revision, output)
        selected = {**tree, "tree": [entry for entry in tree["tree"] if "/" not in entry["path"] or "LICENSES" in PurePosixPath(entry["path"]).parts]}
        collection = repo_notices(repo, revision, selected, output)
        records = collection["files"]
        archive_url = f"https://codeload.github.com/{repo}/tar.gz/{revision}"
    elif parsed.hostname == "chromium.googlesource.com":
        base = url.rstrip("/").removesuffix(".git")
        directories = [""]
        visited = set()
        while directories:
            directory = directories.pop()
            if directory in visited or len(visited) >= 100:
                raise ValueError("Unexpected or excessive SPDX notice directories")
            visited.add(directory)
            tree_url = f"{base}/+/{revision}/{urllib.parse.quote(directory, safe='/')}?format=JSON"
            tree_file = destination / "provenance" / (directory.replace("/", "--") or "root")
            raw_tree = download(tree_url, tree_file.with_suffix(".json"), limit=8 * 1024 * 1024)
            if not raw_tree.startswith(b")]}'"):
                raise ValueError("Unexpected Gitiles tree response")
            tree = json.loads(raw_tree.split(b"\n", 1)[1])
            for entry in tree["entries"]:
                relative = (PurePosixPath(directory) / entry["name"]).as_posix()
                safe_relative(relative)
                if entry["type"] == "tree" and (entry["name"] == "LICENSES" or directory.startswith("LICENSES/")):
                    directories.append(relative + "/")
                elif entry["type"] == "blob" and is_notice(relative):
                    file_url = f"{base}/+/{revision}/{urllib.parse.quote(relative, safe='/')}?format=TEXT"
                    encoded = download(file_url, destination / "provenance" / (relative.replace("/", "--") + ".base64"), limit=16 * 1024 * 1024)
                    content = base64.b64decode(encoded)
                    if git_blob(content) != entry["id"]:
                        raise ValueError("Gitiles notice Git blob mismatch")
                    notice = destination / "notices" / relative
                    write_identical(notice, content)
                    records.append({"file": notice.relative_to(output).as_posix(), "repository_path": relative,
                                    "url": file_url, "revision": revision, "git_blob": entry["id"], "sha256": sha256(content)})
        archive_url = f"{base}/+archive/{revision}.tar.gz"
    else:
        raise ValueError("Unsupported declared Git submodule host")
    record = {"path": label, "repository_url": url, "revision": revision, "files": records,
              "scope": "Root and REUSE LICENSES notices only; archive retains remaining headers when requested."}
    if archive_source:
        archive = destination / "source.tar.gz"
        try:
            content = download(archive_url, archive)
            record["source_archive"] = {"file": archive.relative_to(output).as_posix(), "url": archive_url, "sha256": sha256(content)}
        except Exception as error:
            record["source_archive_failure"] = {"url": archive_url, "error_type": type(error).__name__,
                                                "http_status": getattr(error, "code", None)}
    return record


def collect_deno_local(cache: Path, output: Path) -> tuple[list[dict], dict]:
    tree = json.loads((cache / "tree.json").read_text(encoding="utf-8-sig"))
    if tree.get("truncated"):
        raise ValueError("Incomplete Deno evidence tree")
    paths = {e["path"]: e for e in tree["tree"] if e["type"] == "blob"}
    records = []
    # Cargo.lock is verified against the same immutable source tree before its
    # crate checksums are accepted as authority for downstream downloads.
    for name in [*DENO_LICENSES, "Cargo.lock", "Cargo.toml"]:
        source = cache / "sources" / name
        raw = source.read_bytes()
        if source.is_symlink() or git_blob(raw) != paths[name]["sha"]:
            raise ValueError("Deno source cache does not match pinned tree: " + name)
        destination = output / "upstream" / "denoland--deno" / DENO_REVISION / name
        write_identical(destination, raw)
        records.append({"file": destination.relative_to(output).as_posix(), "repository_path": name,
                        "revision": DENO_REVISION, "git_blob": paths[name]["sha"], "sha256": sha256(raw),
                        "url": f"https://raw.githubusercontent.com/denoland/deno/{DENO_REVISION}/{name}",
                        "kind": "notice" if name in DENO_LICENSES else "source-provenance"})
    lock = tomllib.loads((cache / "sources/Cargo.lock").read_text(encoding="utf-8"))
    for name in DENO_INLINE_NOTICES:
        raw = (cache / "sources" / name).read_bytes()
        if git_blob(raw) != paths[name]["sha"] or not raw.startswith(b"/*!") or b"*/" not in raw[:8192]:
            raise ValueError("Unexpected embedded TypeScript notice/source")
        notice = raw[:raw.index(b"*/") + 2]
        destination = output / "upstream/denoland--deno" / DENO_REVISION / "cli/tsc/TypeScript-header.txt"
        write_identical(destination, notice)
        records.append({"file": destination.relative_to(output).as_posix(), "repository_path": name,
                        "revision": DENO_REVISION, "source_git_blob": paths[name]["sha"],
                        "source_sha256": sha256(raw), "sha256": sha256(notice),
                        "url": f"https://raw.githubusercontent.com/denoland/deno/{DENO_REVISION}/{name}",
                        "kind": "verbatim leading notice comment; source bytes [0, first */ inclusive]"})
    return records, lock


def bootstrap_deno_cache(cache: Path, output: Path) -> None:
    """CI can prepare the same immutable evidence cache without a local checkout."""
    required = [*DENO_LICENSES, *DENO_INLINE_NOTICES, "Cargo.lock", "Cargo.toml"]
    if (cache / "tree.json").is_file() and all((cache / "sources" / name).is_file() for name in required):
        return
    tree = repo_tree("denoland/deno", DENO_REVISION, output)
    tree_file = cache / "tree.json"
    tree_file.parent.mkdir(parents=True, exist_ok=True)
    # The API response contains the immutable per-path Git object identities;
    # preserving semantic JSON here avoids changing any downloaded source bytes.
    if not tree_file.exists():
        tree_file.write_text(json.dumps(tree), encoding="utf-8")
    entries = {entry["path"]: entry for entry in tree["tree"] if entry["type"] == "blob"}
    for name in required:
        url = f"https://raw.githubusercontent.com/denoland/deno/{DENO_REVISION}/{name}"
        raw = download(url, cache / "sources" / name)
        if git_blob(raw) != entries[name]["sha"]:
            raise ValueError("Bootstrapped Deno Git blob mismatch")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cache", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--all-rust", action="store_true")
    parser.add_argument("--native-dependencies", action="store_true", help="Collect each explicitly pinned Rusty V8 submodule's root/SPDX notices")
    parser.add_argument("--native-sources", action="store_true", help="Also retain exact submodule source archives; implies --native-dependencies")
    args = parser.parse_args()
    bootstrap_deno_cache(args.cache, args.output)
    deno, lock = collect_deno_local(args.cache, args.output)
    registry = [p for p in lock["package"] if p.get("source") == "registry+https://github.com/rust-lang/crates.io-index"]
    v8_package = next(p for p in registry if p["name"] == "v8")
    v8_crate = crate_evidence(v8_package, args.output)
    if v8_crate["version"] != "150.2.0" or v8_crate["repository"].rstrip("/").removesuffix(".git") != "https://github.com/denoland/rusty_v8":
        raise ValueError("Unexpected exact Deno V8 crate identity")
    rust_revision = v8_crate["vcs"]["git"]["sha1"]
    rusty_tree = repo_tree("denoland/rusty_v8", rust_revision, args.output)
    rusty = repo_notices("denoland/rusty_v8", rust_revision, rusty_tree, args.output)
    modules_path = args.output / "upstream/denoland--rusty_v8" / rust_revision / ".gitmodules"
    modules = gitmodules(modules_path.read_text(encoding="utf-8"))
    v8_link = next(e for e in rusty_tree["tree"] if e["path"] == "v8" and e["type"] == "commit")
    native_repo = github_repository(modules["v8"])
    native_tree = repo_tree(native_repo, v8_link["sha"], args.output)
    native = repo_notices(native_repo, v8_link["sha"], native_tree, args.output)
    crates, failures = [v8_crate], []
    native_dependencies = []
    if args.native_dependencies or args.native_sources:
        for link in rusty["gitlinks"]:
            try:
                record = module_root_notices(modules[link["path"]], link["revision"], link["path"], args.output, args.native_sources)
                native_dependencies.append(record)
                if record.get("source_archive_failure"):
                    failures.append({"component": link["path"], "revision": link["revision"], "scope": "source-archive", **record["source_archive_failure"]})
            except Exception as error:
                failures.append({"component": link["path"], "revision": link["revision"], "error_type": type(error).__name__,
                                 "http_status": getattr(error, "code", None)})
    if args.all_rust:
        rest = [p for p in registry if p["name"] != "v8"]
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            futures = {pool.submit(crate_evidence, p, args.output): p for p in rest}
            for future in concurrent.futures.as_completed(futures):
                package = futures[future]
                try:
                    crates.append(future.result())
                except Exception as error:
                    failures.append({"name": package["name"], "version": package["version"], "error_type": type(error).__name__})
    native_source_failures = [record for record in native_dependencies if record.get("source_archive_failure")]
    gaps = ["Chromium/V8 DEPS dependency repositories are not resolved by this collector. Retained DEPS/gitlinks require separate source/notice review.",
            "This repository/lockfile superset is not an exact linked-component map or a reproducible-build attestation for the shipped Deno binary."]
    if not args.all_rust:
        gaps.append("Run --all-rust to collect the remaining exact upstream Cargo.lock registry crates.")
    if not args.native_dependencies and not args.native_sources:
        gaps.append("The explicit native submodule root/SPDX notices have not been collected; run --native-dependencies.")
    if not args.native_sources:
        gaps.append("Native gitlink source archives were not requested; run --native-sources or retain them through the separate source-distribution process.")
    if failures:
        gaps.append("Some dependency notice or source downloads failed; see per-component failures.")
    if native_source_failures:
        gaps.append("Some explicitly requested native source archives failed; see native_source_failures.")
    if any(not record["notices"] for record in crates):
        gaps.append("Some locked Rust packages lack standalone notices in their archives; review retained source headers and exact upstream attributions.")
    report = {"schema": 1, "component": "Deno", "version": DENO_VERSION, "revision": DENO_REVISION,
              "status": "evidence-collected-review-required", "deno_files": deno, "rusty_v8": rusty,
              "native_v8": native, "registry_packages_in_lock": len(registry),
              "native_dependencies": native_dependencies,
              "native_source_failures": native_source_failures,
              "crates": sorted(crates, key=lambda p: (p["name"], p["version"])), "failures": failures,
              "remaining_gaps": gaps, "scope": "Conservative exact-source evidence; test/example notices may be overincluded. Not automatic distribution clearance."}
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / "deno-notice-index.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"deno_files": len(deno), "rusty_v8_files": len(rusty["files"]), "native_v8_files": len(native["files"]),
                      "native_dependency_records": len(native_dependencies),
                      "registry_packages_in_lock": len(registry), "collected_crates": len(crates), "failures": len(failures), "remaining_gaps": gaps}, indent=2))
    if failures or native_source_failures:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
