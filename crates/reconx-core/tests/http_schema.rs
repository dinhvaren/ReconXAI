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
fn body_observations(report: &Value) -> Vec<&Value> {
    report["observations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| {
            o["capture_pointer"]
                .as_str()
                .unwrap()
                .ends_with("/response/content/text")
        })
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
        json!({"type":"object","patternProperties":{"(?=x)":{"type":"string"}}}),
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

fn constrained(value: Value, schema: Value, version: &str) -> Value {
    constrained_with_options(value, schema, version, SchemaReviewOptions::default())
}
fn constrained_with_options(
    value: Value,
    schema: Value,
    version: &str,
    options: SchemaReviewOptions,
) -> Value {
    let mut har = fixture("alice.har");
    har["log"]["entries"][0]["response"]["content"] =
        json!({"mimeType":"application/json","text":value.to_string()});
    let mut spec = fixture("openapi30.json");
    spec["openapi"] = json!(version);
    spec["components"]["schemas"]["User"] = schema;
    let (_dir, core, _, capture, op) = imported(har, spec);
    let job = core
        .http_schema_enqueue_with_options(&capture, &op, options)
        .unwrap();
    let done = core.execute_job(&job.id).unwrap();
    assert_eq!(done.status, "completed", "{:?}", done.error_message);
    done.result.unwrap()
}

#[test]
fn formats_are_annotations_by_default_including_inside_boolean_composition() {
    let report = constrained(json!("not-a-date"), json!({"format":"date"}), "3.1.1");
    assert_eq!(report["options"]["assert_formats"], false);
    assert_eq!(
        rule(&report, "format_annotation")[0]["classification"],
        "format_annotation"
    );
    assert!(rule(&report, "string_format").is_empty());
    let report = constrained(
        json!("text"),
        json!({"oneOf":[{"format":"custom"},{"type":"number"}]}),
        "3.1.1",
    );
    assert!(rule(&report, "schema_composition").is_empty(), "{report}");
    let report = constrained(
        json!("text"),
        json!({"oneOf":[{"format":"custom"},{"type":"string"}]}),
        "3.1.1",
    );
    assert_eq!(
        rule(&report, "schema_composition")[0]["classification"],
        "confirmed_documented_mismatch"
    );
}

#[test]
fn conditional_schemas_select_only_the_adjacent_branch_with_original_locations() {
    for (value, suffix, instance) in [
        (
            json!({"kind":"a","payload":"wrong"}),
            "then/properties/payload/type",
            "/payload",
        ),
        (
            json!({"kind":"b","fallback":1}),
            "else/properties/fallback/type",
            "/fallback",
        ),
    ] {
        let report = constrained(
            value,
            json!({
                "$defs":{"guard":{"required":["kind"],"properties":{"kind":{"const":"a"}}}},
                "if":{"$ref":"#/components/schemas/User/$defs/guard"},
                "then":{"properties":{"payload":{"type":"integer"}}},
                "else":{"properties":{"fallback":{"type":"boolean"}}}
            }),
            "3.1.1",
        );
        let errors = rule(&report, "json_type");
        assert_eq!(errors.len(), 1, "{report}");
        assert_eq!(
            errors[0]["openapi_pointer"],
            format!("/components/schemas/User/{suffix}")
        );
        assert_eq!(errors[0]["instance_pointer"], instance);
        assert!(
            rule(&report, "schema_constraint").is_empty(),
            "Guard errors leaked: {report}"
        );
        assert!(rule(&report, "schema_keywords").is_empty(), "{report}");
    }
    for schema in [
        json!({"if":true,"then":true,"else":false}),
        json!({"if":false,"then":false,"else":true}),
        json!({"if":false,"then":false}),
        json!({"if":true,"else":false}),
        json!({"if":false}),
        json!({"then":false,"else":false}),
        json!({"allOf":[{"if":true},{"then":false},{"else":false}]}),
        json!({"if":{"format":"custom"}}),
    ] {
        let report = constrained(json!(5), schema, "3.1.1");
        assert!(
            report["observations"]
                .as_array()
                .unwrap()
                .iter()
                .all(|o| o["classification"] != "confirmed_documented_mismatch"
                    && o["rule"] != "schema_keywords"
                    && o["rule"] != "conditional_branch"),
            "{report}"
        );
    }
}

