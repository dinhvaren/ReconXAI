//! Typed local jobs. SQLite is the queue and concurrency authority across clients.
use super::*;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

const MAX_RUNNING: i64 = 2;
const LEASE_MS: i64 = 10_000;
const JOB_COLUMNS: &str = "id,project_id,target_id,file_id,kind,status,created_at,started_at,finished_at,timeout_ms,progress,stage,error_code,error_message,cancel_requested,result_json,logs_json";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    LocalStaticBinary,
    BinaryMetadata,
    StringsExtraction,
    ArtifactIndex,
    YaraSignatures,
    TargetRegistrationReview,
}
impl JobKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalStaticBinary => "local_static_binary",
            Self::BinaryMetadata => "binary_metadata",
            Self::StringsExtraction => "strings_extraction",
            Self::ArtifactIndex => "artifact_index",
            Self::YaraSignatures => "yara_signatures",
            Self::TargetRegistrationReview => "target_registration_review",
        }
    }
}
fn default_timeout() -> u64 {
    60_000
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobInput {
    pub kind: JobKind,
    pub file_id: Option<String>,
    pub target_id: Option<String>,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisJob {
    pub id: String,
    pub project_id: String,
    pub target_id: Option<String>,
    pub file_id: Option<String>,
    pub kind: String,
    pub status: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub timeout_ms: u64,
    pub progress: u8,
    pub stage: String,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub cancel_requested: bool,
    pub result: Option<serde_json::Value>,
    pub logs: Vec<String>,
}
pub(super) struct PreparedJob {
    pub result: serde_json::Value,
    pub analysis: Option<(AnalysisRecord, String, String)>,
    pub http: Option<super::recon::HttpObservation>,
}
fn decode_job(
    r: &reconx_storage::rusqlite::Row<'_>,
) -> reconx_storage::rusqlite::Result<AnalysisJob> {
    let parse = |column| -> reconx_storage::rusqlite::Result<serde_json::Value> {
        serde_json::from_str(&r.get::<_, String>(column)?).map_err(|e| {
            reconx_storage::rusqlite::Error::FromSqlConversionFailure(
                column,
                reconx_storage::rusqlite::types::Type::Text,
                Box::new(e),
            )
        })
    };
    let logs: Vec<String> = serde_json::from_value(parse(16)?).map_err(|e| {
        reconx_storage::rusqlite::Error::FromSqlConversionFailure(
            16,
            reconx_storage::rusqlite::types::Type::Text,
            Box::new(e),
        )
    })?;
    let result = if r.get::<_, Option<String>>(15)?.is_some() {
        Some(parse(15)?)
    } else {
        None
    };
    Ok(AnalysisJob {
        id: r.get(0)?,
        project_id: r.get(1)?,
        target_id: r.get(2)?,
        file_id: r.get(3)?,
        kind: r.get(4)?,
        status: r.get(5)?,
        created_at: r.get(6)?,
        started_at: r.get(7)?,
        finished_at: r.get(8)?,
        timeout_ms: r.get::<_, u32>(9)? as u64,
        progress: r.get(10)?,
        stage: r.get(11)?,
        error_code: r.get(12)?,
        error_message: r.get(13)?,
        cancel_requested: r.get(14)?,
        result,
        logs,
    })
}
impl Core {
    pub fn enqueue_job(&self, input: JobInput) -> Result<AnalysisJob> {
        if !(1..=3_600_000).contains(&input.timeout_ms) {
            return Err(AppError::new(
                "INVALID_INPUT",
                "Job timeout must be between 1 and 3600000 ms",
            ));
        }
        let (project_id, target_id, file_id) = if input.kind == JobKind::TargetRegistrationReview {
            if input.file_id.is_some() {
                return Err(AppError::new(
                    "INVALID_JOB",
                    "Scope review accepts a target, not a file",
                ));
            }
            let target = self.target(input.target_id.as_deref().ok_or_else(|| {
                AppError::new("INVALID_JOB", "Choose a target for scope review")
            })?)?;
            (target.input.project_id, Some(target.id), None)
        } else {
            let file =
                self.imported_file(input.file_id.as_deref().ok_or_else(|| {
                    AppError::new("INVALID_JOB", "Choose an imported local file")
                })?)?;
            self.require_file_authorization(&file)?;
            if input.target_id.is_some() && input.target_id != file.target_id {
                return Err(AppError::new(
                    "INVALID_JOB",
                    "Target does not match the imported file",
                ));
            }
            (file.project_id, file.target_id, Some(file.id))
        };
        if self.project(&project_id)?.archived {
            return Err(AppError::new(
                "PROJECT_ARCHIVED",
                "Restore the project before scheduling research",
            ));
        }
        let id = Uuid::new_v4().to_string();
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute("INSERT INTO analysis_jobs(id,project_id,target_id,file_id,kind,status,created_at,timeout_ms,logs_json) VALUES(?1,?2,?3,?4,?5,'queued',?6,?7,?8)",params![id,project_id,target_id,file_id,input.kind.as_str(),now(),input.timeout_ms as u32,serde_json::to_string(&vec!["Queued for bounded read-only local execution"]).map_err(json_error)?]).map_err(db_error)?;
        Self::audit(&tx, "job.queued", &id, "queued")?;
        tx.commit().map_err(db_error)?;
        self.job(&id)
    }
    pub fn list_jobs(&self, project: Option<&str>) -> Result<Vec<AnalysisJob>> {
        let project = project.map(|p| self.project(p).map(|p| p.id)).transpose()?;
        self.db.connect()?.prepare(&format!("SELECT {JOB_COLUMNS} FROM analysis_jobs WHERE ?1 IS NULL OR project_id=?1 ORDER BY created_at DESC,id")).map_err(db_error)?
            .query_map([project],decode_job).map_err(db_error)?.collect::<std::result::Result<Vec<_>,_>>().map_err(db_error)
    }
    pub fn job(&self, id: &str) -> Result<AnalysisJob> {
        self.db
            .connect()?
            .query_row(
                &format!("SELECT {JOB_COLUMNS} FROM analysis_jobs WHERE id=?1"),
                [id],
                decode_job,
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| AppError::new("NOT_FOUND", "Job does not exist"))
    }
    pub fn cancel_job(&self, id: &str) -> Result<()> {
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let job = tx
            .query_row(
                &format!("SELECT {JOB_COLUMNS} FROM analysis_jobs WHERE id=?1"),
                [id],
                decode_job,
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| AppError::new("NOT_FOUND", "Job does not exist"))?;
        if !matches!(job.status.as_str(), "queued" | "running") {
            return Err(AppError::new(
                "JOB_FINISHED",
                "Only queued or running jobs can be cancelled",
            ));
        }
        // Running work retains its capacity slot until its worker acknowledges cancellation.
        tx.execute("UPDATE analysis_jobs SET cancel_requested=1,status=CASE WHEN status='queued' THEN 'cancelled' ELSE status END,finished_at=CASE WHEN status='queued' THEN ?1 ELSE finished_at END,stage=CASE WHEN status='queued' THEN 'Cancelled' ELSE 'Cancellation requested' END,logs_json=json_insert(logs_json,'$[#]',?2) WHERE id=?3",params![now(),"Cancellation requested by a user",id]).map_err(db_error)?;
        Self::audit(&tx, "job.cancel_requested", id, "requested")?;
        tx.commit().map_err(db_error)
    }
    /// Only expired or legacy ownerless RUNNING jobs are reconciled. CLI reads and
    /// another Desktop instance never invalidate a live worker's lease.
    pub fn reconcile_jobs(&self) -> Result<usize> {
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let changed=tx.execute("UPDATE analysis_jobs SET status=CASE WHEN cancel_requested=1 THEN 'cancelled' ELSE 'failed' END,error_code=CASE WHEN cancel_requested=1 THEN 'CANCELLED' ELSE 'JOB_INTERRUPTED' END,error_message='Worker stopped before a durable result was committed',stage='Worker interrupted',finished_at=?1,worker_id=NULL,lease_until=NULL,logs_json=json_insert(logs_json,'$[#]','Expired worker lease reconciled; no automatic retry') WHERE status='running' AND (worker_id IS NULL OR lease_until IS NULL OR lease_until<?2)",params![now(),Utc::now().timestamp_millis()]).map_err(db_error)?;
        if changed > 0 {
            Self::audit(&tx, "job.recovered", "workspace", "interrupted")?;
        }
        tx.commit().map_err(db_error)?;
        Ok(changed)
    }
    fn claim_job(&self, id: Option<&str>, owner: &str) -> Result<Option<AnalysisJob>> {
        self.reconcile_jobs()?;
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let active: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM analysis_jobs WHERE status='running'",
                [],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        if active >= MAX_RUNNING {
            return Ok(None);
        }
        let next:Option<String>=tx.query_row("SELECT id FROM analysis_jobs WHERE status='queued' AND (?1 IS NULL OR id=?1) ORDER BY created_at,id LIMIT 1",[id],|r|r.get(0)).optional().map_err(db_error)?;
        let Some(next) = next else { return Ok(None) };
        tx.execute("UPDATE analysis_jobs SET status='running',started_at=?1,worker_id=?2,lease_until=?3,stage='Policy validation',progress=1,logs_json=json_insert(logs_json,'$[#]','Worker claimed job; validating current policy') WHERE id=?4 AND status='queued'",params![now(),owner,Utc::now().timestamp_millis()+LEASE_MS,next]).map_err(db_error)?;
        Self::audit(&tx, "job.started", &next, "running")?;
        tx.commit().map_err(db_error)?;
        self.job(&next).map(Some)
    }
    fn heartbeat(&self, id: &str, owner: &str) -> Result<()> {
        self.db.connect()?.execute("UPDATE analysis_jobs SET lease_until=?1 WHERE id=?2 AND worker_id=?3 AND status='running'",params![Utc::now().timestamp_millis()+LEASE_MS,id,owner]).map_err(db_error)?;
        Ok(())
    }
    fn checkpoint(
        &self,
        job: &AnalysisJob,
        owner: &str,
        stop: &AtomicBool,
        progress: u8,
        message: &str,
    ) -> Result<()> {
        if stop.load(Ordering::Acquire) {
            return Err(AppError::new("JOB_INTERRUPTED", "Worker is shutting down"));
        }
        let current = self.job(&job.id)?;
        if current.status != "running" || current.cancel_requested {
            return Err(AppError::new("CANCELLED", "Local work was cancelled"));
        }
        let started = chrono::DateTime::parse_from_rfc3339(
            job.started_at.as_deref().unwrap_or(&job.created_at),
        )
        .map_err(|_| AppError::new("INVALID_JOB", "Invalid job start time"))?;
        if Utc::now().timestamp_millis() - started.timestamp_millis() >= job.timeout_ms as i64 {
            return Err(AppError::new(
                "JOB_TIMEOUT",
                "Configured execution timeout expired",
            ));
        }
        if let Some(file) = &job.file_id {
            self.require_file_authorization(&self.imported_file(file)?)?;
        }
        if job.kind == "local_http_inventory" {
            self.local_http_job_plan(job)?;
        }
        if self.project(&job.project_id)?.archived {
            return Err(AppError::new(
                "PROJECT_ARCHIVED",
                "Project was archived during execution",
            ));
        }
        let changed=self.db.connect()?.execute("UPDATE analysis_jobs SET progress=max(progress,?1),stage=?2,logs_json=CASE WHEN stage<>?2 THEN json_insert(logs_json,'$[#]',?2) ELSE logs_json END WHERE id=?3 AND worker_id=?4 AND status='running' AND cancel_requested=0",params![progress,message,job.id,owner]).map_err(db_error)?;
        if changed != 1 {
            return Err(AppError::new("CANCELLED", "Job ownership or state changed"));
        }
        Ok(())
    }
    fn complete_job(
        &self,
        job: &AnalysisJob,
        owner: &str,
        mut prepared: PreparedJob,
    ) -> Result<()> {
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let started = chrono::DateTime::parse_from_rfc3339(
            job.started_at.as_deref().unwrap_or(&job.created_at),
        )
        .map_err(|_| AppError::new("INVALID_JOB", "Invalid job timestamp"))?;
        if Utc::now().timestamp_millis() - started.timestamp_millis() >= job.timeout_ms as i64 {
            return Err(AppError::new(
                "JOB_TIMEOUT",
                "Timeout expired before result commit",
            ));
        }
        if let Some(http) = prepared.http.as_ref() {
            prepared.result = self.commit_local_http(&tx, job, http)?;
        }
        let changed=tx.execute("UPDATE analysis_jobs SET status='completed',progress=100,stage='Completed',finished_at=?1,result_json=?2,worker_id=NULL,lease_until=NULL,logs_json=json_insert(logs_json,'$[#]','Result committed') WHERE id=?3 AND status='running' AND worker_id=?4 AND cancel_requested=0",params![now(),serde_json::to_string(&prepared.result).map_err(json_error)?,job.id,owner]).map_err(db_error)?;
        if changed != 1 {
            return Err(AppError::new(
                "CANCELLED",
                "Job cancelled or lease lost before result commit",
            ));
        }
        self.record_job_evidence(
            &tx,
            job,
            &prepared.result,
            prepared.analysis.as_ref().map(|a| &a.0),
        )?;
        if let Some((record, name, digest)) = prepared.analysis {
            tx.execute("INSERT INTO binary_analyses(id,file_id,job_id,created_at,artifact_name,artifact_sha256) VALUES(?1,?2,?3,?4,?5,?6)",params![record.id,record.file_id,record.job_id,record.created_at,name,digest]).map_err(db_error)?;
            if let Some(target) = &job.target_id {
                tx.execute(
                    "UPDATE targets SET last_analyzed_at=?1 WHERE id=?2",
                    params![now(), target],
                )
                .map_err(db_error)?;
            }
        }
        Self::audit(&tx, "job.completed", &job.id, "completed")?;
        tx.commit().map_err(db_error)
    }
    fn fail_job(&self, id: &str, owner: &str, error: &AppError) -> Result<()> {
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute("UPDATE analysis_jobs SET status=CASE WHEN cancel_requested=1 OR ?1='CANCELLED' THEN 'cancelled' ELSE 'failed' END,error_code=?1,error_message=?2,stage=?2,finished_at=?3,worker_id=NULL,lease_until=NULL,logs_json=json_insert(logs_json,'$[#]',?2) WHERE id=?4 AND worker_id=?5 AND status='running'",params![error.code,error.message,now(),id,owner]).map_err(db_error)?;
        Self::audit(&tx, "job.stopped", id, &error.code)?;
        tx.commit().map_err(db_error)
    }
    fn execute_claimed(
        &self,
        job: AnalysisJob,
        owner: &str,
        stop: &AtomicBool,
        emit: &(dyn Fn(AnalysisJob) + Send + Sync),
    ) -> Result<AnalysisJob> {
        let (shutdown, rx) = mpsc::channel::<()>();
        let pulse_core = self.clone();
        let pulse_id = job.id.clone();
        let pulse_owner = owner.to_owned();
        let pulse = thread::Builder::new()
            .name("reconx-job-lease".into())
            .spawn(move || {
                while rx
                    .recv_timeout(Duration::from_millis(500))
                    .is_err_and(|e| e == mpsc::RecvTimeoutError::Timeout)
                {
                    if pulse_core.heartbeat(&pulse_id, &pulse_owner).is_err() {
                        break;
                    }
                }
            })
            .map_err(|_| AppError::new("WORKER_ERROR", "Cannot start job heartbeat"))?;
        emit(job.clone());
        let outcome=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{
            let mut update=|progress,message:&str| {self.checkpoint(&job,owner,stop,progress,message)?;emit(self.job(&job.id)?);Ok(())};
            update(5,"Authorization and scope checked")?;
            let prepared=match job.kind.as_str() {
                "local_static_binary"|"binary_metadata"|"strings_extraction"=>self.prepare_file_analysis(&job,&mut update)?,
                "target_registration_review"=>{
                    let id=job.target_id.as_deref().ok_or_else(||AppError::new("NOT_FOUND","Job target was removed"))?;
                    let decision=self.validate_scope(id,None)?;
                    update(90,"Registration, authorization and exclusions reviewed locally; no network requests")?;
                    PreparedJob{result:serde_json::to_value(decision).map_err(json_error)?,analysis:None,http:None}
                },
                "yara_signatures"=>{
                    let file=self.imported_file(job.file_id.as_deref().ok_or_else(||AppError::new("NOT_FOUND","File was removed"))?)?;
                    let bytes=self.read_sample_with_progress(&file,&mut |done,total|update((10+done*40/total.max(1)) as u8,"Reading verified sample for YARA-X"))?;
                    update(60,"Executing fixed offline YARA-X signatures")?;
                    let result=self.tool_manager().signatures(&bytes,||update(60,"Executing fixed offline YARA-X signatures"))?;
                    PreparedJob{result,analysis:None,http:None}
                },
                "local_http_inventory"=>self.prepare_local_http(&job,&mut update)?,
                "mcp_tool"=>self.prepare_mcp_job(&job,&mut update)?,
                "artifact_index"=>{
                    let id=job.file_id.as_deref().ok_or_else(||AppError::new("NOT_FOUND","Job file was removed"))?;
                    let file=self.imported_file(id)?;
                    self.read_sample_with_progress(&file,&mut |done,total|update((10+done*60/total.max(1)) as u8,"Indexing verified local sample bytes"))?;
                    let record=match self.file_analysis(id){Ok(r)=>Some(r),Err(e) if e.code=="NOT_ANALYZED"=>None,Err(e)=>return Err(e)};
                    update(90,"Sample and available analysis artifact indexed")?;
                    PreparedJob{result:serde_json::json!({"file_id":id,"sha256":file.sha256,"size":file.size,"format":file.format,"analysis_id":record.as_ref().map(|r|&r.id),"sections":record.as_ref().map(|r|r.analysis.sections.len()),"strings":record.as_ref().map(|r|r.analysis.strings.len())}),analysis:None,http:None}
                },
                _=>return Err(AppError::new("INVALID_JOB","No allowlisted handler for this job type")),
            };
            update(95,"Committing local result")?;
            self.complete_job(&job,owner,prepared)
        })).unwrap_or_else(|_|Err(AppError::new("WORKER_PANIC","Local worker rejected an unexpected parser failure")));
        let _ = shutdown.send(());
        let _ = pulse.join();
        if let Err(error) = outcome {
            self.fail_job(&job.id, owner, &error)?;
        }
        let finished = self.job(&job.id)?;
        emit(finished.clone());
        Ok(finished)
    }
    /// Execute one queued job synchronously using the same fenced worker as Desktop.
    pub fn execute_job(&self, id: &str) -> Result<AnalysisJob> {
        let current = self.job(id)?;
        if current.status != "queued" {
            return Ok(current);
        }
        let owner = Uuid::new_v4().to_string();
        let job = self.claim_job(Some(id), &owner)?.ok_or_else(|| {
            AppError::new(
                "JOB_BUSY",
                "Two jobs already run or another worker claimed this job",
            )
        })?;
        self.execute_claimed(job, &owner, &AtomicBool::new(false), &|_| {})
    }
}

