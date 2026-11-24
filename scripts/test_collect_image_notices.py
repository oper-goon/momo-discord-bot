import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import collect_image_notices as collector


class NoticeCollectionTests(unittest.TestCase):
    def test_tools_do_not_inherit_operator_credentials(self):
        with patch.dict(os.environ, {"DISCORD_TOKEN": "test-only-placeholder"}):
            with patch.object(collector.subprocess, "run") as run:
                run.return_value.stdout = b"version"
                run.return_value.stderr = b""
                run.return_value.returncode = 0
                collector.run(["python3", "--version"])
                self.assertNotIn("DISCORD_TOKEN", run.call_args.kwargs["env"])

    def test_rejects_source_and_target_escape_without_symlinks(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "input"
            source.mkdir()
            (root / "outside").write_text("retain")
            with self.assertRaises(ValueError):
                collector.copy_notice(root / "outside", source, Path("notice"), root / "out")
            with self.assertRaises(ValueError):
                collector.copy_notice(root / "outside", root, Path("../outside"), root / "out")

    def test_notice_copy_preserves_bytes_and_digest(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "input"
            source.mkdir()
            data = b"Copyright Example\r\nLicense text\n"
            (source / "copyright").write_bytes(data)
            output = root / "out"
            result = collector.copy_notice(source / "copyright", source, Path("pkg/copyright"), output)
            self.assertEqual((output / result["file"]).read_bytes(), data)
            self.assertEqual(len(result["sha256"]), 64)

    def test_symlink_cannot_escape_approved_root(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "input"
            source.mkdir()
            (root / "private").write_text("not a notice")
            try:
                (source / "LICENSE").symlink_to(root / "private")
            except OSError:
                self.skipTest("symlink creation unavailable")
            with self.assertRaises(ValueError):
                collector.copy_notice(source / "LICENSE", source, Path("LICENSE"), root / "out")
            self.assertFalse((root / "out").exists())

    def test_dpkg_reports_missing_notices_and_source_versions(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            docs = root / "docs"
            (docs / "present").mkdir(parents=True)
            (docs / "present/copyright").write_text("upstream text")
            result = {"status": "ok", "output": "present:amd64\t1.0\tamd64\tsource-pkg\t1.0-1\tinstalled\nmissing\t2\tall\tmissing\t2\tinstalled"}
            with patch.object(collector, "run", return_value=result):
                inventory = collector.collect_dpkg(root / "out", docs)
            self.assertEqual(inventory["packages"][1]["source_version"], "1.0-1")
            self.assertEqual(len(inventory["packages"][1]["notices"]), 1)
            self.assertTrue(inventory["packages"][0]["issues"])

    def test_python_collects_arbitrarily_named_pep639_license(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            site = root / "site"
            info = site / "example-1.0.dist-info"
            (info / "licenses").mkdir(parents=True)
            (info / "METADATA").write_text("Metadata-Version: 2.4\nName: example\nVersion: 1.0\nLicense-Expression: MIT\nLicense-File: legal.txt\n")
            (info / "licenses/legal.txt").write_text("retained upstream notice")
            (info / "RECORD").write_text("example-1.0.dist-info/licenses/legal.txt,,\nexample-1.0.dist-info/METADATA,,\n")
            inventory = collector.collect_python(root / "out", [str(site)])
            package = inventory["packages"][0]
            self.assertEqual(package["license_expression"], "MIT")
            self.assertEqual(len(package["notices"]), 1)
            self.assertNotIn(str(root), json.dumps(inventory))

    def test_refuses_existing_output_to_prevent_overwrites(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "existing").write_text("retain")
            with self.assertRaises(ValueError):
                collector.collect(root, [])
            self.assertEqual((root / "existing").read_text(), "retain")


if __name__ == "__main__":
    unittest.main()
