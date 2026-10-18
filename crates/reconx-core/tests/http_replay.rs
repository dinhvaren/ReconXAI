use reconx_core::*;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};
use zeroize::Zeroizing;

#[derive(Default)]
struct Memory {
    values: Mutex<HashMap<String, String>>,
    fail_delete: AtomicBool,
}
impl SecretStore for Memory {
    fn set(&self, id: &str, key: &str) -> Result<()> {
        self.values.lock().unwrap().insert(id.into(), key.into());
        Ok(())
    }
    fn get(&self, id: &str) -> Result<Option<Zeroizing<String>>> {
        Ok(self
            .values
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .map(Zeroizing::new))
    }
    fn delete(&self, id: &str) -> Result<()> {
        if self.fail_delete.load(Ordering::Acquire) {
            return Err(AppError::new(
                "SECRET_STORE_UNAVAILABLE",
                "Controlled fixture denial",
            ));
        }
        self.values.lock().unwrap().remove(id);
        Ok(())
    }
}
struct Server {
    url: String,
    hits: Arc<AtomicUsize>,
    last: Arc<Mutex<Value>>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Server {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let hits = Arc::new(AtomicUsize::new(0));
        let last = Arc::new(Mutex::new(Value::Null));
        let stop = Arc::new(AtomicBool::new(false));
        let (h, l, s) = (hits.clone(), last.clone(), stop.clone());
        let task = thread::spawn(move || {
            while !s.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        h.fetch_add(1, Ordering::AcqRel);
                        respond(stream, &l);
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5))
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            url,
            hits,
            last,
            stop,
            thread: Some(task),
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(t) = self.thread.take() {
            t.join().unwrap();
        }
    }
}
fn respond(mut stream: TcpStream, last: &Mutex<Value>) {
    // Windows accepted sockets can inherit the listener's nonblocking mode.
    // This fixture reads a complete request using blocking I/O and timeouts.
    stream.set_nonblocking(false).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 4096];
    let header_end;
    loop {
        let Ok(n) = stream.read(&mut chunk) else {
            return;
        };
        if n == 0 {
            return;
        }
        bytes.extend_from_slice(&chunk[..n]);
        if let Some(i) = bytes.windows(4).position(|b| b == b"\r\n\r\n") {
            header_end = i + 4;
            break;
        }
        if bytes.len() > 100_000 {
            return;
        }
    }
    let head = String::from_utf8_lossy(&bytes[..header_end]);
    let mut lines = head.lines();
    let request = lines.next().unwrap_or("").to_owned();
    let headers: HashMap<_, _> = lines
        .filter_map(|l| {
            l.split_once(':')
                .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_owned()))
        })
        .collect();
    let length = headers
        .get("content-length")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);
    while bytes.len() < header_end + length {
        let Ok(n) = stream.read(&mut chunk) else {
            return;
        };
        if n == 0 {
            return;
        }
        bytes.extend_from_slice(&chunk[..n]);
    }
    let raw_body = String::from_utf8_lossy(&bytes[header_end..]).to_string();
    *last.lock().unwrap() = json!({"request":request,"headers":headers,"body":raw_body});
    let target = request.split_whitespace().nth(1).unwrap_or("/");
    if target.starts_with("/slow") {
        thread::sleep(Duration::from_millis(1600));
    }
    let credential = headers
        .get("authorization")
        .or_else(|| headers.get("x-lab-key"))
        .or_else(|| headers.get("cookie"))
        .map(String::as_str)
        .unwrap_or("");
    let owner = if credential.contains("bob") {
        "bob"
    } else {
        "alice"
    };
    let mut body = json!({"owner":owner,"id":1,"echo":credential,"body":raw_body}).to_string();
    let mut status = "200 OK";
    let mut extra = String::new();
    if target.starts_with("/redirect") {
        status = "302 Found";
        extra = "Location: http://192.0.2.1/never-follow\r\n".into();
    }
    if target.starts_with("/large") {
        body = "x".repeat(300_000);
    }
    if target.starts_with("/denied") {
        status = "401 Unauthorized";
    }
    if request.starts_with("POST ") {
        status = "201 Created";
    }
    if target.starts_with("/chunked-large") {
        let mut reply=b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n".to_vec();
        for _ in 0..70 {
            reply.extend_from_slice(b"1000\r\n");
            reply.extend_from_slice(&[b'x'; 4096]);
            reply.extend_from_slice(b"\r\n");
        }
        reply.extend_from_slice(b"0\r\n\r\n");
        let _ = stream.write_all(&reply);
        let _ = stream.shutdown(std::net::Shutdown::Write);
        let _ = stream.read(&mut [0u8; 1]);
        return;
    }
    let reply = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{extra}\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(reply.as_bytes());
    let _ = stream.shutdown(std::net::Shutdown::Write);
    let _ = stream.read(&mut [0u8; 1]);
}
#[test]
fn chunked_fixture_sends_valid_http_frames() {
    let server = Server::start();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let bytes = runtime.block_on(async {
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .http1_only()
            .http1_max_headers(64)
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .no_zstd()
            .build()
            .unwrap();
        let mut response = client
            .get(format!("{}chunked-large", server.url))
            .send()
            .await
            .unwrap();
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.unwrap() {
            bytes.extend_from_slice(&chunk);
            thread::sleep(Duration::from_millis(5));
        }
        drop(client);
        bytes
    });
    assert_eq!(bytes.len(), 70 * 4096);
}
struct Setup {
    dir: tempfile::TempDir,
    core: Core,
    project: String,
    target: String,
    secrets: Arc<Memory>,
    server: Server,
}
fn setup() -> Setup {
    let dir = tempfile::tempdir().unwrap();
    let secrets = Arc::new(Memory::default());
    let core = Core::with_secrets(Some(dir.path().join("replay.db")), secrets.clone()).unwrap();
    let project = core.create_project("Owned local API", "").unwrap().id;
    let server = Server::start();
    let target = core
        .create_target(TargetInput {
            project_id: project.clone(),
            name: "Disposable local fixture".into(),
            kind: TargetKind::RestApi,
            value: server.url.clone(),
            description: String::new(),
            tags: vec![],
            allowed_scope: vec![],
            excluded_scope: vec![],
            authorized: true,
            mode: SecurityMode::LabResearch,
        })
        .unwrap()
        .id;
    let mut settings = core.settings().unwrap();
    settings.security_mode = SecurityMode::LabResearch;
    core.save_settings(&settings).unwrap();
    Setup {
        dir,
        core,
        project,
        target,
        secrets,
        server,
    }
}
fn input(s: &Setup, path: &str) -> HttpDraftInput {
    HttpDraftInput {
        project_id: s.project.clone(),
        target_id: s.target.clone(),
        source_exchange_id: None,
        title: "Reviewed local request".into(),
        method: "GET".into(),
        url: format!("{}{}", s.server.url, path),
        headers: vec![],
        content_type: String::new(),
        body: String::new(),
        auth_profile_id: None,
        timeout_ms: 3000,
        follow_redirects: false,
        proxy: None,
    }
}
fn queue(core: &Core, draft: &HttpDraft) -> AnalysisJob {
    let p = core.http_replay_preview(&draft.id).unwrap();
    core.http_replay_enqueue(HttpReplayApproval {
        draft_id: draft.id.clone(),
        fingerprint: p.fingerprint,
        approved: true,
    })
    .unwrap()
}
fn auth(s: &Setup, kind: HttpAuthKind, secret: &str) -> HttpAuthProfile {
    s.core
        .http_auth_save(
            None,
            HttpAuthInput {
                project_id: s.project.clone(),
                target_id: s.target.clone(),
                name: "Alice".into(),
                role: "user".into(),
                kind,
                header_name: "x-lab-key".into(),
            },
            None,
            Some(Zeroizing::new(secret.into())),
        )
        .unwrap()
}

