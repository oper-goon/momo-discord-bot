"""Offline checks for immutable Deno licensing evidence collection."""
import hashlib
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest

import collect_deno_notices as collector


class DenoEvidenceTests(unittest.TestCase):
    def test_submodule_identity_uses_declared_fork_without_changing_revision(self):
        modules = collector.gitmodules('[submodule "v8"]\npath = v8\nurl = https://github.com/denoland/v8.git\n')
        self.assertEqual(collector.github_repository(modules["v8"]), "denoland/v8")
        for url in ["http://github.com/denoland/v8", "https://example.com/denoland/v8", "https://token@github.com/denoland/v8"]:
            with self.assertRaises(ValueError):
                collector.github_repository(url)

    def test_git_object_identity_is_not_plain_file_hash(self):
        raw = b"release fixture\n"
        self.assertEqual(collector.git_blob(raw), hashlib.sha1(b"blob 16\0" + raw).hexdigest())
        self.assertNotEqual(collector.git_blob(raw), hashlib.sha1(raw).hexdigest())

    def test_notice_paths_include_spdx_files_and_reject_escape(self):
        self.assertTrue(collector.is_notice("LICENSES/MIT.txt"))
        self.assertTrue(collector.is_notice("third_party/component/COPYING"))
        self.assertFalse(collector.is_notice("src/library.rs"))
        self.assertFalse(collector.is_notice("src/copying-phase.cc"))
        for path in ["../LICENSE", "/etc/LICENSE", "C:/LICENSE", "LICENSES/../../outside", "..\\LICENSE"]:
            with self.assertRaises(ValueError):
                collector.safe_relative(path)

    def test_changed_deno_cache_rejected_before_collection(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cache = root / "cache"
            (cache / "sources").mkdir(parents=True)
            (cache / "sources/LICENSE.md").write_bytes(b"modified fixture")
            (cache / "tree.json").write_text(json.dumps({"truncated": False, "tree": [
                {"path": "LICENSE.md", "type": "blob", "sha": collector.git_blob(b"original fixture")}
            ]}), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "does not match pinned tree"):
                collector.collect_deno_local(cache, root / "output")

    def test_verified_crate_archive_retains_notice_bytes_and_vcs(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            archive = output / "sources/fixture-1.0.0.crate"
            archive.parent.mkdir()
            files = {
                "fixture-1.0.0/Cargo.toml": b'[package]\nname="fixture"\nversion="1.0.0"\nlicense="MIT"\n',
                "fixture-1.0.0/LICENSES/MIT.txt": b"test fixture notice; not an upstream license\n",
                "fixture-1.0.0/.cargo_vcs_info.json": b'{"git":{"sha1":"' + b"a" * 40 + b'"}}',
            }
            with tarfile.open(archive, "w:gz") as tar:
                for name, raw in files.items():
                    member = tarfile.TarInfo(name)
                    member.size = len(raw)
                    tar.addfile(member, io.BytesIO(raw))
            package = {"name": "fixture", "version": "1.0.0", "checksum": collector.sha256(archive.read_bytes())}
            record = collector.crate_evidence(package, output)
            self.assertEqual(record["vcs"]["git"]["sha1"], "a" * 40)
            self.assertEqual(len(record["notices"]), 1)
            notice = record["notices"][0]
            self.assertEqual((output / notice["file"]).read_bytes(), files["fixture-1.0.0/LICENSES/MIT.txt"])
            self.assertEqual(notice["sha256"], collector.sha256(files["fixture-1.0.0/LICENSES/MIT.txt"]))
            package["checksum"] = "0" * 64
            with self.assertRaisesRegex(ValueError, "checksum mismatch"):
                collector.crate_evidence(package, output)


if __name__ == "__main__":
    unittest.main()
