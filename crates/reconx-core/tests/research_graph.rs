use reconx_core::*;
use serde_json::json;
use std::{
    thread,
    time::{Duration, Instant},
};

async fn session(core: &Core, project: &str, name: &str) -> ResearchSession {
    let target=core.create_target(serde_json::from_value(json!({"project_id":project,"name":name,"kind":"url","value":"http://127.0.0.1:9/","authorized":true})).unwrap()).unwrap();
    let p = core
        .research_preview(ResearchRequest {
            target_id: target.id,
            provider_id: None,
            mode: ResearchMode::Manual,
        })
        .unwrap();
    core.research_plan(ResearchReview {
        request: p.request,
        fingerprint: p.fingerprint,
        context: p.context,
        consent: false,
    })
    .await
    .unwrap()
}
#[tokio::test]
async fn graph_is_persistent_paginated_idempotent_and_isolated_without_executing_work() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("graph.db");
    let core = Core::open(Some(path.clone())).unwrap();
    let p = core.create_project("Graph fixture", "").unwrap();
    let s = session(&core, &p.id, "Authorization: Bearer graph-synthetic-secret").await;
    let first = core
        .research_graph(&s.plan.id, ResearchGraphQuery::default())
        .unwrap();
    assert_eq!(first.nodes.len(), 4);
    assert_eq!(first.total_nodes, 4);
    assert!(
        !serde_json::to_string(&first)
            .unwrap()
            .contains("graph-synthetic-secret")
    );
    assert!(core.list_jobs(None).unwrap().is_empty());
    let target = first.nodes.iter().find(|n| n.kind == "target").unwrap();
    let p2 = core.create_project("Graph fixture 2", "").unwrap();
    let s2 = session(
        &core,
        &p2.id,
        "Authorization: Bearer graph-synthetic-secret",
    )
    .await;
    let other = core
        .research_graph(&s2.plan.id, ResearchGraphQuery::default())
        .unwrap();
    assert!(!other.nodes.iter().any(|n| n.id == target.id));
    assert_eq!(
        core.research_graph_record(&s2.plan.id, &target.id)
            .unwrap_err()
            .code,
        "NOT_FOUND"
    );
    assert_eq!(
        core.research_graph(
            &s.plan.id,
            ResearchGraphQuery {
                limit: 0,
                ..Default::default()
            }
        )
        .unwrap_err()
        .code,
        "GRAPH_LIMIT"
    );
    let page = core
        .research_graph(
            &s.plan.id,
            ResearchGraphQuery {
                limit: 2,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(page.nodes.len(), 2);
    assert_eq!(page.next_offset, Some(2));
    assert!(
        page.edges
            .iter()
            .all(|e| page.nodes.iter().any(|n| n.id == e.source)
                && page.nodes.iter().any(|n| n.id == e.target))
    );
    let reopened = Core::open(Some(path)).unwrap();
    let again = reopened
        .research_graph(&s.plan.id, ResearchGraphQuery::default())
        .unwrap();
    assert_eq!(again.revision, first.revision);
    assert_eq!(again.source_sha256, first.source_sha256);
    let export = dir.path().join("graph.json");
    let result = core.research_graph_export(&s.plan.id, &export).unwrap();
    assert_eq!(result["nodes"], 4);
    let text = std::fs::read_to_string(&export).unwrap();
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value["edges"].as_array().unwrap().len(), first.total_edges);
    assert!(!text.contains("graph-synthetic-secret"));
    assert!(core.research_graph_export(&s.plan.id, &export).is_err());
}
#[tokio::test]
async fn actual_job_evidence_and_manual_findings_have_real_graph_provenance() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::open(Some(dir.path().join("graph.db"))).unwrap();
    let p = core.create_project("Local graph execution", "").unwrap();
    let s = session(&core, &p.id, "Owned offline target").await;
    core.research_approve(&s.plan.id, &s.plan.preview.fingerprint, true, false)
        .unwrap();
    let engine = JobEngine::start(core.clone(), 1, |_| {}).unwrap();
    let until = Instant::now() + Duration::from_secs(10);
    let done = loop {
        let s = core.research_session(&s.plan.id).unwrap();
        if s.status == "completed" {
            break s;
        }
        assert!(Instant::now() < until, "{s:?}");
        thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(done.jobs.len(), 1);
    assert_eq!(done.evidence.len(), 1);
    let g = core
        .research_graph(&s.plan.id, ResearchGraphQuery::default())
        .unwrap();
    let evidence = g.nodes.iter().find(|n| n.kind == "evidence").unwrap();
    assert_eq!(evidence.source_id, done.evidence[0].id);
    assert!(
        core.research_graph_record(&s.plan.id, &evidence.id)
            .unwrap()
            .get("record")
            .is_some()
    );
    let f = core
        .research_finding(
            &s.plan.id,
            "Manual review note",
            "Not a verified vulnerability",
        )
        .unwrap();
    let next = core
        .research_graph(&s.plan.id, ResearchGraphQuery::default())
        .unwrap();
    assert!(next.revision > g.revision);
    assert!(next.nodes.iter().any(|n| n.kind == "finding"
        && n.source_id == f.id
        && n.attributes["status"] == "needs_verification"));
    assert!(next.edges.iter().any(|e| e.relation == "SUPPORTED_BY"));
    drop(engine);

    core.remove_target(&s.plan.preview.request.target_id)
        .unwrap();
    let historical = core
        .research_graph(&s.plan.id, ResearchGraphQuery::default())
        .unwrap();
    assert_eq!(historical.total_nodes, next.total_nodes);
    assert_eq!(historical.total_edges, next.total_edges);
    assert_eq!(
        historical
            .nodes
            .iter()
            .find(|n| n.kind == "target")
            .unwrap()
            .attributes["source_state"],
        "removed"
    );
    assert!(
        core.research_graph_record(&s.plan.id, &evidence.id)
            .unwrap()
            .get("record")
            .is_some()
    );
    let reopened = Core::open(Some(core.database_path().to_path_buf())).unwrap();
    let again = reopened
        .research_graph(&s.plan.id, ResearchGraphQuery::default())
        .unwrap();
    assert_eq!(again.revision, historical.revision);
    assert_eq!(reopened.list_jobs(None).unwrap().len(), 1);
}

