#!/usr/bin/env python3
"""Prepare private local files and check required settings (Python 3.11+)."""
import os
import pathlib
import shutil
import subprocess
import sys
import tomllib


def configuration_errors(root):
    """Return safe diagnostics, never file contents or parser exceptions."""
    errors = []
    token = ""
    for line in (root / ".env").read_text(encoding="utf-8").splitlines():
        name, separator, value = line.strip().partition("=")
        if separator and name.strip() == "DISCORD_TOKEN":
            token = value.strip().strip("\"'").strip()
    if not token or token.startswith("#") or token.lower().startswith(("replace-", "your-", "your_", "placeholder")):
        errors.append("Edit .env: DISCORD_TOKEN is missing, empty, or still a placeholder.")
    elif "${" in token:
        errors.append("Edit .env: use a literal private DISCORD_TOKEN; this helper does not evaluate variable substitutions.")
    try:
        with (root / "config/config.toml").open("rb") as handle:
            config = tomllib.load(handle)
    except tomllib.TOMLDecodeError:
        errors.append("Fix invalid TOML in config/config.toml; parser details are hidden to protect private values.")
        return errors
    commands = config.get("commands", {})
    if not isinstance(commands, dict):
        errors.append("config/config.toml must contain a [commands] table.")
        return errors
    mode = commands.get("mode", "none")
    guilds = commands.get("development_guilds", [])
    if mode not in ("none", "guild", "global"):
        errors.append('commands.mode must be "none", "guild", or "global".')
    if not isinstance(guilds, list) or any(type(value) is not int or not 0 < value <= 2**64 - 1 for value in guilds):
        errors.append("commands.development_guilds must be a list of positive Discord server IDs.")
    elif mode == "guild" and not guilds:
        errors.append('Guild mode needs commands.development_guilds; set your server ID or choose mode = "global".')
    if mode == "none":
        print('Notice: commands.mode = "none" is valid but does not register commands; first installations should choose guild or global.')
    return errors


def main(root=None):
    root = pathlib.Path(root) if root is not None else pathlib.Path(__file__).resolve().parent.parent
    if shutil.which("docker") is None:
        print("Install Docker with the Compose plugin, then rerun this script.")
        return 1
    result = subprocess.run(["docker", "compose", "version"], capture_output=True)
    if result.returncode:
        print("Docker Compose is unavailable. Install the Compose plugin.")
        return 1
    try:
        for source, target in [(".env.example", ".env"), ("config/example.toml", "config/config.toml")]:
            destination = root / target
            if not destination.exists():
                if target == ".env":
                    with open(destination, "x", encoding="utf-8", opener=lambda path, flags: os.open(path, flags, 0o600)) as handle:
                        handle.write((root / source).read_text(encoding="utf-8"))
                else:
                    shutil.copyfile(root / source, destination)
                print(f"Created {target}.")
            else:
                print(f"Preserved existing {target}.")
        errors = configuration_errors(root)
    except OSError:
        print("Cannot read or create setup files. Check repository templates and local file permissions.")
        return 1
    for error in errors:
        print(error)
    if errors:
        print("Setup is incomplete. Edit the indicated files privately, then rerun this helper.")
        return 1
    print("Required local settings are present. Credentials have not been authenticated.")
    print("Validate privately: docker compose build")
    print("docker compose run --rm --no-deps bot --config /app/config/config.toml --check")
    print("Start: docker compose up -d. Verify: docker compose logs --tail 100 bot")
    print("Setup has not started the bot. See docs/self-hosting/index.md.")
    return 0


if __name__ == "__main__":
    sys.exit(main())


