use reconx_core::*;
use serde_json::json;

#[test]
fn httpx_parser_requires_exact_pinned_response_and_redacts_metadata() {
    let uri = "http://127.0.0.1:3456/";
    let record = json!({"url":uri,"status_code":200,"title":"Authorization: Bearer fixture-token-secret","content_type":"text/html","time":"3ms","host":"127.0.0.1","tech":["fixture"],"tls":null});
    let result = parse_httpx_output(&record.to_string(), uri).unwrap();
    assert_eq!(result["status"], 200);
    assert!(!result.to_string().contains("fixture-token-secret"));
    for text in [
        String::new(),
        "not-json".into(),
        format!("{record}\n{record}"),
        json!({"url":"https://outside.invalid/","status_code":200}).to_string(),
        json!({"url":uri,"status_code":200,"failed":true}).to_string(),
    ] {
        assert!(parse_httpx_output(&text, uri).is_err());
    }
    assert!(parse_httpx_output(&"x".repeat(1024 * 1024 + 1), uri).is_err());
}
#[test]
fn passive_parser_deduplicates_without_inventing_source_success() {
    let input = "{\"host\":\"api.example.test\",\"source\":\"crtsh\"}\n{\"host\":\"API.EXAMPLE.TEST\",\"sources\":[\"crtsh\",\"certspotter\"]}\n";
    let rows = parse_subfinder_output(input, "example.test").unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows["api.example.test"], vec!["crtsh", "certspotter"]);
    assert!(
        parse_subfinder_output("", "example.test")
            .unwrap()
            .is_empty()
    );
    for line in [
        "malformed",
        r#"{"host":"example.test.evil.test"}"#,
        r#"{"host":"-option.example.test"}"#,
        r#"{"sources":["crtsh"]}"#,
    ] {
        assert!(parse_subfinder_output(line, "example.test").is_err());
    }
    assert!(parse_subfinder_output(&"x".repeat(1024 * 1024 + 1), "example.test").is_err());
    let excessive_hosts = (0..1001)
        .map(|i| json!({"host":format!("h{i}.example.test")}).to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(parse_subfinder_output(&excessive_hosts, "example.test").is_err());
}
#[test]
fn recon_plans_are_offline_and_require_authorization_tools_and_budgets() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::open(Some(dir.path().join("fixture.db"))).unwrap();
    let project = core.create_project("Recon policy fixture", "").unwrap();
    let mut target:TargetInput=serde_json::from_value(json!({"project_id":project.id,"name":"Owned local fixture","kind":"url","value":"http://127.0.0.1:1/","authorized":true,"mode":"lab_research"})).unwrap();
    let t = core.create_target(target.clone()).unwrap();
    let mut settings = core.settings().unwrap();
    settings.security_mode = SecurityMode::LabResearch;
    core.save_settings(&settings).unwrap();
    let mut c = ReconConfig::for_target(&t.id, ReconTool::Httpx);
    assert_eq!(
        core.recon_plan(c.clone()).unwrap_err().code,
        "DEPENDENCY_UNAVAILABLE"
    );
    c.max_assets = 9;
    assert_eq!(core.recon_plan(c.clone()).unwrap_err().code, "RECON_POLICY");
    c.max_assets = 4;
    target.authorized = false;
    core.update_target(&t.id, target).unwrap();
    assert_eq!(core.recon_plan(c).unwrap_err().code, "RECON_POLICY");
    let d:TargetInput=serde_json::from_value(json!({"project_id":project.id,"name":"Passive fixture","kind":"domain","value":"example.test","authorized":true})).unwrap();
    let t = core.create_target(d).unwrap();
    let p = ReconConfig::for_target(&t.id, ReconTool::Subfinder);
    assert_eq!(
        core.recon_plan(p).unwrap_err().code,
        "DEPENDENCY_UNAVAILABLE"
    );
}
#[test]
fn marketplace_recon_manifests_are_compiled_and_not_ready_without_health() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::open(Some(dir.path().join("fixture.db"))).unwrap();
    for id in ["httpx", "subfinder"] {
        let e = core.extension_entry(id).unwrap();
        assert_eq!(e.status, "not_installed");
        validate_extension_manifest(&e.manifest).unwrap();
        assert!(e.dependencies.iter().all(|t| !t.integrity_verified));
        let mut corrupt = e.manifest;
        corrupt.entrypoint = "shell:arbitrary".into();
        assert!(validate_extension_manifest(&corrupt).is_err());
    }
}
