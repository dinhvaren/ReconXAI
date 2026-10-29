use reconx_core::*;
use serde_json::json;
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

fn setup() -> (tempfile::TempDir, Core, String, String) {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::open(Some(dir.path().join("research.db"))).unwrap();
    let project = core.create_project("Research Demo", "").unwrap().id;
    let file = dir.path().join("benign.bin");
    std::fs::write(&file, b"BENIGN_RESEARCH_FIXTURE\0hello local evidence").unwrap();
    let imported = core.import_file(&project, &file, true).unwrap();
    (dir, core, project, imported.target_id.unwrap())
}
fn review(p: &ResearchPreview, consent: bool) -> ResearchReview {
    ResearchReview {
        request: p.request.clone(),
        fingerprint: p.fingerprint.clone(),
        context: p.context.clone(),
        consent,
    }
}
fn request(target: &str, provider: Option<&str>, mode: ResearchMode) -> ResearchRequest {
    ResearchRequest {
        target_id: target.into(),
        provider_id: provider.map(str::to_owned),
        mode,
    }
}
fn done(core: &Core, id: &str) -> ResearchSession {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let s = core.research_session(id).unwrap();
        if !matches!(s.status.as_str(), "queued" | "running") {
            return s;
        }
        assert!(Instant::now() < deadline, "session stuck: {s:?}");
        thread::sleep(Duration::from_millis(20));
    }
}
#[test]
fn persisted_workspace_uses_ids_and_cas_instead_of_names() {
    let (dir, core, project, target) = setup();
    let other = core
        .create_project("Research Demo 2", "Similar names; distinct IDs")
        .unwrap();
    let selected = core
        .select_workspace(Some(&project), Some(&target), 0)
        .unwrap();
    assert_eq!(selected.target_id.as_deref(), Some(target.as_str()));
    assert!(
        core.select_workspace(Some(&other.id), Some(&target), selected.revision)
            .is_err()
    );
    assert_eq!(
        core.select_workspace(Some(&project), None, 0)
            .unwrap_err()
            .code,
        "STALE_SELECTION"
    );
    let reopened = Core::open(Some(dir.path().join("research.db"))).unwrap();
    assert_eq!(reopened.workspace_selection().unwrap(), selected);
    core.remove_target(&target).unwrap();
    assert!(core.workspace_selection().unwrap().target_id.is_none());
    assert_eq!(
        core.workspace_selection().unwrap().project_id.as_deref(),
        Some(project.as_str())
    );
}
#[tokio::test]
async fn manual_file_session_executes_existing_jobs_and_exports_reviewed_reports() {
    let (dir, core, project, target) = setup();
    let preview = core
        .research_preview(request(&target, None, ResearchMode::Manual))
        .unwrap();
    assert!(
        !preview
            .context
            .contains(&dir.path().to_string_lossy().to_string())
    );
    let session = core.research_plan(review(&preview, false)).await.unwrap();
    assert_eq!(session.status, "awaiting_approval");
    assert!(core.list_jobs(None).unwrap().is_empty());
    assert!(
        core.research_approve(&session.plan.id, &preview.fingerprint, false, false)
            .is_err()
    );
    let queued = core
        .research_approve(&session.plan.id, &preview.fingerprint, true, false)
        .unwrap();
    assert_eq!(queued.jobs.len(), 3);
    assert!(
        queued
            .jobs
            .iter()
            .all(|j| j.project_id == project && j.target_id.as_deref() == Some(&target))
    );
    assert!(
        core.research_approve(&session.plan.id, &preview.fingerprint, true, false)
            .is_err()
    );
    let engine = JobEngine::start(core.clone(), 2, |_| {}).unwrap();
    let out = done(&core, &session.plan.id);
    assert_eq!(out.status, "completed", "{out:?}");
    assert_eq!(out.evidence.len(), 3);
    let graph = core
        .research_graph(&out.plan.id, ResearchGraphQuery::default())
        .unwrap();
    let target_node = graph.nodes.iter().find(|n| n.kind == "target").unwrap();
    assert_eq!(
        target_node.attributes["value"], "Local artifact location omitted",
        "Graph metadata must omit local import paths"
    );
    for e in &out.evidence {
        core.evidence_verify(&e.id).unwrap();
    }
    assert!(core.research_cancel(&session.plan.id).is_err());
    let finding = core
        .research_finding(
            &session.plan.id,
            "Review fixture metadata",
            "A manual review note, not a verified vulnerability",
        )
        .unwrap();
    assert_eq!(finding.input.status, FindingStatus::NeedsVerification);
    let report = core.research_report(&session.plan.id).unwrap();
    for (format, ext) in [
        (ReportFormat::Markdown, "md"),
        (ReportFormat::Json, "json"),
        (ReportFormat::Html, "html"),
        (ReportFormat::Pdf, "pdf"),
    ] {
        let path = dir.path().join(format!("actual-report.{ext}"));
        core.report_export(&report.report.id, format, &path, &report.report.sha256)
            .unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.len() > 100);
        assert!(!String::from_utf8_lossy(&bytes).contains("synthetic-secret-must-not-leak"));
    }
    drop(engine);
    let reopened = Core::open(Some(dir.path().join("research.db"))).unwrap();
    let persisted = reopened.research_session(&session.plan.id).unwrap();
    assert_eq!(persisted.status, "completed");
    assert_eq!(persisted.findings[0].id, finding.id);
    assert_eq!(persisted.reports[0].id, report.report.id);
}
#[tokio::test]
async fn changed_scope_invalidates_plan_and_cancellation_preserves_history() {
    let (_dir, core, _, target) = setup();
    let p = core
        .research_preview(request(&target, None, ResearchMode::Manual))
        .unwrap();
    let s = core.research_plan(review(&p, false)).await.unwrap();
    let mut t = core.target(&target).unwrap().input;
    t.description = "Policy reviewed after plan".into();
    core.update_target(&target, t.clone()).unwrap();
    assert_eq!(
        core.research_approve(&s.plan.id, &p.fingerprint, true, false)
            .unwrap_err()
            .code,
        "APPROVAL_STALE"
    );
    assert!(core.list_jobs(None).unwrap().is_empty());
    let p = core
        .research_preview(request(&target, None, ResearchMode::Manual))
        .unwrap();
    let s = core.research_plan(review(&p, false)).await.unwrap();
    core.research_approve(&s.plan.id, &p.fingerprint, true, false)
        .unwrap();
    let s = core.research_cancel(&s.plan.id).unwrap();
    assert_eq!(s.status, "cancelled");
    assert!(s.jobs.iter().all(|j| j.status == "cancelled"));
    assert!(core.research_report(&s.plan.id).is_err());
    t.authorized = false;
    core.update_target(&target, t).unwrap();
    assert!(
        core.research_preview(request(&target, None, ResearchMode::Manual))
            .is_err()
    );
}

