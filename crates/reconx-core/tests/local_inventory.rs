use reconx_core::*;
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
struct Service {
    url: String,
    requests: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Service {
    fn start(status: u16, delay: Duration) -> Self {
        let response = format!(
            "HTTP/1.1 {status} Fixture\r\nContent-Type: text/plain\r\nServer: owned-fixture\r\nSet-Cookie: session=fixture-private-cookie\r\nLocation: http://example.test/unfollowed\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        );
        Self::response(response.into_bytes(), delay)
    }
    fn response(response: Vec<u8>, delay: Duration) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let log = requests.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let done = stop.clone();
        let worker = thread::spawn(move || {
            while !done.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        // Winsock may inherit the listener's nonblocking mode.
                        // Read the real request before sending the fixture reply.
                        stream.set_nonblocking(false).unwrap();
                        stream
                            .set_read_timeout(Some(Duration::from_secs(1)))
                            .unwrap();
                        let mut bytes = Vec::new();
                        let mut part = [0; 1024];
                        while bytes.len() < 8192 {
                            match stream.read(&mut part) {
                                Ok(0) | Err(_) => break,
                                Ok(n) => bytes.extend_from_slice(&part[..n]),
                            }
                            if bytes.ends_with(b"\r\n\r\n") {
                                break;
                            }
                        }
                        log.lock()
                            .unwrap()
                            .push(String::from_utf8_lossy(&bytes).into_owned());
                        let until = Instant::now() + delay;
                        while Instant::now() < until && !done.load(Ordering::Acquire) {
                            thread::sleep(Duration::from_millis(10));
                        }
                        let _ = stream.write_all(&response);
                    }
                    Err(_) => thread::sleep(Duration::from_millis(5)),
                }
            }
        });
        Self {
            url,
            requests,
            stop,
            thread: Some(worker),
        }
    }
}
impl Drop for Service {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(t) = self.thread.take() {
            t.join().unwrap();
        }
    }
}
fn setup(url: &str) -> (tempfile::TempDir, Core, Target) {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::open(Some(dir.path().join("local.db"))).unwrap();
    let p = core.create_project("Owned local service", "").unwrap();
    let mut settings = core.settings().unwrap();
    settings.security_mode = SecurityMode::LabResearch;
    core.save_settings(&settings).unwrap();
    let target = core
        .create_target(TargetInput {
            project_id: p.id,
            name: "Owned fixture".into(),
            kind: TargetKind::Url,
            value: url.into(),
            description: String::new(),
            tags: vec![],
            allowed_scope: vec![],
            excluded_scope: vec![],
            authorized: true,
            mode: SecurityMode::LabResearch,
        })
        .unwrap();
    (dir, core, target)
}
#[test]
fn one_actual_head_normalizes_assets_keeps_raw_headers_private_and_survives_restart() {
    let service = Service::start(200, Duration::ZERO);
    let (dir, core, target) = setup(&service.url);
    let plan = core.local_http_preview(&target.id).unwrap();
    assert_eq!(plan.method, "HEAD");
    assert_eq!(plan.request_budget, 1);
    let job = core
        .local_http_enqueue(&target.id, &plan.fingerprint, true)
        .unwrap();
    let done = core.execute_job(&job.id).unwrap();
    assert_eq!(done.status, "completed", "{done:?}");
    assert_eq!(service.requests.lock().unwrap().len(), 1);
    assert!(service.requests.lock().unwrap()[0].starts_with("HEAD / HTTP/1.1"));
    let assets = core.assets(Some(&target.input.project_id)).unwrap();
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].facts["status"], 200);
    assert_eq!(assets[0].facts["reported_server"], "owned-fixture");
    assert!(
        !serde_json::to_string(&done)
            .unwrap()
            .contains("fixture-private-cookie")
    );
    assert!(
        !serde_json::to_string(&assets)
            .unwrap()
            .contains("fixture-private-cookie")
    );
    let evidence = core.evidence_list(None).unwrap();
    assert_eq!(evidence.len(), 2);
    let response = evidence
        .iter()
        .find(|e| e.kind == EvidenceKind::HttpResponse)
        .unwrap();
    assert!(
        core.evidence_show(&response.id)
            .unwrap()
            .text
            .unwrap()
            .contains("fixture-private-cookie")
    );
    let output = dir.path().join("redacted-http.json");
    core.evidence_export(&response.id, &output, false).unwrap();
    assert!(
        !std::fs::read_to_string(output)
            .unwrap()
            .contains("fixture-private-cookie")
    );
    let again = core
        .local_http_enqueue(&target.id, &plan.fingerprint, true)
        .unwrap();
    assert_eq!(core.execute_job(&again.id).unwrap().status, "completed");
    assert_eq!(core.assets(None).unwrap()[0].id, assets[0].id);
    let reopened = Core::open(Some(core.database_path().into())).unwrap();
    assert_eq!(reopened.assets(None).unwrap().len(), 1);
    assert_eq!(reopened.evidence_list(None).unwrap().len(), 4);
}
#[test]
fn redirects_do_not_create_additional_requests_and_localhost_is_pinned() {
    let service = Service::start(302, Duration::ZERO);
    let url = service.url.replace("127.0.0.1", "localhost");
    let (_dir, core, target) = setup(&url);
    let plan = core.local_http_preview(&target.id).unwrap();
    let job = core
        .local_http_enqueue(&target.id, &plan.fingerprint, true)
        .unwrap();
    let done = core.execute_job(&job.id).unwrap();
    assert_eq!(done.status, "completed", "{done:?}");
    assert_eq!(service.requests.lock().unwrap().len(), 1);
    assert_eq!(core.assets(None).unwrap()[0].facts["status"], 302);
}
#[test]
fn unauthorized_public_stale_excluded_and_passive_policies_never_send_traffic() {
    let service = Service::start(200, Duration::ZERO);
    let (_dir, core, mut target) = setup(&service.url);
    let plan = core.local_http_preview(&target.id).unwrap();
    assert_eq!(
        core.local_http_enqueue(&target.id, &plan.fingerprint, false)
            .unwrap_err()
            .code,
        "BLOCKED_BY_POLICY"
    );
    assert_eq!(
        core.local_http_enqueue(&target.id, "invented", true)
            .unwrap_err()
            .code,
        "APPROVAL_STALE"
    );
    let queued = core
        .local_http_enqueue(&target.id, &plan.fingerprint, true)
        .unwrap();
    target.input.authorized = false;
    core.update_target(&target.id, target.input.clone())
        .unwrap();
    assert_eq!(core.execute_job(&queued.id).unwrap().status, "failed");
    target.input.authorized = true;
    target.input.excluded_scope = vec![service.url.clone()];
    core.update_target(&target.id, target.input.clone())
        .unwrap();
    assert!(core.local_http_preview(&target.id).is_err());
    target.input.excluded_scope.clear();
    target.input.value = "http://example.test/".into();
    core.update_target(&target.id, target.input.clone())
        .unwrap();
    assert!(core.local_http_preview(&target.id).is_err());
    target.input.value = service.url.clone();
    core.update_target(&target.id, target.input).unwrap();
    let mut settings = core.settings().unwrap();
    settings.security_mode = SecurityMode::PassiveAuto;
    core.save_settings(&settings).unwrap();
    assert!(core.local_http_preview(&target.id).is_err());
    assert!(service.requests.lock().unwrap().is_empty());
    assert!(core.assets(None).unwrap().is_empty());
}
#[test]
fn approved_budget_is_bounded_and_cancellation_leaves_no_asset_or_evidence() {
    let service = Service::start(200, Duration::from_secs(3));
    let (_dir, core, target) = setup(&service.url);
    let plan = core.local_http_preview(&target.id).unwrap();
    let job = core
        .local_http_enqueue(&target.id, &plan.fingerprint, true)
        .unwrap();
    let runner = core.clone();
    let id = job.id.clone();
    let work = thread::spawn(move || runner.execute_job(&id).unwrap());
    let until = Instant::now() + Duration::from_secs(3);
    while service.requests.lock().unwrap().is_empty() {
        assert!(Instant::now() < until);
        thread::sleep(Duration::from_millis(10));
    }
    core.cancel_job(&job.id).unwrap();
    assert_eq!(work.join().unwrap().status, "cancelled");
    assert!(core.assets(None).unwrap().is_empty());
    assert!(core.evidence_list(None).unwrap().is_empty());
    for _ in 0..19 {
        core.local_http_enqueue(&target.id, &plan.fingerprint, true)
            .unwrap();
    }
    assert_eq!(
        core.local_http_enqueue(&target.id, &plan.fingerprint, true)
            .unwrap_err()
            .code,
        "BLOCKED_BY_POLICY"
    );
}
#[test]
fn unavailable_service_fails_without_fabricating_results() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    drop(listener);
    let (_dir, core, target) = setup(&url);
    let plan = core.local_http_preview(&target.id).unwrap();
    let job = core
        .local_http_enqueue(&target.id, &plan.fingerprint, true)
        .unwrap();
    let done = core.execute_job(&job.id).unwrap();
    assert_eq!(done.status, "failed");
    assert_eq!(done.error_code.as_deref(), Some("HTTP_FAILED"));
    assert!(core.assets(None).unwrap().is_empty());
    assert!(core.evidence_list(None).unwrap().is_empty());
}

