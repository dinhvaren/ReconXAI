use reconx_core::*;
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::Arc,
    thread,
};
fn config(name: &str, url: String) -> ProviderConfig {
    ProviderConfig {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.into(),
        kind: ProviderKind::Compatible,
        base_url: url,
        model: "fixture".into(),
        enabled: true,
        timeout_secs: 2,
        max_output_tokens: 32,
        requires_key: false,
    }
}
fn mock(status: &str) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let status = status.to_owned();
    let worker = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0u8; 4096];
        let _ = stream.read(&mut buf).unwrap();
        let body = r#"{"choices":[{"message":{"content":"local fallback fixture"}}]}"#;
        write!(
            stream,
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
    });
    (url, worker)
}
struct NoSecrets;
impl SecretStore for NoSecrets {
    fn set(&self, _: &str, _: &str) -> Result<()> {
        unreachable!()
    }
    fn get(&self, _: &str) -> Result<Option<zeroize::Zeroizing<String>>> {
        panic!("Keyless local inference must not access credential storage")
    }
    fn delete(&self, _: &str) -> Result<()> {
        unreachable!()
    }
}
#[tokio::test]
async fn fallback_only_handles_pre_transmission_connection_failure() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::with_secrets(Some(dir.path().join("db")), Arc::new(NoSecrets)).unwrap();
    let vacant = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut primary = config(
        "primary",
        format!("http://{}", vacant.local_addr().unwrap()),
    );
    primary.timeout_secs = 10;
    drop(vacant);
    let (url, worker) = mock("200 OK");
    let fallback = config("fallback", url);
    core.save_provider(&primary).unwrap();
    core.save_provider(&fallback).unwrap();
    core.save_settings(&Settings {
        default_provider_id: Some(primary.id.clone()),
        fallback_provider_id: Some(fallback.id.clone()),
        ..Default::default()
    })
    .unwrap();
    let reply = core
        .chat(&primary.id, "Explain local evidence")
        .await
        .unwrap();
    worker.join().unwrap();
    assert_eq!(reply.provider_id, fallback.id);
    assert!(reply.fallback_used);
    assert_eq!(reply.text, "local fallback fixture");
    assert!(
        core.audit_events()
            .unwrap()
            .iter()
            .any(|e| e.action == "ai.fallback" && e.outcome == "completed")
    );
}
#[tokio::test]
async fn http_failures_are_not_automatically_replayed() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::with_secrets(Some(dir.path().join("db")), Arc::new(NoSecrets)).unwrap();
    let (url, worker) = mock("503 Service Unavailable");
    let primary = config("primary", url);
    let fallback_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    fallback_listener.set_nonblocking(true).unwrap();
    let fallback = config(
        "fallback",
        format!("http://{}", fallback_listener.local_addr().unwrap()),
    );
    core.save_provider(&primary).unwrap();
    core.save_provider(&fallback).unwrap();
    core.save_settings(&Settings {
        default_provider_id: Some(primary.id.clone()),
        fallback_provider_id: Some(fallback.id.clone()),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        core.chat(&primary.id, "test").await.unwrap_err().code,
        "AI_HTTP_ERROR"
    );
    worker.join().unwrap();
    assert_eq!(
        fallback_listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
}
