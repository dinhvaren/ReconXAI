"""Run real schema-16 and schema-17 CLIs against an isolated benign workspace.

Preserves the fixture, migration backup and metadata-only proof. Never opens the
normal workspace or executes the imported sample; no AI/network/credential use.
"""
import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import platform
import sqlite3
import subprocess


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def inspect(path):
    with sqlite3.connect(f"{path.as_uri()}?mode=ro", uri=True) as db:
        assert db.execute("PRAGMA integrity_check").fetchone() == ("ok",)
        assert not db.execute("PRAGMA foreign_key_check").fetchall()
        version = db.execute("PRAGMA user_version").fetchone()[0]
        names = [r[0] for r in db.execute("SELECT name FROM sqlite_master WHERE type='table'") if not r[0].startswith("sqlite_")]
        rows = {name: db.execute('SELECT * FROM "' + name.replace('"', '""') + '"').fetchall() for name in names}
        return version, rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("old-binary", "binary", "sample", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    old, current, sample = [p.resolve(strict=True) for p in (args.old_binary, args.binary, args.sample)]
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    env = {**os.environ, "RECONX_DATA_DIR": str(output / "workspace")}
    calls = []

    def command(binary, *arguments, fail=False):
        result = subprocess.run([str(binary), "--json", *map(str, arguments)], env=env,
                                capture_output=True, text=True, encoding="utf-8", timeout=90)
        calls.append({"binary_sha256": digest(binary), "arguments": list(map(str, arguments)),
                      "exit_code": result.returncode, "timestamp": dt.datetime.now(dt.timezone.utc).isoformat()})
        (output / "commands.json").write_text(json.dumps(calls, indent=2), encoding="utf-8")
        if fail:
            assert result.returncode and "SCHEMA_TOO_NEW" in result.stderr + result.stdout
            return
        if result.returncode:
            raise RuntimeError(result.stderr[:1000])
        return json.loads(result.stdout)

    project = command(old, "project", "create", "Research migration fixture")
    imported = command(old, "reverse", "import", sample, "--project", project["id"], "--authorized")
    observed = command(old, "reverse", "analyze", imported["id"])
    queued = command(old, "jobs", "enqueue", "--kind", "artifact_index", "--file", imported["id"])
    database = output / "workspace/reconx.db"
    version, before = inspect(database)
    assert version == 16, f"Expected real old schema 16, got {version}"
    assert observed["analysis"]["sha256"] == digest(sample)
    assert command(current, "workspace", "show")["target_id"] is None
    version, after = inspect(database)
    assert version == 17
    for table, rows in before.items():
        assert all(row in after[table] for row in rows), f"Historical rows changed: {table}"
    backups = list(database.parent.glob("reconx.pre-v17-*.bak"))
    assert len(backups) == 1
    backup_version, backup_rows = inspect(backups[0])
    assert backup_version == 16
    assert before == backup_rows
    command(current, "workspace", "select", "--project", project["id"], "--target", imported["target_id"])
    command(current, "jobs", "run")
    assert command(current, "jobs", "show", queued["id"])["status"] == "completed"
    evidence = command(current, "evidence", "list", "--project", project["id"])
    assert len(evidence) >= 2
    for record in evidence:
        command(current, "evidence", "verify", record["id"])
    assert command(current, "reverse", "metadata", imported["id"])["id"] == observed["id"]
    assert command(current, "workspace", "show")["target_id"] == imported["target_id"]
    # Older clients must reject the newer database without changing its bytes.
    before_rejection = digest(database)
    command(old, "project", "list", fail=True)
    assert digest(database) == before_rejection
    result = {"status": "passed", "platform": platform.platform(), "timestamp": dt.datetime.now(dt.timezone.utc).isoformat(),
              "command_count": len(calls), "old_binary_sha256": digest(old), "binary_sha256": digest(current),
              "checks": ["Real schema-16 records and queued job preserved by schema-17 upgrade; exact valid pre-upgrade backup",
                         "New CLI drains the old queued job and verifies retained/created evidence, metadata and selected IDs across processes",
                         "Old CLI refuses schema-17 without changing database bytes"],
              "preserved_table_row_counts": {name: len(rows) for name, rows in before.items()},
              "backup_sha256": digest(backups[0]), "normal_user_data": "not accessed"}
    (output / "results.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
