use reconx_core::*;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/http")
        .join(name)
}
fn setup() -> (tempfile::TempDir, Core, String) {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::open(Some(dir.path().join("http.db"))).unwrap();
    let project = core.create_project("Offline HTTP fixtures", "").unwrap().id;
    (dir, core, project)
}
fn input(project: &str, identity: &str) -> HttpImportInput {
    HttpImportInput {
        project_id: project.into(),
        target_id: None,
        title: "Offline fixture".into(),
        identity: identity.into(),
        role: if identity == "alice" {
            "reader"
        } else {
            "guest"
        }
        .into(),
    }
}
fn write_json(dir: &Path, name: &str, data: &Value) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, serde_json::to_vec(data).unwrap()).unwrap();
    path
}
#[test]
fn target_associations_partition_inventory_matrix_and_documentation_matches() {
    let (_dir, core, project) = setup();
    let targets: Vec<_> = ["/api/", "/api/users/"]
        .into_iter()
        .map(|path| {
            core.create_target(TargetInput {
                project_id: project.clone(),
                name: format!("Owned {path}"),
                kind: TargetKind::Url,
                value: format!("https://api.example.test{path}"),
                description: String::new(),
                tags: vec![],
                allowed_scope: vec![],
                excluded_scope: vec![],
                authorized: true,
                mode: SecurityMode::PassiveAuto,
            })
            .unwrap()
        })
        .collect();
    for target in &targets {
        let mut imported = input(&project, "alice");
        imported.target_id = Some(target.id.clone());
        core.http_import("har", imported, &fixture("alice.har"))
            .unwrap();
    }
    let before = core.http_workspace(&project, None, "", 0).unwrap();
    assert_eq!(before.endpoints.len(), 2);
    assert_eq!(before.authorization_matrix.len(), 2);
    assert!(
        before
            .authorization_matrix
            .iter()
            .all(|row| row.capture_count == 2)
    );
    let mut spec = input(&project, "");
    spec.target_id = Some(targets[0].id.clone());
    core.http_import("openapi", spec, &fixture("openapi30.json"))
        .unwrap();
    let ws = core.http_workspace(&project, None, "", 0).unwrap();
    assert_eq!(ws.endpoints.len(), 2);
    for endpoint in &ws.endpoints {
        assert_eq!(
            endpoint["documented"],
            endpoint["target_id"] == targets[0].id
        );
    }
    let first = core
        .http_workspace(&project, Some(&targets[0].id), "", 0)
        .unwrap();
    let second = core
        .http_workspace(&project, Some(&targets[1].id), "", 0)
        .unwrap();
    assert_eq!(first.imports.len(), 2);
    assert_eq!(second.imports.len(), 1);
    assert_eq!(first.authorization_matrix.len(), 1);
    assert_eq!(second.authorization_matrix.len(), 1);
    assert_eq!(second.endpoints[0]["documented"], false);
}