/// Drop stops claims and cooperatively interrupts owned work. A killed process is
/// reconciled through expiring leases; jobs are never blindly retried.
pub struct JobEngine {
    stop: Arc<AtomicBool>,
    workers: Vec<JoinHandle<()>>,
}
impl JobEngine {
    pub fn start(
        core: Core,
        concurrency: usize,
        emit: impl Fn(AnalysisJob) + Send + Sync + 'static,
    ) -> Result<Self> {
        if !(1..=MAX_RUNNING as usize).contains(&concurrency) {
            return Err(AppError::new(
                "INVALID_INPUT",
                "Local worker concurrency must be 1 or 2",
            ));
        }
        core.reconcile_jobs()?;
        let stop = Arc::new(AtomicBool::new(false));
        let emit = Arc::new(emit);
        let mut engine = Self {
            stop: stop.clone(),
            workers: Vec::new(),
        };
        for _ in 0..concurrency {
            let core = core.clone();
            let stop = stop.clone();
            let emit = emit.clone();
            let handle = thread::Builder::new()
                .name("reconx-local-worker".into())
                .spawn(move || {
                    let owner = Uuid::new_v4().to_string();
                    while !stop.load(Ordering::Acquire) {
                        match core.claim_job(None, &owner) {
                            Ok(Some(job)) => {
                                if let Err(e) =
                                    core.execute_claimed(job, &owner, &stop, emit.as_ref())
                                {
                                    tracing_error(&e);
                                }
                            }
                            Ok(None) => thread::sleep(Duration::from_millis(100)),
                            Err(e) => {
                                tracing_error(&e);
                                thread::sleep(Duration::from_millis(500));
                            }
                        }
                    }
                })
                .map_err(|_| AppError::new("WORKER_ERROR", "Cannot create bounded local worker"))?;
            engine.workers.push(handle);
        }
        Ok(engine)
    }
}
fn tracing_error(error: &AppError) {
    eprintln!("Local job worker: {}", error.code);
}
impl Drop for JobEngine {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}
