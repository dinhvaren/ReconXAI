//! Reviewed offline agent coordination. Jobs remain the only execution queue.
//! Network/MCP requests and model-driven continuation are deliberately excluded.
use super::*;
use serde_json::{Value, json};
use std::collections::BTreeSet;

const MAX_TASKS: usize = 12;
const MAX_MESSAGES: i64 = 2048;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentMessageType {
    TaskAssigned,
    TaskStarted,
    TaskProgress,
    ToolRequested,
    ApprovalRequired,
    ToolCompleted,
    ObservationCreated,
    EvidenceLinked,
    TaskCompleted,
    TaskFailed,
    ReplanRequested,
    SessionCompleted,
    PlanAccepted,
    PauseRequested,
    SessionResumed,
    SessionCancelled,
}
impl AgentMessageType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TaskAssigned => "TASK_ASSIGNED",
            Self::TaskStarted => "TASK_STARTED",
            Self::TaskProgress => "TASK_PROGRESS",
            Self::ToolRequested => "TOOL_REQUESTED",
            Self::ApprovalRequired => "APPROVAL_REQUIRED",
            Self::ToolCompleted => "TOOL_COMPLETED",
            Self::ObservationCreated => "OBSERVATION_CREATED",
            Self::EvidenceLinked => "EVIDENCE_LINKED",
            Self::TaskCompleted => "TASK_COMPLETED",
            Self::TaskFailed => "TASK_FAILED",
            Self::ReplanRequested => "REPLAN_REQUESTED",
            Self::SessionCompleted => "SESSION_COMPLETED",
            Self::PlanAccepted => "PLAN_ACCEPTED",
            Self::PauseRequested => "PAUSE_REQUESTED",
            Self::SessionResumed => "SESSION_RESUMED",
            Self::SessionCancelled => "SESSION_CANCELLED",
        }
    }
}

