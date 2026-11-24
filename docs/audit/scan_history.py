"""Read-only heuristic scan. Reports locations, never matching values."""
import json
import re
import subprocess

def git(*args):
    return subprocess.check_output(["git", *args])

patterns = {
    "private-key": rb"-----BEGIN (?:OPENSSH|RSA|EC|DSA|ENCRYPTED)? ?PRIVATE KEY-----",
    "discord-token-shape": rb"(?:mfa\.[A-Za-z0-9_-]{50,}|[A-Za-z0-9_-]{24,28}\.[A-Za-z0-9_-]{6}\.[A-Za-z0-9_-]{27,})",
    "credential-assignment": rb"(?im)^\s*(?:DISCORD_TOKEN|TORBOX_API_KEY|EASYNEWS_PASSWORD|TMDB_API_KEY|TMDB_READ_ACCESS_TOKEN)\s*=\s*[\"']?[^\s\"'#]{12,}",
    "personal-path": rb"(?:C:\\Users\\|/home/)[A-Za-z0-9_.-]+",
}
findings = []
count = 0
for row in git("rev-list", "--objects", "--all").splitlines():
    oid, _, path = row.partition(b" ")
    if git("cat-file", "-t", oid.decode()).strip() != b"blob":
        continue
    count += 1
    data = git("cat-file", "-p", oid.decode())
    for name, pattern in patterns.items():
        if re.search(pattern, data):
            findings.append({"kind": name, "object": oid.decode(), "path": path.decode(errors="replace")})
print(json.dumps({"commits": int(git("rev-list", "--all", "--count")), "blobs": count, "findings": findings}, indent=2))
