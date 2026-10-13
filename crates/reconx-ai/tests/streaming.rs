use reconx_ai::{Gateway, ProviderConfig, ProviderKind};
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};
use tokio::sync::watch;
fn serve(first: String, tail: String) -> (Gateway, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let worker = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut input = Vec::new();
        let mut buffer = [0u8; 4096];
        loop {
            let n = socket.read(&mut buffer).unwrap();
            assert!(n > 0);
            input.extend_from_slice(&buffer[..n]);
            if let Some(pos) = input.windows(4).position(|w| w == b"\r\n\r\n") {
                let header = String::from_utf8_lossy(&input[..pos]).to_lowercase();
                let len = header
                    .lines()
                    .find_map(|l| {
                        l.strip_prefix("content-length:")
                            .and_then(|x| x.trim().parse::<usize>().ok())
                    })
                    .unwrap();
                if input.len() >= pos + 4 + len {
                    let body: serde_json::Value =
                        serde_json::from_slice(&input[pos + 4..]).unwrap();
                    assert_eq!(body["stream"], true);
                    assert!(body.get("tools").is_none());
                    break;
                }
            }
        }
        let _ = write!(
            socket,
            "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{first}",
            first.len() + tail.len()
        );
        let _ = socket.flush();
        thread::sleep(Duration::from_millis(60));
        let _ = socket.write_all(tail.as_bytes());
    });
    let c = ProviderConfig {
        id: uuid::Uuid::new_v4().to_string(),
        name: "fixture".into(),
        kind: ProviderKind::Compatible,
        base_url: url,
        model: "fixture".into(),
        enabled: true,
        timeout_secs: 2,
        max_output_tokens: 64,
        requires_key: false,
    };
    (Gateway::new(c).unwrap(), worker)
}
fn delta(text: &str) -> String {
    format!(
        "data: {}\n\n",
        serde_json::json!({"choices":[{"delta":{"content":text}}]})
    )
}
#[tokio::test]
async fn real_http_stream_is_incremental_and_redacts_split_credentials() {
    let (gateway, worker) = serve(
        delta("Answer key-se"),
        format!("{}data: [DONE]\n\n", delta("cret complete")),
    );
    let (_tx, rx) = watch::channel(false);
    let mut updates = Vec::new();
    let reply = gateway
        .stream_chat(
            "Explain supplied evidence",
            Some("key-secret"),
            rx,
            |text| updates.push(text.to_owned()),
        )
        .await
        .unwrap();
    assert_eq!(updates[0], "Answer ");
    assert_eq!(reply.text, "Answer [REDACTED] complete");
    assert!(updates.iter().all(|s| !s.contains("key-se")));
    worker.join().unwrap();
}
#[tokio::test]
async fn cancellation_interrupts_an_active_response() {
    let (gateway, worker) = serve(delta("partial"), "data: [DONE]\n\n".into());
    let (tx, rx) = watch::channel(false);
    let error = gateway
        .stream_chat("test", None, rx, |_| {
            tx.send(true).unwrap();
        })
        .await
        .unwrap_err();
    assert_eq!(error.code, "CANCELLED");
    worker.join().unwrap();
}
#[tokio::test]
async fn eof_without_terminal_event_is_never_success() {
    let (gateway, worker) = serve(delta("partial"), String::new());
    let (_tx, rx) = watch::channel(false);
    assert_eq!(
        gateway
            .stream_chat("test", None, rx, |_| {})
            .await
            .unwrap_err()
            .code,
        "AI_INCOMPLETE"
    );
    worker.join().unwrap();
}