#[tokio::test]
async fn removed_sources_remain_inspectable_without_reauthorizing_or_inventing_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::open(Some(dir.path().join("history.db"))).unwrap();
    let p = core.create_project("File history", "").unwrap();
    // One session was projected before removal, another was never projected.
    for projected in [true, false] {
        let path = dir.path().join(format!("benign-{projected}.bin"));
        std::fs::write(&path, b"BENIGN_STATIC_METADATA_FIXTURE").unwrap();
        let imported = core.import_file(&p.id, &path, true).unwrap();
        let target_id = imported.target_id.as_ref().unwrap();
        let preview = core
            .research_preview(ResearchRequest {
                target_id: target_id.clone(),
                provider_id: None,
                mode: ResearchMode::Manual,
            })
            .unwrap();
        let s = core
            .research_plan(ResearchReview {
                request: preview.request,
                fingerprint: preview.fingerprint,
                context: preview.context,
                consent: false,
            })
            .await
            .unwrap();
        let previous = projected.then(|| {
            core.research_graph(&s.plan.id, ResearchGraphQuery::default())
                .unwrap()
        });
        core.remove_target(target_id).unwrap();
        // Simulate removal of an imported record only in this disposable database;
        // no production delete API or artifact deletion is introduced.
        let db = reconx_storage::Database::open(core.database_path()).unwrap();
        db.connect()
            .unwrap()
            .execute("DELETE FROM imported_files WHERE id=?1", [&imported.id])
            .unwrap();
        let g = core
            .research_graph(&s.plan.id, ResearchGraphQuery::default())
            .unwrap();
        let target = g.nodes.iter().find(|n| n.kind == "target").unwrap();
        let file = g.nodes.iter().find(|n| n.kind == "file").unwrap();
        assert_eq!(target.attributes["source_state"], "removed");
        assert_eq!(target.attributes["scope_status"], "unavailable");
        assert_eq!(file.attributes["source_state"], "removed");
        assert_eq!(file.source_id, imported.id);
        if let Some(previous) = previous {
            let old = previous.nodes.iter().find(|n| n.kind == "file").unwrap();
            assert_eq!(old.id, file.id);
            assert_eq!(file.attributes["sha256"], imported.sha256);
        } else {
            assert!(file.attributes.get("sha256").is_none());
            assert_eq!(file.label, "Removed file record");
        }
        let inspection = core.research_graph_record(&s.plan.id, &file.id).unwrap();
        assert_eq!(inspection["record"]["source_state"], "removed");
        assert!(
            !serde_json::to_string(&g)
                .unwrap()
                .contains(&dir.path().to_string_lossy().to_string())
        );
        assert!(
            core.research_approve(&s.plan.id, &s.plan.preview.fingerprint, true, false)
                .is_err()
        );
        assert!(core.list_jobs(None).unwrap().is_empty());
        let reopened = Core::open(Some(core.database_path().to_path_buf())).unwrap();
        let again = reopened
            .research_graph(&s.plan.id, ResearchGraphQuery::default())
            .unwrap();
        assert_eq!(again.source_sha256, g.source_sha256);
        assert_eq!(again.revision, g.revision);
        let output = dir.path().join(format!("history-{projected}.json"));
        reopened.research_graph_export(&s.plan.id, &output).unwrap();
        assert!(std::fs::read_to_string(output).unwrap().contains("removed"));
    }
}

