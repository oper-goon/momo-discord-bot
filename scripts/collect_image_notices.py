#!/usr/bin/env python3
"""Inventory the completed Debian image; copy notices without claiming clearance.

Run in the image, for example: python3 collect_image_notices.py --output /tmp/notices
No network calls, environment dump, package installation, or source downloads.
"""
from __future__ import annotations

import argparse
import hashlib
from importlib import metadata
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys

NOTICE = re.compile(r"^(?:licen[cs]e|copying|copyright|notice|authors)(?:[._-].*)?$", re.I)
TOOLS = {
    "ffmpeg_license": ["ffmpeg", "-L"],
    "ffmpeg": ["ffmpeg", "-version"],
    "ffmpeg_build": ["ffmpeg", "-buildconf"],
    "ffprobe": ["ffprobe", "-version"],
    "deno": ["deno", "--version"],
    "yt_dlp": ["/opt/ytdlp/bin/yt-dlp", "--ignore-config", "--version"],
    "python": ["python3", "--version"],
    "sqlite": ["sqlite3", "--version"],
    "openssl": ["openssl", "version"],
}


def run(command: list[str]) -> dict:
    # Do not inherit bot tokens, proxy credentials, user configuration or logs.
    env = {"PATH": "/opt/ytdlp/bin:/usr/local/bin:/usr/bin:/bin", "LC_ALL": "C",
           "DENO_NO_UPDATE_CHECK": "1", "DENO_NO_PROMPT": "1"}
    try:
        completed = subprocess.run(command, capture_output=True, timeout=30, env=env, check=False)
    except (OSError, subprocess.TimeoutExpired) as error:
        return {"status": "unavailable", "reason": type(error).__name__}
    raw = completed.stdout + completed.stderr
    if len(raw) > 131072:
        return {"status": "incomplete", "reason": "output limit exceeded", "returncode": completed.returncode}
    return {"status": "ok" if completed.returncode == 0 else "failed",
            "returncode": completed.returncode, "output": raw.decode("utf-8", errors="replace").strip()}


def safe_name(value: str) -> str:
    result = re.sub(r"[^A-Za-z0-9_.+-]", "_", value)
    if not result or result in {".", ".."}:
        raise ValueError("invalid component name")
    return result


def copy_notice(source: Path, allowed: Path, target: Path, output: Path) -> dict:
    """Follow a shared-doc symlink only within the approved source root."""
    if target.is_absolute() or ".." in target.parts:
        raise ValueError("unsafe output path")
    resolved = source.resolve(strict=True)
    if not resolved.is_relative_to(allowed.resolve()) or not resolved.is_file():
        raise ValueError("notice outside approved root or not a regular file")
    if resolved.stat().st_size > 8 * 1024 * 1024:
        raise ValueError("notice exceeds size limit")
    destination = output / target
    destination.parent.mkdir(parents=True, exist_ok=True)
    # Output must start empty, so no pre-existing symlinks can redirect writes.
    shutil.copyfile(resolved, destination)
    return {"file": target.as_posix(), "sha256": hashlib.sha256(destination.read_bytes()).hexdigest()}


def collect_dpkg(output: Path, doc_root: Path = Path("/usr/share/doc")) -> dict:
    result = run(["dpkg-query", "-W", "-f=${binary:Package}\t${Version}\t${Architecture}\t${source:Package}\t${source:Version}\t${db:Status-Status}\\n"])
    if result["status"] != "ok":
        return {"status": result["status"], "reason": "dpkg-query unavailable or failed", "packages": []}
    packages = []
    for line in result["output"].splitlines():
        fields = line.split("\t")
        if len(fields) != 6:
            raise ValueError("unexpected dpkg-query inventory format")
        name, version, architecture, source, source_version, status = fields
        if status != "installed":
            continue
        package = {"name": name, "version": version, "architecture": architecture,
                   "source_package": source, "source_version": source_version,
                   "notices": [], "issues": []}
        # Debian commonly shares copyright files through directory symlinks.
        directory = doc_root / name.split(":", 1)[0]
        if directory.exists() and directory.resolve().is_relative_to(doc_root.resolve()):
            for item in sorted(directory.rglob("*")):
                if item.is_file() and NOTICE.match(item.name):
                    try:
                        relative = item.relative_to(directory)
                        target = Path("debian") / safe_name(name) / relative
                        package["notices"].append(copy_notice(item, doc_root, target, output))
                    except (OSError, ValueError) as error:
                        package["issues"].append({"file": item.name, "reason": type(error).__name__})
        if not package["notices"]:
            package["issues"].append({"reason": "no matching documentation notice found"})
        packages.append(package)
    return {"status": "ok", "packages": sorted(packages, key=lambda p: (p["name"], p["version"]))}


def python_paths(extra: list[str]) -> list[str]:
    paths = {str(Path(p).resolve()) for p in sys.path if p and Path(p).is_dir()}
    # System Python cannot discover the media venv through sys.path alone.
    paths.update(str(p.resolve()) for p in Path("/opt/ytdlp/lib").glob("python*/site-packages") if p.is_dir())
    paths.update(str(Path(p).resolve(strict=True)) for p in extra)
    return sorted(paths)


