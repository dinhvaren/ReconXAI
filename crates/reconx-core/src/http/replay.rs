//! One manually approved request to an owned loopback fixture. No remote scanner.
use super::*;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::{Duration, Instant},
};
const KIND: &str = "manual_http_replay";
const PUBLIC_HEADERS: &[&str] = &[
    "accept",
    "cache-control",
    "if-none-match",
    "if-modified-since",
    "x-request-id",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpDraftInput {
    pub project_id: String,
    pub target_id: String,
    pub source_exchange_id: Option<String>,
    pub title: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<HttpHeader>,
    pub content_type: String,
    pub body: String,
    pub auth_profile_id: Option<String>,
    pub timeout_ms: u32,
    #[serde(default)]
    pub follow_redirects: bool,
    #[serde(default)]
    pub proxy: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpDraft {
    pub id: String,
    pub revision: i64,
    pub updated_at: String,
    #[serde(flatten)]
    pub input: HttpDraftInput,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpReplayPlan {
    pub draft: HttpDraft,
    pub destination_address: String,
    pub request_budget: u8,
    pub state_changing: bool,
    pub identity: Option<HttpAuthProfile>,
    pub fingerprint: String,
    pub safeguards: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HttpReplayApproval {
    pub draft_id: String,
    pub fingerprint: String,
    pub approved: bool,
}
fn denied(message: &str) -> AppError {
    AppError::new("HTTP_POLICY", message)
}
pub(super) fn lab_url(value: &str) -> Result<Url> {
    if value.len() > 8192 {
        return Err(denied("HTTP URL exceeds 8192 bytes"));
    }
    let url = Url::parse(value).map_err(|_| denied("Use an absolute HTTP(S) loopback URL"))?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(denied(
            "HTTP(S), no embedded credentials and no URL fragment are required",
        ));
    }
    let local = match url.host() {
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        Some(url::Host::Domain("localhost")) => true,
        _ => false,
    };
    if !local {
        return Err(denied(
            "This manual adapter accepts loopback IPs or exact localhost only; other destinations are unavailable",
        ));
    }
    Ok(url)
}
fn public_text(value: &str) -> Result<()> {
    if value.contains("[REDACTED]") || super::super::reporting::redact_content(value).1 > 0 {
        return Err(AppError::new(
            "CREDENTIAL_INPUT",
            "Remove redacted placeholders and credentials from the request draft; use a secure identity profile",
        ));
    }
    Ok(())
}
fn public_json(mut value: Value) -> Result<()> {
    if super::super::reporting::redact_value(&mut value) > 0 {
        return Err(AppError::new(
            "CREDENTIAL_INPUT",
            "Sensitive fields belong in secure identities, not ordinary request drafts",
        ));
    }
    Ok(())
}
fn validate_input(mut input: HttpDraftInput) -> Result<HttpDraftInput> {
    let url = lab_url(&input.url)?;
    input.url = url.to_string();
    input.title = validate_text(&input.title, 180, "Request title")?;
    public_text(&input.title)?;
    input.method = input.method.to_ascii_uppercase();
    if !["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"]
        .contains(&input.method.as_str())
    {
        return Err(denied("Unsupported HTTP method"));
    }
    if !(500..=10000).contains(&input.timeout_ms)
        || input.headers.len() > 32
        || input.body.len() > 64 * 1024
    {
        return Err(AppError::new(
            "INPUT_LIMIT",
            "Timeout must be 500-10000 ms, at most 32 headers and 64 KiB request body",
        ));
    }
    if input.follow_redirects || input.proxy.is_some() {
        return Err(denied(
            "Redirect following and proxies are unavailable in this one-request loopback adapter",
        ));
    }
    public_text(&input.url)?;
    let pairs: Vec<_> = url
        .query_pairs()
        .map(|(n, v)| json!({"name":n,"value":v}))
        .collect();
    let map: serde_json::Map<String, Value> = url
        .query_pairs()
        .map(|(n, v)| (n.into_owned(), json!(v)))
        .collect();
    public_json(json!(pairs))?;
    public_json(json!(map))?;
    let mut names = BTreeSet::new();
    for header in &mut input.headers {
        let name = reqwest::header::HeaderName::from_bytes(header.name.as_bytes())
            .map_err(|_| denied("Invalid request header name"))?;
        if !PUBLIC_HEADERS.contains(&name.as_str()) || !names.insert(name.as_str().to_owned()) {
            return Err(denied(
                "Use unique public Accept/Cache-Control/If-None-Match/If-Modified-Since/X-Request-ID headers; credentials use an identity profile",
            ));
        }
        if header.value.len() > 2048 {
            return Err(denied("Request header exceeds 2048 bytes"));
        }
        reqwest::header::HeaderValue::from_str(&header.value)
            .map_err(|_| denied("Invalid request header value"))?;
        public_text(&header.value)?;
        header.name = name.as_str().into();
    }
    if input.content_type.len() > 256 {
        return Err(denied("Content-Type exceeds 256 bytes"));
    }
    reqwest::header::HeaderValue::from_str(&input.content_type)
        .map_err(|_| denied("Invalid Content-Type"))?;
    public_text(&input.content_type)?;
    if input
        .content_type
        .to_ascii_lowercase()
        .starts_with("multipart/")
    {
        return Err(denied(
            "Multipart file upload is not implemented in this bounded editor",
        ));
    }
    public_text(&input.body)?;
    if !input.body.is_empty() {
        let mime = input
            .content_type
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        if mime == "application/json" || mime.ends_with("+json") {
            public_json(parse_json(input.body.as_bytes())?)?;
        } else if mime == "application/x-www-form-urlencoded" {
            let map: serde_json::Map<String, Value> =
                url::form_urlencoded::parse(input.body.as_bytes())
                    .map(|(n, v)| (n.into_owned(), json!(v)))
                    .collect();
            public_json(json!(map))?;
        }
    }
    Ok(input)
}
fn request_error(e: reqwest::Error) -> AppError {
    if e.is_timeout() {
        AppError::new(
            "HTTP_TIMEOUT",
            "The approved HTTP request timed out; it is not retried",
        )
    } else if e.is_connect() {
        AppError::new(
            "HTTP_CONNECT",
            "Cannot connect to the approved loopback service; check the service, port and TLS trust",
        )
    } else {
        AppError::new(
            "HTTP_REQUEST",
            "The approved HTTP request failed; it is not retried",
        )
    }
}
pub(crate) struct PendingReplayArtifact {
    pub result: Value,
    path: PathBuf,
    committed: bool,
}
impl PendingReplayArtifact {
    pub(crate) fn keep(&mut self) {
        self.committed = true;
    }
}
impl Drop for PendingReplayArtifact {
    fn drop(&mut self) {
        if !self.committed {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

impl Core {
    pub fn http_draft_seed(&self, exchange: &str, target: Option<&str>) -> Result<HttpDraftInput> {
        let capture = self.http_exchange(exchange)?;
        self.http_source_integrity(&capture)?;
        let target = target.or(capture.target_id.as_deref()).ok_or_else(|| {
            denied("Select a registered loopback target before creating a replay draft")
        })?;
        if self.target(target)?.input.project_id != capture.project_id
            || capture.target_id.as_deref().is_some_and(|t| t != target)
        {
            return Err(denied(
                "Capture and replay target must belong to the same project/target partition",
            ));
        }
        let mut url = Url::parse(&capture.url).map_err(|_| bad("Invalid captured URL"))?;
        url.set_query(None);
        url.set_fragment(None);
        let headers = capture.request["headers"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|h| {
                let name = h["name"].as_str()?.to_ascii_lowercase();
                let value = h["value"].as_str()?;
                (PUBLIC_HEADERS.contains(&name.as_str()) && public_text(value).is_ok()).then(|| {
                    HttpHeader {
                        name,
                        value: value.into(),
                    }
                })
            })
            .collect();
        let preview = &capture.request["body"]["preview"];
        let body = if preview.is_null()
            || public_json(preview.clone()).is_err()
            || preview.to_string().contains("[REDACTED]")
            || preview
                .as_str()
                .is_some_and(|s| s.starts_with("[Body withheld:"))
        {
            String::new()
        } else {
            serde_json::to_string_pretty(preview).map_err(json_error)?
        };
        Ok(HttpDraftInput {
            project_id: capture.project_id,
            target_id: target.into(),
            source_exchange_id: Some(exchange.into()),
            title: format!("{} captured request", capture.method),
            method: capture.method,
            url: url.to_string(),
            headers,
            content_type: capture.request["body"]["mime_type"]
                .as_str()
                .unwrap_or("")
                .into(),
            body,
            auth_profile_id: None,
            timeout_ms: 5000,
            follow_redirects: false,
            proxy: None,
        })
    }
    pub fn http_draft(&self, id: &str) -> Result<HttpDraft> {
        let row: Option<(String, i64, String)> = self
            .db
            .connect()?
            .query_row(
                "SELECT data_json,revision,updated_at FROM http_request_drafts WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(db_error)?;
        let (data, revision, updated_at) =
            row.ok_or_else(|| AppError::new("NOT_FOUND", "HTTP request draft does not exist"))?;
        Ok(HttpDraft {
            id: id.into(),
            revision,
            updated_at,
            input: serde_json::from_str(&data).map_err(json_error)?,
        })
    }
    pub fn http_drafts(&self, project: &str, target: Option<&str>) -> Result<Vec<HttpDraft>> {
        let project = self.project(project)?.id;
        if let Some(t) = target
            && self.target(t)?.input.project_id != project
        {
            return Err(AppError::new(
                "PROJECT_MISMATCH",
                "Request target belongs to another project",
            ));
        }
        let conn = self.db.connect()?;
        let mut q=conn.prepare("SELECT id FROM http_request_drafts WHERE project_id=?1 AND (?2 IS NULL OR target_id=?2) ORDER BY updated_at DESC LIMIT 100").map_err(db_error)?;
        q.query_map(params![project, target], |r| r.get::<_, String>(0))
            .map_err(db_error)?
            .map(|r| self.http_draft(&r.map_err(db_error)?))
            .collect()
    }
    pub fn http_draft_save(
        &self,
        id: Option<&str>,
        mut input: HttpDraftInput,
        revision: Option<i64>,
    ) -> Result<HttpDraft> {
        input.project_id = self.project(&input.project_id)?.id;
        if self.project(&input.project_id)?.archived {
            return Err(AppError::new(
                "PROJECT_ARCHIVED",
                "Restore project before editing HTTP requests",
            ));
        }
        let input = validate_input(input)?;
        self.check_draft_binding(&input, false)?;
        if let Some(id) = &input.auth_profile_id {
            let identity = self.auth_record(id)?;
            if let Some(reference) = identity.secret_ref
                && let Some(secret) = self.secrets.get(&reference)?
            {
                let header = super::auth::credential_header(identity.profile.input.kind, &secret)?;
                let redactions =
                    credential_redactions(identity.profile.input.kind, &secret, &header);
                let mut public = serde_json::to_value(&input).map_err(json_error)?;
                let original = public.clone();
                redact_exact(&mut public, &redactions);
                if public != original {
                    return Err(AppError::new(
                        "CREDENTIAL_INPUT",
                        "The selected identity credential cannot be stored in request metadata or payload fields",
                    ));
                }
            }
        }
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let old = id.map(|id| self.http_draft(id)).transpose()?;
        if old.as_ref().is_some_and(|d| {
            revision != Some(d.revision)
                || d.input.project_id != input.project_id
                || d.input.target_id != input.target_id
        }) {
            return Err(AppError::new(
                "STALE_REVISION",
                "Request changed or belongs to another target; reload it",
            ));
        }
        let id = id
            .map(str::to_owned)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        tx.execute("INSERT INTO http_request_drafts VALUES(?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(id) DO UPDATE SET source_exchange_id=excluded.source_exchange_id,revision=excluded.revision,data_json=excluded.data_json,updated_at=excluded.updated_at",params![id,input.project_id,input.target_id,input.source_exchange_id,old.map_or(1,|d|d.revision+1),serde_json::to_string(&input).map_err(json_error)?,now()]).map_err(db_error)?;
        Self::audit(&tx, "http.request_saved", &id, "no_network")?;
        tx.commit().map_err(db_error)?;
        self.http_draft(&id)
    }
    fn check_draft_binding(&self, input: &HttpDraftInput, source_integrity: bool) -> Result<()> {
        let target = self.target(&input.target_id)?;
        if target.input.project_id != input.project_id {
            return Err(AppError::new(
                "PROJECT_MISMATCH",
                "Request target belongs to another project",
            ));
        }
        if !matches!(
            target.input.kind,
            TargetKind::Url | TargetKind::RestApi | TargetKind::Graphql
        ) {
            return Err(denied("Replay requires a URL, REST API or GraphQL target"));
        }
        if let Some(source) = &input.source_exchange_id {
            let source = self.http_exchange(source)?;
            if source.project_id != input.project_id
                || source
                    .target_id
                    .as_deref()
                    .is_some_and(|t| t != input.target_id)
            {
                return Err(denied("Source capture belongs to another target/project"));
            }
            if source_integrity {
                self.http_source_integrity(&source)?;
            }
        }
        if let Some(identity) = &input.auth_profile_id {
            let identity = self.auth_record(identity)?.profile;
            if identity.input.project_id != input.project_id
                || identity.input.target_id != input.target_id
                || identity.origin != lab_url(&input.url)?.origin().ascii_serialization()
            {
                return Err(AppError::new(
                    "CREDENTIAL_SCOPE",
                    "Identity is bound to a different target or exact origin",
                ));
            }
        }
        Ok(())
    }
    pub fn http_replay_preview(&self, id: &str) -> Result<HttpReplayPlan> {
        self.replay_plan(id, true)
    }
    fn replay_plan(&self, id: &str, source_integrity: bool) -> Result<HttpReplayPlan> {
        let draft = self.http_draft(id)?;
        validate_input(draft.input.clone())?;
        self.check_draft_binding(&draft.input, source_integrity)?;
        let target = self.target(&draft.input.target_id)?;
        let project = self.project(&draft.input.project_id)?;
        if project.archived {
            return Err(AppError::new(
                "PROJECT_ARCHIVED",
                "Restore project before HTTP replay",
            ));
        }
        let mode = self.settings()?.security_mode;
        if mode == SecurityMode::PassiveAuto
            || target.input.mode == SecurityMode::PassiveAuto
            || !target.input.authorized
        {
            return Err(denied(
                "HTTP sends traffic: declare target authorization and select an active mode for both application and target",
            ));
        }
        let url = lab_url(&draft.input.url)?;
        lab_url(&target.input.value)?;
        if self.validate_scope(&target.id, Some(url.as_str()))?.status != "allowed" {
            return Err(denied(
                "The exact request URL is outside the declared target scope or is excluded",
            ));
        }
        let state_changing = matches!(
            draft.input.method.as_str(),
            "POST" | "PUT" | "PATCH" | "DELETE"
        );
        if state_changing
            && (mode != SecurityMode::LabResearch || target.input.mode != SecurityMode::LabResearch)
        {
            return Err(denied(
                "State-changing methods require Lab Research on both the application and this owned local target",
            ));
        }
        let record = draft
            .input
            .auth_profile_id
            .as_ref()
            .map(|id| self.auth_record(id))
            .transpose()?;
        if record.as_ref().is_some_and(|r| {
            r.profile.expired
                || (r.profile.input.kind != HttpAuthKind::Anonymous && r.secret_ref.is_none())
        }) {
            return Err(AppError::new(
                "CREDENTIAL_UNAVAILABLE",
                "Identity is expired or has no configured credential",
            ));
        }
        let identity_binding=record.as_ref().map(|r|json!({"id":r.profile.id,"revision":r.profile.revision,"input":r.profile.input,"origin":r.profile.origin,"expired":r.profile.expired,"secret_ref":r.secret_ref}));
        let fingerprint=super::super::evidence::digest(&serde_json::to_vec(&json!({"version":1,"draft":draft,"target":target.input,"mode":mode,"identity":identity_binding,"budget":1})).map_err(json_error)?);
        let ip = match url.host() {
            Some(url::Host::Ipv4(ip)) => IpAddr::V4(ip),
            Some(url::Host::Ipv6(ip)) => IpAddr::V6(ip),
            _ => IpAddr::V4(Ipv4Addr::LOCALHOST),
        };
        Ok(HttpReplayPlan {draft,destination_address:SocketAddr::new(ip,url.port_or_known_default().unwrap_or(80)).to_string(),request_budget:1,state_changing,identity:record.map(|r|r.profile),fingerprint,safeguards:vec!["One manual request; no retries, redirects, proxy or automatic cookie jar".into(),"Only loopback IPs; exact localhost is pinned to 127.0.0.1 without DNS".into(),"TLS certificate verification is enabled; bounded headers/body/time and cancellation checkpoints".into(),"At most 20 approvals/project/minute, at least 500 ms between dispatches".into(),"HTTP success is not proof of authentication or authorization. Cancellation cannot undo a server action.".into()]})
    }
    pub fn http_replay_enqueue(&self, approval: HttpReplayApproval) -> Result<AnalysisJob> {
        if !approval.approved {
            return Err(denied(
                "Review the exact destination and request, then explicitly approve Send",
            ));
        }
        let plan = self.http_replay_preview(&approval.draft_id)?;
        if plan.fingerprint != approval.fingerprint {
            return Err(AppError::new(
                "APPROVAL_STALE",
                "Request, identity or policy changed; preview again",
            ));
        }
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let active:i64=tx.query_row("SELECT count(*) FROM http_replay_jobs r JOIN analysis_jobs j ON j.id=r.job_id WHERE r.draft_id=?1 AND j.status IN ('queued','running')",[&approval.draft_id],|r|r.get(0)).map_err(db_error)?;
        if active != 0 {
            return Err(AppError::new(
                "JOB_BUSY",
                "This request already has an active replay job",
            ));
        }
        let recent:i64=tx.query_row("SELECT count(*) FROM http_replay_jobs r JOIN analysis_jobs j ON j.id=r.job_id WHERE j.project_id=?1 AND julianday(r.approved_at)>=julianday('now','-1 minute')",[&plan.draft.input.project_id],|r|r.get(0)).map_err(db_error)?;
        if recent >= 20 {
            return Err(denied(
                "Request budget exhausted: at most 20 approved replays per project/minute",
            ));
        }
        let id = Uuid::new_v4().to_string();
        tx.execute("INSERT INTO analysis_jobs(id,project_id,target_id,kind,status,created_at,timeout_ms,logs_json) VALUES(?1,?2,?3,?4,'queued',?5,?6,'[\"One exact loopback request manually approved; no automatic retry\"]')",params![id,plan.draft.input.project_id,plan.draft.input.target_id,KIND,now(),plan.draft.input.timeout_ms+2000]).map_err(db_error)?;
        tx.execute("INSERT INTO http_replay_jobs(job_id,draft_id,source_exchange_id,fingerprint,approved_at) VALUES(?1,?2,?3,?4,?5)",params![id,plan.draft.id,plan.draft.input.source_exchange_id,plan.fingerprint,now()]).map_err(db_error)?;
        Self::audit(&tx, "http.replay_approved", &id, "manual_loopback")?;
        tx.commit().map_err(db_error)?;
        self.job(&id)
    }
    pub(crate) fn http_replay_job_plan(
        &self,
        job: &AnalysisJob,
        integrity: bool,
    ) -> Result<HttpReplayPlan> {
        let (draft, expected): (String, String) = self
            .db
            .connect()?
            .query_row(
                "SELECT draft_id,fingerprint FROM http_replay_jobs WHERE job_id=?1",
                [&job.id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(db_error)?;
        let plan = self.replay_plan(&draft, integrity)?;
        if plan.fingerprint != expected
            || plan.draft.input.project_id != job.project_id
            || Some(&plan.draft.input.target_id) != job.target_id.as_ref()
        {
            return Err(AppError::new(
                "APPROVAL_STALE",
                "Approved request, identity or authorization changed; no retry occurs",
            ));
        }
        Ok(plan)
    }
    pub fn http_replay_history(&self, id: &str) -> Result<Vec<AnalysisJob>> {
        self.http_draft(id)?;
        let conn = self.db.connect()?;
        let mut q=conn.prepare("SELECT job_id FROM http_replay_jobs WHERE draft_id=?1 ORDER BY approved_at DESC LIMIT 100").map_err(db_error)?;
        q.query_map([id], |r| r.get::<_, String>(0))
            .map_err(db_error)?
            .map(|r| self.job(&r.map_err(db_error)?))
            .collect()
    }
    fn reserve_replay_dispatch(&self, project: &str) -> Result<()> {
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let last: Option<i64> = tx
            .query_row(
                "SELECT started_at_ms FROM http_replay_dispatch WHERE project_id=?1",
                [project],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        let stamp = Utc::now().timestamp_millis();
        if last.is_some_and(|v| stamp - v < 500) {
            return Err(AppError::new(
                "HTTP_RATE_LIMIT",
                "Project dispatch spacing is 500 ms; preview and approve a new request after the current one",
            ));
        }
        tx.execute("INSERT INTO http_replay_dispatch VALUES(?1,?2) ON CONFLICT(project_id) DO UPDATE SET started_at_ms=excluded.started_at_ms",params![project,stamp]).map_err(db_error)?;
        tx.commit().map_err(db_error)
    }
    pub(crate) fn prepare_http_replay(
        &self,
        job: &AnalysisJob,
        update: &mut dyn FnMut(u8, &str) -> Result<()>,
    ) -> Result<super::super::jobs::PreparedJob> {
        let plan = self.http_replay_job_plan(job, true)?;
        let mut redactions = Vec::new();
        let mut credential = None;
        if let Some(profile) = &plan.identity
            && profile.input.kind != HttpAuthKind::Anonymous
        {
            let record = self.auth_record(&profile.id)?;
            let secret = self
                .secrets
                .get(record.secret_ref.as_deref().ok_or_else(|| {
                    AppError::new(
                        "CREDENTIAL_UNAVAILABLE",
                        "Identity credential is unavailable",
                    )
                })?)?
                .ok_or_else(|| {
                    AppError::new(
                        "CREDENTIAL_UNAVAILABLE",
                        "Identity credential is missing from OS storage",
                    )
                })?;
            let header = super::auth::credential_header(profile.input.kind, &secret)?;
            redactions = credential_redactions(profile.input.kind, &secret, &header);
            let mut value = reqwest::header::HeaderValue::from_str(&header)
                .map_err(|_| denied("Invalid credential header"))?;
            value.set_sensitive(true);
            credential = Some((profile.input.header_name.clone(), value));
        }
        self.reserve_replay_dispatch(&job.project_id)?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| AppError::new("WORKER_ERROR", "Cannot start HTTP replay runtime"))?;
        runtime.block_on(async {
            let url=lab_url(&plan.draft.input.url)?;let port=url.port_or_known_default().unwrap_or(80);
            let client=reqwest::Client::builder().no_proxy().redirect(reqwest::redirect::Policy::none()).retry(reqwest::retry::never()).http1_only().http1_max_headers(64).no_gzip().no_brotli().no_deflate().no_zstd().connect_timeout(Duration::from_millis(2000)).timeout(Duration::from_millis(plan.draft.input.timeout_ms as u64)).resolve("localhost",SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST),port)).user_agent("ReconXAI-manual-loopback/0.1").build().map_err(|_|AppError::new("HTTP_CONFIG","Cannot configure verified local HTTP client"))?;
            let method=reqwest::Method::from_bytes(plan.draft.input.method.as_bytes()).map_err(|_|denied("Invalid method"))?;
            let mut request=client.request(method,&plan.draft.input.url).header("accept-encoding","identity");
            let mut headers:Vec<Value>=plan.draft.input.headers.iter().map(|h|json!({"name":h.name,"value":h.value})).collect();
            for h in &plan.draft.input.headers {request=request.header(&h.name,&h.value);}
            if !plan.draft.input.content_type.is_empty(){request=request.header("content-type",&plan.draft.input.content_type);headers.push(json!({"name":"Content-Type","value":plan.draft.input.content_type}));}
            if let Some((name,value))=credential {request=request.header(&name,value);headers.push(json!({"name":name,"value":"[REDACTED]"}));}
            if !plan.draft.input.body.is_empty(){request=request.body(plan.draft.input.body.clone());}
            update(20,"Sending the one approved loopback request; redirects and proxy disabled")?;
            let started_at=now();let start=Instant::now();let future=request.send();tokio::pin!(future);
            let mut response=loop {tokio::select!{r=&mut future=>break r.map_err(request_error)?,_=tokio::time::sleep(Duration::from_millis(50))=>update(25,"Awaiting approved loopback response")?}};
            if !response.remote_addr().is_some_and(|a|a.ip().is_loopback()&&a.port()==port) {return Err(denied("Connection did not reach the approved loopback socket"));}
            update(45,"Actual response headers received")?;
            let header_size:usize=response.headers().iter().map(|(k,v)|k.as_str().len()+v.as_bytes().len()).sum();
            if header_size>64*1024 || response.headers().len()>64 {return Err(AppError::new("RESPONSE_LIMIT","Response headers exceed the bounded capture limit"));}
            if plan.draft.input.method!="HEAD" && response.content_length().is_some_and(|n|n>MAX_BODY as u64) {return Err(AppError::new("RESPONSE_LIMIT","Response exceeds 256 KiB; request is not retried"));}
            let status=response.status().as_u16();let version=format!("{:?}",response.version());
            let mime=response.headers().get("content-type").and_then(|v|v.to_str().ok()).unwrap_or("").to_owned();
            let response_headers:Vec<Value>=response.headers().iter().map(|(k,v)|json!({"name":k.as_str(),"value":v.to_str().unwrap_or("[Non-text header withheld]")})).collect();
            let mut bytes=Zeroizing::new(Vec::new());
            loop {
                let future=response.chunk();tokio::pin!(future);
                let chunk=loop{tokio::select!{r=&mut future=>break r.map_err(request_error)?,_=tokio::time::sleep(Duration::from_millis(50))=>update(55,"Reading bounded response; cancellation and policy checked")?}};
                let Some(chunk)=chunk else {break};
                if bytes.len().saturating_add(chunk.len())>MAX_BODY {return Err(AppError::new("RESPONSE_LIMIT","Response exceeds 256 KiB; request is not retried"));}
                bytes.extend_from_slice(&chunk);update(60,"Actual response bytes captured")?;
            }
            drop(response);drop(client);
            let elapsed=start.elapsed().as_secs_f64()*1000.0;let body_hash=super::super::evidence::digest(&bytes);
            let preview=if let Ok(mut value)=parse_json(&bytes){redact_exact(&mut value,&redactions);Some(serde_json::to_string(&safe_value(value)).map_err(json_error)?)}else if let Ok(text)=std::str::from_utf8(&bytes){let mut value=json!(text);redact_exact(&mut value,&redactions);Some(super::super::reporting::redact_content(value.as_str().unwrap_or("")).0)}else{None};
            let mut har=json!({"log":{"version":"1.2","creator":{"name":"ReconX manual loopback capture","version":"0.1"},"entries":[{"startedDateTime":started_at,"time":elapsed,"timings":{"send":0,"wait":elapsed,"receive":0},"request":{"method":plan.draft.input.method,"url":plan.draft.input.url,"httpVersion":"HTTP/1.1","headers":headers,"cookies":[],"queryString":[],"headersSize":-1,"bodySize":plan.draft.input.body.len(),"postData":{"mimeType":plan.draft.input.content_type,"text":plan.draft.input.body}},"response":{"status":status,"statusText":"","httpVersion":version,"headers":response_headers,"cookies":[],"content":{"mimeType":mime,"size":bytes.len()},"redirectURL":"","headersSize":-1,"bodySize":bytes.len()}}]}});
            if let Some(preview)=preview {har["log"]["entries"][0]["response"]["content"]["text"]=json!(preview);}
            redact_exact(&mut har,&redactions);
            update(85,"Actual response sanitized and hashed; request was not retried")?;
            let facts=json!({"status":status,"elapsed_ms":elapsed,"response_size":bytes.len(),"response_body_sha256":body_hash,"request_budget":1,"redirect_followed":false,"proxy_used":false,"tls_verification":true,"identity":plan.identity.as_ref().map(|p|json!({"id":p.id,"name":p.input.name,"role":p.input.role,"revision":p.revision})),"representation":"Sanitized application-level HTTP capture; credentials removed before persistence, binary/non-UTF8 bodies omitted; timings are total duration, not wire phases","authentication":"Unverified: HTTP response is not proof of identity or authorization"});
            Ok(super::super::jobs::PreparedJob{result:facts.clone(),analysis:None,http:Some(super::super::recon::HttpObservation{snapshot:har,url:plan.draft.input.url,facts})})
        })
    }
    pub(crate) fn commit_http_replay(
        &self,
        tx: &Connection,
        job: &AnalysisJob,
        observation: &super::super::recon::HttpObservation,
    ) -> Result<PendingReplayArtifact> {
        let plan = self.http_replay_job_plan(job, true)?;
        let id = Uuid::new_v4().to_string();
        let bytes = serde_json::to_vec(&observation.snapshot).map_err(json_error)?;
        let identity = plan.identity.as_ref();
        let input = HttpImportInput {
            project_id: job.project_id.clone(),
            target_id: job.target_id.clone(),
            title: plan.draft.input.title.clone(),
            identity: identity
                .map(|i| i.input.name.clone())
                .unwrap_or_else(|| "Anonymous".into()),
            role: identity
                .map(|i| i.input.role.clone())
                .unwrap_or_else(|| "anonymous".into()),
        };
        let (mut captures, mut summary) = parse_har(&bytes, &input, &id)?;
        summary["source"] = json!(KIND);
        summary["warning"] = json!(
            "Actual manually approved loopback response; sanitized before persistence, not a wire capture"
        );
        let evidence = self.insert_observed_evidence(
            tx,
            EvidenceInput {
                project_id: job.project_id.clone(),
                target_id: job.target_id.clone(),
                kind: EvidenceKind::Attachment,
                title: "Private sanitized loopback HTTP capture".into(),
                source_tool: KIND.into(),
                sensitivity: Sensitivity::Restricted,
            },
            None,
            &bytes,
            "application/octet-stream",
        )?;
        let mut pending = PendingReplayArtifact {
            result: Value::Null,
            path: self.output_path("evidence", &evidence.id)?,
            committed: false,
        };
        tx.execute(
            "INSERT INTO http_imports VALUES(?1,?2,?3,'har',?4,?5,?6,?7,?8)",
            params![
                id,
                job.project_id,
                job.target_id,
                plan.draft.input.title,
                evidence.sha256,
                now(),
                evidence.id,
                serde_json::to_string(&summary).map_err(json_error)?
            ],
        )
        .map_err(db_error)?;
        let capture = captures.remove(0);
        tx.execute(
            "INSERT INTO http_exchanges VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                capture.id,
                id,
                job.project_id,
                job.target_id,
                capture.host,
                capture.method,
                capture.path,
                capture.status,
                serde_json::to_string(&capture).map_err(json_error)?
            ],
        )
        .map_err(db_error)?;
        tx.execute(
            "UPDATE http_replay_jobs SET exchange_id=?1 WHERE job_id=?2",
            params![capture.id, job.id],
        )
        .map_err(db_error)?;
        if let Some(profile) = identity {
            tx.execute("UPDATE http_auth_profiles SET last_check_json=?1 WHERE id=?2 AND revision=?3",params![json!({"at":now(),"job_id":job.id,"status":capture.status,"authentication":"unverified","note":"HTTP status alone never authenticates an identity"}).to_string(),profile.id,profile.revision]).map_err(db_error)?;
        }
        pending.result = json!({"exchange_id":capture.id,"import_id":id,"source_exchange_id":plan.draft.input.source_exchange_id,"draft_id":plan.draft.id,"capture_evidence_id":evidence.id,"observations":observation.facts});
        Ok(pending)
    }
}
fn credential_redactions(kind: HttpAuthKind, secret: &str, header: &str) -> Vec<Zeroizing<String>> {
    let mut values = vec![
        Zeroizing::new(secret.to_owned()),
        Zeroizing::new(header.to_owned()),
    ];
    if kind == HttpAuthKind::Basic {
        if let Some((_, password)) = secret.split_once(':') {
            values.push(Zeroizing::new(password.to_owned()));
        }
        values.push(Zeroizing::new(
            base64::engine::general_purpose::STANDARD.encode(secret.as_bytes()),
        ));
    }
    if kind == HttpAuthKind::Cookie {
        for pair in secret.split(';') {
            if let Some((_, value)) = pair.split_once('=') {
                values.push(Zeroizing::new(value.trim().to_owned()));
            }
        }
    }
    values.sort_by_key(|s| std::cmp::Reverse(s.len()));
    values
}
fn redact_exact(value: &mut Value, secrets: &[Zeroizing<String>]) {
    fn text(value: &str, secrets: &[Zeroizing<String>]) -> String {
        let mut s = value.to_owned();
        for secret in secrets {
            s = s.replace(secret.as_str(), "[REDACTED]");
            let encoded: String = url::form_urlencoded::byte_serialize(secret.as_bytes()).collect();
            s = s.replace(&encoded, "[REDACTED]");
            s = s.replace(
                &base64::engine::general_purpose::STANDARD.encode(secret.as_bytes()),
                "[REDACTED]",
            );
        }
        s
    }
    match value {
        Value::String(s) => *s = text(s, secrets),
        Value::Array(a) => a.iter_mut().for_each(|v| redact_exact(v, secrets)),
        Value::Object(o) => {
            let old = std::mem::take(o);
            for (k, mut v) in old {
                redact_exact(&mut v, secrets);
                o.insert(text(&k, secrets), v);
            }
        }
        _ => {}
    }
}