#[tokio::test]
async fn concurrent_refreshes_share_a_single_revision_and_reject_cross_project_file_changes() {
    let dir = tempfile::tempdir().unwrap();
    let core = Core::open(Some(dir.path().join("concurrent.db"))).unwrap();
    let p = core.create_project("Graph readers", "").unwrap();
    let s = session(&core, &p.id, "Owned offline target").await;
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(6));
    let handles: Vec<_> = (0..6)
        .map(|_| {
            let core = core.clone();
            let id = s.plan.id.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                barrier.wait();
                core.research_graph(&id, ResearchGraphQuery::default())
                    .unwrap()
            })
        })
        .collect();
    let pages: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert!(
        pages
            .iter()
            .all(|g| g.revision == 1 && g.source_sha256 == pages[0].source_sha256)
    );
    assert!(core.list_jobs(None).unwrap().is_empty());

    let path = dir.path().join("benign.bin");
    std::fs::write(&path, b"BENIGN_NO_EXECUTION").unwrap();
    let file = core.import_file(&p.id, &path, true).unwrap();
    let preview = core
        .research_preview(ResearchRequest {
            target_id: file.target_id.unwrap(),
            provider_id: None,
            mode: ResearchMode::Manual,
        })
        .unwrap();
    let file_session = core
        .research_plan(ResearchReview {
            request: preview.request,
            fingerprint: preview.fingerprint,
            context: preview.context,
            consent: false,
        })
        .await
        .unwrap();
    let old = core
        .research_graph(&file_session.plan.id, ResearchGraphQuery::default())
        .unwrap();
    let other = core.create_project("Other project", "").unwrap();
    let db = reconx_storage::Database::open(core.database_path()).unwrap();
    let conn = db.connect().unwrap();
    conn.execute(
        "UPDATE imported_files SET project_id=?1 WHERE id=?2",
        [&other.id, &file.id],
    )
    .unwrap();
    assert_eq!(
        core.research_graph(&file_session.plan.id, ResearchGraphQuery::default())
            .unwrap_err()
            .code,
        "GRAPH_LIMIT"
    );
    let stored: String = conn
        .query_row(
            "SELECT source_sha256 FROM research_graph_snapshots WHERE session_id=?1",
            [&file_session.plan.id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(stored, old.source_sha256);
}
