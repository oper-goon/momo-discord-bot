#!/usr/bin/env python3
"""Preserve the pinned documentation theme and bundled icon-set license files."""
import argparse
import hashlib
from importlib import metadata
import json
from pathlib import Path

FILES = {
    "mkdocs-material-LICENSE.txt": "mkdocs_material-9.8.2.dist-info/licenses/LICENSE",
    "fontawesome-LICENSE.txt": "material/templates/.icons/fontawesome/LICENSE.txt",
    "material-icons-LICENSE.txt": "material/templates/.icons/material/LICENSE",
    "octicons-LICENSE.txt": "material/templates/.icons/octicons/LICENSE",
    "simple-icons-LICENSE.txt": "material/templates/.icons/simple/LICENSE.md",
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    distribution = metadata.distribution("mkdocs-material")
    if distribution.version != "9.8.2":
        raise SystemExit("Documentation theme version changed; review and refresh its notices.")
    output = Path(__file__).resolve().parents[1] / "docs/assets/notices"
    payloads = {name: distribution.locate_file(source).read_bytes() for name, source in FILES.items()}
    provenance = {"package": "mkdocs-material", "version": distribution.version,
                  "scope": "Original theme/icon-set notices; icon-set superset, not a claim every icon is used.",
                  "files": [{"file": name, "package_path": FILES[name], "sha256": hashlib.sha256(raw).hexdigest()}
                            for name, raw in sorted(payloads.items())]}
    payloads["provenance.json"] = (json.dumps(provenance, indent=2) + "\n").encode()
    if args.check:
        if any(not (output / name).is_file() or (output / name).read_bytes() != raw for name, raw in payloads.items()):
            raise SystemExit("Documentation theme notices are missing or stale; run scripts/collect_docs_notices.py.")
        print("Documentation theme/icon notices match the pinned package.")
    else:
        output.mkdir(parents=True, exist_ok=True)
        for name, raw in payloads.items():
            (output / name).write_bytes(raw)
        print("Retained five original documentation theme/icon license files.")


if __name__ == "__main__":
    main()
