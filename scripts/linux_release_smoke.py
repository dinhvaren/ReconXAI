"""Exercise an explicit Linux CLI build using an isolated benign ELF fixture.

No network, AI, credentials, package installation or sample execution. Generated
workspaces are retained for inspection; existing output directories are refused.
"""
import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    if platform.system() != "Linux":
        raise SystemExit("This acceptance test requires a real Linux process environment")
    binary = args.binary.resolve(strict=True)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    workspace = Path(tempfile.mkdtemp(prefix="reconx-linux-release-"))
    os.chmod(workspace, 0o700)
    database = workspace / "reconx.db"
    calls = []

    def command(*arguments):
        result = subprocess.run([str(binary), "--json", "--database", str(database), *map(str, arguments)], capture_output=True, text=True, timeout=90)
        calls.append({"arguments": list(map(str, arguments)), "exit_code": result.returncode})
        if result.returncode:
            raise RuntimeError("CLI fixture operation failed: " + result.stderr[:2000])
        return json.loads(result.stdout) if result.stdout.strip() else None

    def data(name, value):
        path = workspace / name
        path.write_text(json.dumps(value), encoding="utf-8")
        return path

    project = command("project", "create", "Linux Release Demo")
    sample = Path("/usr/bin/true").resolve(strict=True)
    source_hash = hashlib.sha256(sample.read_bytes()).hexdigest()
    imported = command("reverse", "import", sample, "--project", project["id"], "--authorized")
    assert imported["sha256"] == source_hash
    analysis = command("reverse", "analyze", imported["id"])
    assert analysis["analysis"]["format"].lower().startswith("elf")
    assert analysis["analysis"]["architecture"]
    assert analysis["analysis"]["sections"]
    strings = command("reverse", "strings", imported["id"])
    assert strings
    jobs = command("jobs", "list", "--project", project["id"])
    job = next(job for job in jobs if job["id"] == analysis["job_id"])
    assert job["status"] == "completed"
    evidence = command("evidence", "from-job", job["id"])
    command("evidence", "verify", evidence["id"])
    finding = command("findings", "create", "--input", data("finding.json", {
        "project_id": project["id"], "title": "Benign ELF metadata acceptance observation",
        "description": "Synthetic release test record, not a vulnerability claim.",
        "severity": "informational", "status": "needs_verification", "evidence_ids": [evidence["id"]],
    }))
    view = command("report", "prepare", "--input", data("report.json", {
        "project_id": project["id"], "title": "Linux CLI release acceptance",
        "template": "reverse_engineering", "summary": "Kiểm tra Linux cục bộ\nAuthorization: Bearer synthetic-linux-release-secret",
        "finding_ids": [finding["id"]], "evidence_ids": [evidence["id"]],
    }))
    assert "synthetic-linux-release-secret" not in json.dumps(view)
    exports = []
    for format, extension in [("markdown", "md"), ("json", "json"), ("html", "html"), ("pdf", "pdf")]:
        path = output / ("report." + extension)
        command("report", "export", view["report"]["id"], "--format", format, "--output", path, "--reviewed-sha256", view["report"]["sha256"])
        blob = path.read_bytes()
        assert blob and b"synthetic-linux-release-secret" not in blob
        if format == "pdf":
            assert blob.startswith(b"%PDF-")
        elif format == "json":
            json.loads(blob)
        exports.append({"format": format, "path": path.name, "sha256": hashlib.sha256(blob).hexdigest(), "bytes": len(blob)})
    # Every invocation starts a separate CLI process and opens the persisted DB.
    assert command("reverse", "metadata", imported["id"])["id"] == analysis["id"]
    assert any(item["id"] == finding["id"] for item in command("findings", "list", "--project", project["id"]))
    command("report", "show", view["report"]["id"])
    result = {
        "timestamp": dt.datetime.now(dt.timezone.utc).isoformat(), "status": "passed",
        "platform": "linux-" + platform.machine(), "kernel_release": platform.release(),
        "cli_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "workspace": str(workspace), "source_sha256": source_hash,
        "format": analysis["analysis"]["format"], "architecture": analysis["analysis"]["architecture"],
        "sections": len(analysis["analysis"]["sections"]), "strings": len(strings),
        "job_id": job["id"], "job_status": job["status"], "evidence_id": evidence["id"],
        "finding_status": "needs_verification", "separate_process_persistence_checked": True,
        "exports": exports, "calls": calls, "linux_desktop_verified": False,
        "secret_service_verified": False,
    }
    (output / "acceptance.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