#[test]
fn conditional_unknown_redacted_invalid_and_wrong_dialect_inputs_stay_inconclusive() {
    for (value, condition) in [
        (
            json!({"kind":"[REDACTED]"}),
            json!({"properties":{"kind":{"const":"a"}}}),
        ),
        (json!(5), json!({"unevaluatedProperties":false})),
        (json!(5), json!(null)),
        (
            json!(5),
            json!({"$schema":"https://example.test/custom-dialect"}),
        ),
    ] {
        let report = constrained(
            value,
            json!({"if":condition,"then":false,"else":false}),
            "3.1.1",
        );
        assert_eq!(rule(&report, "conditional_branch").len(), 1, "{report}");
        assert_eq!(
            rule(&report, "conditional_branch")[0]["openapi_pointer"],
            "/components/schemas/User/if"
        );
        assert!(rule(&report, "false_schema").is_empty(), "{report}");
        assert_eq!(report["network_requests"], 0);
    }
    let report = constrained(
        json!(5),
        json!({"if":true,"then":null,"else":false}),
        "3.1.1",
    );
    assert_eq!(
        rule(&report, "invalid_schema")[0]["openapi_pointer"],
        "/components/schemas/User/then"
    );
    let report = constrained(json!(5), json!({"if":null}), "3.1.1");
    assert_eq!(rule(&report, "invalid_conditional").len(), 1, "{report}");
    for keyword in ["if", "then", "else"] {
        let report = constrained(json!(5), json!({keyword:false}), "3.0.4");
        assert_eq!(
            rule(&report, "schema_dialect")[0]["openapi_pointer"],
            format!("/components/schemas/User/{keyword}")
        );
        assert!(rule(&report, "false_schema").is_empty());
    }
}

#[test]
fn conditional_predicates_obey_the_saved_format_policy_and_compose_without_leaks() {
    for (assert_formats, failures) in [(false, 1), (true, 0)] {
        let report = constrained_with_options(
            json!("bad-date"),
            json!({
                "if":{"format":"date"},"then":false,"else":true
            }),
            "3.1.1",
            SchemaReviewOptions { assert_formats },
        );
        assert_eq!(rule(&report, "false_schema").len(), failures, "{report}");
        assert!(
            rule(&report, "string_format").is_empty(),
            "Predicate error leaked: {report}"
        );
    }
    let report = constrained(
        json!(5),
        json!({"oneOf":[
            {"if":{"type":"integer"},"then":{"minimum":6},"else":false},
            {"type":"integer"}
        ]}),
        "3.1.1",
    );
    assert!(
        report["observations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|o| o["classification"] != "confirmed_documented_mismatch"
                && o["rule"] != "schema_keywords"
                && o["rule"] != "conditional_branch"),
        "{report}"
    );
    let report = constrained(json!(5), json!({"not":{"if":true,"then":false}}), "3.1.1");
    assert!(rule(&report, "schema_not").is_empty(), "{report}");
}

#[test]
fn nested_conditionals_cycles_and_predicate_expansion_share_existing_limits() {
    let report = constrained(
        json!(5),
        json!({
            "if":true,"then":{"if":{"type":"integer"},"then":{"maximum":3},"else":false},"else":false
        }),
        "3.1.1",
    );
    assert_eq!(
        rule(&report, "schema_constraint")[0]["openapi_pointer"],
        "/components/schemas/User/then/then/maximum"
    );
    let report = constrained(
        json!(5),
        json!({
            "if":{"$ref":"#/components/schemas/User"},"then":false,"else":false
        }),
        "3.1.1",
    );
    assert_eq!(rule(&report, "conditional_branch").len(), 1, "{report}");
    assert!(rule(&report, "false_schema").is_empty());
    let report = constrained(
        json!(5),
        json!({
            "if":{"allOf":(0..12_000).map(|_|json!({"type":"integer"})).collect::<Vec<_>>()},
            "then":false,"else":false
        }),
        "3.1.1",
    );
    assert!(report["checks"].as_u64().unwrap() <= 10_000);
    assert_eq!(rule(&report, "conditional_branch").len(), 1, "{report}");
    assert!(rule(&report, "false_schema").is_empty());
}

#[test]
fn opted_in_syntactic_formats_use_real_validators_without_network_access() {
    for (format, valid, invalid) in [
        ("date", "2024-02-29", "2025-02-29"),
        ("time", "23:59:59Z", "25:01:00Z"),
        ("date-time", "2026-10-19T03:04:05+07:00", "not-a-date"),
        ("duration", "P3DT4H", "3days"),
        ("email", "owned@example.test", "bad-address"),
        ("hostname", "example.test", "bad host"),
        ("ipv4", "127.0.0.1", "999.2.3.4"),
        ("ipv6", "::1", "::zz"),
        ("uri", "https://example.test/path#part", "not a uri"),
        ("uri-reference", "../path#section", "bad path"),
        ("uri-template", "/items/{id}", "{unclosed"),
        (
            "uuid",
            "550e8400-e29b-41d4-a716-446655440000",
            "urn:uuid:550e8400-e29b-41d4-a716-446655440000",
        ),
        ("json-pointer", "/a~1b/~0", "a/b"),
        ("relative-json-pointer", "0#", "x"),
    ] {
        for (text, mismatches) in [(valid, 0), (invalid, 1)] {
            let report = constrained_with_options(
                json!(text),
                json!({"format":format}),
                "3.1.1",
                SchemaReviewOptions {
                    assert_formats: true,
                },
            );
            assert_eq!(
                rule(&report, "string_format").len(),
                mismatches,
                "format {format}: {report}"
            );
            assert!(rule(&report, "format_evaluation").is_empty(), "{report}");
            assert_eq!(report["network_requests"], 0);
            for observation in rule(&report, "string_format") {
                assert_eq!(
                    observation["openapi_pointer"],
                    "/components/schemas/User/format"
                );
                assert_eq!(
                    observation["classification"],
                    "confirmed_documented_mismatch"
                );
            }
        }
    }
    let report = constrained_with_options(
        json!("not-a-date"),
        json!({"format":"date"}),
        "3.0.4",
        SchemaReviewOptions {
            assert_formats: true,
        },
    );
    assert_eq!(rule(&report, "string_format").len(), 1);
}

