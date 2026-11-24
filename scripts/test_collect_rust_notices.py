import tempfile
import unittest
from pathlib import Path

from collect_rust_notices import notice_files, reachable


class RustNoticeTests(unittest.TestCase):
    def test_non_development_resolved_edges_only(self):
        packages = [{"id": x, "name": x, "version": "1"} for x in ("bot-app", "run", "build", "dev", "unused")]
        edges = [{"pkg": p, "dep_kinds": [{"kind": k}]} for p, k in
                 (("run", None), ("build", "build"), ("dev", "dev"))]
        metadata = {"packages": packages, "resolve": {"nodes": [
            {"id": p["id"], "deps": edges if p["id"] == "bot-app" else []} for p in packages]}}
        self.assertEqual([p["name"] for p in reachable(metadata, "bot-app")], ["bot-app", "build", "run"])

    def test_nested_component_notices_are_retained(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "nested").mkdir()
            (root / "LICENSES").mkdir()
            for filename in ("LICENSE", "LICENSES/MIT.txt", "nested/LICENSE-BoringSSL", "nested/NOTICE.txt"):
                (root / filename).write_text("original text")
            self.assertEqual([p.relative_to(root).as_posix() for p in notice_files(root)],
                             ["LICENSE", "LICENSES/MIT.txt", "nested/LICENSE-BoringSSL", "nested/NOTICE.txt"])

    def test_declared_path_escape_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "package"
            root.mkdir()
            (root.parent / "private").write_text("must not copy")
            with self.assertRaises(ValueError):
                notice_files(root, "../private")


if __name__ == "__main__":
    unittest.main()
