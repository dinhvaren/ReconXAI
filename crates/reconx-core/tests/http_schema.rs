use reconx_core::*;
use serde_json::{Value, json};
use std::{fs, path::Path};

fn fixture(name: &str) -> Value {
    serde_json::from_slice(
        &fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/http")
                .join(name),
        )
        .unwrap(),
    )
    .unwrap()
}
fn input(project: &str) -> HttpImportInput {
    HttpImportInput {
        project_id: project.into(),
        target_id: None,
        title: "Offline schema fixture".into(),
        identity: "Owned capture".into(),
        role: "reader".into(),
    }
}
fn imported(mut har: Value, spec: Value) -> (tempfile::TempDir, Core, String, String, String) {
    // A single distinct entry keeps each validation case independent of inventory sorting.
    har["log"]["entries"].as_array_mut().unwrap().truncate(1);
    let dir = tempfile::tempdir().unwrap();
    let core = Core::open(Some(dir.path().join("schema.db"))).unwrap();
    let project = core.create_project("Schema fixtures", "").unwrap().id;
    for (name, kind, value) in [("source.har", "har", har), ("source.json", "openapi", spec)] {
        let p = dir.path().join(name);
        fs::write(&p, serde_json::to_vec(&value).unwrap()).unwrap();
        core.http_import(kind, input(&project), &p).unwrap();
    }
    let ws = core.http_workspace(&project, None, "", 0).unwrap();
    (
        dir,
        core,
        project,
        ws.traffic[0].id.clone(),
        ws.operations[0].id.clone(),
    )
}
fn analyze(core: &Core, capture: &str, op: &str) -> Value {
    let job = core.http_schema_enqueue(capture, op).unwrap();
    let job = core.execute_job(&job.id).unwrap();
    assert_eq!(job.status, "completed", "{:?}", job.error_message);
    job.result.unwrap()
}
fn rule<'a>(report: &'a Value, rule: &str) -> Vec<&'a Value> {
    report["observations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["rule"] == rule)
        .collect()
}
#[test]
fn required_type_and_redaction_have_integrity_linked_provenance_and_persist() {
    let mut har = fixture("alice.har");
    har["log"]["entries"][0]["response"]["content"] = json!({"mimeType":"application/json","size":90,"text":r#"{"id":"wrong","token":"never-export-this"}"#});
    let mut spec = fixture("openapi30.json");
    spec["components"]["schemas"]["User"]["properties"]["token"] = json!({"type":"integer"});
    let (dir, core, project, capture, op) = imported(har, spec);
    let job = core.http_schema_enqueue(&capture, &op).unwrap();
    assert_eq!(
        core.http_schema_enqueue(&capture, &op).unwrap_err().code,
        "JOB_BUSY"
    );
    let done = core.execute_job(&job.id).unwrap();
    assert_eq!(done.status, "completed");
    let report = done.result.unwrap();
    assert_eq!(report["network_requests"], 0);
    assert_eq!(rule(&report, "json_type").len(), 1);
    assert_eq!(rule(&report, "required_field").len(), 1);
    assert_eq!(
        rule(&report, "redacted_value")[0]["classification"],
        "insufficient_evidence"
    );
    assert_eq!(
        rule(&report, "json_type")[0]["openapi_pointer"],
        "/components/schemas/User/properties/id/type"
    );
    for o in report["observations"].as_array().unwrap() {
        assert!(
            o["capture_pointer"]
                .as_str()
                .unwrap()
                .starts_with("/log/entries/0/")
        );
        assert!(
            core.evidence_verify(o["capture_evidence_id"].as_str().unwrap())
                .is_ok()
        );
        assert!(
            core.evidence_verify(o["openapi_evidence_id"].as_str().unwrap())
                .is_ok()
        );
    }
    assert!(!report.to_string().contains("never-export-this"));
    assert!(core.findings(Some(&project)).unwrap().is_empty());
    let evidence = core.evidence_from_job(&job.id).unwrap();
    assert_eq!(evidence.job_id.as_deref(), Some(job.id.as_str()));
    drop(core);
    let core = Core::open(Some(dir.path().join("schema.db"))).unwrap();
    assert_eq!(core.job(&job.id).unwrap().result.unwrap(), report);
    assert!(
        core.http_workspace(&project, None, "", 0)
            .unwrap()
            .analysis_jobs
            .iter()
            .any(|v| v["job"]["id"] == job.id)
    );
}
#[test]
fn exact_status_precedes_range_and_default_and_undocumented_is_observation() {
    for (status, responses, expect_type, expect_status) in [
        (
            201,
            json!({"2XX":{"description":"range","content":{"application/json":{"schema":{"type":"array"}}}},"default":{"description":"fallback"}}),
            true,
            false,
        ),
        (
            200,
            json!({"200":{"description":"exact"},"2XX":{"description":"range","content":{"application/json":{"schema":{"type":"array"}}}}}),
            false,
            false,
        ),
        (
            418,
            json!({"default":{"description":"fallback"}}),
            false,
            false,
        ),
        (418, json!({"200":{"description":"ok"}}), false, true),
    ] {
        let mut har = fixture("alice.har");
        har["log"]["entries"][0]["response"]["status"] = json!(status);
        let mut spec = fixture("openapi30.json");
        spec["paths"]["/users/{userId}"]["get"]["responses"] = responses;
        let (_dir, core, _, capture, op) = imported(har, spec);
        let report = analyze(&core, &capture, &op);
        assert_eq!(!rule(&report, "json_type").is_empty(), expect_type);
        assert_eq!(!rule(&report, "response_status").is_empty(), expect_status);
    }
}
#[test]
fn incomplete_bodies_headers_query_and_auth_are_not_confirmed_vulnerabilities() {
    let mut har = fixture("alice.har");
    har["log"]["entries"][0]["request"]["headers"] = json!([]);
    har["log"]["entries"][0]["request"]["cookies"] = json!([]);
    har["log"]["entries"][0]["response"]["content"] =
        json!({"mimeType":"application/json","size":100});
    let mut spec = fixture("openapi30.json");
    spec["paths"]["/users/{userId}"]["get"]["parameters"] =
        json!([{"name":"filter","in":"query","required":true,"schema":{"type":"integer"}}]);
    spec["paths"]["/users/{userId}"]["get"]["responses"]["200"]["headers"] =
        json!({"X-Trace":{"required":true,"schema":{"type":"string"}}});
    let (_dir, core, _, capture, op) = imported(har, spec);
    let report = analyze(&core, &capture, &op);
    for o in report["observations"].as_array().unwrap() {
        assert_ne!(o["classification"], "confirmed_documented_mismatch");
    }
    assert_eq!(rule(&report, "body_unavailable").len(), 1);
    assert_eq!(rule(&report, "required_parameter").len(), 1);
    assert_eq!(rule(&report, "required_response_header").len(), 1);
    assert_eq!(
        rule(&report, "authentication_scheme")[0]["classification"],
        "possible_mismatch"
    );
}
#[test]
fn mime_wildcards_and_mismatches_and_request_body_checks() {
    let mut har = fixture("alice.har");
    har["log"]["entries"][0]["request"]["method"] = json!("POST");
    har["log"]["entries"][0]["request"]["postData"] =
        json!({"mimeType":"application/json; charset=utf-8","text":r#"{"enabled":"yes"}"#});
    let mut spec = fixture("openapi30.json");
    let get = spec["paths"]["/users/{userId}"]
        .as_object_mut()
        .unwrap()
        .remove("get")
        .unwrap();
    spec["paths"]["/users/{userId}"]["post"] = get;
    spec["paths"]["/users/{userId}"]["post"]["requestBody"] = json!({"required":true,"content":{"application/*":{"schema":{"type":"object","properties":{"enabled":{"type":"boolean"}}}}}});
    spec["paths"]["/users/{userId}"]["post"]["responses"]["200"]["content"] =
        json!({"text/plain":{"schema":{"type":"string"}}});
    let (_dir, core, _, capture, op) = imported(har, spec);
    let report = analyze(&core, &capture, &op);
    assert_eq!(
        rule(&report, "content_type")[0]["classification"],
        "confirmed_documented_mismatch"
    );
    assert_eq!(
        rule(&report, "json_type")[0]["instance_pointer"],
        "/enabled"
    );
    assert_eq!(
        rule(&report, "json_type")[0]["capture_pointer"],
        "/log/entries/0/request/postData/text"
    );
}
#[test]
fn schema_cycles_dialects_and_unimplemented_constraints_are_explicit() {
    for schema in [
        json!({"$ref":"#/components/schemas/User"}),
        json!({"type":"object","oneOf":[{"type":"object"}]}),
        json!({"$schema":"https://example.test/custom-dialect","type":"array"}),
        json!({"$id":"relative.json","type":"array"}),
    ] {
        let mut spec = fixture("openapi30.json");
        spec["openapi"] = json!("3.1.1");
        spec["components"]["schemas"]["User"] = schema;
        let (_dir, core, _, capture, op) = imported(fixture("alice.har"), spec);
        let report = analyze(&core, &capture, &op);
        assert!(
            report["observations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|o| o["classification"] == "unsupported_validation_case")
        );
        assert!(rule(&report, "json_type").is_empty());
    }
}
#[test]
fn recursive_schema_can_follow_finite_instance_without_false_cycle() {
    let mut har = fixture("alice.har");
    har["log"]["entries"][0]["response"]["content"] = json!({"mimeType":"application/json","text":r#"{"id":1,"owner":"alice","friend":{"id":2,"owner":"bob"}}"#});
    let (_dir, core, _, capture, op) = imported(har, fixture("openapi30.json"));
    let report = analyze(&core, &capture, &op);
    assert!(rule(&report, "schema_reference_cycle").is_empty());
    assert!(rule(&report, "required_field").is_empty());
    assert!(rule(&report, "json_type").is_empty());
}
#[test]
fn nullable_boolean_and_directional_required_respect_openapi_versions() {
    for version in ["3.0.4", "3.1.1"] {
        let mut har = fixture("alice.har");
        har["log"]["entries"][0]["response"]["content"] =
            json!({"mimeType":"application/json","text":r#"{"id":null,"owner":"alice"}"#});
        let mut spec = fixture("openapi30.json");
        spec["openapi"] = json!(version);
        spec["components"]["schemas"]["User"]["required"] = json!(["id", "owner", "password"]);
        spec["components"]["schemas"]["User"]["properties"]["password"] =
            json!({"type":"string","writeOnly":true});
        spec["components"]["schemas"]["User"]["properties"]["id"] = if version.starts_with("3.0") {
            json!({"type":"integer","nullable":true})
        } else {
            json!({"type":["integer","null"]})
        };
        let (_dir, core, _, capture, op) = imported(har, spec);
        let report = analyze(&core, &capture, &op);
        assert!(rule(&report, "json_type").is_empty());
        assert!(rule(&report, "required_field").is_empty());
    }
}
#[test]
fn exact_origin_and_target_partitions_are_enforced_even_for_offline_jobs() {
    for url in [
        "http://api.example.test/api/users/1",
        "https://api.example.test:8443/api/users/1",
        "https://other.example.test/api/users/1",
    ] {
        let mut har = fixture("alice.har");
        har["log"]["entries"][0]["request"]["url"] = json!(url);
        let (_dir, core, _, capture, op) = imported(har, fixture("openapi30.json"));
        assert_eq!(
            core.http_schema_enqueue(&capture, &op).unwrap_err().code,
            "OPERATION_MISMATCH"
        );
    }
    let mut spec = fixture("openapi30.json");
    spec["servers"] = json!([{"url":"/api"}]);
    let (_dir, core, _, capture, op) = imported(fixture("alice.har"), spec);
    assert_eq!(
        core.http_schema_enqueue(&capture, &op).unwrap_err().code,
        "OPERATION_MISMATCH"
    );
}
#[test]
fn overlapping_target_and_project_documentation_cannot_cross_bind() {
    let (dir, core, project, capture, op) =
        imported(fixture("alice.har"), fixture("openapi30.json"));
    let targets = ["/api/", "/api/users/"].map(|path| {
        core.create_target(TargetInput {
            project_id: project.clone(),
            name: path.into(),
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
    });
    let mut args = input(&project);
    args.target_id = Some(targets[0].id.clone());
    core.http_import("har", args, &dir.path().join("source.har"))
        .unwrap();
    let mut args = input(&project);
    args.target_id = Some(targets[1].id.clone());
    core.http_import("openapi", args, &dir.path().join("source.json"))
        .unwrap();
    let ws = core.http_workspace(&project, None, "", 0).unwrap();
    let scoped_capture = ws.traffic.iter().find(|e| e.target_id.is_some()).unwrap();
    let scoped_op = ws
        .operations
        .iter()
        .find(|e| e.target_id.is_some())
        .unwrap();
    for (c, o) in [
        (&capture, &scoped_op.id),
        (&scoped_capture.id, &op),
        (&scoped_capture.id, &scoped_op.id),
    ] {
        assert_eq!(
            core.http_schema_enqueue(c, o).unwrap_err().code,
            "TARGET_MISMATCH"
        );
    }
}
#[test]
fn queued_job_cancellation_tampering_and_legacy_locator_fail_safely() {
    let (dir, core, _, capture, op) = imported(fixture("alice.har"), fixture("openapi30.json"));
    let job = core.http_schema_enqueue(&capture, &op).unwrap();
    core.cancel_job(&job.id).unwrap();
    assert_eq!(core.job(&job.id).unwrap().status, "cancelled");
    let db = reconx_storage::Database::open(dir.path().join("schema.db")).unwrap();
    db.connect().unwrap().execute("UPDATE http_exchanges SET data_json=json_remove(data_json,'$.source_pointer') WHERE id=?1",[&capture]).unwrap();
    let report = analyze(&core, &capture, &op);
    assert_eq!(report["network_requests"], 0);
    let job = core.http_schema_enqueue(&capture, &op).unwrap();
    db.connect()
        .unwrap()
        .execute(
            "UPDATE http_exchanges SET data_json=json_set(data_json,'$.status',201) WHERE id=?1",
            [&capture],
        )
        .unwrap();
    let failed = core.execute_job(&job.id).unwrap();
    assert_eq!(failed.status, "failed");
    assert_eq!(failed.error_code.as_deref(), Some("SOURCE_CHANGED"));
}
#[test]
fn schema_work_and_observation_budgets_produce_incomplete_coverage() {
    let mut har = fixture("alice.har");
    har["log"]["entries"][0]["response"]["content"] = json!({"mimeType":"application/json","text":serde_json::to_string(&vec![1;12_000]).unwrap()});
    let mut spec = fixture("openapi30.json");
    spec["components"]["schemas"]["User"] = json!({"type":"array","items":{"type":"integer"}});
    let (_dir, core, _, capture, op) = imported(har, spec);
    let report = analyze(&core, &capture, &op);
    assert!(report["checks"].as_u64().unwrap() <= 10_000);
    assert!(!rule(&report, "resource_budget").is_empty());
}