#[test]
fn unsupported_redacted_or_over_budget_formats_never_count_as_successful_assertions() {
    let options = SchemaReviewOptions {
        assert_formats: true,
    };
    for (value, schema, expected) in [
        (json!("x"), json!({"format":false}), "invalid_format"),
        (json!("x"), json!({"format":"custom"}), "unsupported_format"),
        (json!("x"), json!({"format":"regex"}), "unsupported_format"),
        (
            json!("[REDACTED]"),
            json!({"format":"date"}),
            "redacted_value",
        ),
        (
            json!("x"),
            json!({"format":"[REDACTED]"}),
            "format_evaluation",
        ),
        (
            json!("x".repeat(4097)),
            json!({"format":"uri-reference"}),
            "format_budget",
        ),
    ] {
        let report = constrained_with_options(value, schema, "3.1.1", options);
        assert_eq!(rule(&report, expected).len(), 1, "{report}");
        assert!(rule(&report, "string_format").is_empty(), "{report}");
    }
    let report = constrained_with_options(
        json!("x"),
        json!({"oneOf":[{"format":"custom"},{"type":"string"}]}),
        "3.1.1",
        options,
    );
    assert_eq!(
        rule(&report, "schema_composition")[0]["classification"],
        "insufficient_evidence"
    );
    let alternatives: Vec<_> = (0..300)
        .map(|_| json!({"format":"uri-reference"}))
        .collect();
    let report = constrained_with_options(
        json!("a".repeat(4096)),
        json!({"allOf":alternatives}),
        "3.1.1",
        options,
    );
    assert!(!rule(&report, "format_budget").is_empty(), "{report}");
    assert!(rule(&report, "string_format").is_empty());
}

#[test]
fn format_options_survive_restart_and_tampering_cannot_change_queued_policy() {
    let mut har = fixture("alice.har");
    har["log"]["entries"][0]["response"]["content"] =
        json!({"mimeType":"application/json","text":"\"not-a-date\""});
    let mut spec = fixture("openapi30.json");
    spec["components"]["schemas"]["User"] = json!({"format":"date"});
    let (dir, core, project, capture, operation) = imported(har, spec);
    let job = core
        .http_schema_enqueue_with_options(
            &capture,
            &operation,
            SchemaReviewOptions {
                assert_formats: true,
            },
        )
        .unwrap();
    drop(core);
    let core = Core::open(Some(dir.path().join("schema.db"))).unwrap();
    let done = core.execute_job(&job.id).unwrap();
    assert_eq!(done.status, "completed");
    let result = done.result.unwrap();
    assert_eq!(result["options"]["assert_formats"], true);
    assert_eq!(rule(&result, "string_format").len(), 1);
    assert!(core.findings(Some(&project)).unwrap().is_empty());
    core.evidence_verify(&core.evidence_from_job(&job.id).unwrap().id)
        .unwrap();
    for change in [
        "UPDATE http_schema_job_options SET options_json='{\"assert_formats\":false}' WHERE job_id=?1",
        "DELETE FROM http_schema_job_options WHERE job_id=?1",
    ] {
        let job = core
            .http_schema_enqueue_with_options(
                &capture,
                &operation,
                SchemaReviewOptions {
                    assert_formats: true,
                },
            )
            .unwrap();
        reconx_storage::Database::open(dir.path().join("schema.db"))
            .unwrap()
            .connect()
            .unwrap()
            .execute(change, [&job.id])
            .unwrap();
        let failed = core.execute_job(&job.id).unwrap();
        assert_eq!(failed.status, "failed");
        assert_eq!(failed.error_code.as_deref(), Some("SOURCE_CHANGED"));
    }
}

#[test]
fn legacy_queued_schema_fingerprint_still_executes_with_default_options() {
    let (dir, core, _, capture, operation) =
        imported(fixture("alice.har"), fixture("openapi30.json"));
    let report = analyze(&core, &capture, &operation);
    let bytes = serde_json::to_vec(&json!([
        core.http_exchange(&capture).unwrap(),
        core.api_operation(&operation).unwrap(),
        report["sources"]
    ]))
    .unwrap();
    let fingerprint = reconx_analysis::hash(&bytes);
    let job = core.http_schema_enqueue(&capture, &operation).unwrap();
    let db = reconx_storage::Database::open(dir.path().join("schema.db")).unwrap();
    let conn = db.connect().unwrap();
    conn.execute(
        "DELETE FROM http_schema_job_options WHERE job_id=?1",
        [&job.id],
    )
    .unwrap();
    conn.execute(
        "UPDATE http_schema_jobs SET fingerprint=?1 WHERE job_id=?2",
        reconx_storage::rusqlite::params![fingerprint, job.id],
    )
    .unwrap();
    let done = core.execute_job(&job.id).unwrap();
    assert_eq!(done.status, "completed", "{:?}", done.error_message);
    assert_eq!(done.result.unwrap()["options"]["assert_formats"], false);
}

