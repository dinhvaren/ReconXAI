use reconx_core::{
    Core, JobEngine, JobReview, McpApproval, McpServerConfig, PlanProposal, PlanReview, PlanStep,
    Result, SecretStore,
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::{Read, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use zeroize::Zeroizing;
#[derive(Default)]
struct Secrets(Mutex<HashMap<String, String>>);
impl SecretStore for Secrets {
    fn set(&self, id: &str, s: &str) -> Result<()> {
        self.0.lock().unwrap().insert(id.into(), s.into());
        Ok(())
    }
    fn get(&self, id: &str) -> Result<Option<Zeroizing<String>>> {
        Ok(self.0.lock().unwrap().get(id).cloned().map(Zeroizing::new))
    }
    fn delete(&self, id: &str) -> Result<()> {
        self.0.lock().unwrap().remove(id);
        Ok(())
    }
}
fn node() -> PathBuf {
    std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|p| p.join(if cfg!(windows) { "node.exe" } else { "node" }))
        .find(|p| p.is_file())
        .expect("Node is required by the local MCP fixture")
}
fn config(id: &str) -> McpServerConfig {
    McpServerConfig {
        id: id.into(),
        name: "Owned fixture".into(),
        transport: "stdio".into(),
        executable: Some(node().to_string_lossy().into()),
        args: vec![
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/e2e/mcp-fixture.mjs")
                .to_string_lossy()
                .into(),
        ],
        working_directory: None,
        endpoint: None,
        environment: vec![],
        bearer_auth: false,
        enabled: true,
        trusted: true,
        timeout_ms: 3000,
        allowed_tools: vec!["echo".into()],
    }
}
fn setup() -> (tempfile::TempDir, Core, String, String) {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::with_secrets(
        Some(dir.path().join("test.db")),
        Arc::new(Secrets::default()),
    )
    .unwrap();
    let p = core.create_project("Integration fixture", "").unwrap();
    let path = dir.path().join("owned.bin");
    std::fs::write(&path, b"RECONX_BENIGN_FIXTURE\0hello").unwrap();
    let f = core.import_file(&p.id, &path, true).unwrap();
    (dir, core, p.id, f.id)
}
fn done(core: &Core, id: &str) -> reconx_core::AnalysisJob {
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        let j = core.job(id).unwrap();
        if !matches!(j.status.as_str(), "running" | "queued") {
            return j;
        }
        assert!(Instant::now() < until, "job stuck {j:?}");
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn approve(core: &Core, project: &str, input: Value) -> reconx_core::AnalysisJob {
    let p = core
        .mcp_invocation_preview("fixture", "echo", &input)
        .unwrap();
    core.mcp_enqueue(McpApproval {
        server_id: "fixture".into(),
        tool: "echo".into(),
        input,
        project_id: project.into(),
        fingerprint: p["fingerprint"].as_str().unwrap().into(),
        approved: true,
    })
    .unwrap()
}

#[tokio::test]
async fn real_mcp_stdio_discovery_approval_job_and_restart() {
    let (dir, core, project, _) = setup();
    core.mcp_save(config("fixture")).await.unwrap();
    let s = core.mcp_connect("fixture").await.unwrap();
    assert_eq!(s.tools.len(), 2);
    assert_eq!(core.mcp_state("fixture").await, "connected");
    assert!(
        core.capabilities(&project)
            .unwrap()
            .iter()
            .any(|c| c["provider"] == "mcp" && c["available"] == true)
    );
    assert_eq!(
        core.mcp_invocation_preview("fixture", "echo", &json!({"text":15}))
            .unwrap_err()
            .code,
        "MCP_INPUT"
    );
    assert_eq!(
        core.mcp_invocation_preview("fixture", "denied", &json!({}))
            .unwrap_err()
            .code,
        "BLOCKED_BY_POLICY"
    );
    let j = approve(&core, &project, json!({"text":"actual fixture result"}));
    let engine = JobEngine::start(core.clone(), 1, |_| {}).unwrap();
    let out = done(&core, &j.id);
    assert_eq!(out.status, "completed", "{out:?}");
    let text = out.result.as_ref().unwrap().to_string();
    assert!(text.contains("actual fixture result"));
    assert!(text.contains("\"host_secret_inherited\":false"));
    drop(engine);
    core.mcp_disconnect("fixture").await.unwrap();
    assert_eq!(core.mcp_state("fixture").await, "disconnected");
    let reopened = Core::with_secrets(
        Some(dir.path().join("test.db")),
        Arc::new(Secrets::default()),
    )
    .unwrap();
    assert_eq!(reopened.mcp_servers().unwrap().len(), 1);
    assert_eq!(
        reopened.mcp_history("fixture").unwrap()[0].status,
        "completed"
    );
    assert_eq!(reopened.job(&j.id).unwrap().result, out.result);
}
#[tokio::test]
async fn mcp_trust_schema_stale_permissions_timeout_and_cancel() {
    let (_dir, core, project, _) = setup();
    let mut c = config("fixture");
    c.trusted = false;
    core.mcp_save(c.clone()).await.unwrap();
    assert_eq!(
        core.mcp_connect("fixture").await.unwrap_err().code,
        "MCP_TRUST_REQUIRED"
    );
    c.trusted = true;
    c.args.push("--bad-schema".into());
    core.mcp_save(c.clone()).await.unwrap();
    assert_eq!(
        core.mcp_connect("fixture").await.unwrap_err().code,
        "MCP_SCHEMA"
    );
    c.args.pop();
    // A 500 ms process-start budget can fail before the tested invocation on
    // loaded Windows hosts. Keep a bounded startup budget and make the tool
    // deliberately exceed it, then prove the job actually reached invocation.
    c.timeout_ms = 3000;
    core.mcp_save(c.clone()).await.unwrap();
    core.mcp_connect("fixture").await.unwrap();
    let stale = approve(&core, &project, json!({"text":"stale"}));
    c.allowed_tools.clear();
    core.mcp_save(c.clone()).await.unwrap();
    let engine = JobEngine::start(core.clone(), 1, |_| {}).unwrap();
    assert_eq!(done(&core, &stale.id).status, "failed");
    drop(engine);
    c.allowed_tools = vec!["echo".into()];
    core.mcp_save(c).await.unwrap();
    core.mcp_connect("fixture").await.unwrap();
    let timeout = approve(&core, &project, json!({"text":"delay","delay_ms":10000}));
    let engine = JobEngine::start(core.clone(), 1, |_| {}).unwrap();
    let timed_out = done(&core, &timeout.id);
    assert_eq!(timed_out.status, "failed");
    assert_eq!(timed_out.error_code.as_deref(), Some("JOB_TIMEOUT"));
    assert!(
        timed_out.progress >= 30,
        "Timeout must interrupt a real tool invocation: {timed_out:?}"
    );
    // Cancellation tests need enough time to observe a real in-flight call.
    // Reusing the short timeout could fail during process startup on loaded
    // CI, before the tool was ever called; that tests timeout, not cancellation.
    drop(engine);
    let mut cancel_config = config("fixture");
    cancel_config.timeout_ms = 15_000;
    core.mcp_save(cancel_config).await.unwrap();
    core.mcp_connect("fixture").await.unwrap();
    let cancel = approve(&core, &project, json!({"text":"cancel","delay_ms":10000}));
    let engine = JobEngine::start(core.clone(), 1, |_| {}).unwrap();
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        let current = core.job(&cancel.id).unwrap();
        assert!(
            matches!(current.status.as_str(), "queued" | "running"),
            "Cancellation fixture ended before reaching the tool call: {current:?}"
        );
        if current.progress >= 30 {
            break;
        }
        assert!(Instant::now() < until, "No in-flight MCP call: {current:?}");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    core.cancel_job(&cancel.id).unwrap();
    let j = done(&core, &cancel.id);
    assert_eq!(j.status, "cancelled");
    assert!(j.result.is_none());
    drop(engine);
    core.mcp_disconnect("fixture").await.unwrap();
}
#[test]
fn plan_validation_cannot_select_shell_network_or_unavailable_tools() {
    let (_dir, core, _, file) = setup();
    for id in [
        "shell.exec",
        "nuclei.scan",
        "mcp.fixture.echo",
        "yara-x.signatures",
    ] {
        assert!(
            core.validate_local_plan(
                &file,
                &PlanProposal {
                    steps: vec![PlanStep {
                        capability_id: id.into(),
                        reason: "requested".into()
                    }]
                }
            )
            .is_err()
        );
    }
    let step = PlanStep {
        capability_id: "local.metadata".into(),
        reason: "Inspect an owned sample".into(),
    };
    core.validate_local_plan(
        &file,
        &PlanProposal {
            steps: vec![step.clone()],
        },
    )
    .unwrap();
    assert!(
        core.validate_local_plan(
            &file,
            &PlanProposal {
                steps: vec![step.clone(), step]
            }
        )
        .is_err()
    );
    assert!(
        core.tool_profile_plan("reverse")
            .unwrap()
            .iter()
            .any(|m| m.id == "yara-x")
    );
    assert_eq!(
        core.tool_profile_plan("full").unwrap_err().code,
        "PROFILE_INCOMPLETE"
    );
}
#[tokio::test]
async fn mcp_http_bearer_auth_and_real_job_output_are_bounded() {
    use std::io::BufRead;
    let (_dir, core, project, _) = setup();
    let c = config("fixture");
    let mut child = std::process::Command::new(node())
        .args(&c.args)
        .args(["--http", "--auth"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let mut line = String::new();
    std::io::BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    let port: Value = serde_json::from_str(&line).unwrap();
    let mut http = c.clone();
    http.transport = "streamable_http".into();
    http.executable = None;
    http.args.clear();
    http.endpoint = Some(format!("http://127.0.0.1:{}/mcp", port["port"]));
    http.bearer_auth = true;
    let scenario = async {
        core.mcp_save(http).await.unwrap();
        assert_eq!(
            core.mcp_connect("fixture").await.unwrap_err().code,
            "MCP_CREDENTIAL_MISSING"
        );
        core.mcp_set_secret("fixture", "bearer", "invalid-fixture-token")
            .unwrap();
        assert!(core.mcp_connect("fixture").await.is_err());
        core.mcp_set_secret("fixture", "bearer", "reconx-fixture-token")
            .unwrap();
        assert_eq!(core.mcp_connect("fixture").await.unwrap().tools.len(), 2);
        assert!(
            !serde_json::to_string(&core.mcp_server("fixture").unwrap())
                .unwrap()
                .contains("reconx-fixture-token")
        );
        let job = approve(&core, &project, json!({"text":"actual HTTP result"}));
        let engine = JobEngine::start(core.clone(), 1, |_| {}).unwrap();
        let output = done(&core, &job.id);
        drop(engine);
        assert_eq!(output.status, "completed", "{output:?}");
        assert!(
            output
                .result
                .unwrap()
                .to_string()
                .contains("actual HTTP result")
        );
        core.mcp_disconnect("fixture").await.unwrap();
    };
    // Always reap this owned server, including when an assertion fails.
    use futures::FutureExt;
    let outcome = std::panic::AssertUnwindSafe(scenario).catch_unwind().await;
    let _ = child.kill();
    let _ = child.wait();
    if let Err(e) = outcome {
        std::panic::resume_unwind(e);
    }
}
#[tokio::test]
async fn missing_executable_unavailable_server_and_oversized_output_fail_truthfully() {
    let (_dir, core, project, _) = setup();
    let mut c = config("fixture");
    c.executable = Some("/missing/reconx-fixture-executable".into());
    assert_eq!(core.mcp_save(c).await.unwrap_err().code, "MCP_EXECUTABLE");
    let mut c = config("fixture");
    c.args.push("--large-output".into());
    core.mcp_save(c).await.unwrap();
    core.mcp_connect("fixture").await.unwrap();
    let job = approve(&core, &project, json!({"text":"bounded"}));
    let engine = JobEngine::start(core.clone(), 1, |_| {}).unwrap();
    let out = done(&core, &job.id);
    drop(engine);
    assert_eq!(out.error_code.as_deref(), Some("MCP_OUTPUT_LIMIT"));
    assert!(out.result.is_none());
    core.mcp_disconnect("fixture").await.unwrap();
    let mut c = config("offline");
    c.transport = "streamable_http".into();
    c.executable = None;
    c.args.clear();
    c.endpoint = Some("http://127.0.0.1:1/mcp".into());
    c.timeout_ms = 200;
    core.mcp_save(c).await.unwrap();
    assert!(core.mcp_connect("offline").await.is_err());
}
#[tokio::test]
async fn mcp_secrets_are_redacted_from_discovery_and_results_including_escaped_values() {
    let (_dir, core, project, _) = setup();
    let mut c = config("fixture");
    c.environment = vec!["MCP_TEST_TOKEN".into()];
    c.args.push("--echo-server-secret".into());
    core.mcp_save(c).await.unwrap();
    let secret = "fixture-quote\"-slash\\-private";
    core.mcp_set_secret("fixture", "MCP_TEST_TOKEN", secret)
        .unwrap();
    let discovered = core.mcp_connect("fixture").await.unwrap();
    assert_eq!(discovered.tools[0]["description"], "[REDACTED]");
    assert_eq!(
        core.mcp_invocation_preview(
            "fixture",
            "echo",
            &json!({"text":"safe","api_key":"never persisted"})
        )
        .unwrap_err()
        .code,
        "MCP_SECRET_INPUT"
    );
    let job = approve(&core, &project, json!({"text":"safe"}));
    let engine = JobEngine::start(core.clone(), 1, |_| {}).unwrap();
    let result = done(&core, &job.id);
    drop(engine);
    assert_eq!(result.status, "completed", "{result:?}");
    assert_eq!(
        result.result.as_ref().unwrap()["response"]["content"][0]["text"],
        "[REDACTED]"
    );
    assert!(
        !serde_json::to_string(&core.mcp_history("fixture").unwrap())
            .unwrap()
            .contains("fixture-quote")
    );
    core.mcp_disconnect("fixture").await.unwrap();
}
#[tokio::test]
async fn mock_model_plan_runs_actual_local_jobs_and_interprets_persisted_evidence() {
    let (_dir, core, project, file) = setup();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let gateway = format!("http://{}/v1", listener.local_addr().unwrap());
    let calls = Arc::new(Mutex::new(Vec::<String>::new()));
    let record = calls.clone();
    let server = std::thread::spawn(move || {
        for n in 0..2 {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut b = Vec::new();
            let mut data = [0; 4096];
            loop {
                let len = stream.read(&mut data).unwrap();
                b.extend_from_slice(&data[..len]);
                if let Some(at) = b.windows(4).position(|w| w == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&b[..at]);
                    let length = headers
                        .lines()
                        .find_map(|l| {
                            l.to_lowercase()
                                .strip_prefix("content-length:")
                                .map(|s| s.trim().parse::<usize>().unwrap())
                        })
                        .unwrap_or(0);
                    if b.len() >= at + 4 + length {
                        break;
                    }
                }
            }
            record
                .lock()
                .unwrap()
                .push(String::from_utf8_lossy(&b).into_owned());
            let content = if n == 0 {
                json!({"steps":[{"capability_id":"local.metadata","reason":"Read actual metadata"},{"capability_id":"local.index","reason":"Index verified sample"}]}).to_string()
            } else {
                "Fixture model interpretation of actual persisted evidence".into()
            };
            let body = json!({"choices":[{"message":{"content":content}}]}).to_string();
            write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body).unwrap();
        }
    });
    let provider = reconx_core::ProviderConfig {
        id: "ac69c519-72c4-402d-a59a-5368db1a7766".into(),
        name: "Deterministic mock adapter".into(),
        kind: reconx_core::ProviderKind::Compatible,
        base_url: gateway.clone(),
        model: "fixture".into(),
        enabled: true,
        timeout_secs: 5,
        max_output_tokens: 512,
        requires_key: false,
    };
    core.save_provider(&provider).unwrap();
    let preview = core.plan_preview(&file).unwrap();
    let review = |consent| PlanReview {
        file_id: file.clone(),
        provider_id: provider.id.clone(),
        gateway: gateway.clone(),
        preview: preview.clone(),
        consent,
    };
    assert_eq!(
        core.plan_local_tools(review(false)).await.unwrap_err().code,
        "CONSENT_REQUIRED"
    );
    assert!(calls.lock().unwrap().is_empty());
    let plan = core.plan_local_tools(review(true)).await.unwrap();
    assert_eq!(
        core.approve_tool_plan(&plan.id, false).unwrap_err().code,
        "APPROVAL_REQUIRED"
    );
    let jobs = core.approve_tool_plan(&plan.id, true).unwrap();
    assert_eq!(
        core.approve_tool_plan(&plan.id, true).unwrap_err().code,
        "PLAN_ALREADY_EXECUTED"
    );
    let engine = JobEngine::start(core.clone(), 2, |_| {}).unwrap();
    for j in &jobs {
        assert_eq!(done(&core, &j.id).status, "completed");
    }
    drop(engine);
    let preview = core.job_evidence_preview(&jobs[0].id).unwrap();
    let reply = core
        .interpret_job(JobReview {
            job_id: jobs[0].id.clone(),
            provider_id: provider.id,
            gateway,
            preview,
            consent: true,
        })
        .await
        .unwrap();
    assert!(reply.text.contains("actual persisted evidence"));
    assert_eq!(core.job_interpretations(&jobs[0].id).unwrap().len(), 1);
    assert_eq!(core.tool_plans(&project).unwrap().len(), 1);
    server.join().unwrap();
    let calls = calls.lock().unwrap();
    assert_eq!(calls.len(), 2);
    assert!(!calls[0].contains("owned.bin"));
    assert!(!calls[0].contains("source_path"));
}
