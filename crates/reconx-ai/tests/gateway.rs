use reconx_ai::{Gateway, OsSecretStore, ProviderConfig, ProviderKind, SecretStore};
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::{Duration, Instant},
};
fn server(response: Value, status: &str) -> (String, thread::JoinHandle<(String, Value)>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    listener.set_nonblocking(true).unwrap();
    let body = response.to_string();
    let status = status.to_owned();
    let thread = thread::spawn(move || {
        let start = Instant::now();
        let mut stream = loop {
            match listener.accept() {
                Ok((s, _)) => break s,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(start.elapsed() < Duration::from_secs(10));
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => panic!("{e}"),
            }
        };
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut bytes = Vec::new();
        let mut buffer = [0u8; 4096];
        let mut target = None;
        loop {
            let n = stream.read(&mut buffer).unwrap();
            assert!(n > 0);
            bytes.extend_from_slice(&buffer[..n]);
            if let Some(pos) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                let header = String::from_utf8_lossy(&bytes[..pos]).to_lowercase();
                let len = header
                    .lines()
                    .find_map(|l| {
                        l.strip_prefix("content-length:")
                            .and_then(|v| v.trim().parse::<usize>().ok())
                    })
                    .unwrap_or(0);
                target = Some((pos + 4, len));
            }
            if target.is_some_and(|(p, len)| bytes.len() >= p + len) {
                break;
            }
        }
        let (pos, len) = target.unwrap();
        let headers = String::from_utf8_lossy(&bytes[..pos]).to_string();
        let request = if len > 0 {
            serde_json::from_slice(&bytes[pos..pos + len]).unwrap()
        } else {
            Value::Null
        };
        write!(stream,"HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nLocation: http://127.0.0.1:1/secret\r\n\r\n{body}",body.len()).unwrap();
        (headers, request)
    });
    (url, thread)
}
fn config(kind: ProviderKind, url: String) -> ProviderConfig {
    ProviderConfig {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Local test".into(),
        kind,
        base_url: url,
        model: "fixture".into(),
        enabled: true,
        timeout_secs: 2,
        max_output_tokens: 512,
        requires_key: kind != ProviderKind::Ollama,
    }
}
#[tokio::test]
async fn all_adapters_use_correct_endpoints_and_payloads() {
    let fixtures = [
        (
            ProviderKind::Openai,
            "/responses",
            json!({"output":[{"content":[{"type":"output_text","text":"reply"}]}]}),
        ),
        (
            ProviderKind::Anthropic,
            "/messages",
            json!({"content":[{"type":"text","text":"reply"}]}),
        ),
        (
            ProviderKind::Gemini,
            "/models/fixture:generateContent",
            json!({"candidates":[{"content":{"parts":[{"text":"reply"}]}}]}),
        ),
        (
            ProviderKind::Ollama,
            "/api/chat",
            json!({"message":{"content":"reply"}}),
        ),
        (
            ProviderKind::Deepseek,
            "/chat/completions",
            json!({"choices":[{"message":{"content":"reply"}}]}),
        ),
        (
            ProviderKind::Openrouter,
            "/chat/completions",
            json!({"choices":[{"message":{"content":"reply"}}]}),
        ),
        (
            ProviderKind::LmStudio,
            "/chat/completions",
            json!({"choices":[{"message":{"content":"reply"}}]}),
        ),
        (
            ProviderKind::Compatible,
            "/chat/completions",
            json!({"choices":[{"message":{"content":"reply"}}]}),
        ),
    ];
    for (kind, path, response) in fixtures {
        let (url, worker) = server(response, "200 OK");
        let gateway = Gateway::new(config(kind, url)).unwrap();
        let reply = gateway
            .chat("Explain local evidence", Some("fixture-key"))
            .await
            .unwrap();
        assert_eq!(reply.text, "reply");
        let (headers, body) = worker.join().unwrap();
        assert!(
            headers.starts_with(&format!("POST {path} HTTP/1.1")),
            "{headers}"
        );
        let lower = headers.to_lowercase();
        if kind == ProviderKind::Anthropic {
            assert!(lower.contains("x-api-key: fixture-key"));
            assert!(lower.contains("anthropic-version: 2023-06-01"));
        } else if kind == ProviderKind::Gemini {
            assert!(lower.contains("x-goog-api-key: fixture-key"));
        } else {
            assert!(lower.contains("authorization: bearer fixture-key"));
        }
        assert!(body.get("tools").is_none());
        assert_eq!(body["model"].as_str().unwrap_or("fixture"), "fixture");
        if kind == ProviderKind::Openai {
            assert_eq!(body["store"], false);
            assert_eq!(body["input"], "Explain local evidence");
        }
    }
}
#[tokio::test]
async fn provider_errors_omit_body_and_redirects_are_not_followed() {
    for status in ["401 Unauthorized", "302 Found"] {
        let (url, worker) = server(json!({"error":"fixture-secret"}), status);
        let error = Gateway::new(config(ProviderKind::Compatible, url))
            .unwrap()
            .chat("test", Some("fixture-secret"))
            .await
            .unwrap_err();
        assert_eq!(error.code, "AI_HTTP_ERROR");
        assert!(!error.to_string().contains("fixture-secret"));
        worker.join().unwrap();
    }
}
#[tokio::test]
async fn echoed_credentials_are_redacted() {
    let (url, worker) = server(
        json!({"choices":[{"message":{"content":"echo fixture-secret"}}]}),
        "200 OK",
    );
    let reply = Gateway::new(config(ProviderKind::Compatible, url))
        .unwrap()
        .chat("test", Some("fixture-secret"))
        .await
        .unwrap();
    assert_eq!(reply.text, "echo [REDACTED]");
    worker.join().unwrap();
}
#[tokio::test]
async fn model_discovery_normalizes_provider_shapes() {
    for (kind, body) in [
        (
            ProviderKind::Compatible,
            json!({"data":[{"id":"one"},{"id":"one"},{"id":"two"}]}),
        ),
        (
            ProviderKind::Gemini,
            json!({"models":[{"name":"models/one"},{"name":"models/two"}]}),
        ),
        (
            ProviderKind::Ollama,
            json!({"models":[{"name":"one"},{"name":"two"}]}),
        ),
    ] {
        let (url, worker) = server(body, "200 OK");
        assert_eq!(
            Gateway::new(config(kind, url))
                .unwrap()
                .models(Some("key"))
                .await
                .unwrap(),
            vec!["one", "two"]
        );
        let (headers, body) = worker.join().unwrap();
        assert!(headers.starts_with("GET "));
        assert!(body.is_null());
    }
}
#[test]
#[ignore = "Requires an available OS credential store; writes only an ephemeral test entry"]
fn real_os_secret_round_trip() {
    let id = format!("test-{}", uuid::Uuid::new_v4());
    let store = OsSecretStore;
    store.set(&id, "ephemeral-fixture-secret").unwrap();
    let read = store.get(&id).unwrap().unwrap();
    store.delete(&id).unwrap();
    assert_eq!(read.as_str(), "ephemeral-fixture-secret");
    assert!(store.get(&id).unwrap().is_none());
}
