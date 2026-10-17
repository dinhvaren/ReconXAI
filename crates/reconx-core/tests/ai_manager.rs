use reconx_core::*;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap},
    io::{Read, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use zeroize::Zeroizing;
#[derive(Default)]
struct Store {
    keys: Mutex<HashMap<String, String>>,
    fail: bool,
}
impl SecretStore for Store {
    fn set(&self, id: &str, key: &str) -> Result<()> {
        if self.fail {
            return Err(AppError::new(
                "SECRET_STORE_UNAVAILABLE",
                "Synthetic store failure",
            ));
        }
        self.keys.lock().unwrap().insert(id.into(), key.into());
        Ok(())
    }
    fn get(&self, id: &str) -> Result<Option<Zeroizing<String>>> {
        Ok(self
            .keys
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .map(Zeroizing::new))
    }
    fn delete(&self, id: &str) -> Result<()> {
        self.keys.lock().unwrap().remove(id);
        Ok(())
    }
}
fn config(name: &str) -> ProviderConfig {
    ProviderConfig {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.into(),
        kind: ProviderKind::Compatible,
        base_url: "http://127.0.0.1:1234/v1".into(),
        model: "fixture-a".into(),
        enabled: true,
        timeout_secs: 2,
        max_output_tokens: 2048,
        requires_key: true,
    }
}
fn server(status: &str, reply: Value) -> (String, thread::JoinHandle<(String, Value)>) {
    server_before(status, reply, None)
}
fn server_before(
    status: &str,
    reply: Value,
    before: Option<Box<dyn FnOnce() + Send>>,
) -> (String, thread::JoinHandle<(String, Value)>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let status = status.to_owned();
    let worker = thread::spawn(move || {
        let start = Instant::now();
        let mut socket = loop {
            match listener.accept() {
                Ok((s, _)) => break s,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(start.elapsed() < Duration::from_secs(8));
                    thread::sleep(Duration::from_millis(5));
                }
                Err(e) => panic!("{e}"),
            }
        };
        socket.set_nonblocking(false).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut bytes = Vec::new();
        let mut buf = [0; 4096];
        let (offset, size) = loop {
            let n = socket.read(&mut buf).unwrap();
            assert!(n > 0);
            bytes.extend_from_slice(&buf[..n]);
            if let Some(pos) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&bytes[..pos]).to_lowercase();
                let len = headers
                    .lines()
                    .find_map(|l| {
                        l.strip_prefix("content-length:")
                            .and_then(|s| s.trim().parse::<usize>().ok())
                    })
                    .unwrap_or(0);
                if bytes.len() >= pos + 4 + len {
                    break (pos + 4, len);
                }
            }
        };
        let headers = String::from_utf8_lossy(&bytes[..offset]).into_owned();
        let body = if size > 0 {
            serde_json::from_slice(&bytes[offset..offset + size]).unwrap()
        } else {
            Value::Null
        };
        if let Some(callback) = before {
            callback();
        }
        let response = reply.to_string();
        write!(socket,"HTTP/1.1 {status}\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{response}",response.len()).unwrap();
        (headers, body)
    });
    (url, worker)
}
#[test]
fn profile_key_and_default_are_atomic_and_survive_reopening_without_plaintext_secrets() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("ai.db");
    let store = Arc::new(Store::default());
    let core = Core::with_secrets(Some(db.clone()), store.clone()).unwrap();
    let project = core.create_project("Retained project", "Notes").unwrap();
    let p = config("Primary reasoning");
    assert_eq!(
        core.save_ai_profile(&p, "custom", None, true)
            .unwrap_err()
            .code,
        "MISSING_CREDENTIAL"
    );
    assert!(core.providers().unwrap().is_empty());
    let headers = BTreeMap::from([("X-Tenant".into(), "private-header-value".into())]);
    core.save_ai_setup(
        AiProfileInput {
            provider: p.clone(),
            vendor_id: "custom".into(),
            make_default: true,
            options: GatewayOptions::default(),
        },
        Some("synthetic-key-not-for-cloud"),
        Some(&headers),
    )
    .unwrap();
    assert!(core.credential_status(&p.id).unwrap());
    assert_eq!(
        core.settings().unwrap().default_provider_id,
        Some(p.id.clone())
    );
    core.save_ai_routing(&[AgentRoute {
        agent_id: "orchestrator".into(),
        provider_id: p.id.clone(),
    }])
    .unwrap();
    drop(core);
    let core = Core::with_secrets(Some(db), store.clone()).unwrap();
    assert_eq!(core.project(&project.id).unwrap().description, "Notes");
    assert_eq!(core.ai_agent_provider("orchestrator").unwrap().id, p.id);
    let state = core.ai_manager().unwrap();
    assert_eq!(state.profiles[0].credential_state, "stored");
    assert!(state.profiles[0].connection.is_none());
    assert_eq!(state.profiles[0].header_names, vec!["X-Tenant"]);
    assert!(
        !serde_json::to_string(&state)
            .unwrap()
            .contains("private-header-value")
    );
    for file in std::fs::read_dir(dir.path()).unwrap().flatten() {
        if file.file_type().unwrap().is_file() {
            let bytes = std::fs::read(file.path()).unwrap();
            let raw = String::from_utf8_lossy(&bytes);
            assert!(!raw.contains("synthetic-key-not-for-cloud"));
            assert!(!raw.contains("private-header-value"));
        }
    }
    core.set_key(&p.id, "replacement-key").unwrap();
    let mut changed = p.clone();
    changed.base_url = "https://another.example/v1".into();
    assert_eq!(
        core.save_ai_profile(&changed, "custom", Some("new-key"), true)
            .unwrap_err()
            .code,
        "GATEWAY_CREDENTIAL_BOUND"
    );
    core.delete_provider(&p.id).unwrap();
    assert!(core.ai_routing().unwrap().is_empty());
    assert!(core.settings().unwrap().default_provider_id.is_none());
    assert!(store.keys.lock().unwrap().is_empty());
}
#[test]
fn secure_store_failures_and_duplicate_names_leave_configuration_intact() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::with_secrets(
        Some(dir.path().join("fail.db")),
        Arc::new(Store {
            fail: true,
            ..Default::default()
        }),
    )
    .unwrap();
    let p = config("Failed save");
    assert_eq!(
        core.save_ai_profile(&p, "custom", Some("synthetic"), true)
            .unwrap_err()
            .code,
        "SECRET_STORE_UNAVAILABLE"
    );
    assert!(core.providers().unwrap().is_empty());
    assert!(core.settings().unwrap().default_provider_id.is_none());
    let core = Core::with_secrets(
        Some(dir.path().join("valid.db")),
        Arc::new(Store::default()),
    )
    .unwrap();
    core.save_ai_profile(&p, "custom", Some("original"), true)
        .unwrap();
    let duplicate = config("Failed save");
    assert_eq!(
        core.save_ai_profile(&duplicate, "custom", Some("second"), false)
            .unwrap_err()
            .code,
        "DUPLICATE_NAME"
    );
    assert_eq!(core.providers().unwrap().len(), 1);
    assert!(core.credential_status(&p.id).unwrap());
}
#[tokio::test]
async fn connection_test_performs_selected_model_inference_with_native_protocols() {
    for kind in [
        ProviderKind::Openai,
        ProviderKind::Anthropic,
        ProviderKind::Gemini,
        ProviderKind::Compatible,
        ProviderKind::Ollama,
    ] {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("ai.db");
        let store = Arc::new(Store::default());
        let core = Core::with_secrets(Some(db.clone()), store.clone()).unwrap();
        let reply = match kind {
            ProviderKind::Openai => {
                json!({"output":[{"content":[{"type":"output_text","text":"OK"}]}]})
            }
            ProviderKind::Anthropic => json!({"content":[{"text":"OK"}]}),
            ProviderKind::Gemini => json!({"candidates":[{"content":{"parts":[{"text":"OK"}]}}]}),
            ProviderKind::Ollama => json!({"message":{"content":"OK"}}),
            _ => json!({"choices":[{"message":{"content":"OK"}}]}),
        };
        let (url, worker) = server("200 OK", reply);
        let mut p = config("Protocol fixture");
        p.kind = kind;
        p.base_url = url;
        p.requires_key = kind != ProviderKind::Ollama;
        core.save_ai_profile(&p, "custom", p.requires_key.then_some("fixture-key"), true)
            .unwrap();
        assert_eq!(
            core.test_ai_connection(&p.id).await.unwrap().state,
            "connected"
        );
        let (head, body) = worker.join().unwrap();
        assert!(head.starts_with("POST "));
        if kind == ProviderKind::Gemini {
            assert!(head.contains("models/fixture-a:generateContent"));
        } else {
            assert_eq!(body["model"], p.model);
        }
        drop(core);
        let core = Core::with_secrets(Some(db), store).unwrap();
        assert_eq!(
            core.ai_manager().unwrap().profiles[0]
                .connection
                .as_ref()
                .unwrap()
                .state,
            "connected"
        );
        p.model = "fixture-b".into();
        core.save_provider(&p).unwrap();
        assert!(core.ai_manager().unwrap().profiles[0].connection.is_none());
    }
}
#[tokio::test]
async fn auth_model_quota_and_rate_limit_errors_are_safe_persistent_failed_states() {
    for (status, code) in [
        ("401 Unauthorized", "AI_AUTHENTICATION_FAILED"),
        ("403 Forbidden", "AI_ACCESS_DENIED"),
        ("404 Not Found", "AI_MODEL_NOT_FOUND"),
        ("429 Too Many Requests", "AI_RATE_LIMITED"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let core =
            Core::with_secrets(Some(dir.path().join("ai.db")), Arc::new(Store::default())).unwrap();
        let (url, worker) = server(status, json!({"error":"fixture-secret-MUST-NOT-LEAK"}));
        let mut p = config("Auth fixture");
        p.base_url = url;
        core.save_ai_profile(&p, "custom", Some("fixture-secret-MUST-NOT-LEAK"), true)
            .unwrap();
        let health = core.test_ai_connection(&p.id).await.unwrap();
        assert_eq!(health.state, "failed");
        assert_eq!(health.error_code.as_deref(), Some(code));
        assert!(!health.message.contains("fixture-secret"));
        assert!(core.ai_manager().unwrap().profiles[0].connection.is_some());
        worker.join().unwrap();
    }
}
#[tokio::test]
async fn model_refresh_preserves_selection_and_never_claims_connected() {
    let dir = tempfile::tempdir().unwrap();
    let core =
        Core::with_secrets(Some(dir.path().join("ai.db")), Arc::new(Store::default())).unwrap();
    let (url, worker) = server(
        "200 OK",
        json!({"data":[{"id":"fixture-b"},{"id":"fixture-a"},{"id":"fixture-a"},{"id":"unsafe?key=secret"}]}),
    );
    let mut p = config("Discovery fixture");
    p.base_url = url;
    core.save_ai_profile(&p, "custom", Some("synthetic"), true)
        .unwrap();
    assert_eq!(
        core.refresh_ai_models(&p.id).await.unwrap(),
        vec!["fixture-a", "fixture-b"]
    );
    worker.join().unwrap();
    assert_eq!(core.provider(&p.id).unwrap().model, "fixture-a");
    let state = core.ai_manager().unwrap();
    assert!(state.profiles[0].connection.is_none());
    assert_eq!(state.profiles[0].models.len(), 2);
    assert_eq!(
        core.save_ai_routing(&[AgentRoute {
            agent_id: "shell_executor".into(),
            provider_id: p.id.clone()
        }])
        .unwrap_err()
        .code,
        "INVALID_ROUTING"
    );
    core.save_ai_routing(&[AgentRoute {
        agent_id: "reverse".into(),
        provider_id: p.id.clone(),
    }])
    .unwrap();
    assert_eq!(core.ai_agent_provider("reverse").unwrap().id, p.id);
    core.save_ai_routing(&[]).unwrap();
    assert_eq!(core.ai_agent_provider("reverse").unwrap().id, p.id);
}
#[tokio::test]
async fn replacing_a_key_during_inference_cannot_publish_a_stale_connected_state() {
    let dir = tempfile::tempdir().unwrap();
    let core =
        Core::with_secrets(Some(dir.path().join("ai.db")), Arc::new(Store::default())).unwrap();
    let mut p = config("Concurrent key change");
    let other = core.clone();
    let id = p.id.clone();
    let (url, worker) = server_before(
        "200 OK",
        json!({"choices":[{"message":{"content":"OK"}}]}),
        Some(Box::new(move || other.set_key(&id, "replacement").unwrap())),
    );
    p.base_url = url;
    core.save_ai_profile(&p, "custom", Some("original"), true)
        .unwrap();
    assert_eq!(
        core.test_ai_connection(&p.id).await.unwrap_err().code,
        "STALE_CONFIGURATION"
    );
    worker.join().unwrap();
    assert!(core.ai_manager().unwrap().profiles[0].connection.is_none());
}