#[test]
fn scalar_constraints_use_dialect_aware_numeric_equality_and_unicode_lengths() {
    for (value, schema, version, mismatches) in [
        (json!(1), json!({"enum":[1.0]}), "3.1.1", 0),
        (json!({"n":1}), json!({"const":{"n":1.0}}), "3.1.1", 0),
        (
            json!("Tiếng Việt"),
            json!({"minLength":10,"maxLength":10}),
            "3.0.4",
            0,
        ),
        (json!("é"), json!({"minLength":2}), "3.1.1", 1),
        (json!(4), json!({"minimum":5}), "3.0.4", 1),
        (
            json!(5),
            json!({"minimum":5,"exclusiveMinimum":true}),
            "3.0.4",
            1,
        ),
        (json!(5), json!({"exclusiveMinimum":5}), "3.1.1", 1),
        (json!(0.3), json!({"multipleOf":0.1}), "3.1.1", 0),
        (json!([1, 2]), json!({"maxItems":1}), "3.1.1", 1),
        (json!({"id":1}), json!({"minProperties":2}), "3.1.1", 1),
        (
            json!(9007199254740993_u64),
            json!({"enum":[9007199254740992_u64]}),
            "3.1.1",
            1,
        ),
    ] {
        let report = constrained(value, schema, version);
        assert_eq!(
            rule(&report, "schema_constraint").len(),
            mismatches,
            "{report}"
        );
        assert!(rule(&report, "invalid_constraint").is_empty(), "{report}");
        for o in rule(&report, "schema_constraint") {
            assert!(
                o["openapi_pointer"]
                    .as_str()
                    .unwrap()
                    .starts_with("/components/schemas/User/")
            );
            assert_eq!(o["classification"], "confirmed_documented_mismatch");
        }
    }
}

#[test]
fn invalid_scalar_declarations_and_30_const_are_explicit_not_assumed_valid() {
    for (schema, version, expected) in [
        (
            json!({"minimum":3,"exclusiveMinimum":3}),
            "3.0.4",
            "invalid_constraint",
        ),
        (
            json!({"exclusiveMinimum":true}),
            "3.1.1",
            "invalid_constraint",
        ),
        (json!({"multipleOf":0}), "3.1.1", "invalid_constraint"),
        (json!({"minLength":-1}), "3.1.1", "invalid_constraint"),
        (json!({"const":3}), "3.0.4", "schema_dialect"),
    ] {
        let report = constrained(json!(3), schema, version);
        assert!(!rule(&report, expected).is_empty(), "{report}");
        assert!(rule(&report, "schema_constraint").is_empty());
    }
}

#[test]
fn alternatives_do_not_publish_failed_branch_errors_or_guess_unknown_branches() {
    for (schema, mismatch, unknown) in [
        (
            json!({"oneOf":[{"type":"integer"},{"type":"string"}]}),
            false,
            false,
        ),
        (
            json!({"oneOf":[{"type":"integer"},{"type":"number"}]}),
            true,
            false,
        ),
        (
            json!({"anyOf":[{"type":"string"},{"minimum":4}]}),
            false,
            false,
        ),
        (
            json!({"anyOf":[{"type":"string"},{"minimum":6}]}),
            true,
            false,
        ),
        (
            json!({"oneOf":[{"type":"integer"},{"unevaluatedProperties":false}]}),
            false,
            true,
        ),
        (
            json!({"anyOf":[{"type":"string"},{"unevaluatedProperties":false}]}),
            false,
            true,
        ),
        (
            json!({"anyOf":[{"type":"integer"},{"unevaluatedProperties":false}]}),
            false,
            false,
        ),
    ] {
        let report = constrained(json!(5), schema, "3.1.1");
        let observations = rule(&report, "schema_composition");
        assert_eq!(
            observations
                .iter()
                .any(|o| o["classification"] == "confirmed_documented_mismatch"),
            mismatch,
            "{report}"
        );
        assert_eq!(
            observations
                .iter()
                .any(|o| o["classification"] == "insufficient_evidence"),
            unknown,
            "{report}"
        );
        assert!(
            rule(&report, "json_type").is_empty(),
            "Alternative errors must remain speculative: {report}"
        );
    }
}

#[test]
fn allof_and_not_keep_original_constraint_locations() {
    let report = constrained(
        json!(5),
        json!({"allOf":[{"minimum":4},{"maximum":3}],"not":{"enum":[5]}}),
        "3.1.1",
    );
    assert_eq!(
        rule(&report, "schema_constraint")[0]["openapi_pointer"],
        "/components/schemas/User/allOf/1/maximum"
    );
    assert_eq!(
        rule(&report, "schema_not")[0]["classification"],
        "confirmed_documented_mismatch"
    );
    let report = constrained(
        json!(5),
        json!({"not":{"unevaluatedProperties":false}}),
        "3.1.1",
    );
    assert_eq!(
        rule(&report, "schema_not")[0]["classification"],
        "insufficient_evidence"
    );
}

