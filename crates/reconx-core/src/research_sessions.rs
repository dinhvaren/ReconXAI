//! Target-bound, explicitly reviewed sessions over the existing Job Engine.
//! No model-supplied arguments, shell, remote scanning or implicit AI egress.
use super::*;
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceSelection {
    pub revision: i64,
    pub project_id: Option<String>,
    pub target_id: Option<String>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResearchMode {
    Manual,
    AiAssisted,
    AuthorizedAuto,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchRequest {
    pub target_id: String,
    pub provider_id: Option<String>,
    pub mode: ResearchMode,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchCapability {
    pub id: String,
    pub name: String,
    pub available: bool,
    pub reason: String,
    pub network_requests: Option<u8>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchPreview {
    pub request: ResearchRequest,
    pub project_id: String,
    pub target_name: String,
    pub target_kind: TargetKind,
    pub file_id: Option<String>,
    pub model: Option<String>,
    pub gateway: Option<String>,
    pub context: String,
    pub fingerprint: String,
    pub capabilities: Vec<ResearchCapability>,
    pub limitations: Vec<String>,
    #[serde(default)]
    pub recon_reviews: Vec<ReconPlan>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchReview {
    pub request: ResearchRequest,
    pub fingerprint: String,
    pub context: String,
    pub consent: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchPlan {
    pub id: String,
    pub preview: ResearchPreview,
    pub proposal: PlanProposal,
    pub planning_usage: Option<AiReply>,
    pub created_at: String,
}
#[derive(Debug, Serialize)]
pub struct ResearchSession {
    pub plan: ResearchPlan,
    pub status: String,
    pub approved_at: Option<String>,
    pub cancelled_at: Option<String>,
    pub jobs: Vec<AnalysisJob>,
    pub evidence: Vec<Evidence>,
    pub findings: Vec<Finding>,
    pub reports: Vec<ReportRecord>,
    pub interpretation: Option<AiReply>,
    pub runtime_enabled: bool,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchInterpretReview {
    pub id: String,
    pub provider_id: String,
    pub gateway: String,
    pub model: String,
    pub preview: String,
    pub consent: bool,
}

fn policy(message: &str) -> AppError {
    AppError::new("RESEARCH_POLICY", message)
}
impl Core {
    pub fn workspace_selection(&self) -> Result<WorkspaceSelection> {
        self.db
            .connect()?
            .query_row(
                "SELECT revision,project_id,target_id FROM workspace_selection WHERE id=1",
                [],
                |r| {
                    Ok(WorkspaceSelection {
                        revision: r.get(0)?,
                        project_id: r.get(1)?,
                        target_id: r.get(2)?,
                    })
                },
            )
            .map_err(db_error)
    }
    pub fn select_workspace(
        &self,
        project: Option<&str>,
        target: Option<&str>,
        revision: i64,
    ) -> Result<WorkspaceSelection> {
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let project = project.map(|id| self.project(id)).transpose()?;
        if project.as_ref().is_some_and(|p| p.archived) {
            return Err(policy("Select an active project"));
        }
        let target = target.map(|id| self.target(id)).transpose()?;
        if target
            .as_ref()
            .is_some_and(|t| Some(&t.input.project_id) != project.as_ref().map(|p| &p.id))
        {
            return Err(policy("The selected target belongs to a different project"));
        }
        let changed = tx.execute("UPDATE workspace_selection SET revision=revision+1,project_id=?1,target_id=?2 WHERE id=1 AND revision=?3", params![project.as_ref().map(|p| &p.id),target.as_ref().map(|t| &t.id),revision]).map_err(db_error)?;
        if changed != 1 {
            return Err(AppError::new(
                "STALE_SELECTION",
                "Workspace selection changed in another client; refresh",
            ));
        }
        tx.commit().map_err(db_error)?;
        self.workspace_selection()
    }
    pub fn research_preview(&self, request: ResearchRequest) -> Result<ResearchPreview> {
        let target = self.target(&request.target_id)?;
        let project = self.project(&target.input.project_id)?;
        if project.archived
            || !target.input.authorized
            || self.validate_scope(&target.id, None)?.status != "allowed"
        {
            return Err(policy(
                "Select an authorized, in-scope target in an active project",
            ));
        }
        let provider = request
            .provider_id
            .as_deref()
            .map(|id| self.provider(id))
            .transpose()?;
        if request.mode != ResearchMode::Manual && provider.is_none() {
            return Err(AppError::new(
                "MODEL_UNAVAILABLE",
                "Configure and select an AI model first",
            ));
        }
        if let Some(p) = &provider
            && (!p.enabled || (p.requires_key && !self.credential_status(&p.id)?))
        {
            return Err(AppError::new(
                "MODEL_UNAVAILABLE",
                "Enable the selected model and configure its credential",
            ));
        }
        let file = self
            .list_files(Some(&project.id))?
            .into_iter()
            .find(|f| f.target_id.as_deref() == Some(&target.id));
        if let Some(file) = &file {
            self.require_file_authorization(file)?;
        }
        let registry = self.capabilities(&project.id)?;
        let mut caps = vec![ResearchCapability {
            id: "local.scope_review".into(),
            name: "Review registered scope (offline)".into(),
            available: true,
            reason: "Existing target-registration handler; no network traffic".into(),
            network_requests: Some(0),
        }];
        for (id, name) in [
            ("local.metadata", "Extract binary metadata"),
            ("local.strings", "Extract local strings"),
            ("yara-x.signatures", "Run fixed local YARA-X signatures"),
        ] {
            let too_large = id == "yara-x.signatures"
                && file
                    .as_ref()
                    .is_some_and(|f| f.size > reconx_analysis::MAX_FILE_BYTES as u64);
            let available = !too_large
                && file.is_some()
                && registry
                    .iter()
                    .any(|c| c["id"] == id && c["available"] == true);
            caps.push(ResearchCapability {
                id: id.into(),
                name: name.into(),
                available,
                reason: if file.is_none() {
                    "Import a local file associated with this target first".into()
                } else if too_large {
                    "This file exceeds the fixed YARA-X memory budget; full artifact remains imported".into()
                } else if !available {
                    "Install and verify the reviewed YARA-X extension in Marketplace".into()
                } else {
                    "Existing read-only local-file Job Engine capability".into()
                },
                network_requests: Some(0),
            });
        }
        let web = self.local_http_preview(&target.id);
        caps.push(ResearchCapability{id:"local.http_head".into(),name:"One reviewed owned-loopback HEAD request".into(),available:web.is_ok() && request.mode!=ResearchMode::AuthorizedAuto,reason:if request.mode==ResearchMode::AuthorizedAuto{"Network requests always require separate manual execution review; unavailable in Authorized Auto".into()}else{web.as_ref().map(|_|"One request; no redirects, proxy, retries or scanner".into()).unwrap_or_else(|e|e.message.clone())},network_requests:Some(1)});
        let mut recon_reviews = Vec::new();
        for (tool, cap, name) in [
            (
                ReconTool::Subfinder,
                "subfinder.passive",
                "Passive subdomain intelligence",
            ),
            (
                ReconTool::Httpx,
                "httpx.probe",
                "Approved scoped HTTPX probing",
            ),
        ] {
            let candidate = self.recon_plan(ReconConfig::for_target(&target.id, tool.clone()));
            let available = candidate.is_ok() && request.mode != ResearchMode::AuthorizedAuto;
            let reason = if request.mode == ResearchMode::AuthorizedAuto {
                "Network/source disclosure requires a separate reviewed session".into()
            } else {
                candidate
                    .as_ref()
                    .map(|p| p.permissions.join("; "))
                    .unwrap_or_else(|e| e.message.clone())
            };
            if let Ok(p) = candidate {
                recon_reviews.push(p);
            }
            caps.push(ResearchCapability {
                id: cap.into(),
                name: name.into(),
                available,
                reason,
                network_requests: if tool == ReconTool::Subfinder {
                    None
                } else {
                    Some(4)
                },
            });
        }
        let permitted: Vec<_> = caps.iter().filter(|c| c.available).collect();
        let context=serde_json::to_string_pretty(&json!({"target_id":target.id,"target_type":target.input.kind,"scope":"authorized and allowed","file":file.as_ref().map(|f|json!({"format":f.format,"size":f.size,"sha256":f.sha256})),"capabilities":permitted,"task":"Return strict JSON only: {\"steps\":[{\"capability_id\":\"an available ID\",\"reason\":\"brief rationale\"}]}. Choose 1 to 4 unique available capabilities. No tool arguments, commands, claims of vulnerabilities or unsupported analyzers. Network inventory, if available, is a single separately reviewed local HEAD, not a pentest."})).map_err(json_error)?;
        let options = provider
            .as_ref()
            .map(|p| self.ai_options(&p.id))
            .transpose()?;
        let fingerprint=evidence::digest(&serde_json::to_vec(&json!({"target":target.input,"application_mode":self.settings()?.security_mode,"file":file.as_ref().map(|f|(&f.id,&f.sha256)),"request":request,"context":context,"provider":provider,"options":options,"recon_reviews":recon_reviews})).map_err(json_error)?);
        Ok(ResearchPreview{request,project_id:project.id,target_name:target.input.name,target_kind:target.input.kind,file_id:file.map(|f|f.id),model:provider.as_ref().map(|p|p.model.clone()),gateway:provider.map(|p|p.base_url),context,fingerprint,capabilities:caps,recon_reviews,limitations:vec!["This session runs only the listed existing bounded capabilities, not an automated penetration test".into(),"Domain, source, APK, cloud, decompiler and arbitrary MCP/scanner analyzers remain unavailable unless explicitly listed".into(),"AI interpretation and any resulting finding require review; no automatic Verified status".into(),"Legacy batches cannot pause; the offline DAG pauses at job boundaries. Cost estimates are unavailable; model token usage is reported only when supplied".into()]})
    }
    fn validate_research_proposal(
        &self,
        preview: &ResearchPreview,
        proposal: &PlanProposal,
    ) -> Result<()> {
        if proposal.steps.is_empty() || proposal.steps.len() > 4 {
            return Err(policy("Choose 1 to 4 bounded steps"));
        }
        let mut seen = std::collections::HashSet::new();
        for step in &proposal.steps {
            validate_text(&step.reason, 500, "Step reason")?;
            if !seen.insert(&step.capability_id)
                || !preview
                    .capabilities
                    .iter()
                    .any(|c| c.id == step.capability_id && c.available)
            {
                return Err(policy(
                    "Unknown, duplicate, unavailable or unapproved capability",
                ));
            }
        }
        if preview.target_kind == TargetKind::Domain
            && let Some(http) = proposal
                .steps
                .iter()
                .position(|s| s.capability_id == "httpx.probe")
        {
            if let Some(passive) = proposal
                .steps
                .iter()
                .position(|s| s.capability_id == "subfinder.passive")
            {
                if passive > http {
                    return Err(policy(
                        "Passive discovery must precede its HTTPX dependency",
                    ));
                }
            } else if !self.assets(Some(&preview.project_id))?.iter().any(|a| {
                a.target_id.as_deref() == Some(&preview.request.target_id) && a.kind == "subdomain"
            }) {
                return Err(policy(
                    "Discover in-scope assets before HTTPX; include Subfinder before the HTTPX step",
                ));
            }
        }
        Ok(())
    }
    pub async fn research_plan(&self, review: ResearchReview) -> Result<ResearchSession> {
        let preview = self.research_preview(review.request.clone())?;
        if preview.fingerprint != review.fingerprint || preview.context != review.context {
            return Err(AppError::new(
                "APPROVAL_STALE",
                "Target, provider or capabilities changed; review context again",
            ));
        }
        let reply = if preview.request.mode == ResearchMode::Manual {
            None
        } else {
            if !review.consent {
                return Err(AppError::new(
                    "CONSENT_REQUIRED",
                    "Approve this exact metadata and model destination before AI planning",
                ));
            }
            let config = self.provider(
                preview
                    .request
                    .provider_id
                    .as_deref()
                    .ok_or_else(|| policy("Select a model"))?,
            )?;
            Some(
                self.chat_artifacts_to_gateway(config, &preview.context)
                    .await?,
            )
        };
        let proposal = if let Some(reply) = &reply {
            serde_json::from_str::<PlanProposal>(reply.text.trim()).map_err(|_| {
                AppError::new(
                    "INVALID_AI_PLAN",
                    "The model did not return a valid typed plan; no jobs were created",
                )
            })?
        } else {
            PlanProposal {
                steps: preview
                    .capabilities
                    .iter()
                    .filter(|c| c.available && c.network_requests == Some(0))
                    .take(4)
                    .map(|c| PlanStep {
                        capability_id: c.id.clone(),
                        reason: c.name.clone(),
                    })
                    .collect(),
            }
        };
        self.validate_research_proposal(&preview, &proposal)?;
        if self.research_preview(review.request)?.fingerprint != preview.fingerprint {
            return Err(AppError::new(
                "APPROVAL_STALE",
                "Configuration changed during planning",
            ));
        }
        let plan = ResearchPlan {
            id: Uuid::new_v4().to_string(),
            preview,
            proposal,
            planning_usage: reply,
            created_at: now(),
        };
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if self
            .research_preview(plan.preview.request.clone())?
            .fingerprint
            != plan.preview.fingerprint
        {
            return Err(policy(
                "Configuration changed before persisting this plan; review again",
            ));
        }
        tx.execute("INSERT INTO research_sessions(id,project_id,target_id,provider_id,created_at,plan_json) VALUES(?1,?2,?3,?4,?5,?6)",params![plan.id,plan.preview.project_id,plan.preview.request.target_id,plan.preview.request.provider_id,plan.created_at,serde_json::to_string(&plan).map_err(json_error)?]).map_err(db_error)?;
        Self::audit(&tx, "research.planned", &plan.id, "awaiting_approval")?;
        tx.commit().map_err(db_error)?;
        self.research_session(&plan.id)
    }
    pub fn research_list(&self, project: &str) -> Result<Vec<String>> {
        let project = self.project(project)?;
        self.db.connect()?.prepare("SELECT id FROM research_sessions WHERE project_id=?1 ORDER BY created_at DESC LIMIT 100").map_err(db_error)?.query_map([project.id],|r|r.get(0)).map_err(db_error)?.collect::<std::result::Result<_,_>>().map_err(db_error)
    }
    pub fn research_session(&self, id: &str) -> Result<ResearchSession> {
        let conn = self.db.connect()?;
        let (text,approved_at,cancelled_at,reply):(String,Option<String>,Option<String>,Option<String>)=conn.query_row("SELECT plan_json,approved_at,cancelled_at,reply_json FROM research_sessions WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(db_error)?.ok_or_else(||AppError::new("NOT_FOUND","Research session does not exist"))?;
        let plan: ResearchPlan = serde_json::from_str(&text).map_err(json_error)?;
        let ids: Vec<String> = conn
            .prepare("SELECT job_id FROM research_session_jobs WHERE session_id=?1 ORDER BY step")
            .map_err(db_error)?
            .query_map([id], |r| r.get(0))
            .map_err(db_error)?
            .collect::<std::result::Result<_, _>>()
            .map_err(db_error)?;
        let jobs: Vec<_> = ids.iter().map(|id| self.job(id)).collect::<Result<_>>()?;
        let status = if cancelled_at.is_some() {
            if jobs.iter().any(|j| j.status == "running") {
                "cancelling"
            } else {
                "cancelled"
            }
        } else if approved_at.is_none() {
            "awaiting_approval"
        } else if jobs.iter().any(|j| j.status == "running") {
            "running"
        } else if jobs.iter().any(|j| j.status == "queued") {
            "queued"
        } else if jobs.iter().any(|j| j.status != "completed") {
            "failed"
        } else {
            "completed"
        };
        let runtime: Option<String> = conn
            .query_row(
                "SELECT status FROM research_runtimes WHERE session_id=?1",
                [id],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        let status = if cancelled_at.is_none() {
            runtime.as_deref().unwrap_or(status)
        } else {
            status
        };
        let evidence_ids:Vec<String>=conn.prepare("SELECT id FROM evidence WHERE job_id IN (SELECT job_id FROM research_session_jobs WHERE session_id=?1) UNION SELECT record_id FROM research_session_records WHERE session_id=?1 AND kind='interpretation'").map_err(db_error)?.query_map([id],|r|r.get(0)).map_err(db_error)?.collect::<std::result::Result<_,_>>().map_err(db_error)?;
        let mut findings = Vec::new();
        let mut reports = Vec::new();
        let records:Vec<(String,String)>=conn.prepare("SELECT record_id,kind FROM research_session_records WHERE session_id=?1 AND kind!='interpretation' ORDER BY created_at").map_err(db_error)?.query_map([id],|r|Ok((r.get(0)?,r.get(1)?))).map_err(db_error)?.collect::<std::result::Result<_,_>>().map_err(db_error)?;
        for (record, kind) in records {
            if kind == "finding" {
                if let Ok(f) = self.finding(&record) {
                    findings.push(f);
                }
            } else {
                reports.push(self.report_show(&record)?.report);
            }
        }
        Ok(ResearchSession {
            plan,
            status: status.into(),
            runtime_enabled: runtime.is_some(),
            approved_at,
            cancelled_at,
            jobs,
            evidence: evidence_ids
                .iter()
                .map(|id| self.evidence(id))
                .collect::<Result<_>>()?,
            findings,
            reports,
            interpretation: reply
                .map(|r| serde_json::from_str(&r).map_err(json_error))
                .transpose()?,
        })
    }
    pub fn research_approve(
        &self,
        id: &str,
        fingerprint: &str,
        approved: bool,
        network_approved: bool,
    ) -> Result<ResearchSession> {
        if !approved {
            return Err(AppError::new(
                "APPROVAL_REQUIRED",
                "Review all planned steps before execution",
            ));
        }
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let session = self.research_session(id)?;
        if session.approved_at.is_some() || session.cancelled_at.is_some() {
            return Err(policy(
                "This session approval is already consumed or cancelled",
            ));
        }
        let preview = self.research_preview(session.plan.preview.request.clone())?;
        if preview.fingerprint != fingerprint
            || preview.fingerprint != session.plan.preview.fingerprint
        {
            return Err(AppError::new(
                "APPROVAL_STALE",
                "Target, model, policy or capabilities changed; create a new plan",
            ));
        }
        self.validate_research_proposal(&preview, &session.plan.proposal)?;
        let mut discovery_dependency: Option<String> = None;
        for (index, step) in session.plan.proposal.steps.iter().enumerate() {
            let job = if matches!(
                step.capability_id.as_str(),
                "httpx.probe" | "subfinder.passive"
            ) {
                if !network_approved || preview.request.mode == ResearchMode::AuthorizedAuto {
                    return Err(policy(
                        "Separately approve the displayed recon destinations, source sharing and budgets",
                    ));
                }
                let tool = if step.capability_id == "httpx.probe" {
                    ReconTool::Httpx
                } else {
                    ReconTool::Subfinder
                };
                let p = self.recon_plan(ReconConfig::for_target(
                    &preview.request.target_id,
                    tool.clone(),
                ))?;
                let id = self.recon_enqueue_in_tx(
                    &tx,
                    &p,
                    true,
                    if tool == ReconTool::Httpx {
                        discovery_dependency.as_deref()
                    } else {
                        None
                    },
                )?;
                if tool == ReconTool::Subfinder {
                    discovery_dependency = Some(id.clone());
                }
                id
            } else if step.capability_id == "local.http_head" {
                if !network_approved || preview.request.mode == ResearchMode::AuthorizedAuto {
                    return Err(policy(
                        "Separately approve the single owned-loopback HEAD request; it is unavailable in Authorized Auto",
                    ));
                }
                let head = self.local_http_preview(&preview.request.target_id)?;
                self.local_http_enqueue_in_tx(&tx, &head.target_id, &head.fingerprint, true)?
            } else {
                let kind = match step.capability_id.as_str() {
                    "local.scope_review" => JobKind::TargetRegistrationReview,
                    "local.metadata" => JobKind::BinaryMetadata,
                    "local.strings" => JobKind::StringsExtraction,
                    "yara-x.signatures" => JobKind::YaraSignatures,
                    _ => return Err(policy("Capability is not registered")),
                };
                self.enqueue_job_in_tx(
                    &tx,
                    JobInput {
                        kind,
                        file_id: if kind == JobKind::TargetRegistrationReview {
                            None
                        } else {
                            preview.file_id.clone()
                        },
                        target_id: Some(preview.request.target_id.clone()),
                        timeout_ms: 60000,
                    },
                    None,
                )?
            };
            tx.execute(
                "INSERT INTO research_session_jobs(session_id,step,job_id) VALUES(?1,?2,?3)",
                params![id, index as i64, job],
            )
            .map_err(db_error)?;
        }
        tx.execute(
            "UPDATE research_sessions SET approved_at=?1 WHERE id=?2",
            params![now(), id],
        )
        .map_err(db_error)?;
        Self::audit(&tx, "research.approved", id, "queued")?;
        tx.commit().map_err(db_error)?;
        self.research_session(id)
    }
    pub fn research_cancel(&self, id: &str) -> Result<ResearchSession> {
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let session = self.research_session(id)?;
        if matches!(session.status.as_str(), "completed" | "failed") {
            return Err(policy(
                "A finished session cannot be cancelled; results remain available",
            ));
        }
        for job in &session.jobs {
            if matches!(job.status.as_str(), "queued" | "running") {
                self.cancel_job_in_tx(&tx, &job.id)?;
            }
        }
        Self::cancel_agent_runtime(&tx, id)?;
        tx.execute(
            "UPDATE research_sessions SET cancelled_at=COALESCE(cancelled_at,?1) WHERE id=?2",
            params![now(), id],
        )
        .map_err(db_error)?;
        Self::audit(&tx, "research.cancelled", id, "requested")?;
        tx.commit().map_err(db_error)?;
        self.research_session(id)
    }
    fn complete_research(&self, id: &str) -> Result<ResearchSession> {
        let session = self.research_session(id)?;
        if session.status != "completed" || session.evidence.is_empty() {
            return Err(policy(
                "Complete the real analysis jobs and collect evidence first",
            ));
        }
        Ok(session)
    }
    pub fn research_evidence_preview(&self, id: &str) -> Result<String> {
        let session = self.complete_research(id)?;
        let previews: Vec<_> = session
            .jobs
            .iter()
            .map(|j| self.job_evidence_preview(&j.id))
            .collect::<Result<_>>()?;
        let text=serde_json::to_string_pretty(&json!({"session_id":id,"source_evidence_ids":session.evidence.iter().filter(|e|e.kind!=EvidenceKind::AiSummary).map(|e|&e.id).collect::<Vec<_>>(),"observations":previews,"task":"Explain only these actual observations. Cite the evidence IDs. Separate facts, inferences and hypotheses. Do not claim vulnerabilities are verified; no additional tools may be executed."})).map_err(json_error)?;
        if text.len() > 128 * 1024 {
            return Err(policy(
                "Evidence preview exceeds 128 KiB; review selected jobs separately",
            ));
        }
        Ok(reporting::redact_content(&text).0)
    }
    pub async fn research_interpret(
        &self,
        review: ResearchInterpretReview,
    ) -> Result<ResearchSession> {
        if !review.consent {
            return Err(AppError::new(
                "CONSENT_REQUIRED",
                "Approve the exact evidence preview and destination",
            ));
        }
        let session = self.complete_research(&review.id)?;
        if self.research_evidence_preview(&review.id)? != review.preview {
            return Err(AppError::new(
                "PREVIEW_CHANGED",
                "Review the current evidence again",
            ));
        }
        let provider = self.provider(&review.provider_id)?;
        if provider.base_url != review.gateway || provider.model != review.model {
            return Err(AppError::new(
                "DESTINATION_CHANGED",
                "The model destination changed",
            ));
        }
        let mut reply = self
            .chat_artifacts_to_gateway(provider, &review.preview)
            .await?;
        if self.research_evidence_preview(&review.id)? != review.preview {
            return Err(AppError::new(
                "PREVIEW_CHANGED",
                "Session changed during interpretation",
            ));
        }
        reply.text = reporting::redact_content(&reply.text).0;
        let evidence = self.evidence_add(
            EvidenceInput {
                project_id: session.plan.preview.project_id,
                target_id: Some(session.plan.preview.request.target_id),
                kind: EvidenceKind::AiSummary,
                title: "Unverified AI interpretation of research evidence".into(),
                source_tool: "AI research interpretation".into(),
                sensitivity: Sensitivity::Confidential,
            },
            &reply.text,
        )?;
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute(
            "UPDATE research_sessions SET reply_json=?1 WHERE id=?2",
            params![
                serde_json::to_string(&reply).map_err(json_error)?,
                review.id
            ],
        )
        .map_err(db_error)?;
        tx.execute(
            "INSERT INTO research_session_records VALUES(?1,?2,'interpretation',?3)",
            params![review.id, evidence.id, now()],
        )
        .map_err(db_error)?;
        Self::audit(&tx, "research.interpreted", &review.id, "unverified")?;
        tx.commit().map_err(db_error)?;
        self.research_session(&review.id)
    }
    pub fn research_finding(&self, id: &str, title: &str, details: &str) -> Result<Finding> {
        let session = self.complete_research(id)?;
        let finding=self.finding_save(None,FindingInput{project_id:session.plan.preview.project_id,target_id:Some(session.plan.preview.request.target_id),title:title.into(),description:details.into(),status:FindingStatus::NeedsVerification,evidence_ids:session.evidence.iter().map(|e|e.id.clone()).collect(),notes:"Manually recorded from an AI-assisted session. Interpretation is not deterministic verification.".into(),..Default::default()},None,false)?;
        self.db
            .connect()?
            .execute(
                "INSERT INTO research_session_records VALUES(?1,?2,'finding',?3)",
                params![id, finding.id, now()],
            )
            .map_err(db_error)?;
        Ok(finding)
    }
    pub fn research_report(&self, id: &str) -> Result<ReportView> {
        let session = self.complete_research(id)?;
        let report=self.report_prepare(ReportInput{project_id:session.plan.preview.project_id,title:format!("Research session {}",&id[..id.len().min(8)]),template:ReportTemplate::Research,summary:"Bounded research session. Evidence is observed; AI interpretations and findings require independent review. This is not a complete penetration test.".into(),methodology:session.plan.proposal.steps.iter().map(|s|format!("{}: {}",s.capability_id,s.reason)).collect::<Vec<_>>().join("\n"),finding_ids:session.findings.iter().map(|f|f.id.clone()).collect(),evidence_ids:session.evidence.iter().map(|e|e.id.clone()).collect()})?;
        self.db
            .connect()?
            .execute(
                "INSERT INTO research_session_records VALUES(?1,?2,'report',?3)",
                params![id, report.report.id, now()],
            )
            .map_err(db_error)?;
        Ok(report)
    }
}
