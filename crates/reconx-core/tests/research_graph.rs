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
}
