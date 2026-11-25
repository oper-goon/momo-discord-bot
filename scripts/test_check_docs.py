"""Regression checks for Pages subpaths, missing assets and fragment targets."""
import tempfile
import unittest
from pathlib import Path

from check_docs import validate


class LinkCheckerTests(unittest.TestCase):
    def test_pages_relative_links_and_fragments(self):
        with tempfile.TemporaryDirectory() as temporary:
            site = Path(temporary)
            (site / "guide").mkdir()
            (site / "index.html").write_text('<a href="guide/#install">Install</a><img src="image.svg">')
            (site / "image.svg").write_text('<svg/>')
            (site / "guide/index.html").write_text('<h1 id="install">Install</h1><a href="../">Home</a>')
            errors, count = validate(site, "https://example.org/bot/")
            self.assertEqual(errors, [])
            self.assertEqual(count, 3)

    def test_broken_assets_anchors_and_root_path_fail(self):
        with tempfile.TemporaryDirectory() as temporary:
            site = Path(temporary)
            (site / "index.html").write_text('<a href="#absent">Bad</a><img src="lost.svg"><a href="/guide/">Bad</a>')
            errors, _ = validate(site, "https://example.org/bot/")
            self.assertEqual(len(errors), 3)


if __name__ == "__main__":
    unittest.main()