#[test]
fn redacted_equality_and_alternatives_never_create_fabricated_mismatches() {
    let value = json!({"token":"private-test-value"});
    let report = constrained(
        value,
        json!({"oneOf":[{"const":{"token":"something-else"}}]}),
        "3.1.1",
    );
    assert_eq!(
        rule(&report, "schema_composition")[0]["classification"],
        "insufficient_evidence"
    );
    assert!(!report.to_string().contains("private-test-value"));
    let report = constrained(
        json!({"token":"private-test-value"}),
        json!({"enum":[{"token":"different"}]}),
        "3.1.1",
    );
    assert!(rule(&report, "schema_constraint").is_empty());
    assert_eq!(
        rule(&report, "equality_constraint")[0]["classification"],
        "insufficient_evidence"
    );
}

#[test]
fn additional_properties_validate_values_and_respect_evaluated_patterns() {
    let report = constrained(
        json!({"id":1,"extra":"wrong"}),
        json!({"properties":{"id":{"type":"integer"}},"additionalProperties":{"type":"integer"}}),
        "3.1.1",
    );
    assert_eq!(rule(&report, "json_type")[0]["instance_pointer"], "/extra");
    assert_eq!(
        rule(&report, "json_type")[0]["openapi_pointer"],
        "/components/schemas/User/additionalProperties/type"
    );
    let report = constrained(
        json!({"x":"allowed"}),
        json!({"patternProperties":{"^x$":{"type":"string"}},"additionalProperties":false}),
        "3.1.1",
    );
    assert!(body_observations(&report).is_empty(), "{report}");
    let report = constrained(
        json!({"id":1,"extra":2}),
        json!({"properties":{"id":{}},"additionalProperties":false}),
        "3.0.4",
    );
    assert_eq!(
        rule(&report, "additional_properties")[0]["instance_pointer"],
        "/extra"
    );
}

#[test]
fn string_patterns_preserve_unanchored_search_and_ecma_character_classes() {
    for version in ["3.0.4", "3.1.1"] {
        for (value, pattern, mismatch) in [
            (json!("prefixABCtail"), "ABC", false),
            (json!("abc"), "^a[a-z]+$", false),
            (json!("ABC"), "^a[a-z]+$", true),
            (json!("123"), r"^\d+$", false),
            (json!("\u{0661}\u{0662}"), r"^\d+$", true),
            (json!("ascii_09"), r"^\w+$", false),
            (json!("\u{e9}"), r"^\w+$", true),
            (json!("\u{feff}"), r"^\s$", false),
            (json!("anything"), "", false),
            (json!(42), "^letters$", false),
        ] {
            let report = constrained(value, json!({"pattern":pattern}), version);
            assert_eq!(
                !rule(&report, "string_pattern").is_empty(),
                mismatch,
                "{report}"
            );
            assert!(rule(&report, "invalid_pattern").is_empty(), "{report}");
            for observation in rule(&report, "string_pattern") {
                assert_eq!(
                    observation["openapi_pointer"],
                    "/components/schemas/User/pattern"
                );
                assert_eq!(
                    observation["classification"],
                    "confirmed_documented_mismatch"
                );
            }
        }
    }
}

#[test]
fn pattern_properties_apply_all_matching_schemas_alongside_explicit_properties() {
    let schema = json!({
        "properties":{"x":{"type":"string"}},
        "patternProperties":{"^x":{"type":"integer"},"x$":{"minimum":5}},
        "additionalProperties":false
    });
    let report = constrained(json!({"x":1,"extra":1}), schema, "3.1.1");
    assert_eq!(
        rule(&report, "json_type")[0]["openapi_pointer"],
        "/components/schemas/User/properties/x/type"
    );
    assert_eq!(
        rule(&report, "schema_constraint")[0]["openapi_pointer"],
        "/components/schemas/User/patternProperties/x$/minimum"
    );
    assert_eq!(rule(&report, "additional_properties").len(), 1);
    assert_eq!(
        rule(&report, "additional_properties")[0]["instance_pointer"],
        "/extra"
    );
    let report = constrained(
        json!({"surround-x-tail":"bad","free":false}),
        json!({"patternProperties":{"x":{"type":"integer"}},"additionalProperties":{"type":"boolean"}}),
        "3.1.1",
    );
    assert_eq!(rule(&report, "json_type").len(), 1);
    assert_eq!(
        rule(&report, "json_type")[0]["instance_pointer"],
        "/surround-x-tail"
    );
}

