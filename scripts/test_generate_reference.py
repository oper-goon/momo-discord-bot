"""Regression checks for reference coverage and stale registry detection."""
import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import generate_reference as reference


class ReferenceCoverageTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.registry = json.loads(reference.SNAPSHOT.read_text(encoding="utf-8"))

    def test_every_registered_entry_and_optional_argument_has_reference(self):
        page = reference.command_page(self.registry)
        for command in self.registry["commands"]:
            label = command["context_menu"] or "/" + command["path"].replace(".", " ")
            self.assertIn("### `" + label + "`", page)
        self.assertNotIn("See behavior and feature guide; no fixed default", page)
        self.assertIn("why", page)
        self.assertIn("No reason provided.", page)

    def test_new_and_removed_configuration_require_description_update(self):
        for operation in ("add", "remove"):
            changed = copy.deepcopy(self.registry)
            if operation == "add":
                changed["config_defaults"]["discord"]["new_option"] = True
            else:
                del changed["config_defaults"]["discord"]["owners"]
            with self.assertRaisesRegex(ValueError, "Config coverage mismatch"):
                reference.configuration_page(changed)

    def test_new_environment_variable_fails_coverage(self):
        # Removing an existing known variable exercises the same unknown-name
        # detector used when a developer introduces a new env::var literal.
        existing = reference.ENV.pop("TORBOX_API_KEY")
        try:
            with self.assertRaisesRegex(ValueError, "Undocumented environment"):
                reference.configuration_page(self.registry)
        finally:
            reference.ENV["TORBOX_API_KEY"] = existing

    def test_fresh_runtime_export_drift_fails_check(self):
        changed = copy.deepcopy(self.registry)
        changed["commands"][0]["description"] += " Changed"
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "new-registry.json"
            path.write_text(json.dumps(changed), encoding="utf-8")
            result = subprocess.run(
                [sys.executable, str(reference.ROOT / "scripts/generate_reference.py"),
                 "--registry", str(path), "--check"],
                capture_output=True, text=True, check=False,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("registry snapshot is stale", result.stderr)


if __name__ == "__main__":
    unittest.main()
