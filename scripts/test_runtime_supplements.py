"""Guard the exact third-party assets shipped by the Dockerfile."""
import base64
import hashlib
import json
from pathlib import Path
import unittest
import tarfile
import io


class SupplementTests(unittest.TestCase):
    def test_deno_bundle_checksum_paths_and_notice_bytes(self):
        root = Path(__file__).resolve().parents[1] / "licenses"
        path = root / "deno-2.9.4-notices.tar.gz"
        raw = path.read_bytes()
        self.assertEqual(hashlib.sha256(raw).hexdigest(), path.with_suffix(path.suffix + ".sha256").read_text().split()[0])
        with tarfile.open(fileobj=io.BytesIO(raw), mode="r:gz") as archive:
            for member in archive.getmembers():
                self.assertTrue(member.isfile())
                self.assertFalse(member.name.startswith("/"))
                self.assertNotIn("..", Path(member.name).parts)
            index = json.load(archive.extractfile("deno-notice-index.json"))
            self.assertEqual(index["version"], "2.9.4")
            self.assertTrue(any(record["repository_path"] == "cli/tsc/00_typescript.js" for record in index["deno_files"]))
            records = list(index["deno_files"])
            for section in [index["rusty_v8"], index["native_v8"], *index["native_dependencies"]]:
                records.extend(section["files"])
            for crate in index["crates"]:
                records.extend(crate["notices"])
            for record in records:
                self.assertEqual(hashlib.sha256(archive.extractfile(record["file"]).read()).hexdigest(), record["sha256"])

    def test_ejs_original_archives_and_notices_match_provenance(self):
        root = Path(__file__).resolve().parents[1] / "licenses"
        report = json.loads((root / "ejs-dependencies.json").read_text())
        self.assertEqual(report["parent"]["version"], "0.8.0")
        for package in report["dependencies"]:
            directory = root / "upstream/npm" / (package["name"] + "-" + package["version"])
            archive = (directory / "source.tgz").read_bytes()
            self.assertEqual(hashlib.sha256(archive).hexdigest(), package["archive_sha256"])
            self.assertEqual("sha512-" + base64.b64encode(hashlib.sha512(archive).digest()).decode(), package["integrity"])
            for notice in package["notices"]:
                self.assertEqual(hashlib.sha256((directory / notice["file"]).read_bytes()).hexdigest(), notice["sha256"])


if __name__ == "__main__":
    unittest.main()