#[test]
fn nested_pattern_schema_references_and_escaped_source_locations_are_real() {
    let report = constrained(
        json!({"a/b~c":{"n":"wrong"}}),
        json!({"$defs":{"Payload":{"properties":{"n":{"type":"integer"}}}},"patternProperties":{"^a/b~c$":{"$ref":"#/components/schemas/User/$defs/Payload"}},"additionalProperties":false}),
        "3.1.1",
    );
    assert_eq!(
        rule(&report, "json_type")[0]["openapi_pointer"],
        "/components/schemas/User/$defs/Payload/properties/n/type"
    );
    assert_eq!(
        rule(&report, "json_type")[0]["instance_pointer"],
        "/a~1b~0c/n"
    );
    assert!(rule(&report, "additional_properties").is_empty());
    let report = constrained(
        json!({"a/b~c":"bad"}),
        json!({"patternProperties":{"^a/b~c$":{"type":"integer"}}}),
        "3.1.1",
    );
    assert_eq!(
        rule(&report, "json_type")[0]["openapi_pointer"],
        "/components/schemas/User/patternProperties/^a~1b~0c$/type"
    );
}

#[test]
fn unknown_regex_semantics_never_become_false_additional_property_mismatches() {
    for pattern in ["(?=x)", r"^(a)\1$", "[", "^a{100000}$"] {
        let report = constrained(
            json!({"other":"value"}),
            json!({"patternProperties":{pattern:{"type":"string"}},"additionalProperties":false}),
            "3.1.1",
        );
        assert!(!rule(&report, "invalid_pattern").is_empty(), "{report}");
        assert!(
            rule(&report, "additional_properties")
                .iter()
                .all(|o| o["classification"] == "insufficient_evidence"),
            "{report}"
        );
        assert!(
            report["observations"]
                .as_array()
                .unwrap()
                .iter()
                .all(|o| o["classification"] != "confirmed_documented_mismatch")
        );
    }
    let report = constrained(
        json!({"x":"ok"}),
        json!({"patternProperties":{"^x$":{"type":"string"}},"additionalProperties":false}),
        "3.0.4",
    );
    assert_eq!(rule(&report, "schema_dialect").len(), 1);
    assert!(
        rule(&report, "additional_properties")
            .iter()
            .all(|o| o["classification"] == "insufficient_evidence")
    );
    for declaration in [json!([]), json!({"^x$":42})] {
        let report = constrained(
            json!({"x":1}),
            json!({"patternProperties":declaration,"additionalProperties":false}),
            "3.1.1",
        );
        assert_eq!(
            rule(&report, "invalid_pattern_properties").len(),
            1,
            "{report}"
        );
        assert!(
            rule(&report, "additional_properties")
                .iter()
                .all(|o| o["classification"] == "insufficient_evidence")
        );
    }
    let report = constrained(json!([]), json!({"patternProperties":false}), "3.1.1");
    assert_eq!(rule(&report, "invalid_pattern_properties").len(), 1);
}

#[test]
fn redacted_pattern_data_and_speculative_branches_remain_inconclusive() {
    for (value, schema) in [
        (json!("[REDACTED]"), json!({"pattern":"^x$"})),
        (json!("x"), json!({"pattern":"[REDACTED]"})),
        (
            json!({"[REDACTED]":"x"}),
            json!({"patternProperties":{"^x$":false},"additionalProperties":false}),
        ),
        (json!("x"), json!({"not":{"pattern":"(?=x)"}})),
        (
            json!("x"),
            json!({"oneOf":[{"pattern":"(?=x)"},{"pattern":"^x$"}]}),
        ),
    ] {
        let report = constrained(value, schema, "3.1.1");
        assert!(!body_observations(&report).is_empty(), "{report}");
        assert!(
            report["observations"]
                .as_array()
                .unwrap()
                .iter()
                .all(|o| o["classification"] != "confirmed_documented_mismatch"),
            "{report}"
        );
    }
    let report = constrained(
        json!({"token":"synthetic-pattern-secret"}),
        json!({"patternProperties":{"token":{"type":"integer","pattern":"^x$"}}}),
        "3.1.1",
    );
    assert_eq!(rule(&report, "redacted_value").len(), 1);
    assert!(!report.to_string().contains("synthetic-pattern-secret"));
    let report = constrained(
        json!("x"),
        json!({"anyOf":[{"pattern":"^wrong$"},{"pattern":"^x$"}]}),
        "3.1.1",
    );
    assert!(body_observations(&report).is_empty(), "{report}");
}

#[test]
fn regex_compilation_matching_and_catalog_expansion_have_shared_budgets() {
    let report = constrained(json!("x"), json!({"pattern":"a".repeat(1025)}), "3.1.1");
    assert_eq!(rule(&report, "pattern_budget").len(), 1);
    let report = constrained(
        json!("a".repeat(16_384)),
        json!({"pattern":"a".repeat(1024)}),
        "3.1.1",
    );
    assert_eq!(rule(&report, "pattern_budget").len(), 1);
    assert!(rule(&report, "string_pattern").is_empty());
    let alternatives: Vec<_> = (0..65)
        .map(|i| json!({"pattern":format!("pattern-{i}")}))
        .collect();
    let report = constrained(json!("none"), json!({"allOf":alternatives}), "3.1.1");
    assert_eq!(rule(&report, "pattern_budget").len(), 1);
    assert!(report["checks"].as_u64().unwrap() <= 10_000);
    // A nested-quantifier expression uses the non-backtracking engine and is a
    // normal mismatch; a lookaround requiring backtracking is explicitly unsupported.
    let report = constrained(
        json!(format!("{}!", "a".repeat(2000))),
        json!({"pattern":"^(a+)+$"}),
        "3.1.1",
    );
    assert_eq!(rule(&report, "string_pattern").len(), 1);
    let report = constrained(json!("x"), json!({"pattern":false}), "3.1.1");
    assert_eq!(rule(&report, "invalid_pattern").len(), 1);
}

