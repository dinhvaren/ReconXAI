//! Explicitly trusted MCP connections. Tool metadata and results remain untrusted.
use super::*;
use rmcp::{
    ClientLifecycleMode, ClientServiceExt, RoleClient,
    model::{
        CallToolRequestParams, ClientCapabilities, ClientConfig, Implementation, ProtocolVersion,
    },
    service::RunningService,
    transport::{
        StreamableHttpClientTransport, streamable_http_client::StreamableHttpClientTransportConfig,
    },
};
use std::{collections::HashMap, process::Stdio, time::Duration};
use tokio::{io::AsyncReadExt, process::Child, sync::Mutex};

const OUTPUT_LIMIT: usize = 256 * 1024;
type Client = RunningService<RoleClient, ClientConfig>;
struct Session {
    client: Client,
    child: Option<Child>,
    revision: String,
    secrets: Vec<Zeroizing<String>>,
}
#[derive(Default)]
pub(super) struct McpGateway {
    sessions: Mutex<HashMap<String, Session>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    pub transport: String,
    pub executable: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    pub working_directory: Option<String>,
    pub endpoint: Option<String>,
    #[serde(default)]
    pub environment: Vec<String>,
    #[serde(default)]
    pub bearer_auth: bool,
    pub enabled: bool,
    pub trusted: bool,
    pub timeout_ms: u64,
    /// Only manually reviewed, read-only, closed-world tools may be called.
    #[serde(default)]
    pub allowed_tools: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub config: McpServerConfig,
    pub revision: String,
    pub tools: Vec<serde_json::Value>,
    pub last_connected_at: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpApproval {
    pub server_id: String,
    pub tool: String,
    pub input: serde_json::Value,
    pub project_id: String,
    pub fingerprint: String,
    pub approved: bool,
}
fn fingerprint(value: &impl Serialize) -> Result<String> {
    Ok(reconx_tools::hash(
        &serde_json::to_vec(value).map_err(json_error)?,
    ))
}
fn mcp_error() -> AppError {
    AppError::new(
        "MCP_CONNECTION",
        "MCP communication failed; check the configured server and transport",
    )
}
fn bounded(value: serde_json::Value) -> Result<serde_json::Value> {
    if serde_json::to_vec(&value).map_err(json_error)?.len() > OUTPUT_LIMIT {
        return Err(AppError::new(
            "MCP_OUTPUT_LIMIT",
            "MCP data exceeds 256 KiB",
        ));
    }
    Ok(value)
}
fn redact(value: &mut serde_json::Value, secrets: &[Zeroizing<String>]) {
    fn text(s: &str, secrets: &[Zeroizing<String>]) -> String {
        let mut value = s.to_owned();
        for secret in secrets {
            value = value.replace(secret.as_str(), "[REDACTED]");
        }
        value
    }
    match value {
        serde_json::Value::String(s) => *s = text(s, secrets),
        serde_json::Value::Array(items) => items.iter_mut().for_each(|v| redact(v, secrets)),
        serde_json::Value::Object(fields) => {
            let original = std::mem::take(fields);
            for (key, mut v) in original {
                redact(&mut v, secrets);
                fields.insert(text(&key, secrets), v);
            }
        }
        _ => {}
    }
}
fn credential_input(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(fields) => fields.iter().any(|(key, v)| {
            let key = key.to_ascii_lowercase().replace(['-', '_'], "");
            [
                "authorization",
                "cookie",
                "password",
                "secret",
                "apikey",
                "accesstoken",
                "refreshtoken",
                "token",
            ]
            .contains(&key.as_str())
                || credential_input(v)
        }),
        serde_json::Value::Array(items) => items.iter().any(credential_input),
        _ => false,
    }
}
fn validate_schema(schema: &serde_json::Value) -> Result<jsonschema::Validator> {
    // External schema retrieval is disabled in Cargo; reject all references anyway.
    fn refs(v: &serde_json::Value) -> bool {
        match v {
            serde_json::Value::Object(o) => {
                o.contains_key("$ref") || o.contains_key("$dynamicRef") || o.values().any(refs)
            }
            serde_json::Value::Array(a) => a.iter().any(refs),
            _ => false,
        }
    }
    if refs(schema) {
        return Err(AppError::new(
            "MCP_SCHEMA",
            "Referenced schemas are not supported in this bounded adapter",
        ));
    }
    jsonschema::validator_for(schema)
        .map_err(|_| AppError::new("MCP_SCHEMA", "MCP input schema is invalid"))
}
impl Core {
    pub fn mcp_history(&self, id: &str) -> Result<Vec<AnalysisJob>> {
        let ids=self.db.connect()?.prepare("SELECT job_id FROM capability_invocations WHERE server_id=?1 ORDER BY approved_at DESC LIMIT 100").map_err(db_error)?.query_map([id],|r|r.get::<_,String>(0)).map_err(db_error)?.collect::<std::result::Result<Vec<_>,_>>().map_err(db_error)?;
        ids.iter().map(|id| self.job(id)).collect()
    }
    pub fn mcp_servers(&self) -> Result<Vec<McpServer>> {
        let rows=self.db.connect()?.prepare("SELECT config_json,revision,tools_json,last_connected_at FROM mcp_servers ORDER BY updated_at DESC").map_err(db_error)?.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,Option<String>>(3)?))).map_err(db_error)?.collect::<std::result::Result<Vec<_>,_>>().map_err(db_error)?;
        rows.into_iter()
            .map(|(c, revision, t, last_connected_at)| {
                Ok(McpServer {
                    config: serde_json::from_str(&c).map_err(json_error)?,
                    revision,
                    tools: serde_json::from_str(&t).map_err(json_error)?,
                    last_connected_at,
                })
            })
            .collect()
    }
    pub fn mcp_server(&self, id: &str) -> Result<McpServer> {
        self.mcp_servers()?
            .into_iter()
            .find(|s| s.config.id == id)
            .ok_or_else(|| AppError::new("NOT_FOUND", "MCP server does not exist"))
    }
    pub async fn mcp_save(&self, mut config: McpServerConfig) -> Result<McpServer> {
        config.name = validate_text(&config.name, 120, "Server name")?;
        if config.id.is_empty() {
            config.id = Uuid::new_v4().to_string();
        }
        if config.id.len() > 80
            || !config
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || !(100..=120_000).contains(&config.timeout_ms)
            || config.args.len() > 32
            || config
                .args
                .iter()
                .any(|a| a.len() > 2048 || a.contains('\0'))
            || config.allowed_tools.len() > 64
            || config.environment.len() > 16
            || config
                .allowed_tools
                .iter()
                .any(|n| n.is_empty() || n.len() > 128)
        {
            return Err(AppError::new(
                "INVALID_INPUT",
                "Invalid MCP limits, identifier or arguments",
            ));
        }
        for key in &config.environment {
            if key.is_empty()
                || key.len() > 64
                || !key
                    .bytes()
                    .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
                || ["PATH", "SYSTEMROOT", "WINDIR", "HOME", "USERPROFILE"].contains(&key.as_str())
            {
                return Err(AppError::new(
                    "INVALID_INPUT",
                    "Environment names must be uppercase; host configuration overrides are prohibited",
                ));
            }
        }
        match config.transport.as_str() {
            "stdio" => {
                let path = std::path::Path::new(config.executable.as_deref().unwrap_or(""));
                if !path.is_absolute() || !path.is_file() {
                    return Err(AppError::new(
                        "MCP_EXECUTABLE",
                        "Select an existing absolute executable path",
                    ));
                }
                if config.endpoint.is_some() || config.bearer_auth {
                    return Err(AppError::new(
                        "INVALID_INPUT",
                        "stdio does not use HTTP authentication",
                    ));
                }
                if let Some(p) = &config.working_directory
                    && (!std::path::Path::new(p).is_absolute() || !std::path::Path::new(p).is_dir())
                {
                    return Err(AppError::new(
                        "INVALID_INPUT",
                        "Working directory must be an existing absolute directory",
                    ));
                }
            }
            "streamable_http" => {
                reconx_security::validate_gateway(config.endpoint.as_deref().unwrap_or(""))?;
                if config.executable.is_some()
                    || !config.args.is_empty()
                    || !config.environment.is_empty()
                {
                    return Err(AppError::new(
                        "INVALID_INPUT",
                        "HTTP connections cannot configure a subprocess",
                    ));
                }
            }
            _ => {
                return Err(AppError::new(
                    "MCP_TRANSPORT",
                    "Choose stdio or streamable_http",
                ));
            }
        }
        self.mcp_disconnect(&config.id).await?;
        let revision = fingerprint(&config)?;
        self.db.connect()?.execute("INSERT INTO mcp_servers(id,config_json,revision,updated_at) VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET config_json=excluded.config_json,revision=excluded.revision,tools_json='[]',updated_at=excluded.updated_at",params![config.id,serde_json::to_string(&config).map_err(json_error)?,revision,now()]).map_err(db_error)?;
        Self::audit(&self.db.connect()?, "mcp.configured", &config.id, "saved")?;
        self.mcp_server(&config.id)
    }
    pub fn mcp_set_secret(&self, id: &str, name: &str, key: &str) -> Result<()> {
        let s = self.mcp_server(id)?;
        if !(s.config.environment.iter().any(|n| n == name)
            || name == "bearer" && s.config.bearer_auth)
        {
            return Err(AppError::new(
                "MCP_SECRET",
                "Secret is not declared in server configuration",
            ));
        }
        if key.is_empty() || key.len() > 8192 || key.contains(['\r', '\n', '\0']) {
            return Err(AppError::new("INVALID_INPUT", "Invalid MCP credential"));
        }
        self.secrets
            .set(&format!("mcp:{}:{}:{name}", id, s.revision), key)?;
        Self::audit(&self.db.connect()?, "mcp.credential.changed", id, "stored")
    }
    pub async fn mcp_disconnect(&self, id: &str) -> Result<()> {
        if let Some(mut session) = self.mcp_gateway.sessions.lock().await.remove(id) {
            let _ = tokio::time::timeout(Duration::from_secs(2), session.client.close()).await;
            if let Some(child) = session.child.as_mut() {
                let _ = child.kill().await;
                let _ = child.wait().await;
            }
        }
        Ok(())
    }
    pub async fn mcp_remove(&self, id: &str) -> Result<()> {
        let s = self.mcp_server(id)?;
        self.mcp_disconnect(id).await?;
        for name in s
            .config
            .environment
            .iter()
            .map(String::as_str)
            .chain(s.config.bearer_auth.then_some("bearer"))
        {
            self.secrets
                .delete(&format!("mcp:{id}:{}:{name}", s.revision))?;
        }
        self.db
            .connect()?
            .execute("DELETE FROM mcp_servers WHERE id=?1", [id])
            .map_err(db_error)?;
        Ok(())
    }
    pub async fn mcp_state(&self, id: &str) -> String {
        let sessions = self.mcp_gateway.sessions.lock().await;
        if sessions.get(id).is_some_and(|s| !s.client.is_closed()) {
            "connected"
        } else {
            "disconnected"
        }
        .into()
    }
    pub async fn mcp_connect(&self, id: &str) -> Result<McpServer> {
        let s = self.mcp_server(id)?;
        if !s.config.enabled || !s.config.trusted {
            return Err(AppError::new(
                "MCP_TRUST_REQUIRED",
                "Enable and explicitly trust the executable or endpoint before connection; local servers run with your account permissions",
            ));
        }
        let mut sessions = self.mcp_gateway.sessions.lock().await;
        if sessions
            .get(id)
            .is_some_and(|c| c.revision == s.revision && !c.client.is_closed())
        {
            return Ok(s);
        }
        sessions.remove(id);
        if sessions.len() >= 8 {
            return Err(AppError::new(
                "MCP_LIMIT",
                "Disconnect a server before opening more than eight sessions",
            ));
        }
        let mut secrets = Vec::new();
        let mut values = HashMap::new();
        for name in s
            .config
            .environment
            .iter()
            .map(String::as_str)
            .chain(s.config.bearer_auth.then_some("bearer"))
        {
            let key = self
                .secrets
                .get(&format!("mcp:{id}:{}:{name}", s.revision))?
                .ok_or_else(|| {
                    AppError::new(
                        "MCP_CREDENTIAL_MISSING",
                        "Set the declared MCP secret in OS credential storage",
                    )
                })?;
            values.insert(name.to_owned(), key.to_string());
            secrets.push(key);
        }
        let config = ClientConfig::new(
            ClientCapabilities::default(),
            Implementation::new("ReconXAI", env!("CARGO_PKG_VERSION")),
        );
        let lifecycle = ClientLifecycleMode::Auto {
            preferred_versions: vec![ProtocolVersion::V_2026_07_28],
            legacy_version: Some(ProtocolVersion::V_2025_11_25),
        };
        let mut child = None;
        let timeout = Duration::from_millis(s.config.timeout_ms);
        let client = if s.config.transport == "stdio" {
            let mut command =
                tokio::process::Command::new(s.config.executable.as_deref().unwrap_or(""));
            command
                .env_clear()
                .args(&s.config.args)
                .kill_on_drop(true)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null());
            for name in ["SystemRoot", "WINDIR", "TEMP", "TMP"] {
                if let Some(value) = std::env::var_os(name) {
                    command.env(name, value);
                }
            }
            command.envs(&values);
            if let Some(path) = &s.config.working_directory {
                command.current_dir(path);
            }
            #[cfg(windows)]
            {
                command.creation_flags(0x08000000);
            }
            let mut process = command.spawn().map_err(|_| mcp_error())?;
            let stdout = process
                .stdout
                .take()
                .ok_or_else(mcp_error)?
                .take(4 * 1024 * 1024);
            let stdin = process.stdin.take().ok_or_else(mcp_error)?;
            child = Some(process);
            tokio::time::timeout(
                timeout,
                config.serve_with_lifecycle((stdout, stdin), lifecycle),
            )
            .await
            .map_err(|_| AppError::new("MCP_TIMEOUT", "MCP connection timed out"))?
            .map_err(|_| mcp_error())?
        } else {
            let http = reqwest::Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(timeout)
                .build()
                .map_err(|_| mcp_error())?;
            let mut settings = StreamableHttpClientTransportConfig::with_uri(
                s.config.endpoint.clone().unwrap_or_default(),
            );
            settings.max_sse_event_size = OUTPUT_LIMIT;
            settings.reinit_on_expired_session = false; // Never transparently replay tools/call.
            settings.max_concurrent_requests = 1;
            settings.auth_header = values.remove("bearer");
            let transport = StreamableHttpClientTransport::with_client(
                super::mcp_http::BoundedHttp(http),
                settings,
            );
            tokio::time::timeout(timeout, config.serve_with_lifecycle(transport, lifecycle))
                .await
                .map_err(|_| AppError::new("MCP_TIMEOUT", "MCP connection timed out"))?
                .map_err(|_| mcp_error())?
        };
        // One bounded page. Do not follow an untrusted infinite pagination chain.
        let page = tokio::time::timeout(timeout, client.list_tools(None))
            .await
            .map_err(|_| AppError::new("MCP_TIMEOUT", "Tool discovery timed out"))?
            .map_err(|_| mcp_error())?;
        if page.next_cursor.is_some() || page.tools.len() > 64 {
            return Err(AppError::new(
                "MCP_LIMIT",
                "Server exceeds 64-tool single-page discovery limit",
            ));
        }
        let mut tools = page
            .tools
            .into_iter()
            .map(|t| serde_json::to_value(t).map_err(json_error))
            .collect::<Result<Vec<_>>>()?;
        for tool in &mut tools {
            redact(tool, &secrets);
        }
        bounded(serde_json::json!(&tools))?;
        for tool in &tools {
            validate_schema(&tool["inputSchema"])?;
        }
        self.db.connect()?.execute("UPDATE mcp_servers SET tools_json=?1,last_connected_at=?2 WHERE id=?3 AND revision=?4",params![serde_json::to_string(&tools).map_err(json_error)?,now(),id,s.revision]).map_err(db_error)?;
        sessions.insert(
            id.into(),
            Session {
                client,
                child,
                revision: s.revision,
                secrets,
            },
        );
        Self::audit(&self.db.connect()?, "mcp.connected", id, "discovered")?;
        self.mcp_server(id)
    }
    pub fn mcp_invocation_preview(
        &self,
        id: &str,
        name: &str,
        input: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let s = self.mcp_server(id)?;
        if !s.config.enabled
            || !s.config.trusted
            || !s.config.allowed_tools.iter().any(|n| n == name)
        {
            return Err(AppError::new(
                "BLOCKED_BY_POLICY",
                "Enable, trust and explicitly allow this MCP tool before invocation",
            ));
        }
        let tool = s
            .tools
            .iter()
            .find(|t| t["name"].as_str() == Some(name))
            .ok_or_else(|| {
                AppError::new("MCP_TOOL_NOT_FOUND", "Connect and discover this tool first")
            })?;
        let a = &tool["annotations"];
        if a["readOnlyHint"] != true || a["destructiveHint"] == true || a["openWorldHint"] != false
        {
            return Err(AppError::new(
                "BLOCKED_BY_POLICY",
                "This milestone permits only reviewed read-only, closed-world MCP tools",
            ));
        }
        if !input.is_object() {
            return Err(AppError::new(
                "MCP_INPUT",
                "Tool input must be a JSON object",
            ));
        }
        if credential_input(input) {
            return Err(AppError::new(
                "MCP_SECRET_INPUT",
                "Credential fields cannot be persisted as tool inputs; configure server authentication in secure storage",
            ));
        }
        bounded(input.clone())?;
        if !validate_schema(&tool["inputSchema"])?.is_valid(input) {
            return Err(AppError::new(
                "MCP_INPUT",
                "Input does not match the discovered tool schema",
            ));
        }
        let fp = fingerprint(&(s.revision, tool, input))?;
        Ok(
            serde_json::json!({"server_id":id,"tool":name,"input":input,"fingerprint":fp,"permissions":["manual approval","read-only","closed-world"],"warning":"Server code runs outside ReconX's sandbox. Trust and restrict the server itself. MCP output is untrusted."}),
        )
    }
    pub fn mcp_enqueue(&self, approval: McpApproval) -> Result<AnalysisJob> {
        if !approval.approved {
            return Err(AppError::new(
                "APPROVAL_REQUIRED",
                "Review and approve the exact MCP invocation",
            ));
        }
        let preview =
            self.mcp_invocation_preview(&approval.server_id, &approval.tool, &approval.input)?;
        if preview["fingerprint"] != approval.fingerprint {
            return Err(AppError::new(
                "APPROVAL_STALE",
                "Server, schema or input changed; review the invocation again",
            ));
        }
        let project = self.project(&approval.project_id)?;
        if project.archived {
            return Err(AppError::new(
                "PROJECT_ARCHIVED",
                "Restore the project first",
            ));
        }
        let s = self.mcp_server(&approval.server_id)?;
        let id = Uuid::new_v4().to_string();
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute("INSERT INTO analysis_jobs(id,project_id,kind,status,created_at,timeout_ms,logs_json) VALUES(?1,?2,'mcp_tool','queued',?3,?4,'[\"Approved manual MCP invocation queued\"]')",params![id,project.id,now(),s.config.timeout_ms as u32]).map_err(db_error)?;
        tx.execute("INSERT INTO capability_invocations(job_id,server_id,tool_name,input_json,approval_fingerprint,approved_at) VALUES(?1,?2,?3,?4,?5,?6)",params![id,approval.server_id,approval.tool,serde_json::to_string(&approval.input).map_err(json_error)?,approval.fingerprint,now()]).map_err(db_error)?;
        Self::audit(&tx, "mcp.approved", &id, "queued")?;
        tx.commit().map_err(db_error)?;
        self.job(&id)
    }
    pub(super) fn prepare_mcp_job(
        &self,
        job: &AnalysisJob,
        update: &mut impl FnMut(u8, &str) -> Result<()>,
    ) -> Result<jobs::PreparedJob> {
        let (id,tool,input,fp):(String,String,String,String)=self.db.connect()?.query_row("SELECT server_id,tool_name,input_json,approval_fingerprint FROM capability_invocations WHERE job_id=?1",[&job.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).map_err(db_error)?;
        let input: serde_json::Value = serde_json::from_str(&input).map_err(json_error)?;
        let preview = self.mcp_invocation_preview(&id, &tool, &input)?;
        if preview["fingerprint"] != fp {
            return Err(AppError::new(
                "APPROVAL_STALE",
                "MCP configuration or schema changed since approval",
            ));
        }
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| mcp_error())?;
        let isolated = Core {
            db: self.db.clone(),
            secrets: self.secrets.clone(),
            mcp_gateway: Arc::new(McpGateway::default()),
        };
        runtime.block_on(async {
          let result=async {
            update(15,"Connecting to explicitly trusted MCP server")?;
            {
              let connection=isolated.mcp_connect(&id);tokio::pin!(connection);
              loop {tokio::select! {connected=&mut connection=>{connected?;break;},_=tokio::time::sleep(Duration::from_millis(50))=>{update(15,"Connecting to explicitly trusted MCP server")?;}}}
            }
            let preview=self.mcp_invocation_preview(&id,&tool,&input)?;
            if preview["fingerprint"]!=fp{return Err(AppError::new("APPROVAL_STALE","Discovered schema changed; request new approval"));}
            let result={
                let sessions=isolated.mcp_gateway.sessions.lock().await;
                let session=sessions.get(&id).ok_or_else(mcp_error)?;
                let call=session.client.call_tool_once(CallToolRequestParams::new(tool.clone()).with_arguments(input.as_object().cloned().unwrap_or_default()));
                tokio::pin!(call);
                update(30,"Invoking approved MCP tool; no automatic retries")?;
                let value=loop { tokio::select! {
                    result=&mut call=>break result.map_err(|_|mcp_error())?,
                    _=tokio::time::sleep(Duration::from_millis(50))=>{ update(30,"Waiting for actual MCP response")?; }
                }};
                let value=match value { rmcp::model::CallToolResponse::Complete(v)=>v,_=>return Err(AppError::new("MCP_UNSUPPORTED_RESULT","Interactive follow-up and task responses require separate approval and are not executed")) };
                let mut value=serde_json::to_value(value).map_err(json_error)?;
                redact(&mut value,&session.secrets);
                if value.get("isError")==Some(&serde_json::json!(true)){return Err(AppError::new("MCP_TOOL_ERROR","MCP tool reported an execution error"));}
                bounded(value)?
            };
            if self.mcp_invocation_preview(&id,&tool,&input)?["fingerprint"]!=fp{return Err(AppError::new("APPROVAL_STALE","MCP permission changed during execution"));}
            update(90,"Received untrusted MCP result")?;
            Ok(jobs::PreparedJob{result:serde_json::json!({"provider":"mcp","server_id":id,"tool":tool,"untrusted":true,"response":result}),analysis:None,http:None})
          }.await;
          isolated.mcp_disconnect(&id).await?;
          result
        })
    }
}
