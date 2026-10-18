use serde_json::Value;
use std::process::Command;
#[test]
fn offline_http_import_inventory_and_compare_use_the_same_core_records() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("http.db");
    let core = reconx_core::Core::open(Some(db.clone())).unwrap();
    let project = core.create_project("CLI traffic fixture", "").unwrap().id;
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/reconx-core/tests/fixtures/http");
    for name in ["alice.har", "bob.har"] {
        let file = fixture.join(name);
        let r = run(
            &db,
            &[
                "web",
                "import-har",
                file.to_str().unwrap(),
                "--project",
                &project,
                "--identity",
                name,
            ],
        );
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    }
    let spec = fixture.join("openapi31.yaml");
    let r = run(
        &db,
        &[
            "api",
            "import-openapi",
            spec.to_str().unwrap(),
            "--project",
            &project,
        ],
    );
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let out = run(&db, &["web", "traffic", "--project", &project]);
    assert!(out.status.success());
    let ws: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(ws["total"], 3);
    assert_eq!(ws["endpoints"][0]["documented"], true);
    let schema = run(
        &db,
        &[
            "api",
            "schema-check",
            ws["traffic"][0]["id"].as_str().unwrap(),
            ws["operations"][0]["id"].as_str().unwrap(),
        ],
    );
    assert!(
        schema.status.success(),
        "{}",
        String::from_utf8_lossy(&schema.stderr)
    );
    let schema: Value = serde_json::from_slice(&schema.stdout).unwrap();
    let executed = run(&db, &["jobs", "run"]);
    assert!(
        executed.status.success(),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    let shared = core.job(schema["id"].as_str().unwrap()).unwrap();
    assert_eq!(shared.status, "completed");
    assert_eq!(shared.result.unwrap()["network_requests"], 0);
    let matrix = run(&db, &["api", "authz-matrix", "--project", &project]);
    assert!(matrix.status.success());
    let matrix: Value = serde_json::from_slice(&matrix.stdout).unwrap();
    assert_eq!(matrix["authorization_matrix"].as_array().unwrap().len(), 2);
    assert_eq!(matrix["authorization_matrix"][0]["capture_count"], 2);
    let first = ws["traffic"][0]["id"].as_str().unwrap();
    let second = ws["traffic"][1]["id"].as_str().unwrap();
    let out = run(
        &db,
        &["api", "diff", first, second, "--owner-field", "owner"],
    );
    assert!(out.status.success());
    let comparison: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(comparison["network_requests"], 0);
    assert_eq!(core.http_exchange(first).unwrap().id, first);
    assert_eq!(
        core.http_workspace(&project, None, "", 0)
            .unwrap()
            .comparisons
            .len(),
        1
    );
    let out = run(&db, &["api", "finding", comparison["id"].as_str().unwrap()]);
    assert!(out.status.success());
    assert_eq!(
        core.findings(Some(&project)).unwrap()[0].input.status,
        reconx_core::FindingStatus::NeedsVerification
    );
    let malformed = dir.path().join("malformed.har");
    std::fs::write(&malformed, "not JSON").unwrap();
    let out = run(
        &db,
        &[
            "web",
            "import-har",
            malformed.to_str().unwrap(),
            "--project",
            &project,
        ],
    );
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("INVALID_HTTP_IMPORT"));
}
#[test]
fn ai_catalog_keyless_profiles_and_agent_routing_share_the_desktop_database() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("ai.db");
    let catalog = run(&db, &["ai", "catalog"]);
    assert!(catalog.status.success());
    let catalog: Value = serde_json::from_slice(&catalog.stdout).unwrap();
    assert_eq!(catalog.as_array().unwrap().len(), 24);
    let result = run(
        &db,
        &[
            "ai",
            "providers",
            "add",
            "--vendor",
            "lmstudio",
            "--name",
            "Private model",
            "--model",
            "installed-fixture",
            "--default",
        ],
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let config: Value = serde_json::from_slice(&result.stdout).unwrap();
    let id = config["id"].as_str().unwrap();
    let profiles = run(&db, &["ai", "profiles", "list"]);
    let profiles: Value = serde_json::from_slice(&profiles.stdout).unwrap();
    assert_eq!(profiles[0]["credential_state"], "not_required");
    assert!(profiles[0]["connection"].is_null());
    let core = reconx_core::Core::open(Some(db.clone())).unwrap();
    assert_eq!(
        core.settings().unwrap().default_provider_id.as_deref(),
        Some(id)
    );
    assert_eq!(
        core.ai_agent_provider("orchestrator").unwrap().model,
        "installed-fixture"
    );
    let routes = dir.path().join("routes.json");
    std::fs::write(
        &routes,
        serde_json::to_vec(&serde_json::json!([{"agent_id":"reverse","provider_id":id}])).unwrap(),
    )
    .unwrap();
    let result = run(
        &db,
        &["ai", "routing", "configure", routes.to_str().unwrap()],
    );
    assert!(result.status.success());
    let result = run(&db, &["ai", "routing", "resolve", "reverse"]);
    assert_eq!(
        serde_json::from_slice::<Value>(&result.stdout).unwrap()["id"],
        id
    );
    let result = run(&db, &["ai", "models", "select", id]);
    assert!(result.status.success());
    let result = run(&db, &["ai", "providers", "remove", id]);
    if result.status.success() {
        assert!(core.ai_routing().unwrap().is_empty());
        assert!(core.settings().unwrap().default_provider_id.is_none());
        assert!(core.provider(id).is_err());
    } else {
        // A headless Linux runner has no Secret Service. Even a keyless
        // profile may have optional stored headers, so deletion must fail
        // closed rather than silently orphaning credentials. Native tests
        // separately cover successful removal with an actual OS store.
        let error: Value = serde_json::from_slice(&result.stderr).unwrap();
        assert_eq!(error["error"]["code"], "SECRET_STORE_UNAVAILABLE");
        assert_eq!(core.provider(id).unwrap().model, "installed-fixture");
        assert_eq!(core.ai_routing().unwrap()[0].provider_id, id);
        assert_eq!(
            core.settings().unwrap().default_provider_id.as_deref(),
            Some(id)
        );
    }
}
#[path = "../../../tests/e2e/native-sample.rs"]
mod native_sample;
fn run(db: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_reconx"))
        .arg("--database")
        .arg(db)
        .arg("--json")
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn native_cli_persists_projects_and_emits_machine_errors() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("cli.db");
    let created = run(&db, &["project", "create", "CLI test"]);
    assert!(created.status.success());
    let value: Value = serde_json::from_slice(&created.stdout).unwrap();
    assert_eq!(value["name"], "CLI test");
    let listed = run(&db, &["project", "list"]);
    assert!(listed.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&listed.stdout).unwrap()[0]["id"],
        value["id"]
    );
    let duplicate = run(&db, &["project", "create", "CLI test"]);
    assert_eq!(duplicate.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&duplicate.stderr).unwrap()["error"]["code"],
        "DUPLICATE_NAME"
    );
    let missing = run(&db, &["project", "open", "missing"]);
    assert_eq!(missing.status.code(), Some(3));
    assert_eq!(
        serde_json::from_slice::<Value>(&missing.stderr).unwrap()["error"]["code"],
        "NOT_FOUND"
    );
}
#[test]
fn native_cli_durable_queue_runs_typed_work_and_cancels_queued_jobs() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("jobs.db");
    assert!(run(&db, &["project", "create", "demo"]).status.success());
    let sample = dir.path().join("owned.raw");
    std::fs::write(&sample, b"real owned fixture strings").unwrap();
    let imported = run(
        &db,
        &[
            "reverse",
            "import",
            sample.to_str().unwrap(),
            "--project",
            "demo",
            "--authorized",
        ],
    );
    assert!(imported.status.success());
    let file: Value = serde_json::from_slice(&imported.stdout).unwrap();
    let id = file["id"].as_str().unwrap();
    let note_path = dir.path().join("note.txt");
    std::fs::write(&note_path, "owned local notes").unwrap();
    let note = run(
        &db,
        &[
            "reverse",
            "note",
            id,
            "--input",
            note_path.to_str().unwrap(),
        ],
    );
    assert!(note.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&note.stdout).unwrap()["note"],
        "owned local notes"
    );
    let queued = run(
        &db,
        &[
            "jobs",
            "enqueue",
            "--kind",
            "strings_extraction",
            "--file",
            id,
        ],
    );
    assert!(queued.status.success());
    let job: Value = serde_json::from_slice(&queued.stdout).unwrap();
    assert_eq!(job["status"], "queued");
    let drain = run(&db, &["jobs", "run", "--concurrency", "1"]);
    assert!(
        drain.status.success(),
        "{}",
        String::from_utf8_lossy(&drain.stderr)
    );
    let result = run(&db, &["jobs", "result", job["id"].as_str().unwrap()]);
    assert!(result.status.success());
    let result: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(result["analysis"]["sha256"], file["sha256"]);
    let queued = run(
        &db,
        &["jobs", "enqueue", "--kind", "artifact_index", "--file", id],
    );
    let job: Value = serde_json::from_slice(&queued.stdout).unwrap();
    let cancelled = run(&db, &["jobs", "cancel", job["id"].as_str().unwrap()]);
    assert!(cancelled.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&cancelled.stdout).unwrap()["status"],
        "cancelled"
    );
    assert!(
        !run(&db, &["jobs", "enqueue", "--kind", "shell", "--file", id])
            .status
            .success()
    );
}
#[test]
fn native_cli_targets_and_reverse_use_shared_core_and_restart_persistence() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("inputs.db");
    assert!(run(&db, &["project", "create", "demo"]).status.success());
    let add = run(
        &db,
        &[
            "target",
            "add",
            "--project",
            "demo",
            "--type",
            "domain",
            "--value",
            "example.test",
            "--authorized",
        ],
    );
    assert!(
        add.status.success(),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );
    let target: Value = serde_json::from_slice(&add.stdout).unwrap();
    let id = target["id"].as_str().unwrap();
    let listed: Value =
        serde_json::from_slice(&run(&db, &["target", "list", "--project", "demo"]).stdout).unwrap();
    assert_eq!(listed[0]["id"], id);
    let scope: Value =
        serde_json::from_slice(&run(&db, &["scope", "validate", id]).stdout).unwrap();
    assert_eq!(scope["status"], "allowed");
    let invalid = run(
        &db,
        &[
            "target",
            "add",
            "--project",
            "demo",
            "--type",
            "ipv4",
            "--value",
            "bad",
        ],
    );
    assert!(!invalid.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&invalid.stderr).unwrap()["error"]["code"],
        "INVALID_TARGET"
    );
    let sample = native_sample::owned_native(dir.path());
    let imported = run(
        &db,
        &[
            "reverse",
            "import",
            sample.to_str().unwrap(),
            "--project",
            "demo",
            "--authorized",
        ],
    );
    assert!(
        imported.status.success(),
        "{}",
        String::from_utf8_lossy(&imported.stderr)
    );
    let f: Value = serde_json::from_slice(&imported.stdout).unwrap();
    let fid = f["id"].as_str().unwrap();
    assert!(run(&db, &["reverse", "analyze", fid]).status.success());
    let metadata: Value =
        serde_json::from_slice(&run(&db, &["reverse", "metadata", fid]).stdout).unwrap();
    assert_eq!(metadata["analysis"]["sha256"], f["sha256"]);
    assert!(
        !metadata["analysis"]["sections"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let jobs: Value = serde_json::from_slice(&run(&db, &["jobs", "list"]).stdout).unwrap();
    assert_eq!(jobs[0]["status"], "completed");
    assert!(run(&db, &["target", "remove", id]).status.success());
}

#[test]
fn evidence_findings_reports_share_core_records_and_require_review() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("outputs.db");
    let invalid = run(&db, &["findings", "cvss", "CVSS:3.1/AV:N"]);
    assert_eq!(invalid.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&invalid.stderr).unwrap()["error"]["code"],
        "INVALID_CVSS"
    );
    let core = reconx_core::Core::open(Some(db.clone())).unwrap();
    let project = core.create_project("output demo", "").unwrap();
    let file = dir.path().join("observed.txt");
    std::fs::write(
        &file,
        "Owned local observation. password=fixture-private-value",
    )
    .unwrap();
    let result = run(
        &db,
        &[
            "evidence",
            "import",
            file.to_str().unwrap(),
            "--project",
            &project.id,
            "--kind",
            "user_note",
        ],
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let evidence: Value = serde_json::from_slice(&result.stdout).unwrap();
    let id = evidence["id"].as_str().unwrap();
    core.evidence_verify(id).unwrap();
    let note = dir.path().join("review.txt");
    std::fs::write(&note, "Manually compared with actual fixture content").unwrap();
    assert!(
        run(
            &db,
            &["evidence", "review", id, "--note", note.to_str().unwrap()]
        )
        .status
        .success()
    );
    let finding_input = dir.path().join("finding.json");
    std::fs::write(
        &finding_input,
        serde_json::to_vec(&reconx_core::FindingInput {
            project_id: project.id.clone(),
            title: "Manual local observation".into(),
            evidence_ids: vec![id.into()],
            ..Default::default()
        })
        .unwrap(),
    )
    .unwrap();
    let result = run(
        &db,
        &[
            "findings",
            "create",
            "--input",
            finding_input.to_str().unwrap(),
        ],
    );
    assert!(result.status.success());
    let finding: Value = serde_json::from_slice(&result.stdout).unwrap();
    let report = core
        .report_prepare(reconx_core::ReportInput {
            project_id: project.id,
            title: "Reviewed local report".into(),
            template: reconx_core::ReportTemplate::Research,
            summary: "Owned fixture only".into(),
            methodology: "Offline observation".into(),
            finding_ids: vec![finding["id"].as_str().unwrap().into()],
            evidence_ids: vec![],
        })
        .unwrap();
    let output = dir.path().join("report.json");
    let result = run(
        &db,
        &[
            "report",
            "export",
            &report.report.id,
            "--format",
            "json",
            "--output",
            output.to_str().unwrap(),
            "--reviewed-sha256",
            &report.report.sha256,
        ],
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let text = std::fs::read_to_string(output).unwrap();
    assert!(!text.contains("fixture-private-value"));
    assert!(text.contains("[REDACTED]"));
    assert_eq!(core.findings(None).unwrap().len(), 1);
}

#[test]
fn cli_exact_local_http_approval_reuses_core_jobs_assets_and_evidence() {
    use reconx_core::{Core, SecurityMode, TargetInput, TargetKind};
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("local-http.db");
    let core = Core::open(Some(db.clone())).unwrap();
    let project = core.create_project("CLI owned local fixture", "").unwrap();
    let mut settings = core.settings().unwrap();
    settings.security_mode = SecurityMode::LabResearch;
    core.save_settings(&settings).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let target = core
        .create_target(TargetInput {
            project_id: project.id.clone(),
            name: "Owned local HTTP".into(),
            kind: TargetKind::Url,
            value: format!("http://{}/", listener.local_addr().unwrap()),
            description: String::new(),
            tags: vec![],
            allowed_scope: vec![],
            excluded_scope: vec![],
            authorized: true,
            mode: SecurityMode::LabResearch,
        })
        .unwrap();
    let plan: Value =
        serde_json::from_slice(&run(&db, &["recon", "preview", &target.id]).stdout).unwrap();
    let fp = plan["fingerprint"].as_str().unwrap();
    let denied = run(&db, &["recon", "run", &target.id, "--fingerprint", fp]);
    assert_eq!(denied.status.code(), Some(4));
    assert!(listener.accept().is_err());
    let queued = run(
        &db,
        &["recon", "run", &target.id, "--fingerprint", fp, "--approve"],
    );
    assert!(
        queued.status.success(),
        "{}",
        String::from_utf8_lossy(&queued.stderr)
    );
    let job: Value = serde_json::from_slice(&queued.stdout).unwrap();
    let worker = thread::spawn(move || {
        let until = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            if let Ok((mut stream, _)) = listener.accept() {
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                    .unwrap();
                let mut buf = [0; 8192];
                let n = stream.read(&mut buf).unwrap();
                assert!(buf[..n].starts_with(b"HEAD / HTTP/1.1"));
                stream.write_all(b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
                break;
            }
            assert!(
                std::time::Instant::now() < until,
                "Expected actual local HEAD"
            );
            thread::sleep(std::time::Duration::from_millis(5));
        }
    });
    let ran = run(&db, &["jobs", "run"]);
    assert!(
        ran.status.success(),
        "{}",
        String::from_utf8_lossy(&ran.stderr)
    );
    worker.join().unwrap();
    assert_eq!(
        core.job(job["id"].as_str().unwrap()).unwrap().status,
        "completed"
    );
    let assets: Value =
        serde_json::from_slice(&run(&db, &["recon", "assets", "--project", &project.id]).stdout)
            .unwrap();
    assert_eq!(assets[0]["facts"]["status"], 204);
    assert_eq!(
        core.assets(None).unwrap()[0].id,
        assets[0]["id"].as_str().unwrap()
    );
    assert_eq!(core.evidence_list(None).unwrap().len(), 2);
}
