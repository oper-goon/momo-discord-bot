#!/usr/bin/env python3
"""Package checksum-verified Deno notice evidence, separately from source archives."""
import argparse
import gzip
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import tarfile


def package(source, output):
    index_bytes = (source / "deno-notice-index.json").read_bytes()
    index = json.loads(index_bytes)
    if index["version"] != "2.9.4" or index["failures"] or index.get("native_source_failures"):
        raise ValueError("unexpected or incomplete Deno evidence")
    records = list(index["deno_files"])
    for section in [index["rusty_v8"], index["native_v8"], *index["native_dependencies"]]:
        records.extend(section["files"])
    for crate in index["crates"]:
        records.extend(crate["notices"])
    files = {"deno-notice-index.json": index_bytes,
             "README.txt": b"Deno 2.9.4 conservative notice evidence\n\nOriginal notice bytes and exact-source provenance are retained.\nThis bundle is a repository/lockfile superset, not an exact linked-code inventory.\nSource archives are separate; their exact URLs and hashes are in deno-notice-index.json.\nReview remaining_gaps and native dependency source records.\nThird-party source and notices retain their own terms, not Momokun PolyForm restrictions.\n"}
    for record in records:
        name = record["file"]
        relative = PurePosixPath(name)
        if relative.is_absolute() or ".." in relative.parts or "\\" in name or ":" in name:
            raise ValueError("unsafe notice path")
        path = source / name
        if path.is_symlink() or not path.resolve().is_relative_to(source.resolve()):
            raise ValueError("notice escapes evidence directory")
        raw = path.read_bytes()
        if hashlib.sha256(raw).hexdigest() != record["sha256"]:
            raise ValueError("notice hash mismatch")
        files[name] = raw
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("wb") as stream, gzip.GzipFile(fileobj=stream, mode="wb", filename="", mtime=0) as compressed, tarfile.open(fileobj=compressed, mode="w") as archive:
        for name, raw in sorted(files.items()):
            member = tarfile.TarInfo(name)
            member.mode = 0o644
            member.size = len(raw)
            archive.addfile(member, io.BytesIO(raw))
    digest = hashlib.sha256(output.read_bytes()).hexdigest()
    output.with_suffix(output.suffix + ".sha256").write_text(digest + "  " + output.name + "\n", encoding="ascii")
    return {"files": len(files), "sha256": digest, "bytes": output.stat().st_size}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    print(json.dumps(package(args.source, args.output), sort_keys=True))
