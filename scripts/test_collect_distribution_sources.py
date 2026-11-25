import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import io
import json

import collect_distribution_sources as collector


class SourceTests(unittest.TestCase):
    def test_python_source_hash_is_checked_without_execution(self):
        data = b"source bytes; never executed"
        package = {"urls": [{"packagetype": "sdist", "filename": "example.tar.gz",
                             "url": "https://files.pythonhosted.org/example.tar.gz",
                             "digests": {"sha256": hashlib.sha256(data).hexdigest()}, "size": len(data)}]}
        def response(content, url):
            stream = io.BytesIO(content)
            stream.url = url
            return stream
        with tempfile.TemporaryDirectory() as temporary:
            with patch.object(collector, "urlopen", side_effect=[response(json.dumps(package).encode(), "https://pypi.org"), response(data, package["urls"][0]["url"])]):
                result = collector.fetch_python("example", "1", Path(temporary))
                self.assertEqual(result[0]["sha256"], hashlib.sha256(data).hexdigest())
            package["urls"][0]["filename"] = "tampered.tar.gz"
            with patch.object(collector, "urlopen", side_effect=[response(json.dumps(package).encode(), "https://pypi.org"), response(b"tampered", package["urls"][0]["url"])]):
                with self.assertRaisesRegex(ValueError, "checksum"):
                    collector.fetch_python("example", "1", Path(temporary))

    def test_signed_descriptor_identity_and_archive_integrity(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            data = b"upstream source archive"
            archive = root / "example.orig.tar.xz"
            archive.write_bytes(data)
            digest = hashlib.sha256(data).hexdigest()
            descriptor = root / "example.dsc"
            descriptor.write_text(f"-----BEGIN PGP SIGNED MESSAGE-----\nHash: SHA256\n\nFormat: 3.0 (quilt)\nSource: example\nVersion: 1:2.0-1\nChecksums-Sha256:\n {digest} {len(data)} {archive.name}\n-----BEGIN PGP SIGNATURE-----\nignored\n")
            self.assertEqual(len(collector.verify_dsc(root, "example", "1:2.0-1")), 2)
            with self.assertRaisesRegex(ValueError, "identity"):
                collector.verify_dsc(root, "example", "2.0-1")
            archive.write_bytes(b"tampered")
            with self.assertRaisesRegex(ValueError, "checksum"):
                collector.verify_dsc(root, "example", "1:2.0-1")

    def test_unsafe_descriptor_paths_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "example.dsc").write_text("Source: example\nVersion: 1\nChecksums-Sha256:\n " + "0" * 64 + " 1 ../escape\n")
            with self.assertRaisesRegex(ValueError, "unsafe"):
                collector.verify_dsc(root, "example", "1")

    def test_output_cannot_overwrite_existing_evidence(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "retained").write_bytes(b"keep")
            with self.assertRaisesRegex(ValueError, "empty"):
                collector.collect({}, root)
            self.assertEqual((root / "retained").read_bytes(), b"keep")


if __name__ == "__main__":
    unittest.main()