#[test]
fn opaque_sensitive_headers_are_preserved_privately_but_redacted_on_export() {
    let service = Service::response(b"HTTP/1.1 200 OK\r\nSet-Cookie: session=\xff\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec(), Duration::ZERO);
    let (dir, core, target) = setup(&service.url);
    let plan = core.local_http_preview(&target.id).unwrap();
    let job = core
        .local_http_enqueue(&target.id, &plan.fingerprint, true)
        .unwrap();
    assert_eq!(core.execute_job(&job.id).unwrap().status, "completed");
    let evidence = core
        .evidence_list(None)
        .unwrap()
        .into_iter()
        .find(|e| e.kind == EvidenceKind::HttpResponse)
        .unwrap();
    let text = core.evidence_show(&evidence.id).unwrap().text.unwrap();
    assert!(text.contains("value_base64"));
    let original: serde_json::Value = serde_json::from_str(&text).unwrap();
    let header = original["response"]["headers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["name"] == "set-cookie")
        .unwrap();
    let encoded = header["value_base64"].as_str().unwrap();
    let out = dir.path().join("safe.json");
    core.evidence_export(&evidence.id, &out, false).unwrap();
    let exported = std::fs::read_to_string(out).unwrap();
    assert!(!exported.contains(encoded));
    assert!(exported.contains("[REDACTED]"));
    let report = core
        .report_prepare(ReportInput {
            project_id: target.input.project_id,
            title: "Actual opaque header observation".into(),
            template: ReportTemplate::Research,
            summary: String::new(),
            methodology: String::new(),
            finding_ids: vec![],
            evidence_ids: vec![evidence.id],
        })
        .unwrap();
    assert!(!report.document.text.contains(encoded));
    assert!(
        !serde_json::to_string(&report.document)
            .unwrap()
            .contains(encoded)
    );
}
#[test]
fn timeout_and_excessive_headers_fail_without_publishing_partial_results() {
    for service in [
        Service::start(200, Duration::from_secs(6)),
        Service::response(
            format!(
                "HTTP/1.1 200 OK\r\nX-Excess: {}\r\nContent-Length: 0\r\n\r\n",
                "x".repeat(80 * 1024)
            )
            .into_bytes(),
            Duration::ZERO,
        ),
    ] {
        let (_dir, core, target) = setup(&service.url);
        let plan = core.local_http_preview(&target.id).unwrap();
        let job = core
            .local_http_enqueue(&target.id, &plan.fingerprint, true)
            .unwrap();
        let begin = Instant::now();
        let done = core.execute_job(&job.id).unwrap();
        assert_eq!(done.status, "failed", "{done:?}");
        assert!(begin.elapsed() < Duration::from_secs(8));
        assert!(
            matches!(
                done.error_code.as_deref(),
                Some("HTTP_FAILED" | "INPUT_LIMIT" | "JOB_TIMEOUT")
            ),
            "{done:?}"
        );
        assert!(core.assets(None).unwrap().is_empty());
        assert!(core.evidence_list(None).unwrap().is_empty());
        assert_eq!(service.requests.lock().unwrap().len(), 1);
    }
}