#[test]
fn correlation_respects_origins_ports_defaults_and_concrete_path_precedence() {
    let (dir, core, project) = setup();
    let mut har: Value = serde_json::from_slice(&fs::read(fixture("alice.har")).unwrap()).unwrap();
    let entry = har["log"]["entries"][0].clone();
    har["log"]["entries"] = json!(
        [
            "https://api.example.test/api/users/1",
            "https://api.example.test:443/api/users/2",
            "http://api.example.test/api/users/3",
            "https://api.example.test:8443/api/users/4",
            "https://another.example.test/api/users/5",
            "https://api.example.test/api/users/me"
        ]
        .map(|url| {
            let mut e = entry.clone();
            e["request"]["url"] = json!(url);
            e
        })
    );
    core.http_import(
        "har",
        input(&project, "alice"),
        &write_json(dir.path(), "origins.har", &har),
    )
    .unwrap();
    let mut spec: Value =
        serde_json::from_slice(&fs::read(fixture("openapi30.json")).unwrap()).unwrap();
    spec["servers"] = json!([{"url":"{scheme}://{host}:{port}/api/","variables":{
        "scheme":{"default":"https","enum":["http","https"]},
        "host":{"default":"api.example.test"},"port":{"default":"443"}
    }}]);
    spec["paths"]["/users/me"] = json!({"get":{"summary":"Concrete user endpoint","responses":{"200":{"description":"ok"}}}});
    core.http_import(
        "openapi",
        input(&project, ""),
        &write_json(dir.path(), "origins.json", &spec),
    )
    .unwrap();
    let ws = core.http_workspace(&project, None, "", 0).unwrap();
    assert_eq!(ws.endpoints.len(), 5);
    assert_eq!(
        ws.endpoints
            .iter()
            .filter(|e| e["documented"] == true)
            .count(),
        2
    );
    let documented = ws
        .endpoints
        .iter()
        .find(|e| e["path"] == "/users/{userId}")
        .unwrap();
    assert_eq!(documented["requests"], 2);
    assert_eq!(documented["origin"], "https://api.example.test");
    let concrete = ws
        .endpoints
        .iter()
        .find(|e| e["path"] == "/users/me")
        .unwrap();
    assert_eq!(concrete["operation_ids"].as_array().unwrap().len(), 1);
    let capture = ws.traffic.iter().find(|e| e.path.ends_with("/me")).unwrap();
    assert_eq!(capture.request["path_parameters"], json!([]));
    let parameter = ws.traffic.iter().find(|e| e.path.ends_with("/1")).unwrap();
    assert_eq!(parameter.request["path_parameters"], json!(["userId"]));
}

#[test]
fn server_templates_and_workspace_correlation_have_explicit_resource_budgets() {
    let (dir, core, project) = setup();
    let source: Value =
        serde_json::from_slice(&fs::read(fixture("openapi30.json")).unwrap()).unwrap();
    for servers in [
        json!({"url":"/"}),
        json!([{}]),
        json!([{"url":"ftp://example.test"}]),
        json!([{"url":"https://user:password@example.test"}]),
        json!([{"url":"https://example.test?token=x"}]),
        json!([{"url":"//different.example.test/api"}]),
        json!([{"url":"/{missing}"}]),
        json!([{"url":"/{v}","variables":{"v":{"default":"{v}"}}}]),
        json!([{"url":"/{v}","variables":{"v":{"default":"api","enum":["v2"]}}}]),
        json!(vec![json!({"url":"/api"}); 33]),
    ] {
        let mut spec = source.clone();
        spec["servers"] = servers;
        assert!(
            core.http_import(
                "openapi",
                input(&project, ""),
                &write_json(dir.path(), "invalid-server.json", &spec)
            )
            .is_err()
        );
    }
    assert!(core.http_imports(&project).unwrap().is_empty());
    let mut spec = source;
    spec["servers"] = json!(vec![json!({"url":"/api"}); 32]);
    let mut paths = serde_json::Map::new();
    for i in 0..32 {
        paths.insert(
            format!("/endpoint-{i}"),
            json!({"get":{"responses":{"200":{"description":"ok"}}}}),
        );
    }
    spec["paths"] = json!(paths);
    core.http_import(
        "openapi",
        input(&project, ""),
        &write_json(dir.path(), "bounded.json", &spec),
    )
    .unwrap();
    let mut har: Value = serde_json::from_slice(&fs::read(fixture("alice.har")).unwrap()).unwrap();
    har["log"]["entries"] = json!(vec![har["log"]["entries"][0].clone(); 1000]);
    core.http_import(
        "har",
        input(&project, "alice"),
        &write_json(dir.path(), "bounded.har", &har),
    )
    .unwrap();
    assert_eq!(
        core.http_workspace(&project, None, "", 0).unwrap_err().code,
        "WORKSPACE_LIMIT"
    );
}

