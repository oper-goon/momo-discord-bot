"""Offline mixed-provider DAVE wire interoperability, using two distinct binaries.

python check_interop.py --old PATH_TO_OLD_EXE --new PATH_TO_NEW_EXE
Build the old helper from tests/voice-interop/legacy without provider patches.
All keys are ephemeral; no Discord credentials, sockets or production data.
"""
import argparse
import json
import queue
import subprocess
import threading


class Peer:
    def __init__(self, executable, user, expected_name):
        self.user = user
        self.expected_name = expected_name
        self.process = subprocess.Popen(
            [str(executable)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            text=True, encoding="utf-8", bufsize=1,
        )
        self.lines = queue.Queue()
        self.reader = threading.Thread(target=self._read, daemon=True)
        self.reader.start()

    def _read(self):
        for line in self.process.stdout:
            self.lines.put(line)
        self.lines.put(None)

    def rpc(self, op, expect_error=False, **arguments):
        self.process.stdin.write(json.dumps({"op": op, **arguments}) + "\n")
        self.process.stdin.flush()
        line = self.lines.get(timeout=30)
        assert line is not None, f"{self.expected_name} exited unexpectedly"
        response = json.loads(line)
        if expect_error:
            assert "error" in response, f"{op} unexpectedly succeeded"
            return None
        assert "ok" in response, f"{self.expected_name} {op} failed: {response.get('error')}"
        return response["ok"]

    def setup(self, external):
        output = self.rpc("setup", user=self.user, channel=9001, external=external)
        assert output["helper"] == self.expected_name, "Wrong binary passed to --old/--new"

    def info(self, verify_user=None):
        return self.rpc("info", verify_user=verify_user)

    def close(self):
        if self.process.poll() is None:
            self.process.stdin.close()
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=5)
        self.process.stdout.close()


def same_epoch(peers):
    infos = [peer.info() for peer in peers]
    expected_users = sorted(peer.user for peer in peers)
    for info in infos:
        assert info["ready"] and info["status"] == "ACTIVE"
        assert sorted(info["users"]) == expected_users
        assert info["privacy_code"]
        for field in ["epoch", "authenticator", "privacy_code"]:
            assert info[field] == infos[0][field], f"Mismatch: {field}"
    return infos[0]


def audio(sender, receiver, marker):
    plaintext = [0x78, marker, 0x32, 0x43, 0x54, 0x65, 0x76, 0x87]
    ciphertext = sender.rpc("encrypt", bytes=plaintext)
    assert ciphertext != plaintext and len(ciphertext) > len(plaintext)
    assert receiver.rpc("decrypt", user=sender.user, bytes=ciphertext) == plaintext


def scenario(host_exe, joiner_exe, host_name, joiner_name):
    peers = [Peer(host_exe, 1001, host_name), Peer(joiner_exe, 1002, joiner_name)]
    try:
        host, joiner = peers
        external = host.rpc("delivery")["external"]
        for peer in peers:
            peer.setup(external)
            assert not peer.info()["ready"]
            peer.rpc("encrypt", bytes=[0x78, 1, 2, 3], expect_error=True)
        package = joiner.rpc("keypackage")
        proposal = host.rpc("add", keypackage=package)
        transition = host.rpc("proposals", bytes=proposal, expected=[1001, 1002])
        host.rpc("commit", bytes=transition["commit"])
        joiner.rpc("welcome", bytes=transition["welcome"])
        initial = same_epoch(peers)
        assert host.info(1002)["verification"] == joiner.info(1001)["verification"]
        audio(host, joiner, 1)
        audio(joiner, host, 2)

        # Same backend as host, so the existing joiner must verify a remote
        # commit and refresh its media ratchet against the other backend.
        third = Peer(host_exe, 1003, host_name)
        peers.append(third)
        third.setup(external)
        proposal = host.rpc("add", keypackage=third.rpc("keypackage"))
        transition = host.rpc("proposals", bytes=proposal, expected=[1001, 1002, 1003])
        joiner.rpc("proposals", bytes=proposal, expected=[1001, 1002, 1003])
        host.rpc("commit", bytes=transition["commit"])
        joiner.rpc("commit", bytes=transition["commit"])
        third.rpc("welcome", bytes=transition["welcome"])
        rotated = same_epoch(peers)
        assert rotated["epoch"] > initial["epoch"]
        assert rotated["authenticator"] != initial["authenticator"]
        audio(host, joiner, 3)
        audio(joiner, host, 4)
        audio(third, joiner, 5)
        audio(joiner, third, 6)

        tampered = host.rpc("encrypt", bytes=[0x78, 7, 8, 9])
        tampered[1] ^= 0x40
        joiner.rpc("decrypt", user=host.user, bytes=tampered, expect_error=True)
        return {"host": host_name, "joiner": joiner_name, "peers": 3,
                "initial_epoch": initial["epoch"], "rotated_epoch": rotated["epoch"],
                "audio_roundtrips": 6, "tamper_rejected": True}
    finally:
        for peer in peers:
            peer.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--old", required=True)
    parser.add_argument("--new", required=True)
    args = parser.parse_args()
    results = [
        scenario(args.old, args.new, "dave-interop-old", "dave-interop-new"),
        scenario(args.new, args.old, "dave-interop-new", "dave-interop-old"),
    ]
    print(json.dumps({"passed": True, "scenarios": results,
                      "scope": "MLS/DAVE serialized handshake and packet crypto; no live Discord/RTP/Opus decode"}, indent=2))


if __name__ == "__main__":
    main()
