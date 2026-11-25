#!/usr/bin/env python3
"""Compare installed plugin Python files to retained upstream source; no execution."""
import argparse
import hashlib
from importlib import metadata
import json
from pathlib import Path, PurePosixPath
import tarfile


def verify(archive: Path, digest: str, installed: Path, version: str) -> dict:
    if hashlib.sha256(archive.read_bytes()).hexdigest() != digest:
        raise ValueError("source archive checksum mismatch")
    expected = {}
    with tarfile.open(archive) as source:
        for member in source.getmembers():
            path = PurePosixPath(member.name)
            if path.is_absolute() or ".." in path.parts or not (member.isfile() or member.isdir()):
                raise ValueError("unsafe archive member")
            parts = path.parts
            if len(parts) > 3 and parts[1:3] == ("plugin", "yt_dlp_plugins") and path.suffix == ".py":
                if member.size > 2 * 1024 * 1024:
                    raise ValueError("source file exceeds limit")
                relative = PurePosixPath(*parts[2:]).as_posix()
                expected[relative] = source.extractfile(member).read()
    if not expected:
        raise ValueError("source contains no plugin Python files")
    distribution = metadata.distribution("bgutil-ytdlp-pot-provider")
    if distribution.version != version:
        raise ValueError("installed plugin version mismatch")
    actual = {}
    for item in distribution.files or []:
        path = PurePosixPath(str(item).replace("\\", "/"))
        if path.parts and path.parts[0] == "yt_dlp_plugins" and path.suffix == ".py":
            if path.is_absolute() or ".." in path.parts:
                raise ValueError("unsafe installed plugin path")
            file = Path(distribution.locate_file(item)).resolve(strict=True)
            if not file.is_relative_to(installed.resolve()):
                raise ValueError("installed plugin outside approved directory")
            actual[path.as_posix()] = file.read_bytes()
    if actual != expected:
        raise ValueError("installed plugin Python files differ from retained source")
    return {"version": version, "archive_sha256": digest, "matching_python_files": len(actual),
            "scope": "Python file byte equivalence only; not complete distribution license clearance"}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, required=True)
    parser.add_argument("--sha256", required=True)
    parser.add_argument("--installed", type=Path, required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = verify(args.archive, args.sha256, args.installed, args.version)
    args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