#[test]
fn captured_identity_matrix_covers_all_pages_and_exports_all_bounded_captures() {
    let (dir, core, project) = setup();
    let mut har: Value = serde_json::from_slice(&fs::read(fixture("alice.har")).unwrap()).unwrap();
    har["log"]["entries"] = json!(vec![har["log"]["entries"][0].clone(); 105]);
    core.http_import(
        "har",
        input(&project, "alice"),
        &write_json(dir.path(), "many.har", &har),
    )
    .unwrap();
    core.http_import("har", input(&project, "bob"), &fixture("bob.har"))
        .unwrap();
    core.http_import("openapi", input(&project, ""), &fixture("openapi30.json"))
        .unwrap();
    let ws = core.http_workspace(&project, None, "", 0).unwrap();
    assert_eq!(ws.total, 106);
    assert_eq!(ws.traffic.len(), 100);
    assert_eq!(ws.authorization_matrix.len(), 2);
    let alice = ws
        .authorization_matrix
        .iter()
        .find(|r| r.identity == "alice")
        .unwrap();
    assert_eq!(alice.capture_count, 105);
    assert_eq!(alice.capture_ids.len(), 105);
    assert_eq!(alice.statuses, vec![200]);
    assert_eq!(alice.body_variants, 1);
    assert_eq!(alice.observed_paths, 1);
    assert_eq!(alice.path, "/users/{userId}");
    let second = core.http_workspace(&project, None, "", 100).unwrap();
    assert_eq!(second.traffic.len(), 6);
    assert_eq!(second.authorization_matrix[0].capture_count, 105);
    let output = dir.path().join("export.json");
    core.http_export(&project, &output).unwrap();
    let bytes = fs::read_to_string(&output).unwrap();
    let exported: Value = serde_json::from_str(&bytes).unwrap();
    assert_eq!(exported["traffic"].as_array().unwrap().len(), 106);
    assert!(!bytes.contains("synthetic-auth-token"));
    assert!(!bytes.contains("synthetic-body-token"));
    assert!(core.http_export(&project, &output).is_err());
    let other = core.create_project("Unrelated fixture", "").unwrap();
    let target = core
        .create_target(TargetInput {
            project_id: other.id,
            name: "Foreign fixture".into(),
            kind: TargetKind::Domain,
            value: "example.test".into(),
            description: String::new(),
            tags: vec![],
            allowed_scope: vec![],
            excluded_scope: vec![],
            authorized: true,
            mode: SecurityMode::PassiveAuto,
        })
        .unwrap();
    assert_eq!(
        core.http_workspace(&project, Some(&target.id), "", 0)
            .unwrap_err()
            .code,
        "PROJECT_MISMATCH"
    );
}

