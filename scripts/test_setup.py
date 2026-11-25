"""Isolated setup checks; no Docker daemon or real credentials required."""
import contextlib
import importlib.util
import io
import pathlib
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("momokun_setup", pathlib.Path(__file__).with_name("setup.py"))
setup = importlib.util.module_from_spec(spec)
spec.loader.exec_module(setup)


class SetupTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        (self.root / "config").mkdir()
        (self.root / ".env.example").write_text("DISCORD_TOKEN=replace-with-your-discord-bot-token\n")
        (self.root / "config/example.toml").write_text('[commands]\nmode = "guild"\ndevelopment_guilds = []\n')

    def run_setup(self, docker=True, compose_code=0):
        output = io.StringIO()
        with patch.object(setup.shutil, "which", return_value="docker" if docker else None), patch.object(setup.subprocess, "run") as run, contextlib.redirect_stdout(output):
            run.return_value.returncode = compose_code
            status = setup.main(self.root)
        return status, output.getvalue()

    def configure(self, mode='"global"', guilds="[]", token="private-test-value"):
        (self.root / ".env").write_text(f"DISCORD_TOKEN={token}\n")
        (self.root / "config/config.toml").write_text(f"[commands]\nmode = {mode}\ndevelopment_guilds = {guilds}\n")

    def test_missing_docker_creates_nothing(self):
        status, output = self.run_setup(docker=False)
        self.assertEqual(status, 1)
        self.assertIn("Install Docker", output)
        self.assertFalse((self.root / ".env").exists())

    def test_missing_compose_creates_nothing(self):
        self.assertEqual(self.run_setup(compose_code=1)[0], 1)
        self.assertFalse((self.root / ".env").exists())

    def test_first_run_identifies_both_required_edits(self):
        status, output = self.run_setup()
        self.assertEqual(status, 1)
        self.assertIn("placeholder", output)
        self.assertIn("Guild mode needs", output)
        self.assertNotIn("replace-with-your-discord-bot-token", output)

    def test_preserves_private_files_and_accepts_global(self):
        self.configure()
        before = (self.root / ".env").read_bytes()
        status, output = self.run_setup()
        self.assertEqual(status, 0)
        self.assertEqual((self.root / ".env").read_bytes(), before)
        self.assertNotIn("private-test-value", output)

    def test_guild_ids_validation(self):
        for guilds in ("[]", "[0]", "[true]", '["123"]', "[18446744073709551616]"):
            self.configure(mode='"guild"', guilds=guilds)
            self.assertEqual(self.run_setup()[0], 1)
        self.configure(mode='"guild"', guilds="[123456789012345678]")
        self.assertEqual(self.run_setup()[0], 0)

    def test_malformed_toml_does_not_echo_content(self):
        self.configure()
        (self.root / "config/config.toml").write_text('secret-private-value = "unterminated')
        status, output = self.run_setup()
        self.assertEqual(status, 1)
        self.assertNotIn("secret-private-value", output)

    def test_empty_token_and_invalid_mode(self):
        self.configure(token="", mode='"invalid"')
        status, output = self.run_setup()
        self.assertEqual(status, 1)
        self.assertIn("DISCORD_TOKEN", output)
        self.assertIn("commands.mode", output)


if __name__ == "__main__":
    unittest.main()