#[test]
fn imported_request_edit_real_send_evidence_and_restart_use_shared_traffic() {
    let s = setup();
    let mut har: Value = serde_json::from_slice(
        &std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/http/alice.har"),
        )
        .unwrap(),
    )
    .unwrap();
    har["log"]["entries"].as_array_mut().unwrap().truncate(1);
    har["log"]["entries"][0]["request"]["url"] = json!(format!(
        "{}json?token=synthetic-private-query",
        s.server.url
    ));
    // JSON string bodies must keep their JSON quoting when seeded. Non-JSON
    // bodies are withheld by the importer and cannot be silently reconstructed.
    har["log"]["entries"][0]["request"]["postData"] =
        json!({"mimeType":"application/json","text":"\"public scalar\""});
    let file = s.dir.path().join("local.har");
    std::fs::write(&file, har.to_string()).unwrap();
    s.core
        .http_import(
            "har",
            HttpImportInput {
                project_id: s.project.clone(),
                target_id: Some(s.target.clone()),
                title: "Owned capture".into(),
                identity: "captured".into(),
                role: "user".into(),
            },
            &file,
        )
        .unwrap();
    let capture = s
        .core
        .http_workspace(&s.project, None, "", 0)
        .unwrap()
        .traffic
        .remove(0);
    let mut draft = s.core.http_draft_seed(&capture.id, None).unwrap();
    assert_eq!(draft.body, "\"public scalar\"");
    assert!(!draft.url.contains('?'));
    assert!(
        !serde_json::to_string(&draft)
            .unwrap()
            .contains("synthetic-auth-token")
    );
    draft.method = "POST".into();
    draft.content_type = "application/json".into();
    draft.body = r#"{"note":"disposable fixture"}"#.into();
    let saved = s.core.http_draft_save(None, draft, None).unwrap();
    assert_eq!(s.server.hits.load(Ordering::Acquire), 0);
    let job = queue(&s.core, &saved);
    let done = s.core.execute_job(&job.id).unwrap();
    assert_eq!(done.status, "completed", "{:?}", done.error_message);
    let result = done.result.unwrap();
    let response = s
        .core
        .http_exchange(result["exchange_id"].as_str().unwrap())
        .unwrap();
    assert_eq!(response.status, 201);
    assert_eq!(response.method, "POST");
    assert_eq!(s.server.hits.load(Ordering::Acquire), 1);
    assert_eq!(
        s.server.last.lock().unwrap()["body"],
        r#"{"note":"disposable fixture"}"#
    );
    s.core
        .evidence_verify(result["capture_evidence_id"].as_str().unwrap())
        .unwrap();
    let evidence = s.core.evidence_from_job(&job.id).unwrap();
    assert_eq!(evidence.job_id.as_deref(), Some(job.id.as_str()));
    let reopened =
        Core::with_secrets(Some(s.dir.path().join("replay.db")), s.secrets.clone()).unwrap();
    assert_eq!(reopened.job(&job.id).unwrap().status, "completed");
    assert_eq!(
        reopened.http_drafts(&s.project, None).unwrap()[0].id,
        saved.id
    );
    assert_eq!(reopened.http_replay_history(&saved.id).unwrap().len(), 1);
    assert_eq!(
        reopened
            .http_workspace(&s.project, None, "", 0)
            .unwrap()
            .total,
        2
    );
    // Target deletion keeps historical drafts/jobs/captures and does not leave
    // the new identity/draft foreign keys blocking the existing target manager.
    let identity = auth(&s, HttpAuthKind::Bearer, "synthetic-retired-target-key");
    s.core.remove_target(&s.target).unwrap();
    assert_eq!(
        s.core.http_replay_preview(&saved.id).unwrap_err().code,
        "NOT_FOUND"
    );
    assert_eq!(s.core.http_replay_history(&saved.id).unwrap().len(), 1);
    assert_eq!(s.server.hits.load(Ordering::Acquire), 1);
    let retained = s.core.http_auth_profiles(&s.project, None).unwrap();
    assert_eq!(retained.len(), 1);
    assert_eq!(retained[0].credential_state, "target_removed");
    assert!(retained[0].expired);
    s.core
        .http_auth_remove(&identity.id, identity.revision)
        .unwrap();
    assert!(s.secrets.values.lock().unwrap().is_empty());
}
#[test]
fn secret_identities_are_bound_redacted_rotated_and_never_auto_authenticated() {
    for (kind, key) in [
        (HttpAuthKind::Bearer, "alice-private-bearer-token"),
        (HttpAuthKind::ApiKey, "alice-private-api-key"),
        (HttpAuthKind::Basic, "alice:alice-private-password"),
        (HttpAuthKind::Cookie, "session=alice-private-cookie"),
        (HttpAuthKind::CustomHeader, "alice-private-custom-key"),
    ] {
        let s = setup();
        let profile = auth(&s, kind, key);
        assert_eq!(profile.credential_state, "configured");
        assert!(profile.last_check.is_none());
        let mut draft = input(&s, "json");
        draft.auth_profile_id = Some(profile.id.clone());
        let mut unsafe_draft = draft.clone();
        unsafe_draft.content_type = "application/json".into();
        unsafe_draft.body = json!({"note":key}).to_string();
        assert_eq!(
            s.core
                .http_draft_save(None, unsafe_draft, None)
                .unwrap_err()
                .code,
            "CREDENTIAL_INPUT"
        );
        let saved = s.core.http_draft_save(None, draft, None).unwrap();
        let done = s.core.execute_job(&queue(&s.core, &saved).id).unwrap();
        assert_eq!(done.status, "completed", "{:?}", done.error_message);
        let capture = s
            .core
            .http_exchange(done.result.unwrap()["exchange_id"].as_str().unwrap())
            .unwrap();
        let output = serde_json::to_string(&capture).unwrap();
        assert!(!output.contains(key));
        assert!(output.contains("[REDACTED]"));
        let metadata = s
            .core
            .http_auth_profiles(&s.project, None)
            .unwrap()
            .remove(0);
        assert_eq!(metadata.last_check.unwrap()["authentication"], "unverified");
        let db = std::fs::read(s.dir.path().join("replay.db")).unwrap();
        assert!(!db.windows(key.len()).any(|v| v == key.as_bytes()));
        let next = s
            .core
            .http_auth_save(
                Some(&profile.id),
                profile.input.clone(),
                Some(profile.revision),
                Some(Zeroizing::new(if kind == HttpAuthKind::Basic {
                    "alice:new-password-value".into()
                } else if kind == HttpAuthKind::Cookie {
                    "session=new-cookie-value".into()
                } else {
                    "new-private-key-value".into()
                })),
            )
            .unwrap();
        assert_eq!(next.revision, profile.revision + 1);
        assert!(next.last_check.is_none());
        assert_eq!(s.secrets.values.lock().unwrap().len(), 1);
        s.core.http_auth_expire(&next.id, next.revision).unwrap();
        assert_eq!(
            s.core.http_replay_preview(&saved.id).unwrap_err().code,
            "CREDENTIAL_UNAVAILABLE"
        );
        s.core
            .http_auth_remove(&next.id, next.revision + 1)
            .unwrap();
        assert!(s.secrets.values.lock().unwrap().is_empty());
    }
}
#[test]
fn exact_approval_stale_edits_and_expired_keys_send_no_request() {
    let s = setup();
    let profile = auth(&s, HttpAuthKind::Bearer, "alice-secret-for-staleness");
    let mut draft = input(&s, "json");
    draft.auth_profile_id = Some(profile.id.clone());
    let saved = s.core.http_draft_save(None, draft, None).unwrap();
    let plan = s.core.http_replay_preview(&saved.id).unwrap();
    assert_eq!(
        s.core
            .http_replay_enqueue(HttpReplayApproval {
                draft_id: saved.id.clone(),
                fingerprint: plan.fingerprint.clone(),
                approved: false
            })
            .unwrap_err()
            .code,
        "HTTP_POLICY"
    );
    let job = queue(&s.core, &saved);
    s.core
        .http_auth_expire(&profile.id, profile.revision)
        .unwrap();
    let failed = s.core.execute_job(&job.id).unwrap();
    assert_eq!(failed.status, "failed");
    assert_eq!(s.server.hits.load(Ordering::Acquire), 0);
    assert!(
        s.core
            .http_auth_save(
                Some(&profile.id),
                profile.input,
                Some(profile.revision),
                None
            )
            .is_err()
    );
}
#[test]
fn scope_routing_proxy_redirect_and_header_bypasses_fail_closed() {
    let s = setup();
    for url in [
        "http://192.0.2.1/",
        "http://10.0.0.1/",
        "http://127.0.0.1.evil.test/",
        "http://foo.localhost/",
        "file:///etc/passwd",
        "http://user:password@127.0.0.1/",
        "http://[::ffff:127.0.0.1]/",
    ] {
        let mut d = input(&s, "json");
        d.url = url.into();
        assert!(s.core.http_draft_save(None, d, None).is_err());
    }
    for name in [
        "Host",
        "Connection",
        "Transfer-Encoding",
        "Proxy-Authorization",
        "X-Forwarded-Host",
    ] {
        let mut d = input(&s, "json");
        d.headers.push(HttpHeader {
            name: name.into(),
            value: "example.test".into(),
        });
        assert!(s.core.http_draft_save(None, d, None).is_err());
    }
    let mut d = input(&s, "json");
    d.proxy = Some("http://127.0.0.1:9999".into());
    assert!(s.core.http_draft_save(None, d, None).is_err());
    let mut d = input(&s, "json");
    d.follow_redirects = true;
    assert!(s.core.http_draft_save(None, d, None).is_err());
    let saved = s
        .core
        .http_draft_save(None, input(&s, "redirect"), None)
        .unwrap();
    let done = s.core.execute_job(&queue(&s.core, &saved).id).unwrap();
    assert_eq!(done.status, "completed");
    assert_eq!(done.result.unwrap()["observations"]["status"], 302);
    assert_eq!(s.server.hits.load(Ordering::Acquire), 1);
}
#[test]
fn large_responses_timeouts_and_cancellation_are_genuine_job_failures() {
    for endpoint in ["large", "chunked-large", "slow"] {
        let s = setup();
        let mut input = input(&s, endpoint);
        input.timeout_ms = if endpoint == "slow" { 500 } else { 3000 };
        let draft = s.core.http_draft_save(None, input, None).unwrap();
        let done = s.core.execute_job(&queue(&s.core, &draft).id).unwrap();
        assert_eq!(done.status, "failed", "{:?}", done.error_message);
        assert_eq!(
            done.error_code.as_deref(),
            Some(if endpoint == "slow" {
                "HTTP_TIMEOUT"
            } else {
                "RESPONSE_LIMIT"
            }),
            "fixture endpoint: {endpoint}"
        );
        assert!(done.result.is_none());
        assert_eq!(s.server.hits.load(Ordering::Acquire), 1);
    }
    let s = setup();
    let draft = s
        .core
        .http_draft_save(None, input(&s, "slow"), None)
        .unwrap();
    let job = queue(&s.core, &draft);
    let core = s.core.clone();
    let id = job.id.clone();
    let worker = thread::spawn(move || core.execute_job(&id).unwrap());
    for _ in 0..100 {
        if s.server.hits.load(Ordering::Acquire) > 0 {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    s.core.cancel_job(&job.id).unwrap();
    let done = worker.join().unwrap();
    assert_eq!(done.status, "cancelled");
    assert!(done.result.is_none());
    assert_eq!(s.server.hits.load(Ordering::Acquire), 1);
}
#[test]
fn drafts_reject_plaintext_credentials_and_placeholder_replay() {
    let s = setup();
    for (url, body, ct) in [
        ("json?api_key=private-test-value", "", ""),
        (
            "json",
            r#"{"password":"private-test-value"}"#,
            "application/json",
        ),
        (
            "json",
            "password=private-test-value",
            "application/x-www-form-urlencoded",
        ),
        ("json", "[REDACTED]", "text/plain"),
    ] {
        let mut d = input(&s, url);
        d.body = body.into();
        d.content_type = ct.into();
        assert!(s.core.http_draft_save(None, d, None).is_err());
    }
    assert_eq!(s.server.hits.load(Ordering::Acquire), 0);
}
#[test]
fn workspace_and_target_credential_isolation_and_retired_key_recovery() {
    let s = setup();
    let profile = auth(&s, HttpAuthKind::Bearer, "alice-key-before-replacement");
    s.secrets.fail_delete.store(true, Ordering::Release);
    let newer = s
        .core
        .http_auth_save(
            Some(&profile.id),
            profile.input.clone(),
            Some(profile.revision),
            Some(Zeroizing::new("alice-key-after-replacement".into())),
        )
        .unwrap();
    assert!(newer.cleanup_pending);
    assert_eq!(s.secrets.values.lock().unwrap().len(), 2);
    s.secrets.fail_delete.store(false, Ordering::Release);
    s.core.http_auth_cleanup().unwrap();
    assert_eq!(s.secrets.values.lock().unwrap().len(), 1);
    let other = s.core.create_project("Separate project", "").unwrap().id;
    assert!(s.core.http_auth_profiles(&other, Some(&s.target)).is_err());
    let mut d = input(&s, "json");
    d.url = d.url.replace("127.0.0.1", "localhost");
    d.auth_profile_id = Some(profile.id);
    assert_eq!(
        s.core.http_draft_save(None, d, None).unwrap_err().code,
        "CREDENTIAL_SCOPE"
    );
}