def collect_python(output: Path, paths: list[str]) -> dict:
    packages = []
    seen = set()
    distributions = sorted(metadata.distributions(path=paths), key=lambda d: (
        d.metadata.get("Name") or "unnamed", d.version, str(Path(d.locate_file("")).resolve())))
    for instance, distribution in enumerate(distributions):
        name = distribution.metadata.get("Name") or "unnamed"
        version = distribution.version
        root = Path(distribution.locate_file("")).resolve()
        identity = (name, version, str(root))
        if identity in seen:
            continue
        seen.add(identity)
        package = {"name": name, "version": version,
                   "instance": instance,
                   "license_expression": distribution.metadata.get("License-Expression"),
                   "license_metadata": distribution.metadata.get("License"),
                   "declared_license_files": sorted(distribution.metadata.get_all("License-File") or []),
                   "notices": [], "issues": []}
        files = distribution.files
        if files is None:
            package["issues"].append({"reason": "distribution file inventory unavailable"})
        for entry in sorted(files or [], key=str):
            relative = Path(str(entry))
            # PEP 639 places texts under .dist-info/licenses with arbitrary names.
            is_license_directory = "licenses" in relative.parts and any(part.endswith(".dist-info") for part in relative.parts)
            declared = set(package["declared_license_files"])
            candidates = {relative.as_posix()}
            for index, part in enumerate(relative.parts):
                if part.endswith(".dist-info"):
                    candidates.add(Path(*relative.parts[index + 1:]).as_posix())
            if not (NOTICE.match(relative.name) or is_license_directory or candidates.intersection(declared)):
                continue
            try:
                if relative.is_absolute() or ".." in relative.parts:
                    raise ValueError("unsafe distribution path")
                target = Path("python") / (safe_name(name + "-" + version) + f"-{instance:04d}") / relative
                package["notices"].append(copy_notice(Path(distribution.locate_file(entry)), root, target, output))
            except (OSError, ValueError) as error:
                package["issues"].append({"file": relative.name, "reason": type(error).__name__})
        if not package["notices"]:
            package["issues"].append({"reason": "no matching packaged license text found"})
        packages.append(package)
    # Duplicate versions in separate roots require review; never silently merge.
    counts = {}
    for package in packages:
        key = (package["name"], package["version"])
        counts[key] = counts.get(key, 0) + 1
    for package in packages:
        if counts[(package["name"], package["version"])] > 1:
            package["issues"].append({"reason": "same distribution in multiple search roots; review duplicate notices"})
    return {"status": "ok", "packages": sorted(packages, key=lambda p: (p["name"], p["version"], json.dumps(p, sort_keys=True)))}


def collect_common_licenses(output: Path, source: Path = Path("/usr/share/common-licenses")) -> dict:
    """Preserve full texts referenced by Debian copyright files."""
    notices, issues = [], []
    if not source.is_dir():
        return {"notices": [], "issues": ["common-license directory unavailable"]}
    for entry in sorted(source.iterdir()):
        try:
            notices.append(copy_notice(entry, source, Path("common-licenses") / entry.name, output))
        except (OSError, ValueError) as error:
            issues.append({"file": entry.name, "reason": type(error).__name__})
    return {"notices": notices, "issues": issues}


def collect(output: Path, paths: list[str]) -> dict:
    if output.exists() and (output.is_symlink() or not output.is_dir() or any(output.iterdir())):
        raise ValueError("output must be a new or empty real directory")
    output.mkdir(parents=True, exist_ok=True)
    inventory = {"schema_version": 1, "legal_clearance": False,
                 "scope": "installed Debian and discovered Python distributions in this running image",
                 "debian": collect_dpkg(output), "python": collect_python(output, paths),
                 "common_licenses": collect_common_licenses(output),
                 "tools": {name: run(command) for name, command in sorted(TOOLS.items())},
                 "review_required": [
                     "Missing notices and package metadata require review; matching files do not prove compliance.",
                     "FFmpeg configuration records the actual executable build; inspect codecs, linkage and applicable source obligations.",
                     "Debian source-package names identify versions, but this bundle does not supply corresponding source or a source offer.",
                     "Python license fields are upstream metadata, not a legal conclusion; bundled native components may have additional notices.",
                     "Deno and manually copied executables are not necessarily represented by dpkg or Python metadata; collect their upstream notices separately.",
                     "Rust binary notices and MPL-covered source are separate release materials; no Rust source bundle is generated here.",
                     "The optional provider image must be inventoried independently.",
                 ]}
    (output / "inventory.json").write_text(json.dumps(inventory, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return inventory


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--python-path", action="append", default=[], help="additional installed-distribution search directory")
    args = parser.parse_args()
    try:
        inventory = collect(args.output, python_paths(args.python_path))
    except (OSError, ValueError) as error:
        print(f"Notice collection failed: {type(error).__name__}", file=sys.stderr)
        return 1
    counts = {kind: len(inventory[kind]["packages"]) for kind in ("debian", "python")}
    print(json.dumps({"packages": counts, "legal_clearance": False}, sort_keys=True))
    return 0 if inventory["debian"]["status"] == "ok" else 1


if __name__ == "__main__":
    raise SystemExit(main())
