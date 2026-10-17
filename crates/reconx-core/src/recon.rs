//! Explicitly approved loopback HEAD inventory. No remote scanning or AI execution.
use super::*;
use serde_json::{Value, json};
use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};
const KIND: &str = "local_http_inventory";
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalHttpPlan {
    pub target_id: String,
    pub project_id: String,
    pub url: String,
    pub method: String,
    pub request_budget: u8,
    pub timeout_ms: u32,
    pub fingerprint: String,
    pub permissions: Vec<String>,
    pub limitations: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub project_id: String,
    pub target_id: Option<String>,
    pub kind: String,
    pub value: String,
    pub first_seen: String,
    pub last_seen: String,
    pub job_id: String,
    pub facts: Value,
}
pub(super) struct HttpObservation {
    pub snapshot: Value,
    pub url: String,
    pub facts: Value,
}
fn denied(message: &str) -> AppError {
    AppError::new("BLOCKED_BY_POLICY", message)
}
impl Core {
    pub fn local_http_preview(&self, target: &str) -> Result<LocalHttpPlan> {
        let target = self.target(target)?;
        let project = self.project(&target.input.project_id)?;
        if project.archived {
            return Err(denied("Restore the project before local inventory"));
        }
        if !matches!(
            target.input.kind,
            TargetKind::Url | TargetKind::RestApi | TargetKind::Graphql
        ) {
            return Err(denied(
                "This adapter accepts registered URL, REST or GraphQL targets only",
            ));
        }
        let url = url::Url::parse(&target.input.value)
            .map_err(|_| denied("Target needs an absolute local HTTP URL"))?;
        if !matches!(url.scheme(), "http" | "https")
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(denied(
                "Use HTTP(S) without credentials, query or fragment for local inventory",
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
                "Only loopback IPs or exact localhost are implemented; remote targets and public scanners are unavailable",
            ));
        }
        if !target.input.authorized
            || self.validate_scope(&target.id, Some(url.as_str()))?.status != "allowed"
        {
            return Err(denied(
                "Declare authorization and keep the exact URL inside allowed scope/exclusions",
            ));
        }
        let mode = self.settings()?.security_mode;
        if mode == SecurityMode::PassiveAuto || target.input.mode == SecurityMode::PassiveAuto {
            return Err(denied(
                "HTTP sends traffic. Select Authorized Auto or Lab Research in Settings and the target; this operation still needs exact manual approval",
            ));
        }
        let fingerprint=super::evidence::digest(&serde_json::to_vec(&json!({"adapter":1,"target":target.input,"application_mode":mode,"method":"HEAD","budget":1,"timeout_ms":5000})).map_err(json_error)?);
        Ok(LocalHttpPlan {
            target_id: target.id,
            project_id: project.id,
            url: url.to_string(),
            method: "HEAD".into(),
            request_budget: 1,
            timeout_ms: 5000,
            fingerprint,
            permissions: vec!["manual approval".into(), "one loopback HTTP request".into()],
            limitations: vec![
                "No redirects, proxies, authentication, body download or automatic retries".into(),
                "localhost is pinned to 127.0.0.1; use [::1] for IPv6-only services".into(),
                "No vulnerability checks; normalized header observation, not packet capture".into(),
            ],
        })
    }
    pub fn local_http_enqueue(
        &self,
        target: &str,
        fingerprint: &str,
        approved: bool,
    ) -> Result<AnalysisJob> {
        if !approved {
            return Err(denied(
                "Review the exact local HEAD plan and explicitly approve it",
            ));
        }
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let plan = self.local_http_preview(target)?;
        if plan.fingerprint != fingerprint {
            return Err(AppError::new(
                "APPROVAL_STALE",
                "Target or security policy changed; preview again",
            ));
        }
        let recent:i64=tx.query_row("SELECT count(*) FROM local_http_approvals a JOIN analysis_jobs j ON j.id=a.job_id WHERE j.project_id=?1 AND julianday(a.approved_at)>=julianday('now','-1 minute')",[&plan.project_id],|r|r.get(0)).map_err(db_error)?;
        if recent >= 20 {
            return Err(denied(
                "Local inventory budget: at most 20 approved requests per project per minute",
            ));
        }
        let id = Uuid::new_v4().to_string();
        let stamp = now();
        tx.execute("INSERT INTO analysis_jobs(id,project_id,target_id,kind,status,created_at,timeout_ms,logs_json) VALUES(?1,?2,?3,?4,'queued',?5,5000,'[\"Exact loopback HEAD plan approved; one request maximum\"]')",params![id,plan.project_id,plan.target_id,KIND,stamp]).map_err(db_error)?;
        tx.execute(
            "INSERT INTO local_http_approvals(job_id,fingerprint,approved_at) VALUES(?1,?2,?3)",
            params![id, fingerprint, stamp],
        )
        .map_err(db_error)?;
        Self::audit(&tx, "recon.local.queued", &id, "manually_approved")?;
        tx.commit().map_err(db_error)?;
        self.job(&id)
    }
    pub(super) fn local_http_job_plan(&self, job: &AnalysisJob) -> Result<LocalHttpPlan> {
        let fp: String = self
            .db
            .connect()?
            .query_row(
                "SELECT fingerprint FROM local_http_approvals WHERE job_id=?1",
                [&job.id],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| denied("Local HTTP job has no manual approval record"))?;
        let plan = self.local_http_preview(
            job.target_id
                .as_deref()
                .ok_or_else(|| denied("Target was removed"))?,
        )?;
        if plan.project_id != job.project_id || plan.fingerprint != fp {
            return Err(AppError::new(
                "APPROVAL_STALE",
                "Authorization or target changed after approval",
            ));
        }
        Ok(plan)
    }
    pub(super) fn prepare_local_http(
        &self,
        job: &AnalysisJob,
        update: &mut impl FnMut(u8, &str) -> Result<()>,
    ) -> Result<jobs::PreparedJob> {
        let plan = self.local_http_job_plan(job)?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| AppError::new("WORKER_ERROR", "Cannot start local HTTP runtime"))?;
        runtime.block_on(async {
            let url = url::Url::parse(&plan.url)
                .map_err(|_| denied("Invalid approved URL"))?;
            let builder = reqwest::Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .retry(reqwest::retry::never())
                .http1_only()
                .http1_max_headers(64)
                .connect_timeout(Duration::from_secs(2))
                .timeout(Duration::from_secs(5))
                .user_agent("ReconXAI-local-inventory/0.1")
                .resolve("localhost", SocketAddr::new(
                    IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
                    url.port_or_known_default().unwrap_or(80),
                ));
            let client = builder.build().map_err(|_| AppError::new(
                "HTTP_CONFIG", "Cannot configure local HTTP client",
            ))?;
            update(20, "Starting approved loopback HEAD; redirects and retries disabled")?;
            let request = client.head(&plan.url).send();
            tokio::pin!(request);
            let response = loop {
                tokio::select! {
                    result = &mut request => break result.map_err(|_| AppError::new(
                        "HTTP_FAILED", "Local HTTP connection/TLS failed or timed out; no automatic retry",
                    ))?,
                    _ = tokio::time::sleep(Duration::from_millis(50)) => {
                        update(20, "Waiting for local response headers")?;
                    }
                }
            };
            update(65, "Actual response headers received; no response body downloaded")?;
            let total: usize = response.headers().iter()
                .map(|(key, value)| key.as_str().len() + value.as_bytes().len()).sum();
            if total > 64 * 1024 {
                return Err(AppError::new("INPUT_LIMIT", "Response headers exceed 64 KiB"));
            }
            use base64::Engine;
            let headers: Vec<_> = response.headers().iter().map(|(name, value)| {
                if let Ok(value) = value.to_str() {
                    json!({"name": name.as_str(), "value": value})
                } else {
                    json!({
                        "name": name.as_str(),
                        "value_base64": base64::engine::general_purpose::STANDARD.encode(value.as_bytes()),
                    })
                }
            }).collect();
            let hint = |key| response.headers().get(key).and_then(|v| v.to_str().ok())
                .map(|v| super::reporting::redact_text(&v.chars().take(256).collect::<String>()).0);
            let facts = json!({
                "method": "HEAD", "status": response.status().as_u16(),
                "http_version": format!("{:?}", response.version()),
                "reported_server": hint("server"), "content_type": hint("content-type"),
                "header_count": headers.len(), "body_downloaded": false, "requests": 1,
                "observation": "Availability metadata, not a vulnerability assessment",
            });
            let snapshot = json!({
                "target_id": plan.target_id,
                "request": {"method": "HEAD", "url": plan.url, "authentication": "none"},
                "response": {
                    "status": response.status().as_u16(),
                    "http_version": format!("{:?}", response.version()), "headers": headers,
                },
                "representation": "Normalized response headers; not wire/packet capture",
                "observed_at": now(),
            });
            drop(response);
            self.local_http_job_plan(job)?;
            update(85, "Local authorization rechecked; normalized observation ready")?;
            Ok(jobs::PreparedJob {
                result: facts.clone(), analysis: None,
                http: Some(HttpObservation {snapshot, url: plan.url, facts}),
            })
        })
    }

    pub(super) fn commit_local_http(
        &self,
        conn: &Connection,
        job: &AnalysisJob,
        observation: &HttpObservation,
    ) -> Result<Value> {
        self.local_http_job_plan(job)?;
        let bytes = serde_json::to_vec_pretty(&observation.snapshot).map_err(json_error)?;
        let evidence = self.insert_observed_evidence(
            conn,
            EvidenceInput {
                project_id: job.project_id.clone(),
                target_id: job.target_id.clone(),
                kind: EvidenceKind::HttpResponse,
                title: "Approved loopback HTTP response".into(),
                source_tool: KIND.into(),
                sensitivity: Sensitivity::Confidential,
            },
            Some(job.id.clone()),
            &bytes,
            "application/json",
        )?;
        let stamp = now();
        let id = Uuid::new_v4().to_string();
        let id:String=conn.query_row("INSERT INTO assets(id,project_id,target_id,kind,value,first_seen,last_seen,job_id,facts_json) VALUES(?1,?2,?3,'http_endpoint',?4,?5,?5,?6,?7) ON CONFLICT(project_id,target_id,kind,value) DO UPDATE SET last_seen=excluded.last_seen,job_id=excluded.job_id,facts_json=excluded.facts_json RETURNING id",params![id,job.project_id,job.target_id,observation.url,stamp,job.id,serde_json::to_string(&observation.facts).map_err(json_error)?],|r|r.get(0)).map_err(db_error)?;
        conn.execute(
            "UPDATE targets SET last_analyzed_at=?1 WHERE id=?2",
            params![stamp, job.target_id],
        )
        .map_err(db_error)?;
        Ok(
            json!({"asset_id":id,"evidence_id":evidence.id,"url":observation.url,"observation":observation.facts,"request_budget":1}),
        )
    }
    pub fn assets(&self, project: Option<&str>) -> Result<Vec<Asset>> {
        let project = project.map(|p| self.project(p).map(|p| p.id)).transpose()?;
        let conn = self.db.connect()?;
        let mut stmt=conn.prepare("SELECT id,project_id,target_id,kind,value,first_seen,last_seen,job_id,facts_json FROM assets WHERE ?1 IS NULL OR project_id=?1 ORDER BY last_seen DESC,id LIMIT 1000").map_err(db_error)?;
        let rows = stmt
            .query_map([project], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, String>(7)?,
                    r.get::<_, String>(8)?,
                ))
            })
            .map_err(db_error)?;
        rows.map(|r| {
            let (id, project_id, target_id, kind, value, first_seen, last_seen, job_id, facts) =
                r.map_err(db_error)?;
            Ok(Asset {
                id,
                project_id,
                target_id,
                kind,
                value,
                first_seen,
                last_seen,
                job_id,
                facts: serde_json::from_str(&facts).map_err(json_error)?,
            })
        })
        .collect()
    }
}
