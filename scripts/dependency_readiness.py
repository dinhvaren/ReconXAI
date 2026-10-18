"""Assess recorded dependency scans without suppressing warnings or closing alerts.

Inputs are local JSON exports. This tool never reads credentials or Git remotes.
Exit 1 means blocked/incomplete, not a clean audit. It does not certify v1.0.
"""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
from pathlib import Path
import re
import tomllib


def load(path: Path) -> dict:
    data = path.read_bytes()
    return json.loads(data.decode("utf-16" if data.startswith(b"\xff\xfe") else "utf-8-sig"))


def numeric_version(value: str) -> tuple[int, ...] | None:
    return tuple(map(int, value.split("."))) if re.fullmatch(r"\d+\.\d+\.\d+", value) else None


def assess(github: dict, npm: dict, cargo: dict, versions: dict[tuple[str, str], list[str]]) -> dict:
    blockers = []
    if github.get("status") != "RETRIEVED":
        blockers.append("GitHub advisory retrieval is incomplete")
    metadata = npm.get("metadata", {}).get("vulnerabilities")
    if npm.get("error") or not isinstance(metadata, dict) or "total" not in metadata:
        blockers.append("npm audit has no successful vulnerability summary")
    elif any(metadata.get(key, 0) for key in ("moderate", "high", "critical")):
        blockers.append("npm has unresolved Moderate/High/Critical findings")
    vulnerabilities = cargo.get("vulnerabilities")
    if not isinstance(vulnerabilities, dict) or "found" not in vulnerabilities:
        blockers.append("cargo audit has no successful vulnerability summary")
    elif vulnerabilities.get("found") or vulnerabilities.get("list"):
        blockers.append("cargo audit reports vulnerabilities")
    warnings = cargo.get("warnings", {})
    if warnings.get("unsound"):
        blockers.append("cargo audit reports unsound dependency code (not suppressed)")

    alerts = []
    for alert in github.get("alerts", []):
        dependency = alert["dependency"]
        package = dependency["package"]
        installed = versions.get((package["ecosystem"], package["name"]), [])
        patched = (alert.get("first_patched_version") or {}).get("identifier", "")
        patch_number = numeric_version(patched)
        # Conservative: pre-releases, missing entries and unknown formats remain
        # unresolved. The full upstream affected range is retained for review.
        resolved = bool(installed and patch_number and all(
            numeric_version(version) is not None and numeric_version(version) >= patch_number
            for version in installed
        ))
        source_state = "PATCHED_IN_SOURCE" if resolved else "UNRESOLVED"
        if alert.get("state") == "open" and not resolved:
            blockers.append(f"{alert['ghsa_id']}: source dependency remains affected or unknown")
        alerts.append({
            "number": alert["number"], "ghsa_id": alert["ghsa_id"],
            "cve_id": alert.get("cve_id"), "ecosystem": package["ecosystem"],
            "package": package["name"], "manifest": dependency["manifest_path"],
            "severity": alert["severity"], "affected_range": alert.get("vulnerable_version_range"),
            "first_patched_version": patched, "source_versions": installed,
            "source_state": source_state, "remote_state": alert.get("state"),
            "remote_fixed_at": alert.get("fixed_at"), "url": alert.get("url"),
        })
    return {
        "dependency_gate": "BLOCKED" if blockers else "PASSED_RECORDED_CHECKS",
        "release_ready": False,
        "scope": "Recorded dependency checks only; functionality/platform/release gates are separate",
        "blockers": blockers, "alerts": alerts,
        "npm": metadata,
        "cargo_vulnerabilities": vulnerabilities,
        "cargo_warnings": {kind: [{"id": item["advisory"]["id"], "package": item["package"]["name"], "version": item["package"]["version"]} for item in items] for kind, items in warnings.items()},
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--github", type=Path, required=True)
    parser.add_argument("--npm", type=Path, required=True)
    parser.add_argument("--cargo", type=Path, required=True)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    versions: dict[tuple[str, str], list[str]] = {}
    for package in tomllib.loads((args.root / "Cargo.lock").read_text())["package"]:
        versions.setdefault(("rust", package["name"]), []).append(package["version"])
    requirements = (args.root / "tests/e2e/requirements.txt").read_text()
    for name, version in re.findall(r"^([A-Za-z0-9_-]+)==([0-9.]+)\b", requirements, re.MULTILINE):
        versions.setdefault(("pip", name), []).append(version)
    report = assess(load(args.github), load(args.npm), load(args.cargo), versions)
    report["timestamp"] = dt.datetime.now(dt.timezone.utc).isoformat()
    report["input_sha256"] = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in [args.github, args.npm, args.cargo, args.root / "Cargo.lock", args.root / "tests/e2e/requirements.txt"]}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"dependency_gate": report["dependency_gate"], "blockers": report["blockers"], "output": str(args.output)}, indent=2))
    return int(bool(report["blockers"]))


if __name__ == "__main__":
    raise SystemExit(main())
