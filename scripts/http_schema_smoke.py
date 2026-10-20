"""Verify offline schema jobs, provenance, persistence and four report formats.

Uses synthetic captures and a new isolated output directory. No network, AI,
credential store, imported script or build script is invoked. Each CLI command
starts a separate process. Existing output directories are refused.
"""
import argparse
import datetime as dt
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import sqlite3


def main() -> None:
    root = Path(__file__).resolve().parent.parent
    parser = argparse.ArgumentParser(description=__doc__)
    suffix = ".exe" if platform.system() == "Windows" else ""
    parser.add_argument("--binary", type=Path, default=root / "target/release" / ("reconx" + suffix))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--legacy-binary", type=Path, help="Optional verified schema-12 CLI to exercise a real queued-job upgrade")
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    source_binary = args.legacy_binary.resolve(strict=True) if args.legacy_binary else binary
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    database = output / "workspace.db"
    calls = []

    def command(*arguments, engine=binary):
        result = subprocess.run(
            [str(engine), "--json", "--database", str(database), *map(str, arguments)],
            capture_output=True, text=True, encoding="utf-8", timeout=90,
        )
        calls.append({"arguments": list(map(str, arguments)), "exit_code": result.returncode})
        if result.returncode:
            raise RuntimeError("Offline CLI acceptance failed: " + result.stderr[:2000])
        return json.loads(result.stdout) if result.stdout.strip() else None

    def data(name, value):
        path = output / name
        path.write_text(json.dumps(value), encoding="utf-8")
        return path

    fixtures = root / "crates/reconx-core/tests/fixtures/http"
    har = json.loads((fixtures / "schema-mismatch.har").read_text(encoding="utf-8"))
    har["log"]["entries"][0]["response"]["content"] = {
        "mimeType": "application/json",
        "text": json.dumps({"id": 1, "owner": "alice", "published": "2026-02-30", "samples": [1, 1.0, 7], "labels": {"x-name": 1, "other": True}, "token": "synthetic-schema-cli-secret"}),
    }
    spec = json.loads((fixtures / "openapi30.json").read_text(encoding="utf-8"))
    spec["openapi"] = "3.1.1"
    user = spec["components"]["schemas"]["User"]
    user["dependentRequired"] = {"id": ["reviewMarker"]}
    user["dependentSchemas"] = {"id": {"properties": {"owner": {"minLength": 99}}}}
    user["if"] = {"required": ["id"], "properties": {"id": {"const": 1}}}
    user["then"] = {"required": ["conditionalMarker"]}
    user["else"] = False
    user["properties"]["owner"]["pattern"] = "^team-"
    user["properties"]["published"] = {"type": "string", "format": "date"}
    user["properties"]["labels"] = {
        "patternProperties": {"^x-": {"type": "string"}}, "additionalProperties": {"type": "boolean"},
    }
    user["properties"]["samples"] = {
        "type": "array", "uniqueItems": True,
        "prefixItems": [{"type": "integer"}, {"type": "boolean"}], "items": False,
    }
    project = command("project", "create", "Offline schema acceptance", engine=source_binary)
    command("web", "import-har", data("source.har", har), "--project", project["id"], "--identity", "synthetic", engine=source_binary)
    command("api", "import-openapi", data("openapi.json", spec), "--project", project["id"], engine=source_binary)
    workspace = command("web", "traffic", "--project", project["id"], engine=source_binary)
    legacy_job = command("api", "schema-check", workspace["traffic"][0]["id"], workspace["operations"][0]["id"], engine=source_binary)
    if args.legacy_binary:
        with sqlite3.connect(database) as inspection:
            assert inspection.execute("PRAGMA user_version").fetchone()[0] == 12
    command("jobs", "run")
    default_job = next(item for item in command("jobs", "list", "--project", project["id"]) if item["id"] == legacy_job["id"])
    assert default_job["status"] == "completed"
    assert default_job["result"]["options"]["assert_formats"] is False
    assert any(o["rule"] == "format_annotation" for o in default_job["result"]["observations"])
    assert not any(o["rule"] == "string_format" for o in default_job["result"]["observations"])
    upgrade = {"legacy_binary_used": bool(args.legacy_binary), "default_job_id": legacy_job["id"]}
    if args.legacy_binary:
        backups = list(output.glob("workspace.pre-v16-*.bak"))
        assert len(backups) == 1
        with sqlite3.connect(backups[0]) as backup:
            assert backup.execute("PRAGMA user_version").fetchone()[0] == 12
            assert backup.execute("SELECT status FROM analysis_jobs WHERE id=?", [legacy_job["id"]]).fetchone()[0] == "queued"
            assert backup.execute("PRAGMA integrity_check").fetchone()[0] == "ok"
        with sqlite3.connect(database) as upgraded:
            assert upgraded.execute("PRAGMA user_version").fetchone()[0] == 16
            assert upgraded.execute("PRAGMA foreign_key_check").fetchall() == []
        upgrade["backup"] = backups[0].name
        upgrade["legacy_sha256"] = hashlib.sha256(source_binary.read_bytes()).hexdigest()
    job = command("api", "schema-check", workspace["traffic"][0]["id"], workspace["operations"][0]["id"], "--assert-formats")
    command("jobs", "run")
    completed = next(item for item in command("jobs", "list", "--project", project["id"]) if item["id"] == job["id"])
    assert completed["status"] == "completed", completed
    result = completed["result"]
    assert result["options"]["assert_formats"] is True
    assert result["network_requests"] == 0
    assert result["checks"] <= 10_000
    assert "synthetic-schema-cli-secret" not in json.dumps(result)
    for rule, location, instance in [
        ("dependent_required", "dependentRequired/id", "/reviewMarker"),
        ("schema_constraint", "dependentSchemas/id/properties/owner/minLength", "/owner"),
        ("schema_constraint", "properties/samples/uniqueItems", "/samples"),
        ("json_type", "properties/samples/prefixItems/1/type", "/samples/1"),
        ("false_schema", "properties/samples/items", "/samples/2"),
        ("string_pattern", "properties/owner/pattern", "/owner"),
        ("json_type", "properties/labels/patternProperties/^x-/type", "/labels/x-name"),
        ("string_format", "properties/published/format", "/published"),
        ("required_field", "then/required", "/conditionalMarker"),
    ]:
        observation = next(o for o in result["observations"] if o["rule"] == rule and o["openapi_pointer"] == "/components/schemas/User/" + location)
        assert observation["classification"] == "confirmed_documented_mismatch"
        assert observation["instance_pointer"] == instance
        assert observation["capture_pointer"] == "/log/entries/0/response/content/text"
        command("evidence", "verify", observation["capture_evidence_id"])
        command("evidence", "verify", observation["openapi_evidence_id"])
    assert command("findings", "list", "--project", project["id"]) == []
    evidence = command("evidence", "from-job", job["id"])
    command("evidence", "verify", evidence["id"])
    finding = command("findings", "create", "--input", data("finding.json", {
        "project_id": project["id"], "title": "Synthetic captured schema mismatch",
        "description": "Acceptance fixture only. Documentation inconsistency does not establish a security vulnerability.",
        "severity": "informational", "status": "needs_verification", "evidence_ids": [evidence["id"]],
    }))
    report = command("report", "prepare", "--input", data("report-input.json", {
        "project_id": project["id"], "title": "Offline schema acceptance report", "template": "research",
        "summary": "Synthetic observations only.\nAuthorization: Bearer synthetic-schema-report-secret",
        "finding_ids": [finding["id"]], "evidence_ids": [evidence["id"]],
    }))
    assert "synthetic-schema-report-secret" not in json.dumps(report)
    exports = []
    for fmt, extension in [("markdown", "md"), ("json", "json"), ("html", "html"), ("pdf", "pdf")]:
        path = output / ("report." + extension)
        command("report", "export", report["report"]["id"], "--format", fmt, "--output", path, "--reviewed-sha256", report["report"]["sha256"])
        blob = path.read_bytes()
        assert blob and b"synthetic-schema-report-secret" not in blob
        if fmt == "pdf":
            assert blob.startswith(b"%PDF-")
        elif fmt == "json":
            json.loads(blob)
        exports.append({"format": fmt, "path": path.name, "sha256": hashlib.sha256(blob).hexdigest(), "bytes": len(blob)})
    restored = next(item for item in command("jobs", "list", "--project", project["id"]) if item["id"] == job["id"])
    assert restored["result"] == result
    command("evidence", "verify", evidence["id"])
    data("schema-job.json", restored)
    acceptance = {
        "timestamp": dt.datetime.now(dt.timezone.utc).isoformat(), "status": "passed",
        "platform": platform.system() + "-" + platform.machine(), "cli_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "job_id": job["id"], "network_requests": 0, "separate_process_persistence_checked": True,
        "finding_status": "needs_verification", "exports": exports, "calls": calls,
        "format_policy": "default annotation plus separately opted-in assertion", "upgrade": upgrade,
        "scope": "Offline synthetic capture/schema, real jobs/evidence/report exports; no vulnerability or full-release claim",
    }
    data("acceptance.json", acceptance)
    print(json.dumps({"status": "passed", "output": str(output), "job_id": job["id"]}))


if __name__ == "__main__":
    main()
