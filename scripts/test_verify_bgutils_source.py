import hashlib
import io
from pathlib import Path
import tarfile
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import verify_bgutils_source as verifier


class VerificationTests(unittest.TestCase):
    def test_source_equivalence_rejects_modified_installed_file(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive = root / "source.tar.gz"
            relative = "yt_dlp_plugins/extractor/example.py"
            content = b"# upstream source\n"
            with tarfile.open(archive, "w:gz") as output:
                member = tarfile.TarInfo("commit/plugin/" + relative)
                member.size = len(content)
                output.addfile(member, io.BytesIO(content))
            installed = root / relative
            installed.parent.mkdir(parents=True)
            installed.write_bytes(content)
            distribution = SimpleNamespace(version="2.0.2", files=[relative], locate_file=lambda path: root / path)
            digest = hashlib.sha256(archive.read_bytes()).hexdigest()
            with patch.object(verifier.metadata, "distribution", return_value=distribution):
                self.assertEqual(verifier.verify(archive, digest, root, "2.0.2")["matching_python_files"], 1)
                installed.write_bytes(b"modified")
                with self.assertRaisesRegex(ValueError, "differ"):
                    verifier.verify(archive, digest, root, "2.0.2")
                with self.assertRaisesRegex(ValueError, "checksum"):
                    verifier.verify(archive, "0" * 64, root, "2.0.2")

    def test_archive_traversal_is_rejected_without_extraction(self):
        with tempfile.TemporaryDirectory() as temporary:
            archive = Path(temporary) / "source.tar.gz"
            with tarfile.open(archive, "w:gz") as output:
                output.addfile(tarfile.TarInfo("../escape"), io.BytesIO())
            digest = hashlib.sha256(archive.read_bytes()).hexdigest()
            with self.assertRaisesRegex(ValueError, "unsafe"):
                verifier.verify(archive, digest, Path(temporary), "2.0.2")


if __name__ == "__main__":
    unittest.main()