#[test]
fn captured_comparison_records_original_hashes_and_refuses_tampered_sources() {
    let (dir, core, project) = setup();
    let original = core
        .http_import("har", input(&project, "alice"), &fixture("alice.har"))
        .unwrap();
    core.http_import("har", input(&project, "bob"), &fixture("bob.har"))
        .unwrap();
    let ws = core.http_workspace(&project, None, "", 0).unwrap();
    let left = ws
        .traffic
        .iter()
        .find(|e| e.identity == "alice")
        .unwrap()
        .id
        .clone();
    let right = ws
        .traffic
        .iter()
        .find(|e| e.identity == "bob")
        .unwrap()
        .id
        .clone();
    let input = HttpComparisonInput {
        left,
        right,
        owner_field: "owner".into(),
    };
    let comparison = core.http_compare(input.clone()).unwrap();
    assert_eq!(comparison["sources"][0]["sha256"], original.sha256);
    assert_eq!(
        comparison["sources"][0]["evidence_id"],
        original.evidence_id
    );
    let path = dir.path().join(
        core.evidence(&original.evidence_id)
            .unwrap()
            .storage_location,
    );
    fs::write(path, b"tampered original").unwrap();
    assert!(core.http_compare(input).is_err());
    assert_eq!(
        core.http_workspace(&project, None, "", 0)
            .unwrap()
            .comparisons
            .len(),
        1
    );
}
#[test]
fn offline_header_review_reuses_real_jobs_and_checks_original_evidence() {
    let (_dir, core, project) = setup();
    core.http_import("har", input(&project, "alice"), &fixture("alice.har"))
        .unwrap();
    let id = core.http_workspace(&project, None, "", 0).unwrap().traffic[0]
        .id
        .clone();
    let job = core.http_review_enqueue(&id).unwrap();
    assert_eq!(job.status, "queued");
    assert_eq!(core.http_review_enqueue(&id).unwrap_err().code, "JOB_BUSY");
    let done = core.execute_job(&job.id).unwrap();
    assert_eq!(done.status, "completed");
    assert_eq!(done.result.as_ref().unwrap()["network_requests"], 0);
    assert_eq!(
        done.result.as_ref().unwrap()["observations"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    let evidence = core.evidence_from_job(&job.id).unwrap();
    assert_eq!(evidence.job_id, Some(job.id.clone()));
    core.evidence_verify(&evidence.id).unwrap();
    let preview = core.job_evidence_preview(&job.id).unwrap();
    assert!(preview.contains("content-security-policy"));
    assert!(!preview.contains("synthetic-auth-token"));
    let second = core.http_review_enqueue(&id).unwrap();
    core.cancel_job(&second.id).unwrap();
    assert_eq!(core.job(&second.id).unwrap().status, "cancelled");
}
#[test]
fn har_is_redacted_normalized_hashed_and_persistent_with_no_plaintext_credentials_in_sqlite() {
    let (dir, core, project) = setup();
    let original = fs::read(fixture("alice.har")).unwrap();
    let import = core
        .http_import("har", input(&project, "alice"), &fixture("alice.har"))
        .unwrap();
    assert_eq!(import.kind, "har");
    assert_eq!(import.summary["entries"], 2);
    assert_eq!(import.summary["duplicate_endpoints"], 1);
    use sha2::{Digest, Sha256};
    assert_eq!(
        import.sha256,
        Sha256::digest(&original)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    let ws = core.http_workspace(&project, None, "", 0).unwrap();
    assert_eq!(ws.total, 2);
    assert_eq!(ws.endpoints.len(), 1);
    assert!(!ws.endpoints[0]["documented"].as_bool().unwrap());
    let e = &ws.traffic[0];
    assert_eq!(e.response["body"]["preview"]["owner"], "alice");
    assert_eq!(e.response["body"]["preview"]["token"], "[REDACTED]");
    assert_eq!(e.authentication, vec!["Bearer", "Cookie"]);
    assert_eq!(e.query_parameters, vec!["access_token"]);
    let serialized = serde_json::to_string(&ws).unwrap();
    for secret in [
        "synthetic-auth-token",
        "synthetic-query-token",
        "synthetic-cookie-token",
        "synthetic-response-cookie",
        "synthetic-body-token",
        "synthetic-header-token",
    ] {
        assert!(!serialized.contains(secret), "{secret}");
    }
    assert_eq!(
        core.http_workspace(&project, None, "Owned offline capture", 0)
            .unwrap()
            .total,
        2
    );
    assert_eq!(
        core.http_workspace(&project, None, "not-present", 0)
            .unwrap()
            .total,
        0
    );
    assert!(
        core.evidence_show(&import.evidence_id)
            .unwrap()
            .preview_omitted
    );
    core.evidence_verify(&import.evidence_id).unwrap();
    assert_eq!(
        core.evidence_export(
            &import.evidence_id,
            &dir.path().join("raw-denied.har"),
            false
        )
        .unwrap_err()
        .code,
        "RAW_APPROVAL_REQUIRED"
    );
    core.http_note(&e.id, Some("Reviewed capture; token=synthetic-note-token"))
        .unwrap();
    let id = e.id.clone();
    drop(core);
    let core = Core::open(Some(dir.path().join("http.db"))).unwrap();
    assert_eq!(core.http_exchange(&id).unwrap().identity, "alice");
    assert!(
        core.http_note(&id, None)
            .unwrap()
            .contains("Reviewed capture")
    );
    core.http_export(&project, &dir.path().join("safe.json"))
        .unwrap();
    let data = fs::read_to_string(dir.path().join("safe.json")).unwrap();
    assert!(!data.contains("synthetic-auth-token"));
    for name in ["http.db", "http.db-wal"] {
        if let Ok(bytes) = fs::read(dir.path().join(name)) {
            let db = String::from_utf8_lossy(&bytes);
            assert!(!db.contains("synthetic-auth-token"));
            assert!(!db.contains("synthetic-body-token"));
        }
    }
    let original_path = dir
        .path()
        .join(core.evidence(&import.evidence_id).unwrap().storage_location);
    fs::write(original_path, b"tampered").unwrap();
    assert!(core.evidence_verify(&import.evidence_id).is_err());
}
#[test]
fn openapi_json_and_yaml_resolve_local_parameters_and_correlate_captures() {
    let (_dir, core, project) = setup();
    core.http_import("har", input(&project, "alice"), &fixture("alice.har"))
        .unwrap();
    let imported = core
        .http_import("openapi", input(&project, ""), &fixture("openapi30.json"))
        .unwrap();
    assert_eq!(
        imported.summary["components"]["securitySchemes"]["bearerAuth"]["scheme"],
        "bearer"
    );
    let ws = core.http_workspace(&project, None, "", 0).unwrap();
    assert_eq!(ws.operations.len(), 1);
    assert_eq!(ws.operations[0].parameters[0]["name"], "userId");
    assert_eq!(ws.endpoints[0]["path"], "/users/{userId}");
    assert_eq!(ws.endpoints[0]["documented"], true);
    let yaml = core
        .http_import("openapi", input(&project, ""), &fixture("openapi31.yaml"))
        .unwrap();
    assert_eq!(yaml.summary["version"], "3.1.1");
    assert_eq!(
        core.http_workspace(&project, None, "", 0)
            .unwrap()
            .operations
            .len(),
        2
    );
}
#[test]
fn malformed_har_has_no_partial_records_or_artifacts() {
    let (dir, core, project) = setup();
    let source: Value = serde_json::from_slice(&fs::read(fixture("alice.har")).unwrap()).unwrap();
    let mut cases = Vec::new();
    let mut v = source.clone();
    v["log"]["version"] = json!("1.1");
    cases.push(v);
    let mut v = source.clone();
    v["log"]["entries"][1]["response"]["content"]["text"] = json!("not-valid-base64");
    cases.push(v);
    let mut v = source.clone();
    v["log"]["entries"][0]["request"]["url"] = json!("file:///etc/passwd");
    cases.push(v);
    let mut v = source.clone();
    v["log"]["entries"][0]["time"] = json!(-5);
    cases.push(v);
    let mut v = source.clone();
    v["log"]["entries"][0]["response"]["content"] =
        json!({"text":"x".repeat(256*1024+1),"mimeType":"text/plain"});
    cases.push(v);
    let mut v = source.clone();
    v["log"]["entries"] = json!(vec![source["log"]["entries"][0].clone(); 2001]);
    cases.push(v);
    for (i, value) in cases.into_iter().enumerate() {
        let path = dir.path().join(format!("bad-{i}.har"));
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(
            core.http_import("har", input(&project, "alice"), &path)
                .is_err()
        );
    }
    let path = dir.path().join("oversized.har");
    fs::File::create(&path)
        .unwrap()
        .set_len((MAX_HTTP_IMPORT + 1) as u64)
        .unwrap();
    assert_eq!(
        core.http_import("har", input(&project, ""), &path)
            .unwrap_err()
            .code,
        "INPUT_LIMIT"
    );
    assert!(core.http_imports(&project).unwrap().is_empty());
    assert!(core.evidence_list(Some(&project)).unwrap().is_empty());
    assert!(!dir.path().join("evidence").exists());
}
#[test]
fn external_refs_structural_cycles_yaml_aliases_and_duplicate_keys_are_rejected() {
    let (dir, core, project) = setup();
    let source: Value =
        serde_json::from_slice(&fs::read(fixture("openapi30.json")).unwrap()).unwrap();
    for (reference, code) in [
        ("https://example.test/schema.json", "EXTERNAL_REF_DISABLED"),
        ("file:///etc/passwd", "EXTERNAL_REF_DISABLED"),
        ("#/components/schemas/Missing", "INVALID_HTTP_IMPORT"),
    ] {
        let mut v = source.clone();
        v["components"]["schemas"]["User"]["properties"]["friend"]["$ref"] = json!(reference);
        let path = dir.path().join("external.json");
        fs::write(&path, serde_json::to_vec(&v).unwrap()).unwrap();
        assert_eq!(
            core.http_import("openapi", input(&project, ""), &path)
                .unwrap_err()
                .code,
            code
        );
    }
    let mut cycle = source;
    cycle["paths"]["/users/{userId}"]["$ref"] = json!("#/paths/~1users~1{userId}");
    let path = dir.path().join("cycle.json");
    fs::write(&path, serde_json::to_vec(&cycle).unwrap()).unwrap();
    assert_eq!(
        core.http_import("openapi", input(&project, ""), &path)
            .unwrap_err()
            .code,
        "REF_CYCLE"
    );
    for yaml in [
        "a: &x [*x]",
        "openapi: 3.1.1\nopenapi: 3.0.4",
        "test: !include private.yaml",
    ] {
        let path = dir.path().join("unsafe.yaml");
        fs::write(&path, yaml).unwrap();
        assert!(
            core.http_import("openapi", input(&project, ""), &path)
                .is_err()
        );
    }
}
#[test]
fn captured_role_comparison_preserves_provenance_and_creates_only_an_unverified_finding() {
    let (dir, core, project) = setup();
    core.http_import("har", input(&project, "alice"), &fixture("alice.har"))
        .unwrap();
    core.http_import("har", input(&project, "bob"), &fixture("bob.har"))
        .unwrap();
    let ws = core.http_workspace(&project, None, "", 0).unwrap();
    let a = ws.traffic.iter().find(|e| e.identity == "alice").unwrap();
    let b = ws.traffic.iter().find(|e| e.identity == "bob").unwrap();
    let compared = core
        .http_compare(HttpComparisonInput {
            left: a.id.clone(),
            right: b.id.clone(),
            owner_field: "owner".into(),
        })
        .unwrap();
    assert_eq!(compared["status"], "needs_manual_review");
    assert_eq!(compared["network_requests"], 0);
    assert_eq!(compared["owner_indicator"]["equal"], false);
    let finding = core
        .http_comparison_finding(compared["id"].as_str().unwrap())
        .unwrap();
    assert_eq!(finding.input.status, FindingStatus::NeedsVerification);
    assert_eq!(finding.input.evidence_ids.len(), 1);
    let report = core
        .report_prepare(ReportInput {
            project_id: project.clone(),
            title: "Offline HTTP fixture report".into(),
            template: ReportTemplate::Technical,
            summary: "Real offline capture comparison".into(),
            methodology: "Captured data only; no replay or verified authorization finding".into(),
            finding_ids: vec![finding.id],
            evidence_ids: finding.input.evidence_ids,
        })
        .unwrap();
    for (format, ext) in [
        (ReportFormat::Json, "json"),
        (ReportFormat::Markdown, "md"),
        (ReportFormat::Html, "html"),
        (ReportFormat::Pdf, "pdf"),
    ] {
        core.report_export(
            &report.report.id,
            format,
            &dir.path().join(format!("report.{ext}")),
            &report.report.sha256,
        )
        .unwrap();
    }
    assert!(
        fs::read(dir.path().join("report.pdf"))
            .unwrap()
            .starts_with(b"%PDF-")
    );
    let text = fs::read_to_string(dir.path().join("report.json")).unwrap();
    assert!(!text.contains("synthetic-auth-token"));
    let other = core.create_project("Other project", "").unwrap();
    core.http_import("har", input(&other.id, "eve"), &fixture("bob.har"))
        .unwrap();
    let e = core.http_workspace(&other.id, None, "", 0).unwrap().traffic[0]
        .id
        .clone();
    assert_eq!(
        core.http_compare(HttpComparisonInput {
            left: a.id.clone(),
            right: e,
            owner_field: "".into()
        })
        .unwrap_err()
        .code,
        "PROJECT_MISMATCH"
    );
    drop(core);
    let core = Core::open(Some(dir.path().join("http.db"))).unwrap();
    assert_eq!(
        core.http_workspace(&project, None, "", 0)
            .unwrap()
            .comparisons
            .len(),
        1
    );
}