fn runtime_error(code: &str, message: &str) -> AppError {
    AppError::new(code, message)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDescriptor {
    pub id: String,
    pub name: String,
    pub capabilities: Vec<String>,
    pub permissions: Vec<String>,
    pub model_requirement: String,
    pub input_schema: Value,
    pub output_schema: Value,
    pub timeout_ms: u64,
    pub error_policy: String,
}

pub fn research_agents() -> Vec<AgentDescriptor> {
    [
        ("orchestrator", "Offline session coordinator", vec!["dag.lifecycle"]),
        ("planner", "Reviewed capability planner", vec!["research.plan"]),
        ("recon", "Offline scope reviewer", vec!["local.scope_review"]),
        ("reverse", "Read-only file analyzer", vec!["local.metadata", "local.strings", "yara-x.signatures"]),
        ("validation", "Evidence integrity validator", vec!["evidence.integrity"]),
        ("reporting", "Evidence-linked reporting", vec!["report.snapshot"]),
    ].into_iter().map(|(id,name,caps)| AgentDescriptor {
        id:id.into(),name:name.into(),capabilities:caps.into_iter().map(str::to_owned).collect(),
        permissions:vec!["reviewed-session".into(),"offline-only".into(),"no-credential-access".into()],
        model_requirement:if id=="planner" {"Existing research plan; optional explicitly reviewed AI planning"} else {"None; typed Core handler, no model or cloud request"}.into(),
        input_schema:json!({"type":"object","required":["session_id"],"properties":{"session_id":{"type":"string"}},"additionalProperties":false}),
        output_schema:json!({"type":"object","required":["evidence_ids"],"properties":{"evidence_ids":{"type":"array","items":{"type":"string"}}}}),
        timeout_ms:60_000,
        error_policy:"Stop dependent work; no implicit network/model fallback; only explicit bounded read-only retry".into(),
    }).collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct AgentLimits {
    pub concurrency: u8,
    pub max_invocations: u8,
    pub max_duration_ms: u64,
    pub max_retries: u8,
}
impl Default for AgentLimits {
    fn default() -> Self {
        Self {
            concurrency: 2,
            max_invocations: 24,
            max_duration_ms: 600_000,
            max_retries: 1,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentDagStep {
    pub id: String,
    pub agent_id: String,
    pub capability_id: String,
    pub dependencies: Vec<String>,
}
pub fn validate_agent_dag(steps: &[AgentDagStep]) -> Result<()> {
    if steps.is_empty() || steps.len() > MAX_TASKS {
        return Err(runtime_error("DAG_LIMIT", "Choose 1–12 unique tasks"));
    }
    let mut ids = BTreeSet::new();
    for s in steps {
        if s.id.is_empty()
            || s.id.len() > 40
            || !s
                .id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_'))
            || !ids.insert(s.id.clone())
        {
            return Err(runtime_error(
                "DAG_INVALID",
                "Task IDs must be unique, bounded ASCII identifiers",
            ));
        }
    }
    let mut done = BTreeSet::new();
    for s in steps {
        let unique: BTreeSet<_> = s.dependencies.iter().collect();
        if unique.len() != s.dependencies.len()
            || s.dependencies
                .iter()
                .any(|d| d == &s.id || !ids.contains(d))
        {
            return Err(runtime_error(
                "DAG_INVALID",
                "Dependencies must exist, be distinct and remain in this DAG",
            ));
        }
    }
    while done.len() < steps.len() {
        let before = done.len();
        for s in steps {
            if s.dependencies.iter().all(|d| done.contains(d)) {
                done.insert(s.id.clone());
            }
        }
        if done.len() == before {
            return Err(runtime_error(
                "DAG_CYCLE",
                "Dependency cycles are not executable",
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRuntimePreview {
    pub session_id: String,
    pub fingerprint: String,
    pub limits: AgentLimits,
    pub steps: Vec<AgentDagStep>,
    pub limitations: Vec<String>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentRuntimeReview {
    pub session_id: String,
    pub fingerprint: String,
    pub limits: AgentLimits,
    pub approved: bool,
}
#[derive(Debug, Serialize)]
pub struct AgentTask {
    pub id: String,
    pub agent_id: String,
    pub capability_id: String,
    pub state: String,
    pub job_id: String,
    pub attempt: u8,
    pub error_code: Option<String>,
    pub dependencies: Vec<String>,
}
#[derive(Debug, Serialize)]
pub struct AgentMessage {
    pub id: i64,
    pub session_id: String,
    pub task_id: Option<String>,
    pub agent_id: String,
    pub correlation_id: String,
    pub message_type: String,
    pub timestamp: String,
    pub payload: Value,
    pub evidence_ids: Vec<String>,
}
#[derive(Debug, Serialize)]
pub struct AgentRuntime {
    pub session_id: String,
    pub status: String,
    pub stop_reason: Option<String>,
    pub revision: i64,
    pub limits: AgentLimits,
    pub invocations: u64,
    pub deadline_ms: i64,
    pub tasks: Vec<AgentTask>,
    pub messages: Vec<AgentMessage>,
    pub next_message: Option<i64>,
    pub model_requests_by_runtime: u8,
    pub network_requests_by_runtime: u8,
    pub runtime_cost: Option<f64>,
}

fn agent_capability(cap: &str) -> Result<(&'static str, Option<JobKind>)> {
    Ok(match cap {
        "local.scope_review" => ("recon", Some(JobKind::TargetRegistrationReview)),
        "local.metadata" => ("reverse", Some(JobKind::BinaryMetadata)),
        "local.strings" => ("reverse", Some(JobKind::StringsExtraction)),
        "yara-x.signatures" => ("reverse", Some(JobKind::YaraSignatures)),
        "evidence.integrity" => ("validation", None),
        "report.snapshot" => ("reporting", None),
        _ => {
            return Err(runtime_error(
                "AGENT_POLICY",
                "This runtime supports reviewed offline analysis only; network/MCP/generated commands are not dispatched",
            ));
        }
    })
}

impl Core {
    pub fn agent_runtime_preview(
        &self,
        id: &str,
        limits: AgentLimits,
    ) -> Result<AgentRuntimePreview> {
        if !(1..=2).contains(&limits.concurrency)
            || !(1..=24).contains(&limits.max_invocations)
            || !(1000..=3_600_000).contains(&limits.max_duration_ms)
            || limits.max_retries > 2
        {
            return Err(runtime_error(
                "AGENT_BUDGET",
                "Concurrency 1–2, invocations 1–24, duration 1–3600 seconds, retries 0–2",
            ));
        }
        let s = self.research_session(id)?;
        let (owner, target): (String, Option<String>) = self
            .db
            .connect()?
            .query_row(
                "SELECT project_id,target_id FROM research_sessions WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(db_error)?;
        if s.plan.id != id
            || s.plan.preview.project_id != owner
            || target.as_deref() != Some(&s.plan.preview.request.target_id)
        {
            return Err(runtime_error(
                "AGENT_POLICY",
                "Stored plan identity does not match its session",
            ));
        }
        if s.approved_at.is_some() || s.cancelled_at.is_some() {
            return Err(runtime_error(
                "AGENT_STATE",
                "Create a new reviewed session before attaching a DAG",
            ));
        }
        let live = self.research_preview(s.plan.preview.request.clone())?;
        if live.fingerprint != s.plan.preview.fingerprint {
            return Err(runtime_error(
                "APPROVAL_STALE",
                "Target, policy, sample or model changed; prepare a new research plan",
            ));
        }
        let mut steps = vec![AgentDagStep {
            id: "scope".into(),
            agent_id: "recon".into(),
            capability_id: "local.scope_review".into(),
            dependencies: vec![],
        }];
        let mut selected = BTreeSet::new();
        for (i, step) in s.plan.proposal.steps.iter().enumerate() {
            let (agent, _) = agent_capability(&step.capability_id)?;
            if !selected.insert(&step.capability_id)
                || !live.capabilities.iter().any(|c| {
                    c.id == step.capability_id && c.available && c.network_requests == Some(0)
                })
            {
                return Err(runtime_error(
                    "AGENT_POLICY",
                    "Choose unique, actually available offline capabilities",
                ));
            }
            if step.capability_id == "local.scope_review" {
                continue;
            }
            steps.push(AgentDagStep {
                id: format!("analysis-{i}"),
                agent_id: agent.into(),
                capability_id: step.capability_id.clone(),
                dependencies: vec!["scope".into()],
            });
        }
        let dependencies = steps.iter().map(|s| s.id.clone()).collect();
        steps.push(AgentDagStep {
            id: "validate".into(),
            agent_id: "validation".into(),
            capability_id: "evidence.integrity".into(),
            dependencies,
        });
        steps.push(AgentDagStep {
            id: "report".into(),
            agent_id: "reporting".into(),
            capability_id: "report.snapshot".into(),
            dependencies: vec!["validate".into()],
        });
        validate_agent_dag(&steps)?;
        if usize::from(limits.max_invocations) < steps.len() {
            return Err(runtime_error(
                "AGENT_BUDGET",
                "Invocation budget cannot cover this reviewed DAG",
            ));
        }
        let fingerprint = evidence::digest(
            &serde_json::to_vec(&json!({"session":id,"plan":s.plan,"steps":steps,"limits":limits}))
                .map_err(json_error)?,
        );
        Ok(AgentRuntimePreview{session_id:id.into(),fingerprint,limits,steps,limitations:vec!["Offline workers only. No automatic network/MCP/AI requests or replanning".into(),"Pause drains current jobs; parsing/subprocess state is never suspended or restored".into(),"Wall-time includes pauses. Interrupted work requires explicit retry; completed work is never replayed".into(),"Token/cost budgets for cloud planning are not implemented; existing planning usage remains separately reported".into()]})
    }

    #[allow(clippy::too_many_arguments)]
    fn agent_message(
        tx: &Connection,
        session: &str,
        task: Option<&str>,
        agent: &str,
        correlation: &str,
        kind: AgentMessageType,
        payload: Value,
        evidence: &[String],
    ) -> Result<()> {
        let kind = kind.as_str();
        let exists:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM research_agent_messages WHERE session_id=?1 AND correlation_id=?2 AND message_type=?3)",params![session,correlation,kind],|r|r.get(0)).map_err(db_error)?;
        if exists {
            return Ok(());
        }
        let count: i64 = tx
            .query_row(
                "SELECT count(*) FROM research_agent_messages WHERE session_id=?1",
                [session],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        if count >= MAX_MESSAGES {
            // A full telemetry journal must never roll back cancellation or
            // permit another invocation. Authoritative state and SQL transition
            // history still persist even when no further message can be added.
            if !matches!(kind, "SESSION_CANCELLED" | "PAUSE_REQUESTED") {
                tx.execute("UPDATE research_runtimes SET status='failed',stop_reason='message_budget_exhausted',revision=revision+1 WHERE session_id=?1 AND status NOT IN ('cancelled','completed')",[session]).map_err(db_error)?;
            }
            return Ok(());
        }
        if payload.to_string().len() > 8192 || evidence.len() > 32 {
            return Err(runtime_error(
                "AGENT_BACKPRESSURE",
                "Durable event budget reached; no unbounded message queue is allocated",
            ));
        }
        tx.execute("INSERT INTO research_agent_messages(session_id,task_id,agent_id,correlation_id,message_type,created_at,payload_json,evidence_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![session,task,agent,correlation,kind,now(),payload.to_string(),serde_json::to_string(evidence).map_err(json_error)?]).map_err(db_error)?;
        Ok(())
    }
    fn enqueue_agent_task(
        &self,
        tx: &Connection,
        s: &ResearchSession,
        cap: &str,
    ) -> Result<String> {
        let (_, kind) = agent_capability(cap)?;
        let id = if let Some(kind) = kind {
            self.enqueue_job_in_tx(
                tx,
                JobInput {
                    kind,
                    file_id: if kind == JobKind::TargetRegistrationReview {
                        None
                    } else {
                        s.plan.preview.file_id.clone()
                    },
                    target_id: Some(s.plan.preview.request.target_id.clone()),
                    timeout_ms: 60_000,
                },
                None,
            )?
        } else {
            let id = Uuid::new_v4().to_string();
            let kind = if cap == "evidence.integrity" {
                "research_evidence_validation"
            } else {
                "research_report"
            };
            tx.execute("INSERT INTO analysis_jobs(id,project_id,target_id,kind,status,created_at,logs_json) VALUES(?1,?2,?3,?4,'queued',?5,'[]')",params![id,s.plan.preview.project_id,s.plan.preview.request.target_id,kind,now()]).map_err(db_error)?;
            id
        };
        let ordinal: i64 = tx
            .query_row(
                "SELECT coalesce(max(step)+1,0) FROM research_session_jobs WHERE session_id=?1",
                [&s.plan.id],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        tx.execute(
            "INSERT INTO research_session_jobs VALUES(?1,?2,?3)",
            params![s.plan.id, ordinal, id],
        )
        .map_err(db_error)?;
        Ok(id)
    }
    pub fn agent_runtime_start(&self, review: AgentRuntimeReview) -> Result<AgentRuntime> {
        if !review.approved {
            return Err(runtime_error(
                "APPROVAL_REQUIRED",
                "Approve the complete offline DAG and its resource limits",
            ));
        }
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let preview = self.agent_runtime_preview(&review.session_id, review.limits)?;
        if preview.fingerprint != review.fingerprint {
            return Err(runtime_error(
                "APPROVAL_STALE",
                "Review the current DAG and limits",
            ));
        }
        let s = self.research_session(&review.session_id)?;
        tx.execute("INSERT INTO research_runtimes(session_id,fingerprint,limits_json,status,created_at,deadline_ms) VALUES(?1,?2,?3,'running',?4,?5)",params![s.plan.id,preview.fingerprint,serde_json::to_string(&preview.limits).map_err(json_error)?,now(),Utc::now().timestamp_millis()+preview.limits.max_duration_ms as i64]).map_err(db_error)?;
        let assigned: BTreeSet<_> = preview
            .steps
            .iter()
            .map(|t| t.agent_id.as_str())
            .chain(["orchestrator", "planner"])
            .collect();
        for descriptor in research_agents()
            .into_iter()
            .filter(|a| assigned.contains(a.id.as_str()))
        {
            tx.execute(
                "INSERT INTO research_agent_assignments VALUES(?1,?2,?3)",
                params![
                    s.plan.id,
                    descriptor.id,
                    serde_json::to_string(&descriptor).map_err(json_error)?
                ],
            )
            .map_err(db_error)?;
        }
        for (i, step) in preview.steps.iter().enumerate() {
            let task = format!("{}:{}", s.plan.id, step.id);
            let job = self.enqueue_agent_task(&tx, &s, &step.capability_id)?;
            tx.execute("INSERT INTO research_agent_tasks(id,session_id,ordinal,agent_id,capability_id,state,job_id) VALUES(?1,?2,?3,?4,?5,'PENDING',?6)",params![task,s.plan.id,i as i64,step.agent_id,step.capability_id,job]).map_err(db_error)?;
            tx.execute(
                "INSERT INTO research_task_attempts VALUES(?1,0,?2)",
                params![task, job],
            )
            .map_err(db_error)?;
            Self::agent_message(
                &tx,
                &s.plan.id,
                Some(&task),
                &step.agent_id,
                &job,
                AgentMessageType::TaskAssigned,
                json!({"job_id":job,"capability_id":step.capability_id}),
                &[],
            )?;
            Self::agent_message(
                &tx,
                &s.plan.id,
                Some(&task),
                &step.agent_id,
                &job,
                AgentMessageType::ToolRequested,
                json!({"job_id":job,"capability_id":step.capability_id,"source":"typed_core_dispatch"}),
                &[],
            )?;
        }
        for step in &preview.steps {
            for dep in &step.dependencies {
                tx.execute(
                    "INSERT INTO research_task_dependencies VALUES(?1,?2,?3)",
                    params![
                        s.plan.id,
                        format!("{}:{}", s.plan.id, step.id),
                        format!("{}:{dep}", s.plan.id)
                    ],
                )
                .map_err(db_error)?;
            }
        }
        tx.execute(
            "UPDATE research_sessions SET approved_at=?1 WHERE id=?2",
            params![now(), s.plan.id],
        )
        .map_err(db_error)?;
        Self::agent_message(
            &tx,
            &s.plan.id,
            None,
            "planner",
            "reviewed-plan",
            AgentMessageType::PlanAccepted,
            json!({"fingerprint":preview.fingerprint,"source":"persisted_research_plan","task_count":preview.steps.len()}),
            &[],
        )?;
        Self::audit(
            &tx,
            "research.runtime.started",
            &s.plan.id,
            "reviewed_offline_dag",
        )?;
        tx.commit().map_err(db_error)?;
        self.agent_runtime(&s.plan.id, 0)
    }

    pub fn agent_runtime(&self, id: &str, after: i64) -> Result<AgentRuntime> {
        if after < 0 {
            return Err(runtime_error(
                "AGENT_LIMIT",
                "Message cursor cannot be negative",
            ));
        }
        self.reconcile_agent_runtimes_for(Some(id))?;
        let conn = self.db.connect()?;
        let (status,reason,revision,limits,invocations,deadline):(String,Option<String>,i64,String,i64,i64)=conn.query_row("SELECT status,stop_reason,revision,limits_json,invocations,deadline_ms FROM research_runtimes WHERE session_id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).optional().map_err(db_error)?.ok_or_else(||runtime_error("NOT_FOUND","This session has no offline agent runtime"))?;
        let mut tasks:Vec<AgentTask>=conn.prepare("SELECT id,agent_id,capability_id,state,job_id,attempt,error_code FROM research_agent_tasks WHERE session_id=?1 ORDER BY ordinal").map_err(db_error)?.query_map([id],|r|Ok(AgentTask{id:r.get(0)?,agent_id:r.get(1)?,capability_id:r.get(2)?,state:r.get(3)?,job_id:r.get(4)?,attempt:r.get(5)?,error_code:r.get(6)?,dependencies:vec![]})).map_err(db_error)?.collect::<std::result::Result<_,_>>().map_err(db_error)?;
        for t in &mut tasks {
            t.dependencies=conn.prepare("SELECT dependency_id FROM research_task_dependencies WHERE task_id=?1 ORDER BY dependency_id").map_err(db_error)?.query_map([&t.id],|r|r.get(0)).map_err(db_error)?.collect::<std::result::Result<_,_>>().map_err(db_error)?;
        }
        let messages:Vec<AgentMessage>=conn.prepare("SELECT id,task_id,agent_id,correlation_id,message_type,created_at,payload_json,evidence_json FROM research_agent_messages WHERE session_id=?1 AND id>?2 ORDER BY id LIMIT 101").map_err(db_error)?.query_map(params![id,after],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,Option<String>>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?,r.get::<_,String>(6)?,r.get::<_,String>(7)?))).map_err(db_error)?.map(|row|{let (n,task,agent,correlation,kind,stamp,payload,evidence)=row.map_err(db_error)?;Ok(AgentMessage{id:n,session_id:id.into(),task_id:task,agent_id:agent,correlation_id:correlation,message_type:kind,timestamp:stamp,payload:serde_json::from_str(&payload).map_err(json_error)?,evidence_ids:serde_json::from_str(&evidence).map_err(json_error)?})}).collect::<Result<_>>()?;
        let next_message = (messages.len() > 100).then(|| messages[99].id);
        Ok(AgentRuntime {
            session_id: id.into(),
            status,
            stop_reason: reason,
            revision,
            limits: serde_json::from_str(&limits).map_err(json_error)?,
            invocations: u64::try_from(invocations)
                .map_err(|_| runtime_error("AGENT_BUDGET", "Invalid invocation accounting"))?,
            deadline_ms: deadline,
            tasks,
            messages: messages.into_iter().take(100).collect(),
            next_message,
            model_requests_by_runtime: 0,
            network_requests_by_runtime: 0,
            runtime_cost: None,
        })
    }

    pub fn agent_runtime_pause(&self, id: &str) -> Result<AgentRuntime> {
        let mut c = self.db.connect()?;
        let tx = c
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        if tx.execute("UPDATE research_runtimes SET status='pause_requested',revision=revision+1,stop_reason='pause_at_job_boundary' WHERE session_id=?1 AND status='running'",[id]).map_err(db_error)?!=1 {
            return Err(runtime_error("AGENT_STATE","Only a running offline DAG can request a checkpoint pause"));
        }
        tx.execute("UPDATE research_agent_tasks SET state=CASE WHEN state='RUNNING' THEN 'PAUSE_REQUESTED' ELSE 'PAUSED' END WHERE session_id=?1 AND state IN ('PENDING','READY','RETRY_WAIT','RUNNING')",[id]).map_err(db_error)?;
        Self::agent_message(
            &tx,
            id,
            None,
            "orchestrator",
            &Uuid::new_v4().to_string(),
            AgentMessageType::PauseRequested,
            json!({"semantics":"finish current jobs; do not claim another job"}),
            &[],
        )?;
        tx.commit().map_err(db_error)?;
        self.agent_runtime(id, 0)
    }
    fn require_agent_policy(&self, s: &ResearchSession) -> Result<()> {
        let preview = self.research_preview(s.plan.preview.request.clone())?;
        if preview.fingerprint != s.plan.preview.fingerprint {
            return Err(runtime_error(
                "APPROVAL_STALE",
                "Authorization, sample or reviewed configuration changed; create a new plan",
            ));
        }
        Ok(())
    }
    pub fn agent_runtime_resume(&self, id: &str) -> Result<AgentRuntime> {
        self.reconcile_agent_runtimes_for(Some(id))?;
        let mut c = self.db.connect()?;
        let tx = c
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        self.require_agent_policy(&self.research_session(id)?)?;
        let failed:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM research_agent_tasks WHERE session_id=?1 AND state IN ('FAILED','BLOCKED','CANCELLED'))",[id],|r|r.get(0)).map_err(db_error)?;
        if failed {
            return Err(runtime_error(
                "RETRY_REQUIRED",
                "Review and explicitly retry incomplete read-only work; resume never replays a failed job",
            ));
        }
        if tx.execute("UPDATE research_runtimes SET status='running',stop_reason=NULL,revision=revision+1 WHERE session_id=?1 AND status='paused' AND deadline_ms>?2",params![id,Utc::now().timestamp_millis()]).map_err(db_error)?!=1 {return Err(runtime_error("AGENT_STATE","Only a paused DAG within its time budget can resume"));}
        tx.execute("UPDATE research_agent_tasks SET state='PENDING' WHERE session_id=?1 AND state='PAUSED'",[id]).map_err(db_error)?;
        Self::agent_message(
            &tx,
            id,
            None,
            "orchestrator",
            &Uuid::new_v4().to_string(),
            AgentMessageType::SessionResumed,
            json!({"policy_revalidated":true}),
            &[],
        )?;
        tx.commit().map_err(db_error)?;
        self.agent_runtime(id, 0)
    }
    pub fn agent_runtime_retry(&self, id: &str) -> Result<AgentRuntime> {
        let snapshot = self.agent_runtime(id, 0)?;
        if !matches!(snapshot.status.as_str(), "failed" | "paused") {
            return Err(runtime_error(
                "AGENT_STATE",
                "Stop running work before explicitly retrying incomplete offline tasks",
            ));
        }
        if snapshot.deadline_ms <= Utc::now().timestamp_millis() {
            return Err(runtime_error(
                "AGENT_BUDGET",
                "The original wall-time budget is exhausted",
            ));
        }
        if !snapshot
            .tasks
            .iter()
            .any(|t| matches!(t.state.as_str(), "FAILED" | "BLOCKED" | "CANCELLED"))
        {
            return Err(runtime_error(
                "AGENT_STATE",
                "Healthy paused work must resume rather than retry",
            ));
        }
        let mut c = self.db.connect()?;
        let tx = c
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let s = self.research_session(id)?;
        if s.cancelled_at.is_some() {
            return Err(runtime_error(
                "AGENT_STATE",
                "Cancelled sessions cannot be retried",
            ));
        }
        self.require_agent_policy(&s)?;
        let active:i64=tx.query_row("SELECT count(*) FROM analysis_jobs WHERE status='running' AND id IN(SELECT job_id FROM research_agent_tasks WHERE session_id=?1)",[id],|r|r.get(0)).map_err(db_error)?;
        if active != 0 {
            return Err(runtime_error(
                "AGENT_STATE",
                "Wait until running jobs have drained",
            ));
        }
        let pending: Vec<_> = snapshot
            .tasks
            .iter()
            .filter(|t| t.state != "COMPLETED")
            .collect();
        if pending.is_empty()
            || pending
                .iter()
                .any(|t| t.attempt >= snapshot.limits.max_retries)
            || snapshot.invocations + pending.len() as u64
                > u64::from(snapshot.limits.max_invocations)
        {
            return Err(runtime_error(
                "AGENT_BUDGET",
                "Retry or invocation budget cannot cover the remaining offline DAG",
            ));
        }
        // Fenced in this transaction; another client cannot concurrently retry.
        let revision: i64 = tx
            .query_row(
                "SELECT revision FROM research_runtimes WHERE session_id=?1",
                [id],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        if revision != snapshot.revision {
            return Err(runtime_error(
                "AGENT_STATE",
                "Runtime changed in another client; refresh before retry",
            ));
        }
        for task in pending {
            let job = self.enqueue_agent_task(&tx, &s, &task.capability_id)?;
            let attempt = task.attempt + 1;
            tx.execute("UPDATE research_agent_tasks SET job_id=?1,attempt=?2,state='RETRY_WAIT',retry_at_ms=?3,error_code=NULL WHERE id=?4",params![job,attempt,Utc::now().timestamp_millis()+250*(1_i64<<attempt),task.id]).map_err(db_error)?;
            tx.execute(
                "INSERT INTO research_task_attempts VALUES(?1,?2,?3)",
                params![task.id, attempt, job],
            )
            .map_err(db_error)?;
            Self::agent_message(
                &tx,
                id,
                Some(&task.id),
                &task.agent_id,
                &job,
                AgentMessageType::TaskAssigned,
                json!({"job_id":job,"explicit_retry":true,"attempt":attempt}),
                &[],
            )?;
        }
        tx.execute("UPDATE research_runtimes SET status='running',stop_reason=NULL,revision=revision+1 WHERE session_id=?1",[id]).map_err(db_error)?;
        tx.commit().map_err(db_error)?;
        self.agent_runtime(id, 0)
    }
    pub(super) fn cancel_agent_runtime(tx: &Connection, id: &str) -> Result<()> {
        if tx.execute("UPDATE research_runtimes SET status='cancelled',stop_reason='user_cancelled',revision=revision+1 WHERE session_id=?1 AND status NOT IN ('completed','cancelled')",[id]).map_err(db_error)?>0 {
            tx.execute("UPDATE research_agent_tasks SET state='CANCELLED' WHERE session_id=?1 AND state IN ('PENDING','READY','RETRY_WAIT','PAUSED','BLOCKED')",[id]).map_err(db_error)?;
            Self::agent_message(tx,id,None,"orchestrator","cancel",AgentMessageType::SessionCancelled,json!({"running_jobs":"cooperative cancellation requested"}),&[])?;
        }
        Ok(())
    }

    /// Reconcile metadata from real job states. No job/tool is executed here.
    pub(super) fn reconcile_agent_runtimes(&self) -> Result<()> {
        self.reconcile_agent_runtimes_for(None)
    }

    fn reconcile_agent_runtimes_for(&self, session: Option<&str>) -> Result<()> {
        let mut c = self.db.connect()?;
        let tx = c
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        // Stable paused/terminal history must not consume every sweep slot.
        // Current-attempt job changes take priority over otherwise idle running
        // sessions. A user read/resume always reconciles that exact session,
        // independent of the bounded background sweep.
        let sessions: Vec<(String, String, String, i64, i64)> = tx
            .prepare(
                r#"WITH candidates AS (
                  SELECT r.*,
                    EXISTS(SELECT 1 FROM research_agent_tasks t
                      JOIN analysis_jobs j ON j.id=t.job_id
                      WHERE t.session_id=r.session_id AND t.state<>'BLOCKED'
                      AND j.status IN ('completed','failed','cancelled')
                      AND t.state<>upper(j.status)) AS dirty,
                    EXISTS(SELECT 1 FROM research_agent_tasks t
                      JOIN analysis_jobs j ON j.id=t.job_id
                      WHERE t.session_id=r.session_id AND j.status IN ('queued','running')) AS outstanding
                  FROM research_runtimes r WHERE r.status<>'completed'
                    AND (?1 IS NULL OR r.session_id=?1)
                )
                SELECT session_id,status,limits_json,deadline_ms,invocations FROM candidates
                WHERE ?1 IS NOT NULL OR status IN ('running','pause_requested') OR dirty
                  OR (status='paused' AND deadline_ms<=?2)
                  OR (status IN ('failed','cancelled') AND outstanding)
                ORDER BY CASE
                    WHEN dirty OR (status IN ('failed','cancelled') AND outstanding) THEN 0
                    WHEN status='pause_requested' THEN 1
                    WHEN deadline_ms<=?2 OR (status='running'
                      AND invocations>=json_extract(limits_json,'$.max_invocations')) THEN 2
                    ELSE 3 END,created_at,session_id
                LIMIT 100"#,
            )
            .map_err(db_error)?
            .query_map(params![session, Utc::now().timestamp_millis()], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })
            .map_err(db_error)?
            .collect::<std::result::Result<_, _>>()
            .map_err(db_error)?;
        for (sid, mut status, limits, deadline, invocations) in sessions {
            let limits: AgentLimits = serde_json::from_str(&limits).map_err(json_error)?;
            let rows:Vec<(String,String,String,String,String,Option<String>)>=tx.prepare("SELECT t.id,t.agent_id,t.job_id,t.state,j.status,j.error_code FROM research_agent_tasks t JOIN analysis_jobs j ON j.id=t.job_id WHERE t.session_id=?1 ORDER BY t.ordinal").map_err(db_error)?.query_map([&sid],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).map_err(db_error)?.collect::<std::result::Result<_,_>>().map_err(db_error)?;
            let mut any_failed = false;
            let mut interrupted = false;
            let mut running = 0;
            for (tid, agent, job, old, actual, error) in &rows {
                if actual == "running" {
                    running += 1;
                }
                let terminal = match actual.as_str() {
                    "completed" => Some("COMPLETED"),
                    "failed" => Some("FAILED"),
                    "cancelled" => Some("CANCELLED"),
                    _ => None,
                };
                if let Some(state) = terminal {
                    if state != "COMPLETED" && old != "BLOCKED" {
                        any_failed = true;
                        interrupted |= error.as_deref() == Some("JOB_INTERRUPTED");
                    }
                    if old != state && old != "BLOCKED" {
                        tx.execute(
                            "UPDATE research_agent_tasks SET state=?1,error_code=?2 WHERE id=?3",
                            params![state, error, tid],
                        )
                        .map_err(db_error)?;
                        let evidence: Vec<String> = tx
                            .prepare("SELECT id FROM evidence WHERE job_id=?1 ORDER BY id LIMIT 32")
                            .map_err(db_error)?
                            .query_map([job], |r| r.get(0))
                            .map_err(db_error)?
                            .collect::<std::result::Result<_, _>>()
                            .map_err(db_error)?;
                        Self::agent_message(
                            &tx,
                            &sid,
                            Some(tid),
                            agent,
                            job,
                            if state == "COMPLETED" {
                                AgentMessageType::TaskCompleted
                            } else {
                                AgentMessageType::TaskFailed
                            },
                            json!({"job_id":job,"state":state,"error_code":error}),
                            &evidence,
                        )?;
                        if !evidence.is_empty() {
                            Self::agent_message(
                                &tx,
                                &sid,
                                Some(tid),
                                agent,
                                job,
                                AgentMessageType::EvidenceLinked,
                                json!({"job_id":job}),
                                &evidence,
                            )?;
                        }
                        tx.execute(
                            "UPDATE research_runtimes SET revision=revision+1 WHERE session_id=?1",
                            [&sid],
                        )
                        .map_err(db_error)?;
                    }
                }
            }
            let all_completed = rows
                .iter()
                .all(|(_, _, _, _, state, _)| state == "completed");
            let exhausted = deadline <= Utc::now().timestamp_millis() && !all_completed;
            if matches!(status.as_str(), "running" | "pause_requested" | "paused")
                && (any_failed || exhausted)
            {
                status = if interrupted && !exhausted {
                    "paused"
                } else {
                    "failed"
                }
                .into();
                let reason = if exhausted {
                    "time_budget_exhausted"
                } else if interrupted {
                    "interrupted_explicit_retry_required"
                } else {
                    "task_failed"
                };
                tx.execute("UPDATE research_runtimes SET status=?1,stop_reason=?2,revision=revision+1 WHERE session_id=?3 AND (status<>?1 OR stop_reason IS NOT ?2)",params![status,reason,sid]).map_err(db_error)?;
            }
            if matches!(status.as_str(), "failed" | "cancelled") || any_failed {
                tx.execute("UPDATE research_agent_tasks SET state='BLOCKED',error_code=coalesce(error_code,'DEPENDENCY_FAILED') WHERE session_id=?1 AND state IN ('PENDING','READY','RETRY_WAIT','PAUSED')",[&sid]).map_err(db_error)?;
                tx.execute("UPDATE analysis_jobs SET cancel_requested=1,status=CASE WHEN status='queued' THEN 'cancelled' ELSE status END,finished_at=CASE WHEN status='queued' THEN ?2 ELSE finished_at END,stage='Offline DAG stopped' WHERE id IN(SELECT job_id FROM research_agent_tasks WHERE session_id=?1) AND status IN ('queued','running')",params![sid,now()]).map_err(db_error)?;
                continue;
            }
            if rows
                .iter()
                .all(|(_, _, _, _, state, _)| state == "completed")
            {
                tx.execute("UPDATE research_runtimes SET status='completed',stop_reason='reviewed_dag_completed',revision=revision+1 WHERE session_id=?1",[&sid]).map_err(db_error)?;
                Self::agent_message(
                    &tx,
                    &sid,
                    None,
                    "orchestrator",
                    "done",
                    AgentMessageType::SessionCompleted,
                    json!({"reason":"all reviewed offline tasks completed","no_replanning":true}),
                    &[],
                )?;
            } else if status == "pause_requested" && running == 0 {
                tx.execute("UPDATE research_runtimes SET status='paused',revision=revision+1 WHERE session_id=?1",[&sid]).map_err(db_error)?;
            } else if status == "running" {
                let changed=tx.execute("UPDATE research_agent_tasks SET state='READY' WHERE session_id=?1 AND state IN ('PENDING','RETRY_WAIT') AND coalesce(retry_at_ms,0)<=?2 AND NOT EXISTS(SELECT 1 FROM research_task_dependencies d JOIN research_agent_tasks p ON p.id=d.dependency_id WHERE d.task_id=research_agent_tasks.id AND p.state<>'COMPLETED')",params![sid,Utc::now().timestamp_millis()]).map_err(db_error)?;
                if changed > 0 {
                    tx.execute(
                        "UPDATE research_runtimes SET revision=revision+1 WHERE session_id=?1",
                        [&sid],
                    )
                    .map_err(db_error)?;
                }
                let remaining:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM research_agent_tasks WHERE session_id=?1 AND state='READY')",[&sid],|r|r.get(0)).map_err(db_error)?;
                if remaining && invocations >= i64::from(limits.max_invocations) && running == 0 {
                    tx.execute("UPDATE research_runtimes SET status='failed',stop_reason='invocation_budget_exhausted',revision=revision+1 WHERE session_id=?1",[&sid]).map_err(db_error)?;
                }
            }
        }
        tx.commit().map_err(db_error)
    }

    pub(super) fn agent_job_claimed(tx: &Connection, id: &str) -> Result<()> {
        let task: Option<(String, String, String)> = tx
            .query_row(
                "SELECT session_id,id,agent_id FROM research_agent_tasks WHERE job_id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(db_error)?;
        if let Some((sid, tid, agent)) = task {
            tx.execute(
                "UPDATE research_agent_tasks SET state='RUNNING' WHERE id=?1 AND state='READY'",
                [&tid],
            )
            .map_err(db_error)?;
            tx.execute("UPDATE research_runtimes SET invocations=invocations+1,revision=revision+1 WHERE session_id=?1",[&sid]).map_err(db_error)?;
            Self::agent_message(
                tx,
                &sid,
                Some(&tid),
                &agent,
                id,
                AgentMessageType::TaskStarted,
                json!({"job_id":id}),
                &[],
            )?;
        }
        Ok(())
    }
    pub(super) fn agent_job_checkpoint(&self, job: &AnalysisJob) -> Result<()> {
        let context:Option<(String,String,String,String,i64)>=self.db.connect()?.query_row("SELECT t.session_id,t.agent_id,t.capability_id,r.status,r.deadline_ms FROM research_agent_tasks t JOIN research_runtimes r ON r.session_id=t.session_id WHERE t.job_id=?1",[&job.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional().map_err(db_error)?;
        if let Some((sid, agent, cap, status, deadline)) = context {
            if !matches!(status.as_str(), "running" | "pause_requested") {
                return Err(runtime_error("CANCELLED", "Offline DAG has stopped"));
            }
            if Utc::now().timestamp_millis() >= deadline {
                return Err(runtime_error(
                    "AGENT_BUDGET",
                    "Offline DAG wall-time budget exhausted",
                ));
            }
            let (expected, kind) = agent_capability(&cap)?;
            let kind = kind
                .map(JobKind::as_str)
                .unwrap_or(if cap == "evidence.integrity" {
                    "research_evidence_validation"
                } else {
                    "research_report"
                });
            // Do not deserialize every completed artifact at each streaming-file
            // checkpoint. Read only the immutable session identity here.
            let (project,target_id):(String,String)=self.db.connect()?.query_row("SELECT project_id,json_extract(plan_json,'$.preview.request.target_id') FROM research_sessions WHERE id=?1",[&sid],|r|Ok((r.get(0)?,r.get(1)?))).map_err(db_error)?;
            let target = self.target(&target_id)?;
            let scope = self.validate_scope(&target.id, None)?;
            if expected != agent
                || kind != job.kind
                || target.input.project_id != job.project_id
                || project != job.project_id
                || job.target_id.as_deref() != Some(&target.id)
                || !scope.authorized
                || scope.status != "allowed"
                || self.project(&job.project_id)?.archived
            {
                return Err(runtime_error(
                    "AGENT_POLICY",
                    "Task, worker, scope or project no longer matches the reviewed offline session",
                ));
            }
        }
        Ok(())
    }
    pub(super) fn agent_job_progress(&self, job: &AnalysisJob, progress: u8) -> Result<()> {
        if !matches!(progress, 5 | 95) {
            return Ok(());
        }
        let mut c = self.db.connect()?;
        let tx = c
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let task: Option<(String, String, String)> = tx
            .query_row(
                "SELECT session_id,id,agent_id FROM research_agent_tasks WHERE job_id=?1",
                [&job.id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(db_error)?;
        if let Some((sid, tid, agent)) = task {
            Self::agent_message(
                &tx,
                &sid,
                Some(&tid),
                &agent,
                &format!("{}:progress:{progress}", job.id),
                AgentMessageType::TaskProgress,
                json!({"job_id":job.id,"progress":progress}),
                &[],
            )?;
        }
        tx.commit().map_err(db_error)
    }
    pub(super) fn prepare_agent_job(
        &self,
        job: &AnalysisJob,
        update: &mut dyn FnMut(u8, &str) -> Result<()>,
    ) -> Result<jobs::PreparedJob> {
        self.agent_job_checkpoint(job)?;
        let sid: String = self
            .db
            .connect()?
            .query_row(
                "SELECT session_id FROM research_agent_tasks WHERE job_id=?1",
                [&job.id],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        let s = self.research_session(&sid)?;
        let evidence: Vec<_> = s
            .evidence
            .iter()
            .filter(|e| e.job_id.as_deref() != Some(&job.id))
            .collect();
        if evidence.is_empty() || evidence.len() > 32 {
            return Err(runtime_error(
                "EVIDENCE_REQUIRED",
                "Select 1–32 real dependency evidence records",
            ));
        }
        let mut facts = Vec::new();
        for (i, e) in evidence.iter().enumerate() {
            update(
                (10 + i * 70 / evidence.len()) as u8,
                "Verifying real dependency evidence hashes",
            )?;
            let _ = self.evidence_bytes(e)?;
            facts.push(json!({"id":e.id,"sha256":e.sha256,"integrity":"matched"}));
        }
        Ok(jobs::PreparedJob {
            result: json!({"session_id":sid,"evidence":facts,"claim":"Artifact integrity only; no vulnerability is verified","kind":job.kind}),
            analysis: None,
            http: None,
        })
    }
    pub(super) fn commit_agent_report(
        &self,
        tx: &Connection,
        job: &AnalysisJob,
        result: &mut Value,
    ) -> Result<()> {
        let sid:String=tx.query_row("SELECT session_id FROM research_agent_tasks WHERE job_id=?1 AND agent_id='reporting' AND capability_id='report.snapshot'",[&job.id],|r|r.get(0)).map_err(db_error)?;
        let s = self.research_session(&sid)?;
        let report=self.build_report(ReportInput {
            project_id:job.project_id.clone(),title:format!("Offline agent session {}",&sid[..8]),template:ReportTemplate::Research,
            summary:"Read-only offline coordination over real artifacts. Integrity checks do not verify vulnerabilities. No autonomous network assessment was performed.".into(),
            methodology:"Reviewed DAG → offline scope → independent file jobs → evidence hash validation → report snapshot. No model/tool output can add tasks.".into(),
            finding_ids:vec![],evidence_ids:s.evidence.iter().map(|e|e.id.clone()).collect(),
        })?;
        self.persist_report_in_tx(tx, &report)?;
        tx.execute(
            "INSERT INTO research_session_records VALUES(?1,?2,'report',?3)",
            params![sid, report.report.id, now()],
        )
        .map_err(db_error)?;
        result["report_id"] = json!(report.report.id);
        result["report_sha256"] = json!(report.report.sha256);
        Ok(())
    }
}