struct Fixture {
    url: String,
    calls: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    task: Option<thread::JoinHandle<()>>,
}
impl Fixture {
    fn new(reply: String) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let calls = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let (log, quit) = (calls.clone(), stop.clone());
        let task = thread::spawn(move || {
            while !quit.load(Ordering::Acquire) {
                let Ok((mut s, _)) = listener.accept() else {
                    thread::sleep(Duration::from_millis(5));
                    continue;
                };
                s.set_nonblocking(false).unwrap();
                s.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
                let mut bytes = Vec::new();
                let mut buf = [0; 4096];
                while let Ok(n) = s.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    bytes.extend_from_slice(&buf[..n]);
                    if let Some(at) = bytes.windows(4).position(|b| b == b"\r\n\r\n") {
                        let head = String::from_utf8_lossy(&bytes[..at]).to_lowercase();
                        let size = head
                            .lines()
                            .find_map(|l| {
                                l.strip_prefix("content-length:")
                                    .and_then(|s| s.trim().parse::<usize>().ok())
                            })
                            .unwrap_or(0);
                        if bytes.len() >= at + 4 + size {
                            break;
                        }
                    }
                }
                log.lock()
                    .unwrap()
                    .push(String::from_utf8_lossy(&bytes).to_string());
                let _ = write!(
                    s,
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    reply.len(),
                    reply
                );
            }
        });
        Self {
            url,
            calls,
            stop,
            task: Some(task),
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(task) = self.task.take() {
            task.join().unwrap();
        }
    }
}
fn model(core: &Core, fixture: &Fixture) -> ProviderConfig {
    let p = ProviderConfig {
        id: "fc694fe1-c5d7-4ccb-86d8-7466d99302ce".into(),
        name: "Deterministic local fixture".into(),
        kind: ProviderKind::Compatible,
        base_url: format!("{}/v1", fixture.url),
        model: "fixture-model".into(),
        enabled: true,
        requires_key: false,
        timeout_secs: 5,
        max_output_tokens: 1024,
    };
    core.save_provider(&p).unwrap();
    p
}
#[tokio::test]
async fn mock_ai_plan_requires_consent_and_separate_network_approval_and_collects_real_head() {
    let (_dir, core, project, _) = setup();
    let http = Fixture::new(String::new());
    let ai=Fixture::new(json!({"choices":[{"message":{"content":json!({"steps":[{"capability_id":"local.scope_review","reason":"Review authorization"},{"capability_id":"local.http_head","reason":"Observe the owned fixture"}]}).to_string()}}],"usage":{"prompt_tokens":12,"completion_tokens":8}}).to_string());
    let provider = model(&core, &ai);
    let mut settings = core.settings().unwrap();
    settings.security_mode = SecurityMode::LabResearch;
    core.save_settings(&settings).unwrap();
    let target = core
        .create_target(TargetInput {
            project_id: project,
            name: "Owned fixture".into(),
            kind: TargetKind::Url,
            value: format!("{}/", http.url),
            description: String::new(),
            tags: vec![],
            allowed_scope: vec![],
            excluded_scope: vec![],
            authorized: true,
            mode: SecurityMode::LabResearch,
        })
        .unwrap();
    let p = core
        .research_preview(request(
            &target.id,
            Some(&provider.id),
            ResearchMode::AiAssisted,
        ))
        .unwrap();
    assert!(core.research_plan(review(&p, false)).await.is_err());
    assert!(ai.calls.lock().unwrap().is_empty());
    let s = core.research_plan(review(&p, true)).await.unwrap();
    assert_eq!(ai.calls.lock().unwrap().len(), 1);
    assert!(http.calls.lock().unwrap().is_empty());
    let before = core
        .research_graph(&s.plan.id, ResearchGraphQuery::default())
        .unwrap();
    assert!(
        !before
            .nodes
            .iter()
            .any(|n| matches!(n.kind.as_str(), "task" | "tool_invocation" | "observation"))
    );
    assert!(http.calls.lock().unwrap().is_empty());
    assert!(
        core.research_approve(&s.plan.id, &p.fingerprint, true, false)
            .is_err()
    );
    assert!(
        core.list_jobs(None).unwrap().is_empty(),
        "Whole batch must roll back on missing network approval"
    );
    core.research_approve(&s.plan.id, &p.fingerprint, true, true)
        .unwrap();
    let engine = JobEngine::start(core.clone(), 2, |_| {}).unwrap();
    let s = done(&core, &s.plan.id);
    assert_eq!(s.status, "completed", "{s:?}");
    let graph = core
        .research_graph(&s.plan.id, ResearchGraphQuery::default())
        .unwrap();
    assert!(graph.revision > before.revision);
    assert_eq!(graph.nodes.iter().filter(|n| n.kind == "url").count(), 1);
    assert_eq!(graph.nodes.iter().filter(|n| n.kind == "task").count(), 2);
    assert!(graph.edges.iter().any(|e| e.relation == "SUPPORTED_BY"));
    let observation = graph
        .nodes
        .iter()
        .find(|n| n.kind == "observation")
        .unwrap();
    assert_eq!(observation.attributes["status"], 200);
    assert!(s.jobs.iter().any(|j| j.id == observation.source_id));
    assert_eq!(
        core.research_graph(&s.plan.id, ResearchGraphQuery::default())
            .unwrap()
            .revision,
        graph.revision
    );
    assert_eq!(
        ai.calls.lock().unwrap().len(),
        1,
        "Graph review never calls AI"
    );
    {
        let calls = http.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert!(calls[0].starts_with("HEAD / HTTP/1.1"));
    }
    let context = core.research_evidence_preview(&s.plan.id).unwrap();
    assert!(context.contains(&s.evidence[0].id));
    let review = ResearchInterpretReview {
        id: s.plan.id.clone(),
        provider_id: provider.id.clone(),
        gateway: provider.base_url,
        model: provider.model,
        preview: context,
        consent: false,
    };
    assert!(core.research_interpret(review).await.is_err());
    assert_eq!(ai.calls.lock().unwrap().len(), 1);
    let auto = core
        .research_preview(request(
            &target.id,
            Some(&provider.id),
            ResearchMode::AuthorizedAuto,
        ))
        .unwrap();
    assert!(
        !auto
            .capabilities
            .iter()
            .find(|c| c.id == "local.http_head")
            .unwrap()
            .available
    );
    drop(engine);
}
#[tokio::test]
async fn model_cannot_inject_shell_arguments_unknown_or_duplicate_capabilities() {
    for proposal in [
        json!({"steps":[{"capability_id":"shell.exec","reason":"Run command"}]}),
        json!({"steps":[{"capability_id":"local.scope_review","reason":"Review","command":"arbitrary"}]}),
        json!({"steps":[{"capability_id":"local.scope_review","reason":"First"},{"capability_id":"local.scope_review","reason":"Duplicate"}]}),
    ] {
        let (_dir, core, project, target) = setup();
        let ai = Fixture::new(
            json!({"choices":[{"message":{"content":proposal.to_string()}}]}).to_string(),
        );
        let model = model(&core, &ai);
        let p = core
            .research_preview(request(&target, Some(&model.id), ResearchMode::AiAssisted))
            .unwrap();
        assert!(core.research_plan(review(&p, true)).await.is_err());
        assert!(core.research_list(&project).unwrap().is_empty());
        assert!(core.list_jobs(None).unwrap().is_empty());
    }
}
