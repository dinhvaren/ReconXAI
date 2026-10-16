//! Bounded HTTP backend for the official MCP SDK. No redirects or automatic tool replay.
use futures::{StreamExt, stream::BoxStream};
use reqwest::header::{HeaderName, HeaderValue};
use rmcp::{
    model::{ClientJsonRpcMessage, ServerJsonRpcMessage},
    transport::streamable_http_client::{
        SseError, StreamableHttpClient, StreamableHttpError, StreamableHttpPostResponse,
    },
};
use std::{collections::HashMap, sync::Arc};
type Error = StreamableHttpError<reqwest::Error>;
const LIMIT: usize = 256 * 1024;
#[derive(Clone)]
pub(super) struct BoundedHttp(pub reqwest::Client);
fn failure() -> Error {
    Error::UnexpectedServerResponse("MCP HTTP response rejected (status, size or encoding)".into())
}
fn sse(response: reqwest::Response) -> BoxStream<'static, Result<sse_stream::Sse, SseError>> {
    let mut total = 0usize;
    let bytes = response.bytes_stream().map(move |chunk| {
        let chunk = chunk.map_err(|_| std::io::Error::other("MCP HTTP body interrupted"))?;
        total = total.saturating_add(chunk.len());
        if total > LIMIT {
            return Err(std::io::Error::other("MCP HTTP body limit exceeded"));
        }
        Ok(chunk)
    });
    sse_stream::SseStream::from_bytes_stream(bytes).boxed()
}
fn request(
    client: &reqwest::Client,
    method: reqwest::Method,
    uri: &str,
    session: Option<&str>,
    auth: Option<&str>,
    headers: HashMap<HeaderName, HeaderValue>,
) -> reqwest::RequestBuilder {
    let mut builder = client
        .request(method, uri)
        .header("Accept", "application/json, text/event-stream")
        .headers(headers.into_iter().collect());
    if let Some(s) = session {
        builder = builder.header("Mcp-Session-Id", s);
    }
    if let Some(s) = auth {
        builder = builder.bearer_auth(s);
    }
    builder
}
impl StreamableHttpClient for BoundedHttp {
    type Error = reqwest::Error;
    async fn post_message(
        &self,
        uri: Arc<str>,
        message: ClientJsonRpcMessage,
        session: Option<Arc<str>>,
        auth: Option<String>,
        headers: HashMap<HeaderName, HeaderValue>,
    ) -> Result<StreamableHttpPostResponse, Error> {
        let mut response = request(
            &self.0,
            reqwest::Method::POST,
            &uri,
            session.as_deref(),
            auth.as_deref(),
            headers,
        )
        .json(&message)
        .send()
        .await
        .map_err(Error::Client)?;
        if !response.status().is_success() {
            return Err(failure());
        }
        if response.status() == reqwest::StatusCode::ACCEPTED
            || response.status() == reqwest::StatusCode::NO_CONTENT
        {
            return Ok(StreamableHttpPostResponse::Accepted);
        }
        if response.content_length().is_some_and(|n| n > LIMIT as u64) {
            return Err(failure());
        }
        let session = response
            .headers()
            .get("Mcp-Session-Id")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let content = response
            .headers()
            .get("Content-Type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if content.starts_with("text/event-stream") {
            return Ok(StreamableHttpPostResponse::Sse(sse(response), session));
        }
        if !content.starts_with("application/json") {
            return Err(failure());
        }
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(Error::Client)? {
            if body.len() + chunk.len() > LIMIT {
                return Err(failure());
            }
            body.extend_from_slice(&chunk);
        }
        let parsed: ServerJsonRpcMessage = serde_json::from_slice(&body).map_err(|_| failure())?;
        Ok(StreamableHttpPostResponse::Json(parsed, session))
    }
    async fn delete_session(
        &self,
        uri: Arc<str>,
        session: Arc<str>,
        auth: Option<String>,
        headers: HashMap<HeaderName, HeaderValue>,
    ) -> Result<(), Error> {
        let result = request(
            &self.0,
            reqwest::Method::DELETE,
            &uri,
            Some(&session),
            auth.as_deref(),
            headers,
        )
        .send()
        .await
        .map_err(Error::Client)?;
        if !result.status().is_success() {
            return Err(Error::ServerDoesNotSupportDeleteSession);
        }
        Ok(())
    }
    async fn get_stream(
        &self,
        uri: Arc<str>,
        session: Option<Arc<str>>,
        last_event_id: Option<String>,
        auth: Option<String>,
        mut headers: HashMap<HeaderName, HeaderValue>,
    ) -> Result<BoxStream<'static, Result<sse_stream::Sse, SseError>>, Error> {
        if let Some(last) = last_event_id {
            headers.insert(
                HeaderName::from_static("last-event-id"),
                HeaderValue::from_str(&last).map_err(|_| failure())?,
            );
        }
        let result = request(
            &self.0,
            reqwest::Method::GET,
            &uri,
            session.as_deref(),
            auth.as_deref(),
            headers,
        )
        .send()
        .await
        .map_err(Error::Client)?;
        if !result.status().is_success() {
            return Err(Error::ServerDoesNotSupportSse);
        }
        Ok(sse(result))
    }
}