#[test]
fn engine_specific_regex_syntax_is_explicitly_outside_the_supported_subset() {
    for pattern in ["(?i)^word$", r"^\Aword\z$", "(?P<name>word)"] {
        let report = constrained(json!("WORD"), json!({"pattern":pattern}), "3.1.1");
        assert_eq!(rule(&report, "invalid_pattern").len(), 1, "{report}");
        assert!(rule(&report, "string_pattern").is_empty(), "{report}");
    }
}

#[test]
fn equality_and_alternative_expansion_share_the_global_work_budget() {
    let alternatives: Vec<_> = (0..12_000).map(|_| json!({"type":"integer"})).collect();
    let report = constrained(json!(5), json!({"anyOf":alternatives}), "3.1.1");
    assert!(report["checks"].as_u64().unwrap() <= 10_000);
    assert_eq!(
        rule(&report, "schema_composition")[0]["classification"],
        "insufficient_evidence"
    );
    let report = constrained(
        json!(5),
        json!({"enum":(0..12_000).collect::<Vec<_>>()}),
        "3.1.1",
    );
    assert!(report["checks"].as_u64().unwrap() <= 10_000);
    assert!(rule(&report, "schema_constraint").is_empty());
    assert!(!rule(&report, "resource_budget").is_empty());
}

#[test]
fn unique_items_use_json_numeric_and_structural_equality_in_both_dialects() {
    for version in ["3.0.4", "3.1.1"] {
        for (value, unique, mismatches) in [
            (json!([1, 1.0]), true, 1),
            (json!([{"n":[1]}, {"n":[1.0]}]), true, 1),
            (json!([1, true, "1", null, [], {}]), true, 0),
            (json!([9007199254740993_u64, 9007199254740992_u64]), true, 0),
            (json!([1, 1]), false, 0),
            (json!({"n":1}), true, 0),
        ] {
            let report = constrained(value, json!({"uniqueItems":unique}), version);
            assert_eq!(
                rule(&report, "schema_constraint").len(),
                mismatches,
                "{report}"
            );
            for observation in rule(&report, "schema_constraint") {
                assert_eq!(
                    observation["openapi_pointer"],
                    "/components/schemas/User/uniqueItems"
                );
            }
        }
    }
    let report = constrained(json!([1, 1]), json!({"uniqueItems":"true"}), "3.1.1");
    assert_eq!(rule(&report, "invalid_constraint").len(), 1);
}

#[test]
fn redacted_or_over_budget_uniqueness_is_inconclusive_even_inside_not() {
    let report = constrained(
        json!([{"token":"synthetic-first"}, {"token":"synthetic-second"}]),
        json!({"uniqueItems":true, "not":{"uniqueItems":true}}),
        "3.1.1",
    );
    assert!(rule(&report, "schema_constraint").is_empty(), "{report}");
    assert_eq!(
        rule(&report, "equality_constraint")[0]["classification"],
        "insufficient_evidence"
    );
    assert_eq!(
        rule(&report, "schema_not")[0]["classification"],
        "insufficient_evidence"
    );
    assert!(!report.to_string().contains("synthetic-first"));
    let report = constrained(
        json!((0..12_000).collect::<Vec<_>>()),
        json!({"uniqueItems":true}),
        "3.1.1",
    );
    assert!(report["checks"].as_u64().unwrap() <= 10_000);
    assert!(!rule(&report, "resource_budget").is_empty());
    assert!(rule(&report, "schema_constraint").is_empty());
}

#[test]
fn prefix_items_and_tail_items_use_separate_locations_and_do_not_require_length() {
    let schema =
        json!({"prefixItems":[{"type":"integer"},{"type":"string"}],"items":{"type":"boolean"}});
    for value in [
        json!([]),
        json!([1]),
        json!([1, "ok"]),
        json!([1, "ok", true, false]),
    ] {
        let report = constrained(value, schema.clone(), "3.1.1");
        assert!(body_observations(&report).is_empty(), "{report}");
    }
    let report = constrained(json!(["wrong", 3, 4]), schema, "3.1.1");
    let errors = rule(&report, "json_type");
    assert_eq!(errors.len(), 3, "{report}");
    for (i, suffix) in ["prefixItems/0/type", "prefixItems/1/type", "items/type"]
        .iter()
        .enumerate()
    {
        assert_eq!(
            errors[i]["openapi_pointer"],
            format!("/components/schemas/User/{suffix}")
        );
        assert_eq!(errors[i]["instance_pointer"], format!("/{i}"));
    }
    let report = constrained(
        json!([1, 2]),
        json!({"prefixItems":[true],"items":false}),
        "3.1.1",
    );
    assert_eq!(rule(&report, "false_schema").len(), 1);
    assert_eq!(rule(&report, "false_schema")[0]["instance_pointer"], "/1");
    let report = constrained(
        json!([1, "free tail"]),
        json!({"prefixItems":[{"type":"integer"}]}),
        "3.1.1",
    );
    assert!(body_observations(&report).is_empty(), "{report}");
}

