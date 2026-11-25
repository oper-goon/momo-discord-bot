#!/usr/bin/env python3
"""Acquire exact Debian/Python sources in an isolated release-evidence container.

No source extraction, builds, package installation, or plugin imports. Debian
downloads use apt's authenticated source indexes; PyPI archives use API SHA-256.
The resulting inventory is evidence, not a declaration of legal clearance.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
from urllib.parse import urlparse
from urllib.request import Request, urlopen


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def parse_dsc(text):
    fields = {}
    current = None
    for line in text.splitlines():
        if line.startswith("-----BEGIN PGP SIGNATURE-----"):
            break
        if line.startswith((" ", "\t")) and current:
            fields[current] += "\n" + line.strip()
        elif re.match(r"^[A-Za-z][A-Za-z0-9-]*:", line):
            current, value = line.split(":", 1)
            fields[current] = value.strip()
        else:
            current = None
    return fields


def verify_dsc(directory, name, version):
    descriptors = list(directory.glob("*.dsc"))
    if len(descriptors) != 1:
        raise ValueError("expected one Debian source descriptor")
    descriptor = descriptors[0]
    fields = parse_dsc(descriptor.read_text(encoding="utf-8"))
    if fields.get("Source") != name or fields.get("Version") != version:
        raise ValueError("Debian source identity mismatch")
    records = []
    for row in fields.get("Checksums-Sha256", "").strip().splitlines():
        digest, size, filename = row.split()
        if Path(filename).name != filename or filename in {".", ".."}:
            raise ValueError("unsafe Debian source filename")
        file = directory / filename
        if not file.is_file() or file.stat().st_size != int(size) or sha256(file) != digest:
            raise ValueError("Debian source checksum mismatch")
        records.append({"file": filename, "sha256": digest, "size": int(size)})
    if not records:
        raise ValueError("Debian descriptor lacks SHA-256 checksums")
    records.append({"file": descriptor.name, "sha256": sha256(descriptor), "size": descriptor.stat().st_size})
    return records


def fetch_python(name, version, directory):
    if not re.fullmatch(r"[A-Za-z0-9_.+-]+", name) or not re.fullmatch(r"[A-Za-z0-9_.+!-]+", version):
        raise ValueError("invalid Python package identity")
    request = Request(f"https://pypi.org/pypi/{name}/{version}/json", headers={"User-Agent": "Momokun-source-evidence/1.0"})
    with urlopen(request, timeout=60) as response:
        raw = response.read(16 * 1024 * 1024 + 1)
    if len(raw) > 16 * 1024 * 1024:
        raise ValueError("PyPI metadata exceeds limit")
    data = json.loads(raw)
    archives = [item for item in data["urls"] if item["packagetype"] == "sdist"]
    if not archives:
        raise ValueError("no upstream Python source distribution")
    records = []
    for item in archives:
        filename, url = item["filename"], item["url"]
        parsed = urlparse(url)
        if Path(filename).name != filename or parsed.scheme != "https" or parsed.hostname != "files.pythonhosted.org" or parsed.username:
            raise ValueError("unexpected Python source location")
        destination = directory / filename
        with urlopen(Request(url, headers={"User-Agent": "Momokun-source-evidence/1.0"}), timeout=120) as response, destination.open("xb") as output:
            if urlparse(response.url).hostname != "files.pythonhosted.org":
                raise ValueError("unexpected Python source redirect")
            size = 0
            while block := response.read(1024 * 1024):
                size += len(block)
                if size > 1024 * 1024 * 1024:
                    raise ValueError("Python source archive exceeds limit")
                output.write(block)
        digest = sha256(destination)
        if digest != item["digests"]["sha256"] or size != item["size"]:
            raise ValueError("Python source checksum mismatch")
        records.append({"file": filename, "url": url, "sha256": digest, "size": size})
    (directory / "pypi.json").write_bytes(raw)
    return records


def collect(inventory, output):
    if output.exists() and (output.is_symlink() or not output.is_dir() or any(output.iterdir())):
        raise ValueError("output must be a new or empty real directory")
    output.mkdir(parents=True, exist_ok=True)
    report = {"legal_clearance": False, "debian": [], "python": [], "issues": [],
              "scope": "Exact source-package downloads; upstream Python sdists do not prove wheel or Debian patch equivalence."}
    sources = {}
    for package in inventory["debian"]["packages"]:
        identity = (package["source_package"], package["source_version"])
        sources.setdefault(identity, []).append(package["name"])
    env = {"PATH": "/usr/sbin:/usr/bin:/sbin:/bin", "LC_ALL": "C", "HOME": "/tmp"}
    for (name, version), binaries in sorted(sources.items()):
        record = {"name": name, "version": version, "binaries": sorted(binaries)}
        try:
            if not re.fullmatch(r"[a-z0-9][a-z0-9+.-]+", name) or not re.fullmatch(r"[A-Za-z0-9.+:~_-]+", version):
                raise ValueError("invalid Debian source identity")
            directory = output / "debian" / name
            directory.mkdir(parents=True)
            command = ["apt-get", "-o", "APT::Sandbox::User=root", "source", "--download-only", "--only-source", name + "=" + version]
            completed = subprocess.run(command, cwd=directory, env=env, stdin=subprocess.DEVNULL,
                                       stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=1800, check=False)
            if completed.returncode:
                raise ValueError("authenticated Debian source download failed")
            record["files"] = verify_dsc(directory, name, version)
        except (OSError, ValueError, subprocess.TimeoutExpired) as error:
            record["issue"] = type(error).__name__
            report["issues"].append({"kind": "debian", "name": name, "version": version, "reason": str(error) if isinstance(error, ValueError) else type(error).__name__})
        report["debian"].append(record)
        print(f"Debian {name}: {'incomplete' if 'issue' in record else 'verified'}", flush=True)
    identities = sorted({(p["name"], p["version"]) for p in inventory["python"]["packages"]})
    for name, version in identities:
        record = {"name": name, "version": version}
        try:
            if not re.fullmatch(r"[A-Za-z0-9_.+-]+", name):
                raise ValueError("invalid Python package name")
            directory = output / "python" / name
            directory.mkdir(parents=True)
            record["files"] = fetch_python(name, version, directory)
        except (OSError, ValueError, KeyError) as error:
            record["issue"] = type(error).__name__
            report["issues"].append({"kind": "python", "name": name, "version": version, "reason": type(error).__name__})
        report["python"].append(record)
        print(f"Python {name}: {'incomplete' if 'issue' in record else 'verified'}", flush=True)
    (output / "source-index.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inventory", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    result = collect(json.loads(args.inventory.read_text(encoding="utf-8")), args.output)
    raise SystemExit(bool(result["issues"]))
