//! Reviewed, bounded ProjectDiscovery jobs. Passive intelligence is not validation.
use super::*;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    net::{IpAddr, SocketAddr},
    time::Duration,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReconTool {
    Httpx,
    Subfinder,
}
impl ReconTool {
    pub fn id(&self) -> &'static str {
        match self {
            Self::Httpx => "httpx",
            Self::Subfinder => "subfinder",
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReconConfig {
    pub target_id: String,
    pub tool: ReconTool,
    #[serde(default = "asset_budget")]
    pub max_assets: u8,
    #[serde(default = "probe_timeout")]
    pub timeout_secs: u32,
    #[serde(default)]
    pub technology: bool,
    #[serde(default = "passive_sources")]
    pub sources: Vec<String>,
}
fn asset_budget() -> u8 {
    4
}
fn probe_timeout() -> u32 {
    10
}
fn passive_sources() -> Vec<String> {
    vec!["crtsh".into()]
}
impl ReconConfig {
    pub fn for_target(target: &str, tool: ReconTool) -> Self {
        Self {
            target_id: target.into(),
            tool,
            max_assets: 4,
            timeout_secs: 10,
            technology: true,
            sources: passive_sources(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconPlan {
    pub config: ReconConfig,
    pub project_id: String,
    pub target: String,
    pub version: String,
    pub fingerprint: String,
    pub permissions: Vec<String>,
    pub limits: Vec<String>,
}
fn denied(message: &str) -> AppError {
    AppError::new("RECON_POLICY", message)
}
fn domain(value: &str) -> Result<String> {
    let host = url::Host::parse(value).map_err(|_| denied("Use a valid DNS domain"))?;
    match host {
        url::Host::Domain(d)
            if d.contains('.')
                && d.len() <= 253
                && d.split('.').all(|label| {
                    !label.is_empty()
                        && label.len() <= 63
                        && !label.starts_with('-')
                        && !label.ends_with('-')
                        && label
                            .bytes()
                            .all(|c| c.is_ascii_alphanumeric() || c == b'-')
                }) =>
        {
            Ok(d.to_lowercase())
        }
        _ => Err(denied(
            "Passive discovery requires a DNS domain, not an IP, URL or local hostname",
        )),
    }
}
fn public_ip(ip: IpAddr) -> bool {
    const DENY: &[&str] = &[
        "0.0.0.0/8",
        "10.0.0.0/8",
        "100.64.0.0/10",
        "127.0.0.0/8",
        "169.254.0.0/16",
        "172.16.0.0/12",
        "192.0.0.0/24",
        "192.0.2.0/24",
        "192.168.0.0/16",
        "198.18.0.0/15",
        "198.51.100.0/24",
        "203.0.113.0/24",
        "224.0.0.0/3",
        "2001::/23",
        "2001:db8::/32",
        "2002::/16",
    ];
    if matches!(ip,IpAddr::V6(v) if v.to_ipv4_mapped().is_some() || v.segments()[0]&0xe000!=0x2000)
    {
        return false;
    }
    !DENY
        .iter()
        .any(|net| net.parse::<ipnet::IpNet>().is_ok_and(|n| n.contains(&ip)))
}
impl Core {
    fn recon_target_policy(&self, config: &ReconConfig) -> Result<Target> {
        if !(1..=8).contains(&config.max_assets) || !(1..=15).contains(&config.timeout_secs) {
            return Err(denied("Use 1–8 assets and a 1–15 second probe timeout"));
        }
        if config.sources.is_empty()
            || config.sources.len() > 3
            || config
                .sources
                .iter()
                .any(|s| !["crtsh", "hackertarget", "certspotter"].contains(&s.as_str()))
        {
            return Err(denied(
                "Choose reviewed keyless passive sources: crtsh, hackertarget, certspotter",
            ));
        }
        let t = self.target(&config.target_id)?;
        if self.project(&t.input.project_id)?.archived
            || !t.input.authorized
            || self.validate_scope(&t.id, None)?.status != "allowed"
        {
            return Err(denied(
                "An active project, explicit authorization and allowed target scope are required",
            ));
        }
        if config.tool == ReconTool::Subfinder {
            domain(&t.input.value)?;
            if t.input.kind != TargetKind::Domain {
                return Err(denied("Subfinder requires a registered domain target"));
            }
        } else {
            if !matches!(
                t.input.kind,
                TargetKind::Domain | TargetKind::Url | TargetKind::RestApi | TargetKind::Graphql
            ) {
                return Err(denied(
                    "HTTPX is only available for domain, URL and API targets",
                ));
            }
            if self.settings()?.security_mode == SecurityMode::PassiveAuto
                || t.input.mode == SecurityMode::PassiveAuto
            {
                return Err(denied(
                    "HTTPX sends active traffic: use an authorized execution mode and approve its exact plan",
                ));
            }
            if t.input.kind != TargetKind::Domain {
                let u = url::Url::parse(&t.input.value).map_err(|_| {
                    denied("HTTPX requires a domain or absolute HTTP(S) URL target")
                })?;
                if !matches!(u.scheme(), "http" | "https")
                    || !u.username().is_empty()
                    || u.password().is_some()
                    || u.query().is_some()
                    || u.fragment().is_some()
                {
                    return Err(denied("Use HTTP(S) without credentials, query or fragment"));
                }
            } else {
                domain(&t.input.value)?;
            }
        }
        Ok(t)
    }
    fn recon_fingerprint(&self, config: &ReconConfig) -> Result<String> {
        let t = self.recon_target_policy(config)?;
        let state = self.extension_state(config.tool.id())?;
        Ok(evidence::digest(&serde_json::to_vec(&json!({"adapter":1,"config":config,"target":t.input,"mode":self.settings()?.security_mode,"binary":self.tool_plan(config.tool.id())?.binary_sha256,"extension":state.map(|s|(s.revision,s.version,s.updated_at,s.enabled))})).map_err(json_error)?))
    }
    pub fn recon_plan(&self, config: ReconConfig) -> Result<ReconPlan> {
        let t = self.recon_target_policy(&config)?;
        let id = config.tool.id();
        let tool = self.tool_plan(id)?;
        if self.extension_entry(id)?.status != "ready" {
            return Err(AppError::new(
                "DEPENDENCY_UNAVAILABLE",
                "Install and health-check the reviewed recon extension in Marketplace first",
            ));
        }
        let fingerprint = self.recon_fingerprint(&config)?;
        let (permissions, limits): (Vec<&str>, Vec<&str>) = match config.tool {
            ReconTool::Httpx => (
                vec![
                    "explicit active HTTP GET approval",
                    "only registered URL or in-scope discovered hosts",
                ],
                vec![
                    "Fixed arguments; no redirects, protocol fallback, retries, cloud upload, crawling, proxy or credentials",
                    "Numeric transport IP is pinned; private/special DNS answers denied; local IP URLs must be explicitly registered",
                    "TLS certificate metadata is observed, not a certificate trust validation verdict",
                    "At most 8 HTTP URLs, one scheme per URL, 1 request/second, one child, 64 KiB response read and 1 MiB output",
                    "HTTPX may perform TLS metadata handshakes; a probe is not a vulnerability test",
                ],
            ),
            ReconTool::Subfinder => (
                vec![
                    "send parent domain to selected external passive sources",
                    "no DNS resolution or HTTP validation of discovered names",
                ],
                vec![
                    "Selected reviewed keyless sources only; provider HTTP behavior is upstream, not active target probing",
                    "2 source requests/second, two-minute enumeration budget, 1 MiB process output and 1000 normalized hosts",
                    "Empty/error output is inconclusive; individual source success is not independently confirmed",
                    "API-key sources are not configured by this adapter; no imported credentials or AI shell commands",
                ],
            ),
        };
        Ok(ReconPlan {
            project_id: t.input.project_id,
            target: t.input.value,
            version: tool.version,
            fingerprint,
            permissions: permissions.into_iter().map(String::from).collect(),
            limits: limits.into_iter().map(String::from).collect(),
            config,
        })
    }
    pub fn recon_enqueue(&self, plan: ReconPlan, approved: bool) -> Result<AnalysisJob> {
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let id = self.recon_enqueue_in_tx(&tx, &plan, approved, None)?;
        tx.commit().map_err(db_error)?;
        self.job(&id)
    }
    pub(super) fn recon_enqueue_in_tx(
        &self,
        tx: &Connection,
        plan: &ReconPlan,
        approved: bool,
        dependency: Option<&str>,
    ) -> Result<String> {
        if !approved {
            return Err(AppError::new(
                "APPROVAL_REQUIRED",
                "Approve this exact recon destination, passive data sharing and resource budgets",
            ));
        }
        let current = self.recon_plan(plan.config.clone())?;
        if current.fingerprint != plan.fingerprint
            || current.target != plan.target
            || current.project_id != plan.project_id
        {
            return Err(AppError::new(
                "APPROVAL_STALE",
                "Target, scope, tool or configuration changed",
            ));
        }
        if let Some(d) = dependency {
            let (project, target, kind): (String, Option<String>, String) = tx
                .query_row(
                    "SELECT project_id,target_id,kind FROM analysis_jobs WHERE id=?1",
                    [d],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .map_err(db_error)?;
            if project != plan.project_id
                || target.as_deref() != Some(&plan.config.target_id)
                || kind != "recon_subfinder"
            {
                return Err(denied(
                    "Discovery dependency must belong to the same target",
                ));
            }
        }
        let id = Uuid::new_v4().to_string();
        let timeout = if plan.config.tool == ReconTool::Subfinder {
            130000
        } else {
            (plan.config.timeout_secs as u64 + 7) * plan.config.max_assets as u64 * 1000
        };
        tx.execute("INSERT INTO analysis_jobs(id,project_id,target_id,kind,status,created_at,timeout_ms,logs_json) VALUES(?1,?2,?3,?4,'queued',?5,?6,'[\"Approved typed reconnaissance; no arbitrary shell\"]')",params![id,plan.project_id,plan.config.target_id,format!("recon_{}",plan.config.tool.id()),now(),timeout as i64]).map_err(db_error)?;
        tx.execute("INSERT INTO recon_tool_approvals(job_id,plan_json,dependency_job_id,approved_at) VALUES(?1,?2,?3,?4)",params![id,serde_json::to_string(plan).map_err(json_error)?,dependency,now()]).map_err(db_error)?;
        Self::audit(tx, "recon.approved", &id, "queued")?;
        Ok(id)
    }
    pub(super) fn recon_job_plan(&self, job: &AnalysisJob) -> Result<ReconPlan> {
        let text: String = self
            .db
            .connect()?
            .query_row(
                "SELECT plan_json FROM recon_tool_approvals WHERE job_id=?1",
                [&job.id],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        let p: ReconPlan = serde_json::from_str(&text).map_err(json_error)?;
        if job.project_id != p.project_id
            || job.target_id.as_deref() != Some(&p.config.target_id)
            || job.kind != format!("recon_{}", p.config.tool.id())
            || self.recon_fingerprint(&p.config)? != p.fingerprint
        {
            return Err(AppError::new(
                "APPROVAL_STALE",
                "Recon policy changed; no further probe is permitted",
            ));
        }
        let state = self
            .extension_state(p.config.tool.id())?
            .ok_or_else(|| denied("Recon extension was removed"))?;
        if !state.enabled || state.status != "installed" {
            return Err(denied("Recon extension is disabled or unhealthy"));
        }
        Ok(p)
    }
    pub fn recon_history(&self, target: Option<&str>) -> Result<Vec<AnalysisJob>> {
        Ok(self
            .list_jobs(None)?
            .into_iter()
            .filter(|j| {
                j.kind.starts_with("recon_")
                    && target.is_none_or(|t| j.target_id.as_deref() == Some(t))
            })
            .collect())
    }
    pub fn recon_export(&self, project: &str) -> Result<Value> {
        let project = self.project(project)?;
        let mut value = json!({"project_id":project.id,"assets":self.assets(Some(&project.id))?,"verification":"Observations only; passive intelligence and unverified technology hints are not vulnerability findings"});
        reporting::redact_value(&mut value);
        Ok(value)
    }
    pub(super) fn prepare_recon_tool(
        &self,
        job: &AnalysisJob,
        update: &mut impl FnMut(u8, &str) -> Result<()>,
    ) -> Result<jobs::PreparedJob> {
        let plan = self.recon_job_plan(job)?;
        let manager = self.tool_manager();
        let mut observations = Vec::new();
        let mut raw = Vec::new();
        let mut excluded = 0;
        if plan.config.tool == ReconTool::Subfinder {
            update(
                10,
                "Querying approved passive sources; no active host validation",
            )?;
            let output = manager.recon_execute(
                reconx_tools::ReconToolRequest::Subfinder {
                    domain: plan.target.clone(),
                    sources: plan.config.sources.clone(),
                    timeout_secs: 120,
                },
                || update(10, "Waiting for real passive source output"),
            )?;
            let parsed = parse_subfinder_output(&output.stdout, &domain(&plan.target)?)?;
            for (host, sources) in parsed {
                if self
                    .validate_scope(&plan.config.target_id, Some(&host))?
                    .status
                    == "allowed"
                {
                    observations.push(json!({"kind":"subdomain","value":host,"facts":{"hostname":host,"sources":sources,"verification":"third_party_passive_unverified"}}));
                } else {
                    excluded += 1;
                }
            }
            raw.push(json!({"stdout":output.stdout,"stderr":output.stderr}));
        } else {
            let t = self.target(&plan.config.target_id)?;
            let dependency: Option<String> = self
                .db
                .connect()?
                .query_row(
                    "SELECT dependency_job_id FROM recon_tool_approvals WHERE job_id=?1",
                    [&job.id],
                    |r| r.get(0),
                )
                .map_err(db_error)?;
            let urls = if t.input.kind == TargetKind::Domain {
                let conn = self.db.connect()?;
                let names=conn.prepare("SELECT value FROM assets WHERE project_id=?1 AND target_id=?2 AND kind='subdomain' AND (?3 IS NULL OR job_id=?3) ORDER BY value LIMIT 1000").map_err(db_error)?.query_map(params![job.project_id,job.target_id,dependency],|r|r.get::<_,String>(0)).map_err(db_error)?.collect::<std::result::Result<Vec<_>,_>>().map_err(db_error)?;
                let mut approved = Vec::new();
                for name in names {
                    let url = format!("https://{name}/");
                    if self
                        .validate_scope(&plan.config.target_id, Some(&url))?
                        .status
                        == "allowed"
                    {
                        approved.push(url);
                    } else {
                        excluded += 1;
                    }
                    if approved.len() >= plan.config.max_assets as usize {
                        break;
                    }
                }
                approved
            } else {
                vec![plan.target.clone()]
            };
            if urls.is_empty() {
                return Err(AppError::new(
                    "NO_APPROVED_ASSETS",
                    "No in-scope passive assets available; run discovery first",
                ));
            }
            for (index, value) in urls.iter().enumerate() {
                self.recon_job_plan(job)?;
                if self
                    .validate_scope(&plan.config.target_id, Some(value))?
                    .status
                    != "allowed"
                {
                    excluded += 1;
                    continue;
                }
                let original =
                    url::Url::parse(value).map_err(|_| denied("Invalid observed URL"))?;
                let (transport, host) = pinned_destination(&original)?;
                update(
                    15,
                    "Destination resolved and pinned; starting approved HTTPX probe",
                )?;
                let output = manager.recon_execute(
                    reconx_tools::ReconToolRequest::Httpx {
                        transport_url: transport.clone(),
                        host,
                        timeout_secs: plan.config.timeout_secs,
                        technology: plan.config.technology,
                    },
                    || update(15, "Waiting for actual HTTPX output"),
                )?;
                let mut result = parse_httpx_output(&output.stdout, &transport)?;
                result["url"] = json!(value);
                result["source"] = json!("httpx");
                result["tls_trust_verified"] = json!(false);
                observations.push(json!({"kind":"http_endpoint","value":value,"facts":result}));
                raw.push(json!({"url":value,"stdout":output.stdout,"stderr":output.stderr}));
                update(
                    20 + (index * 60 / urls.len()) as u8,
                    "Approved HTTPX result received and normalized",
                )?;
                if index + 1 < urls.len() {
                    for _ in 0..10 {
                        std::thread::sleep(Duration::from_millis(100));
                        update(20, "Rate-limit interval; cancellation and scope rechecked")?;
                    }
                }
            }
        }
        if observations.len() > 1000 {
            return Err(AppError::new("OUTPUT_LIMIT", "Too many recon observations"));
        }
        let source_errors = raw.iter().any(|r| {
            r["stderr"]
                .as_str()
                .is_some_and(|s| s.contains("[ERR]") || s.to_lowercase().contains("failed"))
        });
        let facts = json!({"tool":plan.config.tool.id(),"version":plan.version,"observations":observations,"excluded_by_scope":excluded,"empty_state":if observations.is_empty(){"indeterminate_empty"}else{"observed"},"passive_source_status":if plan.config.tool!=ReconTool::Subfinder {"not_applicable"}else if source_errors{"source_errors_observed"}else{"not_individually_confirmed"},"verification":"Research observations, not vulnerabilities"});
        Ok(jobs::PreparedJob {
            result: Value::Null,
            analysis: None,
            http: Some(recon::HttpObservation {
                snapshot: json!({"plan":plan,"output":raw}),
                url: String::new(),
                facts,
            }),
        })
    }
    pub(super) fn commit_recon_tool(
        &self,
        conn: &Connection,
        job: &AnalysisJob,
        output: &recon::HttpObservation,
    ) -> Result<Value> {
        let plan = self.recon_job_plan(job)?;
        let bytes = serde_json::to_vec(&output.snapshot).map_err(json_error)?;
        let evidence = self.insert_observed_evidence(
            conn,
            EvidenceInput {
                project_id: job.project_id.clone(),
                target_id: job.target_id.clone(),
                kind: EvidenceKind::ToolOutput,
                title: format!("{} observed reconnaissance output", plan.config.tool.id()),
                source_tool: plan.config.tool.id().into(),
                sensitivity: Sensitivity::Confidential,
            },
            Some(job.id.clone()),
            &bytes,
            "application/json",
        )?;
        for row in output.facts["observations"]
            .as_array()
            .ok_or_else(|| denied("Invalid normalized result"))?
        {
            let value = row["value"]
                .as_str()
                .ok_or_else(|| denied("Missing observed asset"))?;
            if self
                .validate_scope(&plan.config.target_id, Some(value))?
                .status
                != "allowed"
            {
                return Err(denied("Scope changed before publication"));
            }
            let mut facts = row["facts"].clone();
            facts["evidence_id"] = json!(evidence.id);
            facts["source_tool"] = json!(plan.config.tool.id());
            conn.execute("INSERT INTO assets(id,project_id,target_id,kind,value,first_seen,last_seen,job_id,facts_json) VALUES(?1,?2,?3,?4,?5,?6,?6,?7,?8) ON CONFLICT(project_id,target_id,kind,value) DO UPDATE SET last_seen=excluded.last_seen,job_id=excluded.job_id,facts_json=excluded.facts_json",params![Uuid::new_v4().to_string(),job.project_id,job.target_id,row["kind"].as_str(),value,now(),job.id,serde_json::to_string(&facts).map_err(json_error)?]).map_err(db_error)?;
        }
        let mut result = output.facts.clone();
        result["evidence_id"] = json!(evidence.id);
        Ok(result)
    }
}
fn pinned_destination(url: &url::Url) -> Result<(String, String)> {
    let host = url.host_str().ok_or_else(|| denied("URL has no host"))?;
    let ip = if host == "localhost" {
        IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
    } else if let Ok(ip) = host.trim_matches(['[', ']']).parse::<IpAddr>() {
        if ip.is_unspecified()
            || ip.is_multicast()
            || match ip {
                IpAddr::V4(v) => v.is_link_local() || v.is_broadcast(),
                IpAddr::V6(v) => v.is_unicast_link_local() || v.to_ipv4_mapped().is_some(),
            }
        {
            return Err(denied(
                "Unspecified or multicast destinations are forbidden",
            ));
        }
        ip
    } else {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| denied("DNS runtime unavailable"))?;
        let port = url
            .port_or_known_default()
            .ok_or_else(|| denied("Invalid port"))?;
        let ips: Vec<SocketAddr> = runtime.block_on(async {
            tokio::time::timeout(
                Duration::from_secs(4),
                tokio::net::lookup_host((host, port)),
            )
            .await
            .map_err(|_| denied("DNS timeout"))?
            .map(|i| i.collect())
            .map_err(|_| denied("DNS resolution failed"))
        })?;
        if ips.is_empty() || ips.iter().any(|s| !public_ip(s.ip())) {
            return Err(denied(
                "DNS returned a private, special or unavailable destination; use an explicit registered local-IP URL for a lab",
            ));
        }
        ips[0].ip()
    };
    let mut transport = url.clone();
    transport
        .set_ip_host(ip)
        .map_err(|_| denied("Cannot pin destination"))?;
    let authority = match url.port() {
        Some(p) => format!("{host}:{p}"),
        None => host.to_string(),
    };
    Ok((transport.to_string(), authority))
}
pub fn parse_subfinder_output(output: &str, parent: &str) -> Result<BTreeMap<String, Vec<String>>> {
    if output.len() > 1024 * 1024 {
        return Err(denied("Passive output exceeds limit"));
    }
    let parent = domain(parent)?;
    let mut rows: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for line in output.lines().filter(|l| !l.trim().is_empty()) {
        let v: Value = serde_json::from_str(line)
            .map_err(|_| AppError::new("TOOL_OUTPUT", "Subfinder returned malformed JSONL"))?;
        let name = domain(
            v["host"]
                .as_str()
                .ok_or_else(|| denied("Passive record missing host"))?,
        )?;
        if name != parent && !name.ends_with(&format!(".{parent}")) {
            return Err(denied("Passive output names a different parent domain"));
        }
        if v["sources"].as_array().is_some_and(|a| a.len() > 16) {
            return Err(denied("Too many source attributions in one passive record"));
        }
        let sources = v["sources"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(|s| s.chars().take(64).collect())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_else(|| {
                v["source"]
                    .as_str()
                    .map(|s| vec![s.chars().take(64).collect()])
                    .unwrap_or_default()
            });
        let item = rows.entry(name).or_default();
        for s in sources {
            if !item.contains(&s) {
                item.push(s);
            }
        }
        if rows.len() > 1000 {
            return Err(denied("Passive host budget exceeded"));
        }
    }
    Ok(rows)
}
pub fn parse_httpx_output(output: &str, transport: &str) -> Result<Value> {
    if output.len() > 1024 * 1024 {
        return Err(denied("HTTPX output exceeds limit"));
    }
    let lines = output
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect::<Vec<_>>();
    if lines.len() != 1 {
        return Err(AppError::new(
            "TOOL_OUTPUT",
            "Expected exactly one HTTPX response; empty or extra records are not successful probes",
        ));
    }
    let v: Value = serde_json::from_str(lines[0])
        .map_err(|_| AppError::new("TOOL_OUTPUT", "HTTPX returned malformed JSON"))?;
    if v["failed"] == true
        || v["url"].as_str() != Some(transport)
        || v["status_code"]
            .as_u64()
            .is_none_or(|n| !(100..=599).contains(&n))
    {
        return Err(denied("HTTPX result does not match the pinned request"));
    }
    let text = |key: &str| {
        v[key]
            .as_str()
            .map(|s| reporting::redact_text(&s.chars().take(512).collect::<String>()).0)
    };
    let tech = v["tech"].as_array().map(|a| {
        a.iter()
            .take(32)
            .filter_map(Value::as_str)
            .map(|s| s.chars().take(128).collect::<String>())
            .collect::<Vec<_>>()
    });
    let tls = if v["tls"].to_string().len() <= 16384 {
        v["tls"].clone()
    } else {
        Value::Null
    };
    Ok(
        json!({"status":v["status_code"],"title":text("title"),"content_type":text("content_type"),"response_time":text("time"),"ip":text("host"),"technology":tech,"tls":tls,"method":"GET","redirects_followed":false,"response_read_limit":65536}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_transports_never_resolve_the_original_name_again() {
        for value in ["http://127.0.0.1:8123/path", "http://[::1]:8123/path"] {
            let u = url::Url::parse(value).unwrap();
            assert_eq!(pinned_destination(&u).unwrap().0, value);
        }
        let local = url::Url::parse("http://localhost:8123/path").unwrap();
        let (uri, host) = pinned_destination(&local).unwrap();
        assert_eq!(uri, "http://127.0.0.1:8123/path");
        assert_eq!(host, "localhost:8123");
        for value in [
            "http://169.254.169.254/",
            "http://0.0.0.0/",
            "http://[::ffff:127.0.0.1]/",
        ] {
            assert!(pinned_destination(&url::Url::parse(value).unwrap()).is_err());
        }
        for ip in [
            "127.0.0.1",
            "10.0.0.1",
            "100.64.0.1",
            "192.168.1.1",
            "198.18.0.1",
            "203.0.113.1",
            "::1",
            "fe80::1",
            "2001:db8::1",
        ] {
            assert!(!public_ip(ip.parse().unwrap()), "{ip}");
        }
        for ip in [
            "1.1.1.1",
            "8.8.8.8",
            "192.9.9.9",
            "203.1.1.1",
            "2001:4860:4860::8888",
        ] {
            assert!(public_ip(ip.parse().unwrap()), "{ip}");
        }
    }
    #[test]
    fn passive_fixture_commit_preserves_provenance_scope_and_integrity() {
        let dir = tempfile::tempdir().unwrap();
        let core = Core::open(Some(dir.path().join("fixture.db"))).unwrap();
        let project = core
            .create_project("Synthetic parser persistence fixture", "")
            .unwrap();
        let target=core.create_target(serde_json::from_value(json!({"project_id":project.id,"name":"Synthetic parent","kind":"domain","value":"example.test","authorized":true,"allowed_scope":["example.test","*.example.test"]})).unwrap()).unwrap();
        // Explicit test policy/record fixture. No external enumeration or tool health success is simulated.
        let conn = core.db.connect().unwrap();
        conn.execute("INSERT INTO extension_installations(extension_id,state_json) VALUES('subfinder',?1)",[json!({"version":"0.1.0","enabled":true,"status":"installed","revision":"synthetic-test-policy","updated_at":now(),"last_health_check":null,"error_code":null}).to_string()]).unwrap();
        let config = ReconConfig::for_target(&target.id, ReconTool::Subfinder);
        let plan = ReconPlan {
            fingerprint: core.recon_fingerprint(&config).unwrap(),
            config,
            project_id: project.id.clone(),
            target: "example.test".into(),
            version: "fixture-only".into(),
            permissions: vec![],
            limits: vec![],
        };
        let id = Uuid::new_v4().to_string();
        conn.execute("INSERT INTO analysis_jobs(id,project_id,target_id,kind,status,created_at,logs_json) VALUES(?1,?2,?3,'recon_subfinder','running',?4,'[]')",params![id,project.id,target.id,now()]).unwrap();
        conn.execute(
            "INSERT INTO recon_tool_approvals(job_id,plan_json,approved_at) VALUES(?1,?2,?3)",
            params![id, serde_json::to_string(&plan).unwrap(), now()],
        )
        .unwrap();
        let job = core.job(&id).unwrap();
        let observation = recon::HttpObservation {
            url: String::new(),
            snapshot: json!({"fixture":"Synthetic recorded output; not live passive execution"}),
            facts: json!({"observations":[{"kind":"subdomain","value":"api.example.test","facts":{"sources":["fixture-only"],"verification":"unverified"}}]}),
        };
        let tx = conn.unchecked_transaction().unwrap();
        let result = core.commit_recon_tool(&tx, &job, &observation).unwrap();
        // Both the original output and normalized result must coexist; a
        // UNIQUE(job_id,kind) collision previously rolled back successful scans.
        core.record_job_evidence(&tx, &job, &result, None).unwrap();
        assert_eq!(
            tx.query_row(
                "SELECT count(*) FROM evidence WHERE job_id=?1",
                [&id],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            2
        );
        tx.commit().unwrap();
        let e = result["evidence_id"].as_str().unwrap();
        assert_eq!(core.evidence_verify(e).unwrap()["integrity"], "matched");
        let assets = core.assets(Some(&project.id)).unwrap();
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].target_id.as_deref(), Some(target.id.as_str()));
        assert_eq!(assets[0].facts["evidence_id"], e);
        let mut changed = core.target(&target.id).unwrap().input;
        changed.excluded_scope = vec!["api.example.test".into()];
        core.update_target(&target.id, changed).unwrap();
        assert_eq!(
            core.commit_recon_tool(&conn, &job, &observation)
                .unwrap_err()
                .code,
            "APPROVAL_STALE"
        );
    }
}
