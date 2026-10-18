//! Offline traffic ingestion. No HTTP client, replay, shell, or automatic AI egress.
use super::*;
use base64::Engine;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use url::Url;

pub const MAX_HTTP_IMPORT: usize = 16 * 1024 * 1024;
const MAX_ENTRIES: usize = 2000;
const MAX_BODY: usize = 256 * 1024;
const MAX_DEPTH: usize = 48;
const MAX_SERVERS: usize = 32;
const MAX_CORRELATIONS: usize = 1_000_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpImportInput {
    pub project_id: String,
    pub target_id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub identity: String,
    #[serde(default)]
    pub role: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpImport {
    pub id: String,
    pub project_id: String,
    pub target_id: Option<String>,
    pub kind: String,
    pub title: String,
    pub sha256: String,
    pub created_at: String,
    pub evidence_id: String,
    pub summary: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpExchange {
    pub id: String,
    pub import_id: String,
    pub project_id: String,
    pub target_id: Option<String>,
    pub method: String,
    pub url: String,
    pub host: String,
    pub path: String,
    pub status: u16,
    pub started_at: String,
    pub elapsed_ms: f64,
    pub identity: String,
    pub role: String,
    pub authentication: Vec<String>,
    pub query_parameters: Vec<String>,
    pub request: Value,
    pub response: Value,
    pub timings: Value,
    pub body_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiOperation {
    pub id: String,
    pub import_id: String,
    pub project_id: String,
    pub target_id: Option<String>,
    pub method: String,
    pub path: String,
    pub summary: String,
    pub servers: Value,
    pub parameters: Value,
    pub security: Value,
    pub request_body: Value,
    pub responses: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpComparisonInput {
    pub left: String,
    pub right: String,
    #[serde(default)]
    pub owner_field: String,
}
#[derive(Debug, Serialize)]
pub struct HttpWorkspace {
    pub imports: Vec<HttpImport>,
    pub traffic: Vec<HttpExchange>,
    pub operations: Vec<ApiOperation>,
    pub endpoints: Vec<Value>,
    pub total: usize,
    pub offset: usize,
    pub page_size: usize,
    pub comparisons: Vec<Value>,
    pub analysis_jobs: Vec<Value>,
    pub authorization_matrix: Vec<CapturedIdentityObservation>,
}
#[derive(Debug, Serialize)]
pub struct CapturedIdentityObservation {
    pub target_id: Option<String>,
    pub origin: String,
    pub method: String,
    pub path: String,
    pub identity: String,
    pub role: String,
    pub capture_count: usize,
    pub observed_paths: usize,
    pub statuses: Vec<u16>,
    pub body_variants: usize,
    pub capture_ids: Vec<String>,
}

#[derive(Default)]
struct IdentityGroup {
    captures: Vec<String>,
    paths: BTreeSet<String>,
    statuses: BTreeSet<u16>,
    bodies: BTreeSet<String>,
}
#[derive(Eq, Ord, PartialEq, PartialOrd)]
struct IdentityKey {
    target_id: Option<String>,
    origin: String,
    method: String,
    path: String,
    identity: String,
    role: String,
}

struct OperationPattern {
    origin: Option<String>,
    segments: Vec<String>,
    parameters: Vec<String>,
    literal_segments: usize,
}

// Expand once per operation, never recursively and never during capture matching.
fn operation_patterns(servers: &Value, path: &str) -> Result<Vec<OperationPattern>> {
    let servers = servers
        .as_array()
        .ok_or_else(|| bad("OpenAPI servers must be an array"))?;
    if servers.len() > MAX_SERVERS {
        return Err(AppError::new(
            "INPUT_LIMIT",
            "At most 32 servers per operation are supported",
        ));
    }
    let default = json!({"url":"/"});
    let entries: Vec<_> = if servers.is_empty() {
        vec![&default]
    } else {
        servers.iter().collect()
    };
    entries
        .into_iter()
        .map(|server| {
            let template = string(server, "url")?;
            bounded(template, 4096)?;
            let variables = server
                .get("variables")
                .map(|v| {
                    v.as_object()
                        .ok_or_else(|| bad("Server variables must be an object"))
                })
                .transpose()?;
            if variables.is_some_and(|v| v.len() > 32) {
                return Err(AppError::new("INPUT_LIMIT", "Too many server variables"));
            }
            let mut expanded = String::new();
            let mut remaining = template;
            while let Some(start) = remaining.find('{') {
                expanded.push_str(&remaining[..start]);
                let after = &remaining[start + 1..];
                let end = after
                    .find('}')
                    .ok_or_else(|| bad("Unclosed server variable"))?;
                let name = &after[..end];
                if name.is_empty() || name.contains('{') {
                    return Err(bad("Invalid server variable"));
                }
                let variable = variables
                    .and_then(|v| v.get(name))
                    .ok_or_else(|| bad("Server variable has no definition"))?;
                let value = string(variable, "default")?;
                bounded(value, 256)?;
                if value.contains(['{', '}']) {
                    return Err(bad("Recursive server variable defaults are unsupported"));
                }
                if let Some(values) = variable.get("enum") {
                    let choices = values
                        .as_array()
                        .ok_or_else(|| bad("Server variable enum must be an array"))?;
                    if choices.is_empty()
                        || choices.len() > 128
                        || !choices.iter().all(Value::is_string)
                        || !choices.iter().any(|v| v.as_str() == Some(value))
                    {
                        return Err(bad("Server variable default must occur in its enum"));
                    }
                }
                expanded.push_str(value);
                if expanded.len() > 8192 {
                    return Err(AppError::new(
                        "INPUT_LIMIT",
                        "Expanded server URL exceeds its limit",
                    ));
                }
                remaining = &after[end + 1..];
            }
            expanded.push_str(remaining);
            if expanded.is_empty()
                || expanded.contains(['{', '}'])
                || expanded.chars().any(char::is_whitespace)
                || expanded.chars().any(char::is_control)
            {
                return Err(bad("Invalid server URL"));
            }
            let (origin, base) = if expanded.starts_with('/') && !expanded.starts_with("//") {
                if expanded.contains(['?', '#', '\\']) {
                    return Err(bad("Invalid root-relative server URL"));
                }
                (None, expanded)
            } else {
                let url = Url::parse(&expanded).map_err(|_| {
                    bad("Only absolute HTTP(S) or root-relative server URLs are supported")
                })?;
                if !["http", "https"].contains(&url.scheme())
                    || url.host_str().is_none()
                    || !url.username().is_empty()
                    || url.password().is_some()
                    || url.query().is_some()
                    || url.fragment().is_some()
                {
                    return Err(bad(
                        "Server URL must be HTTP(S) without credentials, query or fragment",
                    ));
                }
                (
                    Some(url.origin().ascii_serialization()),
                    url.path().to_owned(),
                )
            };
            let joined = format!("{}{}", base.trim_end_matches('/'), path);
            bounded(&joined, 8192)?;
            let segments: Vec<String> = joined.split('/').map(str::to_owned).collect();
            if segments.len() > 128 {
                return Err(AppError::new(
                    "INPUT_LIMIT",
                    "Server path has too many segments",
                ));
            }
            let parameters: Vec<_> = segments
                .iter()
                .filter(|s| s.starts_with('{') && s.ends_with('}'))
                .map(|s| s[1..s.len() - 1].to_owned())
                .collect();
            let literal_segments = segments.len() - parameters.len();
            Ok(OperationPattern {
                origin,
                segments,
                parameters,
                literal_segments,
            })
        })
        .collect()
}

fn matches_pattern(origin: &str, segments: &[&str], pattern: &OperationPattern) -> bool {
    pattern
        .origin
        .as_deref()
        .is_none_or(|expected| expected == origin)
        && segments.len() == pattern.segments.len()
        && segments
            .iter()
            .zip(&pattern.segments)
            .all(|(actual, template)| {
                *actual == template
                    || (template.starts_with('{') && template.ends_with('}') && !actual.is_empty())
            })
}
fn bad(message: &str) -> AppError {
    AppError::new("INVALID_HTTP_IMPORT", message)
}
fn bounded(value: &str, max: usize) -> Result<String> {
    if value.len() > max || value.chars().any(|c| c == '\0') {
        return Err(AppError::new(
            "INPUT_LIMIT",
            "Imported field exceeds its supported limit",
        ));
    }
    Ok(value.to_owned())
}
fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| bad("A required string field is missing"))
}
fn array<'a>(value: &'a Value, key: &str, max: usize) -> Result<&'a Vec<Value>> {
    let list = value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| bad("A required array field is missing"))?;
    if list.len() > max {
        return Err(AppError::new(
            "INPUT_LIMIT",
            "Imported collection exceeds its supported limit",
        ));
    }
    Ok(list)
}
fn tree_budget(value: &Value, depth: usize, nodes: &mut usize) -> Result<()> {
    *nodes += 1;
    if depth > MAX_DEPTH || *nodes > 200_000 {
        return Err(AppError::new(
            "INPUT_LIMIT",
            "Document exceeds depth or node budget",
        ));
    }
    match value {
        Value::Array(a) => {
            for v in a {
                tree_budget(v, depth + 1, nodes)?
            }
        }
        Value::Object(o) => {
            for (key, v) in o {
                bounded(key, 4096)?;
                tree_budget(v, depth + 1, nodes)?
            }
        }
        _ => {}
    }
    Ok(())
}
fn parse_json(bytes: &[u8]) -> Result<Value> {
    let value: Value = serde_json::from_slice(bytes).map_err(|_| bad("Invalid JSON document"))?;
    tree_budget(&value, 0, &mut 0)?;
    Ok(value)
}
fn safe_value(mut value: Value) -> Value {
    super::reporting::redact_value(&mut value);
    value
}
fn safe_headers(value: &Value) -> Result<(Value, Vec<String>)> {
    let mut auth = BTreeSet::new();
    let mut out = Vec::new();
    for h in array(value, "headers", 256)? {
        let name = bounded(string(h, "name")?, 256)?;
        let raw = bounded(string(h, "value")?, 16 * 1024)?;
        let lower = name.to_ascii_lowercase();
        if lower == "authorization" {
            auth.insert(
                if raw.to_ascii_lowercase().starts_with("bearer ") {
                    "Bearer"
                } else if raw.to_ascii_lowercase().starts_with("basic ") {
                    "Basic"
                } else {
                    "Authorization header"
                }
                .to_owned(),
            );
        }
        if lower == "cookie" {
            auth.insert("Cookie".into());
        }
        if lower.contains("api-key") || lower.contains("api_key") || lower == "x-auth-token" {
            auth.insert("API key header".into());
        }
        let public = [
            "content-type",
            "content-length",
            "cache-control",
            "server",
            "strict-transport-security",
            "content-security-policy",
            "x-frame-options",
            "x-content-type-options",
            "access-control-allow-origin",
            "access-control-allow-credentials",
            "vary",
            "accept",
            "accept-encoding",
        ];
        let value = if public.contains(&lower.as_str()) {
            super::reporting::redact_content(&raw).0
        } else {
            "[REDACTED]".into()
        };
        out.push(json!({"name":name,"value":value}));
    }
    Ok((Value::Array(out), auth.into_iter().collect()))
}
fn safe_cookies(value: &Value) -> Result<Value> {
    let mut out = Vec::new();
    for c in array(value, "cookies", 256)? {
        out.push(json!({"name":bounded(string(c,"name")?,256)?,"value":"[REDACTED]","httpOnly":c.get("httpOnly").and_then(Value::as_bool),"secure":c.get("secure").and_then(Value::as_bool),"sameSite":c.get("sameSite").and_then(Value::as_str).filter(|v|["Strict","Lax","None"].contains(v))}));
    }
    Ok(Value::Array(out))
}
fn body(value: &Value) -> Result<(Value, String)> {
    let text = value.get("text").and_then(Value::as_str).unwrap_or("");
    if text.len() > MAX_BODY * 2 {
        return Err(AppError::new(
            "BODY_LIMIT",
            "HAR body exceeds the supported limit",
        ));
    }
    let bytes = match value.get("encoding").and_then(Value::as_str) {
        None => text.as_bytes().to_vec(),
        Some("base64") => base64::engine::general_purpose::STANDARD
            .decode(text)
            .map_err(|_| bad("HAR body has invalid base64 encoding"))?,
        Some(_) => return Err(bad("Unsupported HAR body encoding")),
    };
    if bytes.len() > MAX_BODY {
        return Err(AppError::new(
            "BODY_LIMIT",
            "Decoded HAR body exceeds 256 KiB",
        ));
    }
    let hash = super::evidence::digest(&bytes);
    let preview = if bytes.is_empty() {
        Value::Null
    } else if let Ok(json) = parse_json(&bytes) {
        safe_value(json)
    } else {
        Value::String(
            "[Body withheld: non-JSON or binary content; original retained in private evidence]"
                .into(),
        )
    };
    let mime = value.get("mimeType").and_then(Value::as_str).unwrap_or("");
    bounded(mime, 256)?;
    let mut parameters = Vec::new();
    if let Some(params) = value.get("params") {
        for p in params
            .as_array()
            .filter(|a| a.len() <= 256)
            .ok_or_else(|| bad("Invalid form parameters"))?
        {
            parameters.push(json!({"name":bounded(string(p,"name")?,256)?,"value":"[REDACTED]"}));
        }
    }
    Ok((
        json!({"mime_type":mime,"decoded_size":bytes.len(),"preview":preview,"encoded":value.get("encoding").is_some(),"parameters":parameters}),
        hash,
    ))
}
fn parse_har(
    bytes: &[u8],
    input: &HttpImportInput,
    import_id: &str,
) -> Result<(Vec<HttpExchange>, Value)> {
    let doc = parse_json(bytes)?;
    let log = doc
        .get("log")
        .ok_or_else(|| bad("HAR requires a log object"))?;
    if string(log, "version")? != "1.2" {
        return Err(bad("Only HAR 1.2 is supported"));
    }
    if !log.get("creator").is_some_and(Value::is_object) {
        return Err(bad("HAR creator metadata is missing"));
    }
    let entries = array(log, "entries", MAX_ENTRIES)?;
    if entries.is_empty() {
        return Err(bad("HAR contains no entries"));
    }
    let mut out = Vec::new();
    let mut keys = BTreeSet::new();
    let mut duplicates = 0;
    for entry in entries {
        let req = entry
            .get("request")
            .ok_or_else(|| bad("HAR entry has no request"))?;
        let res = entry
            .get("response")
            .ok_or_else(|| bad("HAR entry has no response"))?;
        let method = bounded(string(req, "method")?, 20)?.to_ascii_uppercase();
        if !method.bytes().all(|b| b.is_ascii_uppercase() || b == b'-') || method.is_empty() {
            return Err(bad("Invalid HTTP method"));
        }
        let mut url = Url::parse(string(req, "url")?).map_err(|_| bad("Invalid HTTP URL"))?;
        if !["http", "https"].contains(&url.scheme())
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err(bad("Use HTTP(S) HAR URLs without embedded credentials"));
        }
        bounded(url.as_str(), 8192)?;
        let host = url.host_str().unwrap().to_owned();
        let path = super::reporting::redact_content(url.path()).0;
        let mut query: BTreeSet<String> = url.query_pairs().map(|(n, _)| n.into_owned()).collect();
        for p in array(req, "queryString", 256)? {
            query.insert(bounded(string(p, "name")?, 256)?);
        }
        if query.len() > 256 {
            return Err(bad("Too many query parameters"));
        }
        url.set_query(None);
        url.set_fragment(None);
        url.set_path(&path);
        if !query.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for key in &query {
                pairs.append_pair(key, "[REDACTED]");
            }
        }
        let (headers, mut authentication) = safe_headers(req)?;
        if !array(req, "cookies", 256)?.is_empty() && !authentication.iter().any(|v| v == "Cookie")
        {
            authentication.push("Cookie".into());
        }
        let (response_headers, _) = safe_headers(res)?;
        let status = res
            .get("status")
            .and_then(Value::as_u64)
            .filter(|s| *s <= 599)
            .ok_or_else(|| bad("Invalid HAR response status"))? as u16;
        let (request_body, _) = body(req.get("postData").unwrap_or(&Value::Null))?;
        let (response_body, body_sha256) = body(
            res.get("content")
                .ok_or_else(|| bad("Missing response content metadata"))?,
        )?;
        let started_at = bounded(string(entry, "startedDateTime")?, 100)?;
        chrono::DateTime::parse_from_rfc3339(&started_at)
            .map_err(|_| bad("Invalid HAR entry timestamp"))?;
        let elapsed_ms = entry
            .get("time")
            .and_then(Value::as_f64)
            .filter(|t| *t >= 0.0)
            .ok_or_else(|| bad("Invalid HAR timing"))?;
        let timings = entry
            .get("timings")
            .filter(|t| t.is_object())
            .ok_or_else(|| bad("Missing HAR timings"))?;
        if [
            "blocked", "dns", "connect", "send", "wait", "receive", "ssl",
        ]
        .into_iter()
        .any(|key| {
            timings
                .get(key)
                .is_some_and(|t| !t.as_f64().is_some_and(|v| v >= -1.0))
        }) {
            return Err(bad("Invalid HAR timing field"));
        }
        if !keys.insert((host.clone(), method.clone(), path.clone())) {
            duplicates += 1;
        }
        let mut safe_timings = serde_json::Map::new();
        for key in [
            "blocked", "dns", "connect", "send", "wait", "receive", "ssl",
        ] {
            if let Some(t) = timings.get(key) {
                safe_timings.insert(key.into(), t.clone());
            }
        }
        let version = |v: &Value| -> Result<String> {
            let raw = bounded(string(v, "httpVersion")?, 32)?;
            if !["", "h2", "h3"].contains(&raw.as_str())
                && (!raw.starts_with("HTTP/")
                    || !raw[5..].bytes().all(|b| b.is_ascii_digit() || b == b'.'))
            {
                return Err(bad("Invalid HTTP version"));
            }
            Ok(raw)
        };
        out.push(HttpExchange{id:Uuid::new_v4().to_string(),import_id:import_id.into(),project_id:input.project_id.clone(),target_id:input.target_id.clone(),method,url:url.to_string(),host,path,status,started_at,elapsed_ms,identity:input.identity.clone(),role:input.role.clone(),authentication,query_parameters:query.into_iter().collect(),request:json!({"headers":headers,"cookies":safe_cookies(req)?,"body":request_body,"http_version":version(req)?}),response:json!({"headers":response_headers,"cookies":safe_cookies(res)?,"body":response_body,"http_version":version(res)?}),timings:Value::Object(safe_timings),body_sha256});
    }
    Ok((
        out,
        json!({"entries":entries.len(),"unique_endpoints":keys.len(),"duplicate_endpoints":duplicates,"warning":"Offline import only; previews redact credentials and may omit content. No requests sent."}),
    ))
}
fn resolve<'a>(doc: &'a Value, mut value: &'a Value) -> Result<&'a Value> {
    let mut seen = BTreeSet::new();
    for _ in 0..32 {
        let Some(reference) = value.get("$ref").and_then(Value::as_str) else {
            return Ok(value);
        };
        if !reference.starts_with("#/") {
            return Err(AppError::new(
                "EXTERNAL_REF_DISABLED",
                "Only in-document JSON Pointer references are supported",
            ));
        }
        if !seen.insert(reference) {
            return Err(AppError::new(
                "REF_CYCLE",
                "Cyclic structural reference cannot be expanded",
            ));
        }
        value = doc
            .pointer(&reference[1..])
            .ok_or_else(|| bad("Unresolved local reference"))?;
    }
    Err(AppError::new(
        "REF_LIMIT",
        "Reference chain exceeds 32 steps",
    ))
}
fn validate_refs(doc: &Value, value: &Value, refs: &mut usize) -> Result<()> {
    if let Some(reference) = value.get("$ref") {
        *refs += 1;
        if *refs > 4096 {
            return Err(AppError::new("REF_LIMIT", "Too many references"));
        }
        let r = reference
            .as_str()
            .ok_or_else(|| bad("Reference must be a string"))?;
        if !r.starts_with("#/") {
            return Err(AppError::new(
                "EXTERNAL_REF_DISABLED",
                "External file and network references are disabled",
            ));
        }
        if doc.pointer(&r[1..]).is_none() {
            return Err(bad("Unresolved local reference"));
        }
    }
    match value {
        Value::Object(o) => {
            for v in o.values() {
                validate_refs(doc, v, refs)?
            }
        }
        Value::Array(a) => {
            for v in a {
                validate_refs(doc, v, refs)?
            }
        }
        _ => {}
    }
    Ok(())
}
fn parse_openapi(
    bytes: &[u8],
    input: &HttpImportInput,
    import_id: &str,
) -> Result<(Vec<ApiOperation>, Value)> {
    let text = std::str::from_utf8(bytes).map_err(|_| bad("OpenAPI must be UTF-8"))?;
    let doc: Value = if text.trim_start().starts_with('{') {
        parse_json(bytes)?
    } else {
        let options = serde_saphyr::options! {with_snippet:false,reject_unsupported_tags:true,strict_booleans:true,budget:serde_saphyr::budget!{max_documents:1,max_depth:48,max_events:200_000,max_nodes:200_000,max_total_scalar_bytes:MAX_HTTP_IMPORT,max_aliases:0,max_anchors:0,max_merge_keys:0}};
        let value: Value = serde_saphyr::from_str_with_options(text, options)
            .map_err(|_| bad("Invalid YAML or YAML parser budget exceeded"))?;
        tree_budget(&value, 0, &mut 0)?;
        value
    };
    let version = string(&doc, "openapi")?;
    if !version.starts_with("3.0.") && !version.starts_with("3.1.") {
        return Err(bad("Only OpenAPI 3.0 and 3.1 are supported"));
    }
    let info = doc
        .get("info")
        .filter(|v| v.is_object())
        .ok_or_else(|| bad("OpenAPI info object is missing"))?;
    string(info, "title")?;
    string(info, "version")?;
    let paths = doc
        .get("paths")
        .and_then(Value::as_object)
        .ok_or_else(|| bad("OpenAPI paths object is missing"))?;
    if paths.len() > MAX_ENTRIES {
        return Err(AppError::new("INPUT_LIMIT", "Too many OpenAPI paths"));
    }
    let mut refs = 0;
    validate_refs(&doc, &doc, &mut refs)?;
    let mut ops = Vec::new();
    let mut normalized_bytes = 0usize;
    for (path, item) in paths {
        if !path.starts_with('/') || path.len() > 4096 || path.contains(['?', '#']) {
            return Err(bad("Invalid OpenAPI path"));
        }
        let item = resolve(&doc, item)?;
        for method in [
            "get", "put", "post", "delete", "options", "head", "patch", "trace",
        ] {
            let Some(op) = item.get(method) else { continue };
            let op = resolve(&doc, op)?;
            if !op.is_object() || !op.get("responses").is_some_and(Value::is_object) {
                return Err(bad("Operation requires a responses object"));
            }
            let mut parameters = Vec::new();
            for set in [item.get("parameters"), op.get("parameters")]
                .into_iter()
                .flatten()
            {
                for parameter in set
                    .as_array()
                    .ok_or_else(|| bad("Parameters must be an array"))?
                {
                    let p = resolve(&doc, parameter)?.clone();
                    let name = string(&p, "name")?;
                    let location = string(&p, "in")?;
                    if !["path", "query", "header", "cookie"].contains(&location) {
                        return Err(bad("Invalid parameter location"));
                    }
                    parameters.retain(|old: &Value| {
                        old.get("name").and_then(Value::as_str) != Some(name)
                            || old.get("in").and_then(Value::as_str) != Some(location)
                    });
                    parameters.push(p);
                }
            }
            if parameters.len() > 256 {
                return Err(bad("Too many operation parameters"));
            }
            let servers = op
                .get("servers")
                .or_else(|| item.get("servers"))
                .or_else(|| doc.get("servers"))
                .cloned()
                .unwrap_or(json!([]));
            operation_patterns(&servers, path)?;
            let request_body = op
                .get("requestBody")
                .map(|v| resolve(&doc, v).cloned())
                .transpose()?
                .unwrap_or(Value::Null);
            let responses = op.get("responses").unwrap();
            let mut response_map = serde_json::Map::new();
            for (status, res) in responses.as_object().unwrap() {
                response_map.insert(status.clone(), resolve(&doc, res)?.clone());
            }
            let summary = op
                .get("summary")
                .or_else(|| op.get("description"))
                .and_then(Value::as_str)
                .unwrap_or("");
            ops.push(ApiOperation {
                id: Uuid::new_v4().to_string(),
                import_id: import_id.into(),
                project_id: input.project_id.clone(),
                target_id: input.target_id.clone(),
                method: method.to_ascii_uppercase(),
                path: path.clone(),
                summary: super::reporting::redact_content(&bounded(summary, 16 * 1024)?).0,
                servers: safe_value(servers),
                parameters: safe_value(Value::Array(parameters)),
                security: safe_value(
                    op.get("security")
                        .or_else(|| doc.get("security"))
                        .cloned()
                        .unwrap_or(json!([])),
                ),
                request_body: safe_value(request_body),
                responses: safe_value(Value::Object(response_map)),
            });
            let size = serde_json::to_vec(ops.last().unwrap())
                .map_err(json_error)?
                .len();
            normalized_bytes = normalized_bytes.saturating_add(size);
            if size > 1024 * 1024 || normalized_bytes > 32 * 1024 * 1024 {
                return Err(AppError::new(
                    "INPUT_LIMIT",
                    "Resolved OpenAPI operation data exceeds its byte budget",
                ));
            }
            if ops.len() > MAX_ENTRIES {
                return Err(AppError::new("INPUT_LIMIT", "Too many OpenAPI operations"));
            }
        }
    }
    if ops.is_empty() {
        return Err(bad("OpenAPI document has no supported HTTP operations"));
    }
    let summary = json!({"version":version,"operations":ops.len(),"servers":safe_value(doc.get("servers").cloned().unwrap_or(json!([]))),"components":safe_value(doc.get("components").cloned().unwrap_or(json!({}))),"references":refs,"warning":"Local references validated; recursive schemas retained without expansion. No external references or requests."});
    Ok((ops, summary))
}
impl Core {
    pub fn http_import(
        &self,
        kind: &str,
        mut input: HttpImportInput,
        path: &Path,
    ) -> Result<HttpImport> {
        input.project_id = self.project(&input.project_id)?.id;
        if self.project(&input.project_id)?.archived {
            return Err(AppError::new(
                "PROJECT_ARCHIVED",
                "Restore project before importing traffic",
            ));
        }
        input.title = validate_text(&input.title, 180, "Import title")?;
        bounded(&input.identity, 100)?;
        bounded(&input.role, 100)?;
        input.identity = super::reporting::redact_content(&input.identity).0;
        input.role = super::reporting::redact_content(&input.role).0;
        if let Some(target) = &input.target_id
            && self.target(target)?.input.project_id != input.project_id
        {
            return Err(AppError::new(
                "PROJECT_MISMATCH",
                "Target belongs to another project",
            ));
        }
        let bytes = super::evidence::read_regular(path, MAX_HTTP_IMPORT)?;
        let id = Uuid::new_v4().to_string();
        let (traffic, operations, summary) = match kind {
            "har" => {
                let (rows, summary) = parse_har(&bytes, &input, &id)?;
                (rows, Vec::new(), summary)
            }
            "openapi" => {
                let (ops, summary) = parse_openapi(&bytes, &input, &id)?;
                (Vec::new(), ops, summary)
            }
            _ => return Err(bad("Choose HAR or OpenAPI import")),
        };
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let raw = self.insert_observed_evidence(
            &tx,
            EvidenceInput {
                project_id: input.project_id.clone(),
                target_id: input.target_id.clone(),
                kind: EvidenceKind::Attachment,
                title: format!("Private original: {}", input.title),
                source_tool: format!("offline-{kind}"),
                sensitivity: Sensitivity::Restricted,
            },
            None,
            &bytes,
            "application/octet-stream",
        )?;
        let result = (|| {
            let import = HttpImport {
                id: id.clone(),
                project_id: input.project_id.clone(),
                target_id: input.target_id.clone(),
                kind: kind.into(),
                title: input.title,
                sha256: super::evidence::digest(&bytes),
                created_at: now(),
                evidence_id: raw.id.clone(),
                summary,
            };
            tx.execute(
                "INSERT INTO http_imports VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![
                    import.id,
                    import.project_id,
                    import.target_id,
                    import.kind,
                    import.title,
                    import.sha256,
                    import.created_at,
                    import.evidence_id,
                    serde_json::to_string(&import.summary).map_err(json_error)?
                ],
            )
            .map_err(db_error)?;
            for e in traffic {
                tx.execute(
                    "INSERT INTO http_exchanges VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                    params![
                        e.id,
                        e.import_id,
                        e.project_id,
                        e.target_id,
                        e.host,
                        e.method,
                        e.path,
                        e.status,
                        serde_json::to_string(&e).map_err(json_error)?
                    ],
                )
                .map_err(db_error)?;
            }
            for o in operations {
                tx.execute(
                    "INSERT INTO api_operations VALUES(?1,?2,?3,?4,?5,?6,?7)",
                    params![
                        o.id,
                        o.import_id,
                        o.project_id,
                        o.target_id,
                        o.method,
                        o.path,
                        serde_json::to_string(&o).map_err(json_error)?
                    ],
                )
                .map_err(db_error)?;
            }
            Self::audit(&tx, "http.imported", &id, "offline")?;
            Ok(import)
        })();
        match result {
            Ok(import) => {
                if let Err(e) = tx.commit() {
                    let _ = std::fs::remove_file(self.output_path("evidence", &raw.id)?);
                    return Err(db_error(e));
                }
                Ok(import)
            }
            Err(e) => {
                let _ = std::fs::remove_file(self.output_path("evidence", &raw.id)?);
                Err(e)
            }
        }
    }
    pub fn http_imports(&self, project: &str) -> Result<Vec<HttpImport>> {
        let project = self.project(project)?.id;
        let conn = self.db.connect()?;
        let mut q=conn.prepare("SELECT id,project_id,target_id,kind,title,sha256,created_at,evidence_id,summary_json FROM http_imports WHERE project_id=?1 ORDER BY created_at DESC LIMIT 1000").map_err(db_error)?;
        let rows = q
            .query_map([project], |r| {
                Ok(HttpImport {
                    id: r.get(0)?,
                    project_id: r.get(1)?,
                    target_id: r.get(2)?,
                    kind: r.get(3)?,
                    title: r.get(4)?,
                    sha256: r.get(5)?,
                    created_at: r.get(6)?,
                    evidence_id: r.get(7)?,
                    summary: Value::String(r.get(8)?),
                })
            })
            .map_err(db_error)?;
        let mut bytes = 0usize;
        rows.map(|r| {
            let mut i = r.map_err(db_error)?;
            bytes = bytes.saturating_add(i.summary.as_str().unwrap().len());
            if bytes > 32 * 1024 * 1024 {
                return Err(AppError::new(
                    "WORKSPACE_LIMIT",
                    "Import metadata exceeds the 32 MiB display budget",
                ));
            }
            i.summary = serde_json::from_str(i.summary.as_str().unwrap()).map_err(json_error)?;
            Ok(i)
        })
        .collect()
    }
    pub fn http_exchange(&self, id: &str) -> Result<HttpExchange> {
        self.http_json_record("http_exchanges", id)
    }
    pub fn api_operation(&self, id: &str) -> Result<ApiOperation> {
        self.http_json_record("api_operations", id)
    }
    fn http_json_record<T: serde::de::DeserializeOwned>(&self, table: &str, id: &str) -> Result<T> {
        let text: Option<String> = self
            .db
            .connect()?
            .query_row(
                &format!("SELECT data_json FROM {table} WHERE id=?1"),
                [id],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        serde_json::from_str(
            &text.ok_or_else(|| AppError::new("NOT_FOUND", "HTTP record not found"))?,
        )
        .map_err(json_error)
    }
    pub fn http_workspace(
        &self,
        project: &str,
        target: Option<&str>,
        search: &str,
        offset: usize,
    ) -> Result<HttpWorkspace> {
        self.http_workspace_page(project, target, search, offset, 100)
    }
    fn http_workspace_page(
        &self,
        project: &str,
        target: Option<&str>,
        search: &str,
        offset: usize,
        page_size: usize,
    ) -> Result<HttpWorkspace> {
        let project = self.project(project)?.id;
        if let Some(target) = target
            && self.target(target)?.input.project_id != project
        {
            return Err(AppError::new(
                "PROJECT_MISMATCH",
                "Target belongs to another project",
            ));
        }
        bounded(search, 256)?;
        if offset > 100_000 {
            return Err(AppError::new("INPUT_LIMIT", "Page offset exceeds limit"));
        }
        let conn = self.db.connect()?;
        fn rows<T: serde::de::DeserializeOwned>(
            conn: &Connection,
            table: &str,
            project: &str,
            target: Option<&str>,
        ) -> Result<Vec<T>> {
            let mut q=conn.prepare(&format!("SELECT data_json FROM {table} WHERE project_id=?1 AND (?2 IS NULL OR target_id=?2) ORDER BY rowid DESC LIMIT 5001")).map_err(db_error)?;
            let records = q
                .query_map(params![project, target], |r| r.get::<_, String>(0))
                .map_err(db_error)?;
            let mut out = Vec::new();
            let mut bytes = 0usize;
            for record in records {
                let text = record.map_err(db_error)?;
                bytes = bytes.saturating_add(text.len());
                if bytes > 32 * 1024 * 1024 {
                    return Err(AppError::new(
                        "WORKSPACE_LIMIT",
                        "Filter to one target; workspace data exceeds 32 MiB per collection",
                    ));
                }
                out.push(serde_json::from_str(&text).map_err(json_error)?);
            }
            if out.len() > 5000 {
                return Err(AppError::new(
                    "WORKSPACE_LIMIT",
                    "Filter to one target; inventory limit is 5000 records",
                ));
            }
            Ok(out)
        }
        let mut traffic: Vec<HttpExchange> = rows(&conn, "http_exchanges", &project, target)?;
        let operations: Vec<ApiOperation> = rows(&conn, "api_operations", &project, target)?;
        let patterns = operations
            .iter()
            .map(|o| operation_patterns(&o.servers, &o.path))
            .collect::<Result<Vec<_>>>()?;
        let pattern_count: usize = patterns.iter().map(Vec::len).sum();
        if traffic.len().saturating_mul(pattern_count) > MAX_CORRELATIONS {
            return Err(AppError::new(
                "WORKSPACE_LIMIT",
                "Filter to one target; correlation exceeds its operation budget",
            ));
        }
        let mut endpoints: BTreeMap<String, Value> = BTreeMap::new();
        let mut groups: BTreeMap<IdentityKey, IdentityGroup> = BTreeMap::new();
        for e in &mut traffic {
            let origin = Url::parse(&e.url)
                .map_err(|_| bad("Invalid retained URL"))?
                .origin()
                .ascii_serialization();
            let segments: Vec<_> = e.path.split('/').collect();
            let mut matches: Vec<_> = operations
                .iter()
                .zip(&patterns)
                .filter(|(o, _)| {
                    o.method == e.method
                        && o.target_id
                            .as_ref()
                            .is_none_or(|t| e.target_id.as_ref() == Some(t))
                })
                .filter_map(|(o, candidates)| {
                    candidates
                        .iter()
                        .filter(|p| matches_pattern(&origin, &segments, p))
                        .max_by_key(|p| p.literal_segments)
                        .map(|p| (o, p))
                })
                .collect();
            // Concrete paths take precedence over templated paths, as specified by OpenAPI.
            if let Some(best) = matches.iter().map(|(_, p)| p.literal_segments).max() {
                matches.retain(|(_, p)| p.literal_segments == best);
            }
            if let Some((_, pattern)) = matches.first() {
                e.request["path_parameters"] = json!(pattern.parameters);
            }
            let path = matches
                .first()
                .map_or(e.path.as_str(), |(o, _)| o.path.as_str());
            let key = format!(
                "{} {} {} {}",
                e.target_id.as_deref().unwrap_or("unassigned"),
                origin,
                e.method,
                path
            );
            let entry=endpoints.entry(key).or_insert_with(||json!({"target_id":e.target_id,"host":e.host,"origin":origin,"method":e.method,"path":path,"requests":0,"documented":!matches.is_empty(),"operation_ids":matches.iter().map(|(o,_)|&o.id).collect::<Vec<_>>(),"observation":if matches.is_empty(){"No matching documented operation in this workspace"}else{"Observed documented endpoint"}}));
            entry["requests"] = json!(entry["requests"].as_u64().unwrap_or(0) + 1);
            let group = groups
                .entry(IdentityKey {
                    target_id: e.target_id.clone(),
                    origin,
                    method: e.method.clone(),
                    path: path.into(),
                    identity: e.identity.clone(),
                    role: e.role.clone(),
                })
                .or_default();
            group.captures.push(e.id.clone());
            group.paths.insert(e.path.clone());
            group.statuses.insert(e.status);
            group.bodies.insert(e.body_sha256.clone());
        }
        let authorization_matrix = groups
            .into_iter()
            .map(|(key, group)| CapturedIdentityObservation {
                target_id: key.target_id,
                origin: key.origin,
                method: key.method,
                path: key.path,
                identity: key.identity,
                role: key.role,
                capture_count: group.captures.len(),
                observed_paths: group.paths.len(),
                statuses: group.statuses.into_iter().collect(),
                body_variants: group.bodies.len(),
                capture_ids: group.captures,
            })
            .collect();
        let needle = search.to_lowercase();
        traffic
            .retain(|e| serde_json::to_string(e).is_ok_and(|s| s.to_lowercase().contains(&needle)));
        let total = traffic.len();
        let traffic = traffic.into_iter().skip(offset).take(page_size).collect();
        let mut q=conn.prepare("SELECT data_json FROM http_comparisons WHERE project_id=?1 AND (?2 IS NULL OR json_extract(data_json,'$.target_id')=?2) ORDER BY created_at DESC LIMIT 100").map_err(db_error)?;
        let comparisons = q
            .query_map(params![project, target], |r| r.get::<_, String>(0))
            .map_err(db_error)?
            .map(|s| serde_json::from_str(&s.map_err(db_error)?).map_err(json_error))
            .collect::<Result<Vec<_>>>()?;
        let mut jq=conn.prepare("SELECT r.job_id,r.exchange_id FROM http_review_jobs r JOIN analysis_jobs j ON j.id=r.job_id WHERE j.project_id=?1 AND (?2 IS NULL OR j.target_id=?2) ORDER BY j.created_at DESC LIMIT 100").map_err(db_error)?;
        let pairs = jq
            .query_map(params![project, target], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(db_error)?;
        let analysis_jobs = pairs
            .map(|r| {
                let (id, exchange_id) = r.map_err(db_error)?;
                Ok(json!({"exchange_id":exchange_id,"job":self.job(&id)?}))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(HttpWorkspace {
            imports: self
                .http_imports(&project)?
                .into_iter()
                .filter(|i| target.is_none_or(|t| i.target_id.as_deref() == Some(t)))
                .collect(),
            traffic,
            operations,
            endpoints: endpoints.into_values().collect(),
            total,
            offset,
            page_size,
            comparisons,
            analysis_jobs,
            authorization_matrix,
        })
    }
    pub fn http_review_enqueue(&self, id: &str) -> Result<AnalysisJob> {
        let exchange = self.http_exchange(id)?;
        self.require_http_review_source(&exchange)?;
        let fingerprint =
            super::evidence::digest(&serde_json::to_vec(&exchange).map_err(json_error)?);
        let job = Uuid::new_v4().to_string();
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let active:i64=tx.query_row("SELECT count(*) FROM http_review_jobs r JOIN analysis_jobs j ON j.id=r.job_id WHERE r.exchange_id=?1 AND j.status IN ('queued','running')",[id],|r|r.get(0)).map_err(db_error)?;
        if active > 0 {
            return Err(AppError::new(
                "JOB_BUSY",
                "This capture already has an active review job",
            ));
        }
        tx.execute("INSERT INTO analysis_jobs(id,project_id,target_id,kind,status,created_at,timeout_ms,logs_json) VALUES(?1,?2,?3,'offline_http_review','queued',?4,30000,'[\"Offline captured-header review requested; no network access\"]')",params![job,exchange.project_id,exchange.target_id,now()]).map_err(db_error)?;
        tx.execute(
            "INSERT INTO http_review_jobs VALUES(?1,?2,?3)",
            params![job, id, fingerprint],
        )
        .map_err(db_error)?;
        Self::audit(&tx, "http.review_queued", &job, "offline")?;
        tx.commit().map_err(db_error)?;
        self.job(&job)
    }
    fn require_http_review_source(&self, exchange: &HttpExchange) -> Result<()> {
        if self.project(&exchange.project_id)?.archived {
            return Err(AppError::new(
                "PROJECT_ARCHIVED",
                "Restore project before capture analysis",
            ));
        }
        if let Some(target) = &exchange.target_id {
            let mut candidate =
                Url::parse(&exchange.url).map_err(|_| bad("Invalid retained URL"))?;
            candidate.set_query(None);
            let decision = self.validate_scope(target, Some(candidate.as_str()))?;
            if !decision.authorized || !decision.in_scope {
                return Err(AppError::new(
                    "SCOPE_BLOCKED",
                    "Capture is not within the declared authorized target scope",
                ));
            }
        }
        self.http_source_integrity(exchange)?;
        Ok(())
    }
    fn http_source_integrity(&self, exchange: &HttpExchange) -> Result<Value> {
        let source: (String, String) = self
            .db
            .connect()?
            .query_row(
                "SELECT evidence_id,sha256 FROM http_imports WHERE id=?1 AND project_id=?2",
                params![exchange.import_id, exchange.project_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| bad("Capture import was removed"))?;
        self.evidence_verify(&source.0)?;
        if self.evidence(&source.0)?.sha256 != source.1 {
            return Err(AppError::new(
                "SOURCE_CHANGED",
                "HTTP import and original evidence hashes disagree",
            ));
        }
        Ok(json!({"import_id":exchange.import_id,"evidence_id":source.0,"sha256":source.1}))
    }
    pub(super) fn prepare_http_review(
        &self,
        job: &AnalysisJob,
        update: &mut dyn FnMut(u8, &str) -> Result<()>,
    ) -> Result<super::jobs::PreparedJob> {
        let (id, expected): (String, String) = self
            .db
            .connect()?
            .query_row(
                "SELECT exchange_id,fingerprint FROM http_review_jobs WHERE job_id=?1",
                [&job.id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(db_error)?;
        let exchange = self.http_exchange(&id)?;
        self.require_http_review_source(&exchange)?;
        if exchange.project_id != job.project_id
            || exchange.target_id != job.target_id
            || super::evidence::digest(&serde_json::to_vec(&exchange).map_err(json_error)?)
                != expected
        {
            return Err(AppError::new(
                "SOURCE_CHANGED",
                "Capture changed after analysis was queued",
            ));
        }
        update(30, "Original evidence integrity checked")?;
        let headers = exchange
            .response
            .get("headers")
            .and_then(Value::as_array)
            .ok_or_else(|| bad("Retained response headers are invalid"))?;
        let names: BTreeSet<_> = headers
            .iter()
            .filter_map(|v| v.get("name").and_then(Value::as_str))
            .map(str::to_ascii_lowercase)
            .collect();
        let mut observations = Vec::new();
        for header in [
            "content-security-policy",
            "x-content-type-options",
            "x-frame-options",
        ] {
            observations.push(json!({"header":header,"captured":names.contains(header),"classification":"observation","note":"Header presence/absence is contextual and does not establish a vulnerability"}));
        }
        if exchange.url.starts_with("https://") {
            observations.push(json!({"header":"strict-transport-security","captured":names.contains("strict-transport-security"),"classification":"observation"}));
        }
        update(75, "Captured header indicators normalized")?;
        self.require_http_review_source(&exchange)?;
        Ok(super::jobs::PreparedJob {
            result: json!({"exchange_id":id,"import_id":exchange.import_id,"host":exchange.host,"path":exchange.path,"method":exchange.method,"status":exchange.status,"observations":observations,"network_requests":0,"verification":"unverified observations from supplied capture"}),
            analysis: None,
            http: None,
        })
    }
    pub fn http_note(&self, id: &str, note: Option<&str>) -> Result<String> {
        let exchange = self.http_exchange(id)?;
        if note.is_some() && self.project(&exchange.project_id)?.archived {
            return Err(AppError::new(
                "PROJECT_ARCHIVED",
                "Restore project before editing notes",
            ));
        }
        let conn = self.db.connect()?;
        if let Some(note) = note {
            bounded(note, 16 * 1024)?;
            let note = super::reporting::redact_content(note).0;
            conn.execute("INSERT INTO http_notes VALUES(?1,?2,?3) ON CONFLICT(exchange_id) DO UPDATE SET note=excluded.note,updated_at=excluded.updated_at",params![id,note,now()]).map_err(db_error)?;
            Self::audit(&conn, "http.note_saved", id, "completed")?;
            Ok(note)
        } else {
            Ok(conn
                .query_row(
                    "SELECT note FROM http_notes WHERE exchange_id=?1",
                    [id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(db_error)?
                .unwrap_or_default())
        }
    }
    pub fn http_compare(&self, input: HttpComparisonInput) -> Result<Value> {
        let left = self.http_exchange(&input.left)?;
        let right = self.http_exchange(&input.right)?;
        if self.project(&left.project_id)?.archived {
            return Err(AppError::new(
                "PROJECT_ARCHIVED",
                "Restore project before saving a comparison",
            ));
        }
        if left.id == right.id {
            return Err(bad("Choose two different captures"));
        }
        if left.project_id != right.project_id || left.target_id != right.target_id {
            return Err(AppError::new(
                "PROJECT_MISMATCH",
                "Compare captures from the same project and target",
            ));
        }
        bounded(&input.owner_field, 100)?;
        let source_left = self.http_source_integrity(&left)?;
        let source_right = self.http_source_integrity(&right)?;
        let mut observations=vec!["Offline captured-response comparison; authorization context labels are user supplied, not verified identities".to_owned()];
        let same_origin = Url::parse(&left.url)
            .map_err(|_| bad("Invalid retained URL"))?
            .origin()
            == Url::parse(&right.url)
                .map_err(|_| bad("Invalid retained URL"))?
                .origin();
        if !same_origin || left.method != right.method || left.path != right.path {
            observations.push(
                "Different origin, method or path: this is not an equivalent-request authorization test"
                    .into(),
            );
        }
        if left.status != right.status {
            observations.push(
                "Response statuses differ; this alone does not establish an authorization weakness"
                    .into(),
            );
        }
        if left.body_sha256 != right.body_sha256 {
            observations.push(
                "Response body bytes differ; inspect ownership and expected permissions manually"
                    .into(),
            );
        }
        let owners = if input.owner_field.is_empty() {
            Value::Null
        } else {
            let a = left
                .response
                .pointer("/body/preview")
                .and_then(|v| v.get(&input.owner_field));
            let b = right
                .response
                .pointer("/body/preview")
                .and_then(|v| v.get(&input.owner_field));
            json!({"field":input.owner_field,"present_in_both":a.is_some()&&b.is_some(),"equal":if a.is_some()&&b.is_some(){Some(a==b)}else{None}})
        };
        let id = Uuid::new_v4().to_string();
        let mut data = json!({"id":id,"project_id":left.project_id,"target_id":left.target_id,"left":left.id,"right":right.id,"left_identity":left.identity,"right_identity":right.identity,"left_role":left.role,"right_role":right.role,"left_status":left.status,"right_status":right.status,"body_bytes_equal":left.body_sha256==right.body_sha256,"owner_indicator":owners,"observations":observations,"status":"needs_manual_review","network_requests":0});
        data["sources"] = json!([source_left, source_right]);
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let evidence = self.insert_observed_evidence(
            &tx,
            EvidenceInput {
                project_id: left.project_id.clone(),
                target_id: left.target_id.clone(),
                kind: EvidenceKind::StructuredJson,
                title: "Offline HTTP comparison".into(),
                source_tool: "offline-http-compare".into(),
                sensitivity: Sensitivity::Confidential,
            },
            None,
            &serde_json::to_vec_pretty(&data).map_err(json_error)?,
            "application/json",
        )?;
        data["evidence_id"] = json!(evidence.id);
        tx.execute(
            "INSERT INTO http_comparisons VALUES(?1,?2,?3,?4,?5)",
            params![
                id,
                left.project_id,
                evidence.id,
                now(),
                serde_json::to_string(&data).map_err(json_error)?
            ],
        )
        .map_err(db_error)?;
        Self::audit(&tx, "http.compared", &id, "needs_manual_review")?;
        tx.commit().map_err(db_error)?;
        Ok(data)
    }
    pub fn http_comparison_finding(&self, id: &str) -> Result<Finding> {
        let data: Value = self.http_json_record("http_comparisons", id)?;
        self.finding_save(None,FindingInput{project_id:string(&data,"project_id")?.into(),target_id:data.get("target_id").and_then(Value::as_str).map(str::to_owned),title:"Captured HTTP behavior requires authorization review".into(),description:"Offline comparison of captured responses. No replay or vulnerability verification was performed.".into(),category:"Authorization observation".into(),status:FindingStatus::NeedsVerification,technical_details:serde_json::to_string_pretty(&data).map_err(json_error)?,evidence_ids:vec![string(&data,"evidence_id")?.into()],..Default::default()},None,false)
    }
    pub fn http_export(&self, project: &str, path: &Path) -> Result<Value> {
        let data = self.http_workspace_page(project, None, "", 0, 5000)?;
        let mut value = serde_json::to_value(data).map_err(json_error)?;
        super::reporting::redact_value(&mut value);
        let bytes = serde_json::to_vec_pretty(&value).map_err(json_error)?;
        if bytes.len() > 64 * 1024 * 1024 {
            return Err(AppError::new(
                "EXPORT_LIMIT",
                "Redacted HTTP export exceeds 64 MiB; export individual evidence instead",
            ));
        }
        super::evidence::export_new(path, &bytes)?;
        Self::audit(&self.db.connect()?, "http.exported", project, "redacted")?;
        Ok(json!({"sha256":super::evidence::digest(&bytes),"raw":false}))
    }
}
