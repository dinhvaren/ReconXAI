use super::{AiReply, Gateway, ProviderKind, text_blocks};
use reconx_security::{AppError, Result, validate_text};
use serde_json::Value;
use std::time::Instant;
use tokio::sync::watch;

#[derive(Default)]
struct Frames {
    bytes: Vec<u8>,
    data: Vec<String>,
    ndjson: bool,
}
impl Frames {
    fn push(&mut self, chunk: &[u8]) -> Result<Vec<String>> {
        self.bytes.extend_from_slice(chunk);
        let mut events = Vec::new();
        while let Some(end) = self.bytes.iter().position(|b| *b == b'\n') {
            let line: Vec<u8> = self.bytes.drain(..=end).collect();
            let line = std::str::from_utf8(&line)
                .map_err(|_| AppError::new("AI_INVALID_STREAM", "Stream is not valid UTF-8"))?
                .trim_end_matches(['\r', '\n']);
            if self.ndjson {
                if !line.trim().is_empty() {
                    events.push(line.to_owned());
                }
            } else if line.is_empty() {
                if !self.data.is_empty() {
                    events.push(self.data.join("\n"));
                    self.data.clear();
                }
            } else if let Some(data) = line.strip_prefix("data:") {
                self.data
                    .push(data.strip_prefix(' ').unwrap_or(data).to_owned());
            }
        }
        Ok(events)
    }
}
#[derive(Default)]
struct Accumulator {
    text: String,
    input: Option<u64>,
    output: Option<u64>,
    done: bool,
}
impl Accumulator {
    fn event(&mut self, kind: ProviderKind, event: &str) -> Result<()> {
        if event == "[DONE]" {
            self.done = true;
            return Ok(());
        }
        let v: Value = serde_json::from_str(event)
            .map_err(|_| AppError::new("AI_INVALID_STREAM", "Stream contains invalid JSON"))?;
        if v.get("error").is_some()
            || matches!(
                v.get("type").and_then(Value::as_str),
                Some("error" | "response.failed" | "response.incomplete")
            )
        {
            return Err(AppError::new(
                "AI_STREAM_ERROR",
                "Gateway signaled an incomplete or failed response; details omitted",
            ));
        }
        match kind {
            ProviderKind::Openai => match v.get("type").and_then(Value::as_str) {
                Some("response.output_text.delta") => self
                    .text
                    .push_str(v.get("delta").and_then(Value::as_str).unwrap_or_default()),
                Some("response.completed") => {
                    self.done = true;
                    self.input = v
                        .pointer("/response/usage/input_tokens")
                        .and_then(Value::as_u64);
                    self.output = v
                        .pointer("/response/usage/output_tokens")
                        .and_then(Value::as_u64);
                }
                _ => {}
            },
            ProviderKind::Anthropic => match v.get("type").and_then(Value::as_str) {
                Some("content_block_delta")
                    if v.pointer("/delta/type").and_then(Value::as_str) == Some("text_delta") =>
                {
                    self.text.push_str(
                        v.pointer("/delta/text")
                            .and_then(Value::as_str)
                            .unwrap_or_default(),
                    )
                }
                Some("message_start") => {
                    self.input = v
                        .pointer("/message/usage/input_tokens")
                        .and_then(Value::as_u64)
                }
                Some("message_delta") => {
                    self.output = v.pointer("/usage/output_tokens").and_then(Value::as_u64);
                    if v.pointer("/delta/stop_reason").and_then(Value::as_str) == Some("max_tokens")
                    {
                        return Err(AppError::new(
                            "AI_INCOMPLETE",
                            "Model reached the output budget before finishing",
                        ));
                    }
                }
                Some("message_stop") => self.done = true,
                _ => {}
            },
            ProviderKind::Gemini => {
                self.text.push_str(&text_blocks(
                    v.pointer("/candidates/0/content/parts"),
                    "text",
                ));
                self.input = v
                    .pointer("/usageMetadata/promptTokenCount")
                    .and_then(Value::as_u64)
                    .or(self.input);
                self.output = v
                    .pointer("/usageMetadata/candidatesTokenCount")
                    .and_then(Value::as_u64)
                    .or(self.output);
                if let Some(reason) = v
                    .pointer("/candidates/0/finishReason")
                    .and_then(Value::as_str)
                {
                    if reason != "STOP" {
                        return Err(AppError::new(
                            "AI_INCOMPLETE",
                            "Model did not finish normally",
                        ));
                    }
                    self.done = true;
                }
            }
            ProviderKind::Ollama => {
                self.text.push_str(
                    v.pointer("/message/content")
                        .and_then(Value::as_str)
                        .unwrap_or_default(),
                );
                if v.get("done").and_then(Value::as_bool) == Some(true) {
                    self.done = true;
                    self.input = v.get("prompt_eval_count").and_then(Value::as_u64);
                    self.output = v.get("eval_count").and_then(Value::as_u64);
                }
            }
            _ => {
                self.text.push_str(
                    v.pointer("/choices/0/delta/content")
                        .and_then(Value::as_str)
                        .unwrap_or_default(),
                );
                if let Some(reason) = v
                    .pointer("/choices/0/finish_reason")
                    .and_then(Value::as_str)
                {
                    if reason == "length" {
                        return Err(AppError::new(
                            "AI_INCOMPLETE",
                            "Model reached the output budget before finishing",
                        ));
                    }
                    self.done = true;
                }
                self.input = v
                    .pointer("/usage/prompt_tokens")
                    .and_then(Value::as_u64)
                    .or(self.input);
                self.output = v
                    .pointer("/usage/completion_tokens")
                    .and_then(Value::as_u64)
                    .or(self.output);
            }
        }
        Ok(())
    }
}
fn redact(text: &str, key: Option<&str>, final_text: bool) -> String {
    let Some(key) = key.filter(|k| !k.is_empty()) else {
        return text.to_owned();
    };
    let mut safe = text.replace(key, "[REDACTED]");
    if !final_text {
        // A credential can straddle model tokens or transport chunks. Withhold its
        // longest incomplete suffix until the next event disambiguates it.
        let suffix = (1..key.len())
            .rev()
            .find(|n| key.is_char_boundary(*n) && safe.ends_with(&key[..*n]));
        if let Some(n) = suffix {
            safe.truncate(safe.len() - n);
        }
    }
    safe
}
async fn cancelled(rx: &mut watch::Receiver<bool>) {
    loop {
        if *rx.borrow() {
            return;
        }
        if rx.changed().await.is_err() {
            std::future::pending::<()>().await;
        }
    }
}
impl Gateway {
    pub async fn stream_chat<F>(
        &self,
        prompt: &str,
        key: Option<&str>,
        mut cancellation: watch::Receiver<bool>,
        mut update: F,
    ) -> Result<AiReply>
    where
        F: FnMut(&str) + Send,
    {
        let prompt = validate_text(prompt, 32768, "Prompt")?;
        tokio::select! {
            biased;
            _=cancelled(&mut cancellation)=>Err(AppError::new("CANCELLED","AI request cancelled; partial output is unverified")),
            result=self.stream_inner(&prompt,key,&mut update)=>result,
        }
    }
    async fn stream_inner<F>(
        &self,
        prompt: &str,
        key: Option<&str>,
        update: &mut F,
    ) -> Result<AiReply>
    where
        F: FnMut(&str) + Send,
    {
        let start = Instant::now();
        let mut response = self
            .authenticate(self.chat_request(prompt, true), key)?
            .send()
            .await
            .map_err(|e| {
                AppError::new(
                    if e.is_timeout() {
                        "AI_TIMEOUT"
                    } else if e.is_connect() {
                        "AI_CONNECT_ERROR"
                    } else {
                        "AI_NETWORK_ERROR"
                    },
                    "AI streaming request failed",
                )
            })?;
        if !response.status().is_success() {
            return Err(AppError::new(
                "AI_HTTP_ERROR",
                &format!(
                    "Gateway returned HTTP {}; response body omitted",
                    response.status().as_u16()
                ),
            ));
        }
        let mut frames = Frames {
            ndjson: self.config.kind == ProviderKind::Ollama,
            ..Default::default()
        };
        let mut acc = Accumulator::default();
        let mut bytes = 0usize;
        let mut last = String::new();
        let mut emitted = 0usize;
        let mut last_emit = Instant::now();
        while let Some(chunk) = response.chunk().await.map_err(|e| {
            AppError::new(
                if e.is_timeout() {
                    "AI_TIMEOUT"
                } else {
                    "AI_NETWORK_ERROR"
                },
                "AI stream interrupted",
            )
        })? {
            bytes += chunk.len();
            if bytes > 2 * 1024 * 1024 {
                return Err(AppError::new(
                    "AI_RESPONSE_LIMIT",
                    "Gateway stream exceeded 2 MiB",
                ));
            }
            for event in frames.push(&chunk)? {
                acc.event(self.config.kind, &event)?;
                if acc.text.len() > 256 * 1024 {
                    return Err(AppError::new(
                        "AI_RESPONSE_LIMIT",
                        "Streamed text exceeded 256 KiB",
                    ));
                }
                let safe = redact(&acc.text, key, false);
                if safe != last
                    && emitted < 256
                    && (emitted == 0 || last_emit.elapsed().as_millis() >= 25)
                {
                    update(&safe);
                    last = safe;
                    emitted += 1;
                    last_emit = Instant::now();
                }
            }
        }
        if !acc.done || !frames.bytes.is_empty() || !frames.data.is_empty() {
            return Err(AppError::new(
                "AI_INCOMPLETE",
                "Gateway stream ended without a complete final event",
            ));
        }
        let text = redact(&acc.text, key, true);
        if text.trim().is_empty() {
            return Err(AppError::new(
                "AI_NO_TEXT",
                "Gateway returned no textual answer",
            ));
        }
        update(&text);
        Ok(AiReply {
            text,
            model: self.config.model.clone(),
            provider_id: self.config.id.clone(),
            fallback_used: false,
            latency_ms: start.elapsed().as_millis() as u64,
            input_tokens: acc.input,
            output_tokens: acc.output,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frames_survive_bytewise_unicode_and_multiline_events() {
        let data = ": comment\r\ndata: {\"delta\":\r\ndata: \"é\"}\r\n\r\n";
        let mut decoder = Frames::default();
        let mut events = Vec::new();
        for byte in data.as_bytes() {
            events.extend(decoder.push(&[*byte]).unwrap());
        }
        assert_eq!(events.len(), 1);
        assert_eq!(
            serde_json::from_str::<Value>(&events[0]).unwrap()["delta"],
            "é"
        );
    }
    #[test]
    fn credentials_are_withheld_across_event_boundaries() {
        assert_eq!(redact("text key-se", Some("key-secret"), false), "text ");
        assert_eq!(
            redact("text key-secret", Some("key-secret"), false),
            "text [REDACTED]"
        );
        assert_eq!(
            redact("text key-se", Some("key-secret"), true),
            "text key-se"
        );
    }
    #[test]
    fn all_provider_streams_decode_and_require_completion() {
        for (kind, events) in [
            (
                ProviderKind::Openai,
                vec![
                    r#"{"type":"response.output_text.delta","delta":"ok"}"#,
                    r#"{"type":"response.completed","response":{"usage":{"input_tokens":2,"output_tokens":1}}}"#,
                ],
            ),
            (
                ProviderKind::Anthropic,
                vec![
                    r#"{"type":"message_start","message":{"usage":{"input_tokens":2}}}"#,
                    r#"{"type":"content_block_delta","delta":{"type":"text_delta","text":"ok"}}"#,
                    r#"{"type":"message_delta","usage":{"output_tokens":1}}"#,
                    r#"{"type":"message_stop"}"#,
                ],
            ),
            (
                ProviderKind::Gemini,
                vec![
                    r#"{"candidates":[{"content":{"parts":[{"text":"ok"}]},"finishReason":"STOP"}],"usageMetadata":{"promptTokenCount":2,"candidatesTokenCount":1}}"#,
                ],
            ),
            (
                ProviderKind::Ollama,
                vec![
                    r#"{"message":{"content":"ok"},"done":true,"prompt_eval_count":2,"eval_count":1}"#,
                ],
            ),
            (
                ProviderKind::Compatible,
                vec![
                    r#"{"choices":[{"delta":{"content":"ok"},"finish_reason":"stop"}],"usage":{"prompt_tokens":2,"completion_tokens":1}}"#,
                    "[DONE]",
                ],
            ),
        ] {
            let mut a = Accumulator::default();
            for e in events {
                a.event(kind, e).unwrap();
            }
            assert_eq!(a.text, "ok");
            assert!(a.done);
            assert_eq!((a.input, a.output), (Some(2), Some(1)));
        }
        let mut a = Accumulator::default();
        assert!(
            a.event(
                ProviderKind::Compatible,
                r#"{"error":{"message":"secret"}}"#
            )
            .is_err()
        );
    }
}
