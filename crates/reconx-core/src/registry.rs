//! Official registry metadata is untrusted and never grants execution permission.
use super::*;
use serde_json::{Value, json};
use std::time::Duration;

pub const MCP_REGISTRY_API: &str = "v0.1";
const BASE: &str = "https://registry.modelcontextprotocol.io/";
const MAX_BODY: usize = 2 * 1024 * 1024;
const TTL_SECONDS: i64 = 900;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct RegistryQuery {
    pub search: String,
    pub limit: Option<u16>,
    pub cursor: Option<String>,
    pub updated_since: Option<String>,
    pub runtime: Option<String>,
    pub platform: Option<String>,
    pub refresh: bool,
    pub offline: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub publisher: String,
    pub version: String,
    pub repository: Option<String>,
    pub packages: Vec<Value>,
    pub transports: Vec<String>,
    pub status: String,
    pub trust: String,
    pub schema_supported: bool,
    pub server_json: Value,
    pub registry_metadata: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryPage {
    pub api_version: String,
    pub entries: Vec<RegistryEntry>,
    pub next_cursor: Option<String>,
    pub fetched_at: String,
    pub expires_at: String,
    pub cached: bool,
    pub stale: bool,
    pub warning: Option<String>,
}
fn invalid() -> AppError {
    AppError::new(
        "REGISTRY_SCHEMA",
        "Registry response is incompatible with the supported v0.1 metadata schema",
    )
}
fn bounded_text(value: &str, max: usize) -> bool {
    value.len() <= max && !value.chars().any(char::is_control)
}
fn check_query(query: &RegistryQuery) -> Result<()> {
    if query.limit.is_some_and(|n| n == 0 || n > 100)
        || !bounded_text(&query.search, 256)
        || query
            .cursor
            .as_ref()
            .is_some_and(|s| !bounded_text(s, 4096))
        || query.runtime.as_ref().is_some_and(|s| {
            !["npm", "pypi", "cargo", "oci", "mcpb", "nuget", "remote"].contains(&s.as_str())
        })
        || query
            .platform
            .as_ref()
            .is_some_and(|s| !["windows", "linux", "darwin"].contains(&s.as_str()))
        || query
            .updated_since
            .as_ref()
            .is_some_and(|s| chrono::DateTime::parse_from_rfc3339(s).is_err())
    {
        return Err(AppError::new(
            "INVALID_REGISTRY_QUERY",
            "Invalid or oversized registry search, cursor, runtime, platform or timestamp",
        ));
    }
    if query.platform.is_some() {
        return Err(AppError::new(
            "REGISTRY_PLATFORM_UNSPECIFIED",
            "The supported upstream schema does not declare universal OS compatibility; inspect the reviewed installer plan for this platform",
        ));
    }
    Ok(())
}
fn check_id(id: &str) -> Result<()> {
    if id.len() > 255
        || id.split('/').count() != 2
        || id.split('/').any(|p| p.is_empty() || p == "." || p == "..")
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/._-".contains(&b))
    {
        return Err(AppError::new(
            "INVALID_REGISTRY_ID",
            "Use the exact namespace/server registry identifier",
        ));
    }
    Ok(())
}
fn endpoint(
    base: &str,
    id: Option<&str>,
    version: Option<&str>,
    versions: bool,
    query: &RegistryQuery,
) -> Result<url::Url> {
    check_query(query)?;
    let mut url = url::Url::parse(base).map_err(|_| invalid())?;
    {
        let mut segments = url.path_segments_mut().map_err(|_| invalid())?;
        segments
            .pop_if_empty()
            .push(MCP_REGISTRY_API)
            .push("servers");
        if let Some(id) = id {
            check_id(id)?;
            segments.push(id).push("versions"); // push encodes an identifier as ONE path segment.
            if !versions {
                let version = version.unwrap_or("latest");
                if version.is_empty()
                    || !bounded_text(version, 255)
                    || [".", ".."].contains(&version)
                {
                    return Err(invalid());
                }
                segments.push(version);
            }
        }
    }
    {
        let mut params = url.query_pairs_mut();
        params.append_pair("limit", &query.limit.unwrap_or(20).to_string());
        if id.is_none() {
            params.append_pair("version", "latest");
            if !query.search.is_empty() {
                params.append_pair("search", &query.search);
            }
        }
        if let Some(cursor) = &query.cursor {
            params.append_pair("cursor", cursor);
        }
        if let Some(since) = &query.updated_since {
            params
                .append_pair("updated_since", since)
                .append_pair("include_deleted", "true");
        }
    }
    Ok(url)
}
fn parse_entry(response: &Value) -> Result<RegistryEntry> {
    let server = response
        .get("server")
        .filter(|s| s.is_object())
        .ok_or_else(invalid)?;
    if server.get("$schema").and_then(Value::as_str)
        == Some("https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json")
    {
        static SCHEMA: std::sync::OnceLock<jsonschema::Validator> = std::sync::OnceLock::new();
        let validator = SCHEMA.get_or_init(|| {
            let schema: Value = serde_json::from_str(include_str!(
                "../../../manifests/mcp/server-2025-12-11.schema.json"
            ))
            .expect("Bundled registry schema is valid JSON");
            jsonschema::validator_for(&schema)
                .expect("Bundled registry schema compiles without network references")
        });
        if !validator.is_valid(server) {
            return Err(invalid());
        }
    }
    let text = |key| server.get(key).and_then(Value::as_str).ok_or_else(invalid);
    let id = text("name")?;
    check_id(id).map_err(|_| invalid())?;
    let version = text("version")?;
    if version.is_empty() || !bounded_text(version, 255) {
        return Err(invalid());
    }
    let description = text("description")?;
    if description.len() > 16384 {
        return Err(invalid());
    }
    let packages = match server.get("packages") {
        None => vec![],
        Some(Value::Array(a)) if a.len() <= 32 => a.clone(),
        _ => return Err(invalid()),
    };
    let mut transports = Vec::new();
    for p in &packages {
        for key in ["registryType", "identifier"] {
            if p.get(key)
                .and_then(Value::as_str)
                .is_none_or(|s| s.is_empty() || !bounded_text(s, 2048))
            {
                return Err(invalid());
            }
        }
        if let Some(hash) = p.get("fileSha256")
            && hash
                .as_str()
                .is_none_or(|s| s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return Err(invalid());
        }
        let transport = p
            .pointer("/transport/type")
            .and_then(Value::as_str)
            .ok_or_else(invalid)?;
        transports.push(transport.to_owned());
    }
    if let Some(remotes) = server.get("remotes") {
        let remotes = remotes
            .as_array()
            .filter(|a| a.len() <= 32)
            .ok_or_else(invalid)?;
        for remote in remotes {
            transports.push(
                remote
                    .get("type")
                    .and_then(Value::as_str)
                    .ok_or_else(invalid)?
                    .to_owned(),
            );
            if remote.get("url").and_then(Value::as_str).is_none() {
                return Err(invalid());
            }
        }
    }
    transports.sort();
    transports.dedup();
    let metadata = response.get("_meta").cloned().unwrap_or_else(|| json!({}));
    let status = metadata
        .pointer("/io.modelcontextprotocol.registry~1official/status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    Ok(RegistryEntry {
        id: id.into(),
        name: server
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or(id)
            .into(),
        description: description.into(),
        publisher: id.split('/').next().unwrap_or("").into(),
        version: version.into(),
        repository: server
            .pointer("/repository/url")
            .and_then(Value::as_str)
            .filter(|s| {
                url::Url::parse(s).is_ok_and(|u| {
                    u.scheme() == "https"
                        && u.host_str().is_some()
                        && u.username().is_empty()
                        && u.password().is_none()
                })
            })
            .map(String::from),
        packages,
        transports,
        status: status.into(),
        trust: "official_registry_entry_unreviewed".into(),
        schema_supported: server.get("$schema").and_then(Value::as_str)
            == Some("https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json"),
        server_json: server.clone(),
        registry_metadata: metadata,
    })
}
fn parse_page(value: &Value, detail: bool) -> Result<RegistryPage> {
    let entries = if detail {
        vec![parse_entry(value)?]
    } else {
        let servers = value
            .get("servers")
            .and_then(Value::as_array)
            .filter(|a| a.len() <= 100)
            .ok_or_else(invalid)?;
        servers
            .iter()
            .map(parse_entry)
            .collect::<Result<Vec<_>>>()?
    };
    let next = value.pointer("/metadata/nextCursor");
    let next_cursor = match next {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) if bounded_text(s, 4096) => {
            if s.is_empty() {
                None
            } else {
                Some(s.clone())
            }
        }
        _ => return Err(invalid()),
    };
    Ok(RegistryPage {
        api_version: MCP_REGISTRY_API.into(),
        entries,
        next_cursor,
        fetched_at: now(),
        expires_at: (Utc::now() + chrono::Duration::seconds(TTL_SECONDS)).to_rfc3339(),
        cached: false,
        stale: false,
        warning: None,
    })
}
#[derive(Clone)]
pub struct RegistryClient {
    base: String,
    timeout: Duration,
}
impl Default for RegistryClient {
    fn default() -> Self {
        Self {
            base: BASE.into(),
            timeout: Duration::from_secs(30),
        }
    }
}
impl RegistryClient {
    async fn fetch(&self, url: url::Url) -> Result<Value> {
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(self.timeout)
            .connect_timeout(Duration::from_secs(5))
            .user_agent(concat!("ReconXAI/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| invalid())?;
        for attempt in 0..3u32 {
            match client
                .get(url.clone())
                .header("Accept", "application/json")
                .send()
                .await
            {
                Ok(mut response) => {
                    let status = response.status();
                    if status.as_u16() == 429 || status.is_server_error() {
                        if attempt == 2 {
                            return Err(AppError::new(
                                if status.as_u16() == 429 {
                                    "REGISTRY_RATE_LIMIT"
                                } else {
                                    "REGISTRY_UNAVAILABLE"
                                },
                                "Registry is unavailable or rate limited; cached metadata may be browsed offline",
                            ));
                        }
                        let retry = response
                            .headers()
                            .get("retry-after")
                            .and_then(|s| s.to_str().ok())
                            .and_then(|s| {
                                s.parse::<u64>().ok().or_else(|| {
                                    chrono::DateTime::parse_from_rfc2822(s).ok().map(|date| {
                                        (date.with_timezone(&Utc) - Utc::now()).num_seconds().max(0)
                                            as u64
                                    })
                                })
                            })
                            .unwrap_or(1 << attempt);
                        if retry > 4 {
                            return Err(AppError::new(
                                "REGISTRY_RATE_LIMIT",
                                "Registry requested a longer retry delay; refresh later or browse cached metadata",
                            ));
                        }
                        tokio::time::sleep(Duration::from_secs(retry)).await;
                        continue;
                    }
                    if status.as_u16() == 404 {
                        return Err(AppError::new(
                            "REGISTRY_NOT_FOUND",
                            "Registry entry/version is deleted or unavailable",
                        ));
                    }
                    if !status.is_success() {
                        return Err(AppError::new(
                            "REGISTRY_HTTP",
                            "Registry rejected the metadata request; redirects are not followed",
                        ));
                    }
                    if response
                        .content_length()
                        .is_some_and(|s| s > MAX_BODY as u64)
                    {
                        return Err(AppError::new(
                            "REGISTRY_LIMIT",
                            "Registry response exceeds 2 MiB",
                        ));
                    }
                    let mut body = Vec::new();
                    while let Some(chunk) = response.chunk().await.map_err(|_| {
                        AppError::new(
                            "REGISTRY_NETWORK",
                            "Registry body was interrupted or timed out",
                        )
                    })? {
                        if body.len() + chunk.len() > MAX_BODY {
                            return Err(AppError::new(
                                "REGISTRY_LIMIT",
                                "Registry response exceeds 2 MiB",
                            ));
                        }
                        body.extend_from_slice(&chunk);
                    }
                    return serde_json::from_slice(&body).map_err(|_| invalid());
                }
                Err(_) if attempt < 2 => {
                    tokio::time::sleep(Duration::from_millis(250 * (1 << attempt))).await
                }
                Err(_) => {
                    return Err(AppError::new(
                        "REGISTRY_NETWORK",
                        "Registry connection timed out or failed; no credentials were sent",
                    ));
                }
            }
        }
        Err(invalid())
    }
}
impl Core {
    pub async fn registry_search(&self, query: RegistryQuery) -> Result<RegistryPage> {
        self.registry_query(&RegistryClient::default(), query, None, None, false)
            .await
    }
    pub async fn registry_detail(
        &self,
        id: &str,
        version: Option<&str>,
        offline: bool,
    ) -> Result<RegistryPage> {
        self.registry_query(
            &RegistryClient::default(),
            RegistryQuery {
                offline,
                ..Default::default()
            },
            Some(id),
            version,
            false,
        )
        .await
    }
    pub async fn registry_versions(&self, id: &str, offline: bool) -> Result<RegistryPage> {
        self.registry_query(
            &RegistryClient::default(),
            RegistryQuery {
                offline,
                ..Default::default()
            },
            Some(id),
            None,
            true,
        )
        .await
    }
    async fn registry_query(
        &self,
        client: &RegistryClient,
        query: RegistryQuery,
        id: Option<&str>,
        version: Option<&str>,
        versions: bool,
    ) -> Result<RegistryPage> {
        let url = endpoint(&client.base, id, version, versions, &query)?;
        let key = reconx_tools::hash(url.as_str().as_bytes());
        let cached:Option<String>=self.db.connect()?.query_row("SELECT body_json FROM mcp_registry_cache WHERE cache_key=?1 AND cache_schema=1 AND api_version=?2",params![key,MCP_REGISTRY_API],|r|r.get(0)).optional().map_err(db_error)?;
        let cached = cached
            .map(|s| {
                if s.len() > MAX_BODY * 3 {
                    return Err(invalid());
                }
                let mut page = serde_json::from_str::<RegistryPage>(&s).map_err(json_error)?;
                page.entries = page
                    .entries
                    .into_iter()
                    .map(|entry| {
                        parse_entry(
                            &json!({"server":entry.server_json,"_meta":entry.registry_metadata}),
                        )
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(page)
            })
            .transpose()?;
        if let Some(mut page) = cached.clone() {
            page.stale = chrono::DateTime::parse_from_rfc3339(&page.expires_at)
                .map_err(|_| invalid())?
                < Utc::now();
            if query.offline || (!query.refresh && !page.stale) {
                page.cached = true;
                page.warning = page
                    .stale
                    .then(|| "Stale cached metadata; this is not a latest-version check".into());
                return Ok(filter_page(page, &query));
            }
        }
        if query.offline {
            return Err(AppError::new(
                "REGISTRY_CACHE_MISS",
                "No cached page for this search/version; connect and refresh the catalog first",
            ));
        }
        let response = client.fetch(url).await;
        let mut page = match response {
            Ok(value) => parse_page(&value, id.is_some() && !versions)?,
            Err(error)
                if cached.is_some()
                    && [
                        "REGISTRY_NETWORK",
                        "REGISTRY_RATE_LIMIT",
                        "REGISTRY_UNAVAILABLE",
                    ]
                    .contains(&error.code.as_str()) =>
            {
                let mut page = cached.ok_or_else(invalid)?;
                page.cached = true;
                page.stale = true;
                page.warning = Some(format!(
                    "{}; showing cached metadata, not latest availability",
                    error.code
                ));
                return Ok(filter_page(page, &query));
            }
            Err(error) => return Err(error),
        };
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute("INSERT INTO mcp_registry_cache VALUES(?1,1,?2,?3,?4,?5) ON CONFLICT(cache_key) DO UPDATE SET cache_schema=1,api_version=excluded.api_version,body_json=excluded.body_json,fetched_at=excluded.fetched_at,expires_at=excluded.expires_at",params![key,MCP_REGISTRY_API,serde_json::to_string(&page).map_err(json_error)?,page.fetched_at,page.expires_at]).map_err(db_error)?;
        for entry in &page.entries {
            tx.execute("INSERT INTO mcp_registry_entries VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(server_id,version) DO UPDATE SET server_json=excluded.server_json,registry_metadata=excluded.registry_metadata,fetched_at=excluded.fetched_at,expires_at=excluded.expires_at",params![entry.id,entry.version,entry.server_json.to_string(),entry.registry_metadata.to_string(),page.fetched_at,page.expires_at]).map_err(db_error)?;
        }
        tx.execute("DELETE FROM mcp_registry_cache WHERE cache_key NOT IN (SELECT cache_key FROM mcp_registry_cache ORDER BY fetched_at DESC LIMIT 128)",[]).map_err(db_error)?;
        tx.execute("DELETE FROM mcp_registry_entries WHERE rowid NOT IN (SELECT rowid FROM mcp_registry_entries ORDER BY fetched_at DESC LIMIT 2000)",[]).map_err(db_error)?;
        Self::audit(
            &tx,
            "registry.fetch",
            "official-mcp-registry",
            "metadata_only",
        )?;
        tx.commit().map_err(db_error)?;
        page.cached = false;
        Ok(filter_page(page, &query))
    }
    pub fn registry_clear_cache(&self) -> Result<()> {
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute("DELETE FROM mcp_registry_cache", [])
            .map_err(db_error)?;
        tx.execute("DELETE FROM mcp_registry_entries", [])
            .map_err(db_error)?;
        Self::audit(
            &tx,
            "registry.clear_cache",
            "official-mcp-registry",
            "completed",
        )?;
        tx.commit().map_err(db_error)
    }
}
fn filter_page(mut page: RegistryPage, query: &RegistryQuery) -> RegistryPage {
    page.entries.retain(|e| {
        query.runtime.as_ref().is_none_or(|runtime| {
            if runtime == "remote" {
                e.server_json
                    .get("remotes")
                    .and_then(Value::as_array)
                    .is_some_and(|a| !a.is_empty())
            } else {
                e.packages
                    .iter()
                    .any(|p| p["registryType"] == runtime.as_str())
            }
        })
    });
    page
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::{Arc, Mutex},
    };
    #[tokio::test]
    #[ignore = "Explicit opt-in public HTTPS diagnostic; no credentials or project data"]
    async fn live_public_https_diagnostic() {
        let response = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap()
            .get(
                endpoint(
                    BASE,
                    None,
                    None,
                    false,
                    &RegistryQuery {
                        search: "everything".into(),
                        ..Default::default()
                    },
                )
                .unwrap(),
            )
            .send()
            .await;
        assert!(response.is_ok(), "Public origin diagnostic: {response:?}");
    }
    fn response() -> Value {
        json!({"servers":[{"server":{"$schema":"https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json","name":"io.example/fixture","description":"Untrusted <script>text</script>","version":"1.2.3","packages":[{"registryType":"npm","identifier":"@example/fixture","version":"1.2.3","transport":{"type":"stdio"}}],"_meta":{"io.example":{"custom":true}}},"_meta":{"io.modelcontextprotocol.registry/official":{"status":"active"}}}],"metadata":{"nextCursor":"opaque+/cursor=&?","count":1}})
    }
    fn fixture(
        replies: Vec<(u16, Vec<u8>, String)>,
    ) -> (
        RegistryClient,
        Arc<Mutex<Vec<String>>>,
        std::thread::JoinHandle<()>,
    ) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        listener.set_nonblocking(true).unwrap();
        let worker = std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + Duration::from_secs(8);
            for (status, body, headers) in replies {
                let mut stream = loop {
                    match listener.accept() {
                        Ok((s, _)) => break s,
                        Err(_) if std::time::Instant::now() < deadline => {
                            std::thread::sleep(Duration::from_millis(2))
                        }
                        _ => return,
                    }
                };
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(1)))
                    .unwrap();
                let mut bytes = Vec::new();
                while bytes.len() < 16384 && !bytes.ends_with(b"\r\n\r\n") {
                    let mut chunk = [0u8; 1024];
                    let length = stream.read(&mut chunk).unwrap_or(0);
                    if length == 0 {
                        break;
                    }
                    bytes.extend_from_slice(&chunk[..length]);
                }
                captured
                    .lock()
                    .unwrap()
                    .push(String::from_utf8_lossy(&bytes).into());
                let _ = write!(
                    stream,
                    "HTTP/1.1 {status} Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{headers}Connection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(&body);
            }
        });
        (
            RegistryClient {
                base: format!("http://{address}/"),
                timeout: Duration::from_secs(2),
            },
            requests,
            worker,
        )
    }
    #[test]
    fn identifiers_and_opaque_cursors_are_encoded_without_path_injection() {
        let query = RegistryQuery {
            cursor: Some("a+/b?&= Unicode".into()),
            ..Default::default()
        };
        let url = endpoint(
            BASE,
            Some("io.example/server"),
            Some("1.0.0+build"),
            false,
            &query,
        )
        .unwrap();
        assert!(
            url.path()
                .contains("io.example%2Fserver/versions/1.0.0+build")
        );
        assert_eq!(
            url.query_pairs().find(|(k, _)| k == "cursor").unwrap().1,
            "a+/b?&= Unicode"
        );
        for name in [
            "../escape",
            "io.example/../../escape",
            "io.example/a?b",
            "io.example/%2Fescape",
        ] {
            assert!(endpoint(BASE, Some(name), None, false, &query).is_err());
        }
        assert!(
            check_query(&RegistryQuery {
                search: "x".repeat(257),
                ..Default::default()
            })
            .is_err()
        );
        assert_eq!(
            check_query(&RegistryQuery {
                platform: Some("windows".into()),
                ..Default::default()
            })
            .unwrap_err()
            .code,
            "REGISTRY_PLATFORM_UNSPECIFIED"
        );
    }
    #[test]
    fn typed_metadata_preserves_server_json_and_never_grants_trust() {
        let value = response();
        let page = parse_page(&value, false).unwrap();
        assert_eq!(page.next_cursor.as_deref(), Some("opaque+/cursor=&?"));
        assert_eq!(page.entries[0].status, "active");
        assert_eq!(page.entries[0].trust, "official_registry_entry_unreviewed");
        assert_eq!(page.entries[0].server_json, value["servers"][0]["server"]);
        let mut changed = value.clone();
        changed["servers"][0]["server"]["$schema"] = json!("https://example.test/future.json");
        assert!(!parse_page(&changed, false).unwrap().entries[0].schema_supported);
        changed["servers"][0]["server"]["packages"][0]["fileSha256"] = json!("invalid");
        assert_eq!(
            parse_page(&changed, false).unwrap_err().code,
            "REGISTRY_SCHEMA"
        );
        assert!(parse_page(&json!({"data":[]}), false).is_err());
        assert!(parse_page(&json!({"servers":[],"metadata":{"nextCursor":123}}), false).is_err());
    }
    #[tokio::test]
    async fn real_http_cache_pagination_offline_restart_and_clear() {
        let bytes = serde_json::to_vec(&response()).unwrap();
        let (client, requests, worker) = fixture(vec![
            (200, bytes.clone(), String::new()),
            (200, bytes, String::new()),
        ]);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db");
        let core = Core::open(Some(path.clone())).unwrap();
        let first = core
            .registry_query(
                &client,
                RegistryQuery {
                    search: "fixture".into(),
                    ..Default::default()
                },
                None,
                None,
                false,
            )
            .await
            .unwrap();
        assert!(!first.cached);
        let second = core
            .registry_query(
                &client,
                RegistryQuery {
                    search: "fixture".into(),
                    cursor: first.next_cursor.clone(),
                    ..Default::default()
                },
                None,
                None,
                false,
            )
            .await
            .unwrap();
        assert!(!second.cached);
        worker.join().unwrap();
        assert!(requests.lock().unwrap()[1].contains("cursor=opaque%2B%2Fcursor%3D%26%3F"));
        let reopened = Core::open(Some(path)).unwrap();
        let cached = reopened
            .registry_query(
                &client,
                RegistryQuery {
                    search: "fixture".into(),
                    offline: true,
                    ..Default::default()
                },
                None,
                None,
                false,
            )
            .await
            .unwrap();
        assert!(cached.cached);
        assert!(!cached.stale);
        reopened.db.connect().unwrap().execute("UPDATE mcp_registry_cache SET body_json=json_set(body_json,'$.expires_at','2000-01-01T00:00:00Z')",[]).unwrap();
        let stale = reopened
            .registry_query(
                &client,
                RegistryQuery {
                    search: "fixture".into(),
                    refresh: true,
                    ..Default::default()
                },
                None,
                None,
                false,
            )
            .await
            .unwrap();
        assert!(stale.cached && stale.stale);
        assert!(stale.warning.unwrap().contains("REGISTRY_NETWORK"));
        reopened.registry_clear_cache().unwrap();
        assert_eq!(
            reopened
                .registry_query(
                    &client,
                    RegistryQuery {
                        offline: true,
                        ..Default::default()
                    },
                    None,
                    None,
                    false
                )
                .await
                .unwrap_err()
                .code,
            "REGISTRY_CACHE_MISS"
        );
    }
    #[tokio::test]
    async fn bounded_retries_rate_limit_invalid_json_and_limits() {
        let data = serde_json::to_vec(&response()).unwrap();
        let (client, requests, worker) = fixture(vec![
            (429, vec![], "Retry-After: 0\r\n".into()),
            (200, data, String::new()),
        ]);
        let result = client
            .fetch(endpoint(&client.base, None, None, false, &RegistryQuery::default()).unwrap())
            .await;
        assert!(
            result.is_ok(),
            "{result:?}; requests={:?}",
            requests.lock().unwrap()
        );
        worker.join().unwrap();
        assert_eq!(requests.lock().unwrap().len(), 2);
        for (status, body, headers, code) in [
            (
                429,
                vec![],
                "Retry-After: 3600\r\n".into(),
                "REGISTRY_RATE_LIMIT",
            ),
            (404, vec![], String::new(), "REGISTRY_NOT_FOUND"),
            (200, b"broken".to_vec(), String::new(), "REGISTRY_SCHEMA"),
            (
                200,
                vec![b'x'; MAX_BODY + 1],
                String::new(),
                "REGISTRY_LIMIT",
            ),
        ] {
            let (client, _, worker) = fixture(vec![(status, body, headers)]);
            assert_eq!(
                client
                    .fetch(
                        endpoint(&client.base, None, None, false, &RegistryQuery::default())
                            .unwrap()
                    )
                    .await
                    .unwrap_err()
                    .code,
                code
            );
            worker.join().unwrap();
        }
    }
}
