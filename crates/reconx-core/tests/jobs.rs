use reconx_core::{
    ArtifactReview, Core, JobEngine, JobInput, JobKind, SecurityMode, TargetInput, TargetKind,
};
use reconx_storage::rusqlite::{Connection, params};
use std::{
    fs,
    sync::{Arc, Condvar, Mutex, mpsc},
    time::{Duration, Instant},
};

fn setup() -> (tempfile::TempDir, Core, String) {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::open(Some(dir.path().join("jobs.db"))).unwrap();
    let project = core.create_project("owned jobs", "").unwrap();
    let path = dir.path().join("owned.bin");
    fs::write(&path, b"owned research strings\0hello\0world").unwrap();
    let file = core.import_file(&project.id, &path, true).unwrap();
    (dir, core, file.id)
}
fn input(file: &str, kind: JobKind) -> JobInput {
    JobInput {
        kind,
        file_id: Some(file.into()),
        target_id: None,
        timeout_ms: 60_000,
    }
}
fn wait_done(core: &Core, id: &str) -> reconx_core::AnalysisJob {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let job = core.job(id).unwrap();
        if !matches!(job.status.as_str(), "queued" | "running") {
            return job;
        }
        assert!(Instant::now() < deadline, "job failed to finish: {job:?}");
        std::thread::sleep(Duration::from_millis(20));
    }
}
#[test]
fn typed_local_handlers_persist_results_logs_and_notes_across_restart() {
    let (dir, core, file) = setup();
    for kind in [
        JobKind::LocalStaticBinary,
        JobKind::BinaryMetadata,
        JobKind::StringsExtraction,
        JobKind::ArtifactIndex,
    ] {
        let queued = core.enqueue_job(input(&file, kind)).unwrap();
        assert_eq!(queued.status, "queued");
        assert!(queued.started_at.is_none());
        let done = core.execute_job(&queued.id).unwrap();
        assert_eq!(done.status, "completed", "{done:?}");
        assert_eq!(done.progress, 100);
        assert!(done.started_at.is_some());
        assert!(done.finished_at.is_some());
        assert!(done.result.is_some());
        assert!(done.logs.len() > 3);
        if kind != JobKind::ArtifactIndex {
            assert_eq!(core.analysis_for_job(&done.id).unwrap().job_id, done.id);
        }
    }
    core.save_file_note(&file, "Nghiên cứu — locally retained notes")
        .unwrap();
    let reopened = Core::open(Some(dir.path().join("jobs.db"))).unwrap();
    assert_eq!(
        reopened.file_note(&file).unwrap(),
        "Nghiên cứu — locally retained notes"
    );
    assert_eq!(reopened.list_jobs(None).unwrap().len(), 4);
    assert!(
        reopened
            .file_artifact_preview(&file, "summary")
            .unwrap()
            .contains("SHA-256")
    );
    assert!(
        !reopened
            .file_artifact_preview(&file, "summary")
            .unwrap()
            .contains("locally retained notes")
    );
    assert!(
        !reopened
            .file_artifact_preview(&file, "summary")
            .unwrap()
            .contains(&dir.path().to_string_lossy().to_string())
    );
}
#[test]
fn scope_review_is_local_and_registration_does_not_require_an_analyzer() {
    let (_dir, core, _file) = setup();
    let project = core.list_projects(false).unwrap()[0].id.clone();
    let target = core
        .create_target(TargetInput {
            project_id: project,
            name: "registered domain".into(),
            kind: TargetKind::Domain,
            value: "example.test".into(),
            description: String::new(),
            tags: vec![],
            allowed_scope: vec![],
            excluded_scope: vec![],
            authorized: false,
            mode: SecurityMode::PassiveAuto,
        })
        .unwrap();
    assert_eq!(
        core.create_target(target.input.clone()).unwrap_err().code,
        "DUPLICATE_TARGET"
    );
    let job = core
        .enqueue_job(JobInput {
            kind: JobKind::TargetRegistrationReview,
            file_id: None,
            target_id: Some(target.id.clone()),
            timeout_ms: 60_000,
        })
        .unwrap();
    let done = core.execute_job(&job.id).unwrap();
    assert_eq!(done.status, "completed");
    assert_eq!(done.result.unwrap()["status"], "awaiting_authorization");
    core.remove_target(&target.id).unwrap();
    assert!(core.job(&job.id).unwrap().target_id.is_none());
}
#[test]
fn queued_cancellation_and_integrity_failure_cannot_commit_results() {
    let (dir, core, file) = setup();
    let queued = core
        .enqueue_job(input(&file, JobKind::LocalStaticBinary))
        .unwrap();
    core.cancel_job(&queued.id).unwrap();
    assert_eq!(core.job(&queued.id).unwrap().status, "cancelled");
    assert!(core.execute_job(&queued.id).unwrap().result.is_none());
    let hash = core.imported_file(&file).unwrap().sha256;
    fs::write(
        dir.path().join("imports").join(format!("{hash}.bin")),
        b"changed bytes",
    )
    .unwrap();
    let queued = core
        .enqueue_job(input(&file, JobKind::LocalStaticBinary))
        .unwrap();
    let failed = core.execute_job(&queued.id).unwrap();
    assert_eq!(failed.status, "failed");
    assert_eq!(failed.error_code.as_deref(), Some("INTEGRITY_ERROR"));
    assert!(failed.result.is_none());
    assert!(core.analysis_for_job(&failed.id).is_err());
}
#[test]
fn configured_timeout_and_invalid_job_input_are_enforced() {
    let (_dir, core, file) = setup();
    let mut request = input(&file, JobKind::LocalStaticBinary);
    request.timeout_ms = 0;
    assert_eq!(
        core.enqueue_job(request.clone()).unwrap_err().code,
        "INVALID_INPUT"
    );
    request.timeout_ms = 1;
    let job = core.enqueue_job(request).unwrap();
    let done = core.execute_job(&job.id).unwrap();
    assert_eq!(done.status, "failed");
    assert_eq!(done.error_code.as_deref(), Some("JOB_TIMEOUT"));
    assert!(done.result.is_none());
    assert!(JobEngine::start(core, 3, |_| {}).is_err());
}
#[derive(Default)]
struct Gate {
    open: Mutex<bool>,
    changed: Condvar,
}
impl Gate {
    fn wait(&self) {
        let _guard = self
            .changed
            .wait_timeout_while(self.open.lock().unwrap(), Duration::from_secs(10), |open| {
                !*open
            })
            .unwrap();
    }
    fn release(&self) {
        *self.open.lock().unwrap() = true;
        self.changed.notify_all();
    }
}
#[test]
fn two_engines_obey_global_capacity_and_do_not_steal_live_leases() {
    let (dir, core, file) = setup();
    let mut jobs = Vec::new();
    for _ in 0..4 {
        jobs.push(
            core.enqueue_job(input(&file, JobKind::LocalStaticBinary))
                .unwrap(),
        );
    }
    let gate = Arc::new(Gate::default());
    let (tx, rx) = mpsc::channel();
    let start = || {
        let gate = gate.clone();
        let tx = tx.clone();
        JobEngine::start(core.clone(), 2, move |j| {
            if j.status == "running" && j.progress == 1 {
                tx.send(j.id).unwrap();
                gate.wait();
            }
        })
        .unwrap()
    };
    let one = start();
    let two = start();
    let a = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let b = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_ne!(a, b);
    let reopened = Core::open(Some(dir.path().join("jobs.db"))).unwrap();
    let rows = reopened.list_jobs(None).unwrap();
    assert_eq!(rows.iter().filter(|j| j.status == "running").count(), 2);
    assert_eq!(rows.iter().filter(|j| j.status == "queued").count(), 2);
    gate.release();
    for job in jobs {
        assert_eq!(wait_done(&core, &job.id).status, "completed");
    }
    drop((one, two));
}
#[test]
fn running_cancellation_is_acknowledged_before_releasing_capacity() {
    let (_dir, core, file) = setup();
    let job = core
        .enqueue_job(input(&file, JobKind::LocalStaticBinary))
        .unwrap();
    let gate = Arc::new(Gate::default());
    let (tx, rx) = mpsc::channel();
    let pause = gate.clone();
    let engine = JobEngine::start(core.clone(), 1, move |j| {
        if j.status == "running" && j.progress == 1 {
            tx.send(j.id).unwrap();
            pause.wait();
        }
    })
    .unwrap();
    rx.recv_timeout(Duration::from_secs(5)).unwrap();
    core.cancel_job(&job.id).unwrap();
    let pending = core.job(&job.id).unwrap();
    assert_eq!(pending.status, "running");
    assert!(pending.cancel_requested);
    gate.release();
    let done = wait_done(&core, &job.id);
    assert_eq!(done.status, "cancelled");
    assert!(done.result.is_none());
    drop(engine);
}
#[test]
fn cli_style_analysis_waits_for_an_already_claimed_desktop_job() {
    let (_dir, core, file) = setup();
    let job = core.begin_file_analysis(&file).unwrap();
    let gate = Arc::new(Gate::default());
    let paused = gate.clone();
    let (started_tx, started_rx) = mpsc::channel();
    let engine = JobEngine::start(core.clone(), 1, move |j| {
        if j.status == "running" && j.progress == 1 {
            started_tx.send(()).unwrap();
            paused.wait();
        }
    })
    .unwrap();
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let worker = core.clone();
    let id = job.id.clone();
    let (tx, rx) = mpsc::channel();
    let waiter = std::thread::spawn(move || {
        tx.send(worker.run_file_analysis(&id)).unwrap();
    });
    assert!(rx.recv_timeout(Duration::from_millis(40)).is_err());
    gate.release();
    let record = rx.recv_timeout(Duration::from_secs(5)).unwrap().unwrap();
    assert_eq!(record.job_id, job.id);
    waiter.join().unwrap();
    drop(engine);
}
#[test]
fn expired_worker_recovers_without_replaying_work_and_queued_jobs_survive() {
    let (dir, core, file) = setup();
    let lost = core
        .enqueue_job(input(&file, JobKind::LocalStaticBinary))
        .unwrap();
    let pending = core
        .enqueue_job(input(&file, JobKind::ArtifactIndex))
        .unwrap();
    let conn = Connection::open(dir.path().join("jobs.db")).unwrap();
    conn.execute("UPDATE analysis_jobs SET status='running',worker_id='terminated-worker',lease_until=1,started_at=created_at WHERE id=?1",[&lost.id]).unwrap();
    drop(conn);
    let reopened = Core::open(Some(dir.path().join("jobs.db"))).unwrap();
    let interrupted = reopened.job(&lost.id).unwrap();
    assert_eq!(interrupted.status, "failed");
    assert_eq!(interrupted.error_code.as_deref(), Some("JOB_INTERRUPTED"));
    assert_eq!(reopened.job(&pending.id).unwrap().status, "queued");
    let engine = JobEngine::start(reopened.clone(), 1, |_| {}).unwrap();
    assert_eq!(wait_done(&reopened, &pending.id).status, "completed");
    drop(engine);
}
#[test]
fn version_two_migration_preserves_binary_results_settings_and_foreign_keys() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("v2.db");
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(include_str!(
        "../../reconx-storage/migrations/001_foundation.sql"
    ))
    .unwrap();
    conn.execute_batch(include_str!(
        "../../reconx-storage/migrations/002_research_inputs.sql"
    ))
    .unwrap();
    conn.execute("INSERT INTO projects(id,name,description,created_at,updated_at,archived) VALUES('p','kept','notes','2026-10-15','2026-10-15',0)",[]).unwrap();
    let a = reconx_analysis::inspect(b"real local fixture").unwrap();
    let record = reconx_core::AnalysisRecord {
        id: "a".into(),
        file_id: "f".into(),
        job_id: "j".into(),
        created_at: "2026-10-15".into(),
        analysis: a,
    };
    let bytes = serde_json::to_vec(&record).unwrap();
    fs::create_dir(dir.path().join("imports")).unwrap();
    fs::write(dir.path().join("imports/a.json"), &bytes).unwrap();
    conn.execute("INSERT INTO imported_files(id,project_id,name,sha256,size,format,created_at,source_value) VALUES('f','p','sample',?1,18,'Raw binary','2026-10-15','C:/owned/sample.bin')",[&record.analysis.sha256]).unwrap();
    conn.execute("INSERT INTO analysis_jobs(id,project_id,file_id,kind,status,created_at,finished_at,logs_json) VALUES('j','p','f','local_static_binary','completed','2026-10-15','2026-10-15','[\"Retained\"]')",[]).unwrap();
    conn.execute("INSERT INTO binary_analyses(id,file_id,job_id,created_at,artifact_name,artifact_sha256) VALUES('a','f','j','2026-10-15','a.json',?1)",[reconx_analysis::hash(&bytes)]).unwrap();
    conn.execute(
        "INSERT INTO settings(id,json) VALUES(1,?1)",
        [r#"{"theme":"light","security_mode":"passive_auto","default_provider_id":null}"#],
    )
    .unwrap();
    conn.pragma_update(None, "user_version", 2).unwrap();
    drop(conn);
    let core = Core::open(Some(path.clone())).unwrap();
    assert_eq!(core.file_analysis("f").unwrap().id, "a");
    assert_eq!(core.job("j").unwrap().progress, 100);
    assert_eq!(core.job("j").unwrap().logs, vec!["Retained"]);
    assert!(matches!(
        core.settings().unwrap().theme,
        reconx_core::ThemePreference::Light
    ));
    let conn = Connection::open(path).unwrap();
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| r
            .get::<_, u32>(
            0
        ))
        .unwrap(),
        0
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM projects WHERE id=?1",
            params!["p"],
            |r| r.get::<_, u32>(0)
        )
        .unwrap(),
        1
    );
}
#[tokio::test]
async fn artifact_transmission_requires_consent_and_exact_preview_destination() {
    let (_dir, core, file) = setup();
    core.analyze_file(&file).unwrap();
    let mut request = ArtifactReview {
        id: file.clone(),
        section: "summary".into(),
        provider: "not-configured".into(),
        preview: String::new(),
        gateway: String::new(),
        consent: false,
    };
    assert_eq!(
        core.explain_file_artifact(request).await.unwrap_err().code,
        "CONSENT_REQUIRED"
    );
    request = ArtifactReview {
        id: file,
        section: "summary".into(),
        provider: "not-configured".into(),
        preview: "changed".into(),
        gateway: String::new(),
        consent: true,
    };
    assert_eq!(
        core.explain_file_artifact(request).await.unwrap_err().code,
        "ARTIFACT_CHANGED"
    );
}
#[tokio::test]
async fn artifact_review_rejects_gateway_changes_and_never_uses_chat_fallback() {
    let (_dir, core, file) = setup();
    core.analyze_file(&file).unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let gateway = format!("http://{}/v1", listener.local_addr().unwrap());
    drop(listener);
    let primary = reconx_core::ProviderConfig {
        id: "296d4d97-ff89-4a81-9514-bb90bc723aa7".into(),
        name: "owned primary".into(),
        kind: reconx_core::ProviderKind::Compatible,
        base_url: gateway.clone(),
        model: "fixture".into(),
        enabled: true,
        timeout_secs: 10,
        max_output_tokens: 256,
        requires_key: false,
    };
    let secondary = reconx_core::ProviderConfig {
        id: "49207d4e-8690-4dd8-9dab-7e12f9d53eb5".into(),
        name: "owned secondary".into(),
        ..primary.clone()
    };
    core.save_provider(&primary).unwrap();
    core.save_provider(&secondary).unwrap();
    let mut settings = core.settings().unwrap();
    settings.default_provider_id = Some(primary.id.clone());
    settings.fallback_provider_id = Some(secondary.id);
    core.save_settings(&settings).unwrap();
    let request = || ArtifactReview {
        id: file.clone(),
        section: "summary".into(),
        provider: primary.id.clone(),
        preview: core.file_artifact_preview(&file, "summary").unwrap(),
        gateway: gateway.clone(),
        consent: true,
    };
    let mut changed = request();
    changed.gateway = "http://127.0.0.1:1/different".into();
    assert_eq!(
        core.explain_file_artifact(changed).await.unwrap_err().code,
        "GATEWAY_CHANGED"
    );
    assert_eq!(
        core.explain_file_artifact(request())
            .await
            .unwrap_err()
            .code,
        "AI_CONNECT_ERROR"
    );
    let audit = core.snapshot().unwrap().audit;
    assert!(audit.iter().any(|e| e.action == "ai.artifact_review"));
    assert!(!audit.iter().any(|e| e.action == "ai.fallback"));
}
