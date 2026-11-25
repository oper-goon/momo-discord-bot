"""Synthetic sysroot tests; no network, credentials, or external compiler required."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("toolchain_notices", Path(__file__).with_name("collect_toolchain_notices.py"))
notices = importlib.util.module_from_spec(spec)
spec.loader.exec_module(notices)


class ToolchainNoticesTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.root = self.base / "sysroot"
        self.output = self.base / "notices"
        self.target = "x86_64-unknown-linux-gnu"
        self.info = {"release": "1.98.1", "host": self.target, "commit-hash": "abc123"}
        for name, content in {
            "share/doc/rust/COPYRIGHT-library.html": b"<html>library notices\r\n</html>\n",
            "share/doc/rust/COPYRIGHT.html": b"<html>toolchain notices</html>",
            "share/doc/rust/licenses/Apache-2.0.txt": b"exact Apache notice fixture\n",
            "share/doc/rust/licenses/MIT.txt": b"exact MIT notice fixture\n",
            "share/doc/rust/licenses/dependency/COPYING": b"nested bundled dependency notice\n",
            f"lib/rustlib/manifest-rust-std-{self.target}": b"file:libstd.rlib\n",
            f"lib/rustlib/manifest-rustc-{self.target}": b"file:rustc\n",
            "lib/rustlib/components": b"rustc\nrust-std\n",
            f"lib/rustlib/{self.target}/lib/libstd-fixture.rlib": b"not a real library",
        }.items():
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)

    def collect(self, **kwargs):
        return notices.collect(self.root, kwargs.get("output", self.output), "rustc fixture\n", self.info,
                               kwargs.get("target", self.target), kwargs.get("release", "1.98.1"))

    def test_verbatim_complete_notices_and_inventory(self):
        manifest = self.collect()
        self.assertIn("not an exact", manifest["scope"])
        self.assertEqual(manifest["target_library_files"], ["libstd-fixture.rlib"])
        self.assertEqual(len(manifest["files"]), 8)
        for entry in manifest["files"]:
            original = (self.root / entry["path"]).read_bytes()
            self.assertEqual((self.output / entry["path"]).read_bytes(), original)
            self.assertEqual(entry["sha256"], hashlib.sha256(original).hexdigest())
        self.assertEqual(json.loads((self.output / "manifest.json").read_text()), manifest)
        self.assertFalse(any(self.output.rglob("*.rlib")))

    def test_missing_library_copyright_fails_before_output(self):
        (self.root / "share/doc/rust/COPYRIGHT-library.html").unlink()
        with self.assertRaises(ValueError):
            self.collect()
        self.assertFalse(self.output.exists())

    def test_missing_mit_and_target_manifest_fail(self):
        for relative in ("share/doc/rust/licenses/MIT.txt", f"lib/rustlib/manifest-rust-std-{self.target}"):
            path = self.root / relative
            original = path.read_bytes()
            path.unlink()
            with self.assertRaises(ValueError):
                self.collect()
            path.write_bytes(original)

    def test_version_and_target_mismatch_fail(self):
        with self.assertRaises(ValueError):
            self.collect(release="1.99.0")
        with self.assertRaises(ValueError):
            self.collect(target="../../escape")
        with self.assertRaises(ValueError):
            self.collect(target="aarch64-unknown-linux-gnu")

    def test_existing_output_is_not_overwritten(self):
        self.output.mkdir()
        marker = self.output / "preserve.txt"
        marker.write_bytes(b"keep")
        with self.assertRaises(ValueError):
            self.collect()
        self.assertEqual(marker.read_bytes(), b"keep")

    def test_output_inside_sysroot_is_rejected(self):
        with self.assertRaises(ValueError):
            self.collect(output=self.root / "generated")

    def test_symlinked_notice_rejected(self):
        path = self.root / "share/doc/rust/licenses/MIT.txt"
        try:
            path.unlink()
            path.symlink_to(self.root / "share/doc/rust/licenses/Apache-2.0.txt")
        except OSError:
            self.skipTest("creating symbolic links is unavailable on this host")
        with self.assertRaises(ValueError):
            self.collect()

    def test_rustc_metadata_requires_release_and_host(self):
        with patch.object(notices.subprocess, "run") as run:
            run.return_value.stdout = "rustc 1.98.1\nrelease: 1.98.1\nhost: test-host\n"
            _, fields = notices.rustc_info("rustc-fixture")
            self.assertEqual(fields["release"], "1.98.1")
            run.assert_called_once_with(["rustc-fixture", "-vV"], check=True, capture_output=True, text=True)
            run.return_value.stdout = "unknown output"
            with self.assertRaises(ValueError):
                notices.rustc_info("rustc-fixture")


if __name__ == "__main__":
    unittest.main()
