"""Actual CLI extension lifecycle in a new synthetic workspace; never uses user data.

--approve-install authorizes this fixture's pinned portable YARA-X dependency only.
No active target traffic, AI provider or credential storage is used.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sqlite3
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--legacy-binary", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--approve-install", action="store_true")
    args = parser.parse_args()
    if not args.approve_install:
        parser.error("Review manifests/tools/yara-x.json and explicitly pass --approve-install for the isolated portable dependency test")
    binary = args.binary.resolve(strict=True)
    legacy = args.legacy_binary.resolve(strict=True) if args.legacy_binary else binary
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    database = output / "workspace.db"
    calls = []

    def command(*arguments, engine=binary, error=None):
        result = subprocess.run([str(engine), "--json", "--database", str(database), *map(str, arguments)], capture_output=True, text=True, encoding="utf-8", timeout=180)
        calls.append({"arguments": list(map(str, arguments)), "exit_code": result.returncode})
        if error:
            assert result.returncode != 0 and error in result.stderr, result.stderr
            return
        assert result.returncode == 0, result.stderr
        return json.loads(result.stdout) if result.stdout.strip() else None

    identifier = "reconx.yara-signatures"
    project = command("project", "create", "Owned extension fixture", "--description", "No cloud or active research", engine=legacy)
    sample = output / "owned.bin"
    sample.write_bytes(b"RECONX_BENIGN_FIXTURE\x00owned static research")
    file = command("reverse", "import", sample, "--project", project["id"], "--authorized", engine=legacy)
    queued = command("jobs", "enqueue", "--kind", "artifact_index", "--file", file["id"], engine=legacy)
    before_version = sqlite3.connect(database).execute("PRAGMA user_version").fetchone()[0]
    assert command("extensions", "search", "yara")[0]["status"] == "not_installed"
    upgrade = {"source_version": before_version}
    if legacy != binary:
        backups = list(output.glob("workspace.pre-v16-*.bak"))
        assert len(backups) == 1
        with sqlite3.connect(backups[0]) as backup:
            assert backup.execute("PRAGMA user_version").fetchone()[0] == before_version
            assert backup.execute("PRAGMA integrity_check").fetchone()[0] == "ok"
            assert backup.execute("SELECT status FROM analysis_jobs WHERE id=?", [queued["id"]]).fetchone()[0] == "queued"
        upgrade["backup"] = backups[0].name
    command("jobs", "run")
    assert command("jobs", "show", queued["id"])["status"] == "completed"
    command("extensions", "install", identifier, "--fingerprint", "unapproved", error="APPROVAL_REQUIRED")
    command("extensions", "install", identifier, "--fingerprint", "stale", "--approve", error="PLAN_CHANGED")
    plan = command("extensions", "plan", identifier)
    assert plan["requires_admin"] is False
    dependency = plan["extension"]["dependencies"][0]["manifest"]
    assert dependency["url"].startswith("https://github.com/") and len(dependency["sha256"]) == 64
    (output / "approved-plan.json").write_text(json.dumps(plan, indent=2), encoding="utf-8")
    installed = command("extensions", "install", identifier, "--fingerprint", plan["fingerprint"], "--approve")
    assert installed["status"] == "ready", installed
    assert installed["installed"]["last_health_check"]
    assert command("extensions", "doctor", identifier)["status"] == "ready"
    command("tools", "remove", "yara-x", error="TOOL_REQUIRED_BY_EXTENSION")
    job = command("extensions", "invoke", identifier, "--file", file["id"])
    command("jobs", "run")
    completed = command("jobs", "show", job["id"])
    assert completed["status"] == "completed", completed
    assert "ReconX_Test_Marker" in json.dumps(completed["result"]["observations"])
    assert completed["result"]["sample_sha256"] == hashlib.sha256(sample.read_bytes()).hexdigest()
    assert any(e["job_id"] == job["id"] for e in command("evidence", "list", "--project", project["id"]))
    queued = command("extensions", "invoke", identifier, "--file", file["id"])
    command("extensions", "disable", identifier)
    assert command("jobs", "show", queued["id"])["status"] == "cancelled"
    command("extensions", "invoke", identifier, "--file", file["id"], error="EXTENSION_NOT_READY")
    command("extensions", "enable", identifier)
    command("extensions", "update", identifier, error="NO_EXTENSION_UPDATE")
    command("extensions", "remove", identifier)
    assert command("extensions", "list") == []
    assert len(command("extensions", "history", identifier)) == 2
    assert next(t for t in command("tools", "list") if t["manifest"]["id"] == "yara-x")["status"] == "tested"
    plan = command("extensions", "plan", identifier)
    assert "Reuse and verify" in plan["actions"][0]
    assert command("extensions", "install", identifier, "--fingerprint", plan["fingerprint"], "--approve")["status"] == "ready"
    with sqlite3.connect(database) as conn:
        assert conn.execute("PRAGMA user_version").fetchone()[0] == 16
        assert conn.execute("PRAGMA integrity_check").fetchone()[0] == "ok"
        assert conn.execute("PRAGMA foreign_key_check").fetchall() == []
    result = {"status": "passed", "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(), "upgrade": upgrade, "job": job["id"], "calls": calls,
              "scope": "Actual portable YARA-X and local CLI processes in a synthetic workspace. No Linux Desktop, OS sandbox, community runtime or update success claimed."}
    (output / "results.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(json.dumps({"output": str(output), "status": "passed", "commands": len(calls)}))


if __name__ == "__main__":
    main()