#[test]
fn invalid_and_wrong_dialect_tuples_never_apply_an_unknown_tail_boundary() {
    for prefix in [json!([]), json!({}), json!([1])] {
        let report = constrained(
            json!([1]),
            json!({"prefixItems":prefix,"items":false}),
            "3.1.1",
        );
        assert!(!rule(&report, "invalid_tuple").is_empty(), "{report}");
        assert!(rule(&report, "false_schema").is_empty(), "{report}");
    }
    let report = constrained(json!([1]), json!({"prefixItems":[true]}), "3.0.4");
    assert_eq!(rule(&report, "schema_dialect").len(), 1);
    for version in ["3.0.4", "3.1.1"] {
        let report = constrained(json!([1]), json!({"items":[{"type":"string"}]}), version);
        assert_eq!(rule(&report, "array_items").len(), 1);
        assert!(rule(&report, "json_type").is_empty());
    }
}

#[test]
fn dependencies_are_triggered_by_presence_and_apply_to_the_whole_object() {
    let schema = json!({
        "dependentRequired":{"a/b~c":["other"]},
        "dependentSchemas":{"a/b~c":{"properties":{"value":{"type":"integer"}}}}
    });
    let report = constrained(
        json!({"a/b~c":null,"value":"wrong"}),
        schema.clone(),
        "3.1.1",
    );
    assert_eq!(
        rule(&report, "dependent_required")[0]["openapi_pointer"],
        "/components/schemas/User/dependentRequired/a~1b~0c"
    );
    assert_eq!(
        rule(&report, "dependent_required")[0]["instance_pointer"],
        "/other"
    );
    assert_eq!(
        rule(&report, "json_type")[0]["openapi_pointer"],
        "/components/schemas/User/dependentSchemas/a~1b~0c/properties/value/type"
    );
    assert_eq!(rule(&report, "json_type")[0]["instance_pointer"], "/value");
    for value in [
        json!({"value":"irrelevant"}),
        json!({"a/b~c":false,"other":0,"value":1}),
        json!([]),
    ] {
        let report = constrained(value, schema.clone(), "3.1.1");
        assert!(body_observations(&report).is_empty(), "{report}");
    }
    let report = constrained(
        json!({"token":"synthetic-secret"}),
        json!({"dependentRequired":{"token":["owner"]}}),
        "3.1.1",
    );
    assert_eq!(
        rule(&report, "dependent_required").len(),
        1,
        "A hidden value still establishes property presence: {report}"
    );
    assert!(!report.to_string().contains("synthetic-secret"));
}

#[test]
fn invalid_dependency_declarations_and_30_dependencies_are_explicit() {
    for schema in [
        json!({"dependentRequired":[]}),
        json!({"dependentRequired":{"absent":false}}),
        json!({"dependentRequired":{"absent":["x","x"]}}),
        json!({"dependentRequired":{"absent":[1]}}),
        json!({"dependentSchemas":false}),
        json!({"dependentSchemas":{"absent":[]}}),
    ] {
        let report = constrained(json!({}), schema, "3.1.1");
        assert_eq!(rule(&report, "invalid_dependency").len(), 1, "{report}");
    }
    let report = constrained(
        json!({"present":true}),
        json!({"dependentRequired":{"present":["absent"]},"dependentSchemas":{"present":false}}),
        "3.0.4",
    );
    assert_eq!(rule(&report, "schema_dialect").len(), 2);
    assert!(rule(&report, "dependent_required").is_empty());
    assert!(rule(&report, "false_schema").is_empty());
}

#[test]
fn dependent_schema_cycles_and_expansion_keep_shared_limits_and_three_way_branches() {
    let report = constrained(
        json!({"present":1}),
        json!({"dependentSchemas":{"present":{"$ref":"#/components/schemas/User"}}}),
        "3.1.1",
    );
    assert_eq!(rule(&report, "schema_reference_cycle").len(), 1, "{report}");
    let report = constrained(
        json!({"present":1}),
        json!({"oneOf":[{"dependentSchemas":{"present":{"unevaluatedProperties":false}}}]}),
        "3.1.1",
    );
    assert_eq!(
        rule(&report, "schema_composition")[0]["classification"],
        "insufficient_evidence"
    );
    let report = constrained(
        json!({"present":1}),
        json!({"dependentRequired":{"present":(0..12_000).map(|i| format!("p{i}")).collect::<Vec<_>>()}}),
        "3.1.1",
    );
    assert!(report["checks"].as_u64().unwrap() <= 10_000);
    assert!(!rule(&report, "resource_budget").is_empty());
    assert!(rule(&report, "dependent_required").is_empty());
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
