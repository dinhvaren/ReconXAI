//! Bounded, materialized provenance. Reading this graph never executes tools/AI.
use super::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const MAX_NODES: usize = 5000;
const MAX_EDGES: usize = 20000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchGraphNode {
    pub id: String,
    pub graph: String,
    pub kind: String,
    pub source_table: String,
    pub source_id: String,
    pub label: String,
    pub attributes: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchGraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub relation: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ResearchGraphQuery {
    pub graph: String,
    pub kind: String,
    pub search: String,
    pub offset: usize,
    pub limit: usize,
}
impl Default for ResearchGraphQuery {
    fn default() -> Self {
        Self {
            graph: String::new(),
            kind: String::new(),
            search: String::new(),
            offset: 0,
            limit: 100,
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ResearchGraphPage {
    pub session_id: String,
    pub revision: i64,
    pub source_sha256: String,
    pub updated_at: String,
    pub total_nodes: usize,
    pub total_edges: usize,
    pub matched_nodes: usize,
    pub next_offset: Option<usize>,
    pub nodes: Vec<ResearchGraphNode>,
    pub edges: Vec<ResearchGraphEdge>,
    pub edges_truncated: bool,
}
#[derive(Default)]
struct Projection {
    nodes: BTreeMap<String, ResearchGraphNode>,
    edges: BTreeMap<String, ResearchGraphEdge>,
}
fn graph_error(message: &str) -> AppError {
    AppError::new("GRAPH_LIMIT", message)
}
fn safe_label(label: &str) -> String {
    let value = if let Ok(mut u) = url::Url::parse(label) {
        if matches!(u.scheme(), "http" | "https") {
            let _ = u.set_username("");
            let _ = u.set_password(None);
            u.set_query(None);
            u.set_fragment(None);
            u.to_string()
        } else {
            label.to_owned()
        }
    } else {
        label.to_owned()
    };
    reporting::redact_content(&value)
        .0
        .chars()
        .take(240)
        .collect()
}
impl Projection {
    fn node(
        &mut self,
        session: &str,
        graph: &str,
        kind: &str,
        origin: (&str, &str),
        label: &str,
        mut attrs: Value,
    ) -> Result<String> {
        let (table, source) = origin;
        reporting::redact_value(&mut attrs);
        if attrs.to_string().len() > 8192 {
            return Err(graph_error(
                "Graph metadata exceeds 8 KiB; raw artifacts are never embedded",
            ));
        }
        let discriminator = if kind == "observation" {
            attrs["asset_id"].as_str().unwrap_or("")
        } else {
            ""
        };
        let id = evidence::digest(
            format!("{session}|{graph}|{kind}|{table}|{source}|{discriminator}").as_bytes(),
        );
        self.nodes.insert(
            id.clone(),
            ResearchGraphNode {
                id: id.clone(),
                graph: graph.into(),
                kind: kind.into(),
                source_table: table.into(),
                source_id: source.into(),
                label: safe_label(label),
                attributes: attrs,
            },
        );
        if self.nodes.len() > MAX_NODES {
            return Err(graph_error(
                "This session exceeds the 5000-node projection budget",
            ));
        }
        Ok(id)
    }
    fn edge(&mut self, source: &str, relation: &str, target: &str) -> Result<()> {
        let id = evidence::digest(format!("{source}|{relation}|{target}").as_bytes());
        self.edges.insert(
            id.clone(),
            ResearchGraphEdge {
                id,
                source: source.into(),
                target: target.into(),
                relation: relation.into(),
            },
        );
        if self.edges.len() > MAX_EDGES {
            return Err(graph_error(
                "This session exceeds the 20000-edge projection budget",
            ));
        }
        Ok(())
    }
}
impl Core {
    pub fn research_graph_record(&self, session: &str, node: &str) -> Result<Value> {
        self.research_session(session)?;
        let conn = self.db.connect()?;
        let source: Option<(String,String,String)> = conn.query_row("SELECT source_table,source_id,attributes_json FROM research_graph_nodes WHERE session_id=?1 AND id=?2",params![session,node],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(db_error)?;
        let (table, id, attributes) = source.ok_or_else(|| {
            AppError::new(
                "NOT_FOUND",
                "This node does not belong to the selected session",
            )
        })?;
        let mut record = match table.as_str() {
            "analysis_jobs" => serde_json::to_value(self.job(&id)?).map_err(json_error)?,
            "evidence" => serde_json::to_value(self.evidence_show(&id)?).map_err(json_error)?,
            "findings" => serde_json::to_value(self.finding(&id)?).map_err(json_error)?,
            "reports" => serde_json::to_value(self.report_show(&id)?.report).map_err(json_error)?,
            _ => serde_json::from_str(&attributes).map_err(json_error)?,
        };
        reporting::redact_value(&mut record);
        if record.to_string().len() > 32768 {
            return Ok(
                json!({"source_table":table,"source_id":id,"omitted":true,"reason":"Record exceeds 32 KiB; inspect it in its original workspace"}),
            );
        }
        Ok(json!({"source_table":table,"source_id":id,"omitted":false,"record":record}))
    }
    fn refresh_research_graph(&self, id: &str) -> Result<()> {
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let s = self.research_session(id)?;
        let project = self.project(&s.plan.preview.project_id)?;
        let target_id = &s.plan.preview.request.target_id;
        // Deleting a target clears live foreign keys, but the reviewed plan keeps
        // its original ID. History reads must not require a currently executable
        // target or reconstruct authorization from historical metadata.
        let target = match self.target(target_id) {
            Ok(target) => Some(target),
            Err(error) if error.code == "NOT_FOUND" => None,
            Err(error) => return Err(error),
        };
        if target
            .as_ref()
            .is_some_and(|t| t.input.project_id != project.id)
        {
            return Err(graph_error(
                "Session target no longer belongs to its project",
            ));
        }
        let mut g = Projection::default();
        let target_value = if let Some(target) = &target {
            if matches!(
                target.input.kind,
                TargetKind::Domain
                    | TargetKind::Subdomain
                    | TargetKind::Url
                    | TargetKind::RestApi
                    | TargetKind::Graphql
                    | TargetKind::Ipv4
                    | TargetKind::Ipv6
                    | TargetKind::Cidr
                    | TargetKind::Hostname
            ) {
                safe_label(&target.input.value)
            } else {
                "Local artifact location omitted".into()
            }
        } else {
            "Source record removed; no current destination".into()
        };
        let p = g.node(
            id,
            "asset",
            "project",
            ("projects", &project.id),
            &project.name,
            json!({"id":project.id}),
        )?;
        let t = g.node(id,"asset","target",("targets",target_id),target.as_ref().map_or(s.plan.preview.target_name.as_str(), |t| t.input.name.as_str()),json!({"kind":target.as_ref().map_or(s.plan.preview.target_kind,|t|t.input.kind),"value":target_value,"scope_status":target.as_ref().map_or("unavailable",|t|t.scope_status.as_str()),"source_state":if target.is_some(){"present"}else{"removed"}}))?;
        g.edge(&t, "BELONGS_TO_PROJECT", &p)?;
        let session = g.node(id,"exploration","session",("research_sessions",id),"Research session",json!({"status":s.status,"created_at":s.plan.created_at,"mode":s.plan.preview.request.mode}))?;
        let plan = g.node(id,"exploration","plan",("research_sessions",id),"Reviewed research plan",json!({"step_count":s.plan.proposal.steps.len(),"fingerprint":s.plan.preview.fingerprint}))?;
        g.edge(&session, "HAS_PLAN", &plan)?;
        g.edge(&session, "INVESTIGATES", &t)?;
        if let Some(at) = &s.approved_at {
            let a = g.node(
                id,
                "exploration",
                "approval",
                ("research_sessions", id),
                "Recorded execution approval",
                json!({"approved_at":at,"fingerprint":s.plan.preview.fingerprint}),
            )?;
            g.edge(&plan, "APPROVED_BY", &a)?;
        }
        if let Some(file_id) = &s.plan.preview.file_id {
            let file:Option<(String,String,Option<String>,String)>=tx.query_row(
                "SELECT name,project_id,target_id,json_object('sha256',sha256,'size',size,'format',format) FROM imported_files WHERE id=?1",
                [file_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(db_error)?;
            let (name, mut attributes) = if let Some((name, owner, file_target, attributes)) = file
            {
                if owner != project.id
                    || !(file_target.as_ref() == Some(target_id)
                        || (target.is_none() && file_target.is_none()))
                {
                    return Err(graph_error(
                        "The session file no longer belongs to its original project and target",
                    ));
                }
                let mut attributes: Value =
                    serde_json::from_str(&attributes).map_err(json_error)?;
                attributes["source_state"] = json!("present");
                (name, attributes)
            } else {
                // Reuse only this session's previously materialized metadata. If
                // never projected, the saved plan proves the ID, not a file hash.
                let previous: Option<(String,String)> = tx.query_row(
                    "SELECT label,attributes_json FROM research_graph_nodes WHERE session_id=?1 AND source_table='imported_files' AND source_id=?2 AND kind='file'",
                    params![id,file_id],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(db_error)?;
                let (name, attributes) =
                    previous.unwrap_or_else(|| ("Removed file record".into(), "{}".into()));
                let mut attributes: Value =
                    serde_json::from_str(&attributes).map_err(json_error)?;
                attributes["source_state"] = json!("removed");
                attributes["integrity"] =
                    json!("Historical metadata only; source record unavailable");
                (name, attributes)
            };
            // Never reconstruct or expose an original local path on history reads.
            reporting::redact_value(&mut attributes);
            let file = g.node(
                id,
                "asset",
                "file",
                ("imported_files", file_id),
                &name,
                attributes,
            )?;
            g.edge(&t, "CONTAINS", &file)?;
        }
        let mut jobs = BTreeMap::new();
        for job in &s.jobs {
            if job.project_id != project.id
                || !(job.target_id.as_ref() == Some(target_id)
                    || (target.is_none() && job.target_id.is_none()))
            {
                return Err(graph_error("Job association crosses session boundaries"));
            }
            let n = g.node(id,"exploration","task",("analysis_jobs",&job.id),&job.kind,json!({"status":job.status,"created_at":job.created_at,"started_at":job.started_at,"finished_at":job.finished_at,"error_code":job.error_code}))?;
            g.edge(&plan, "DECOMPOSES_INTO", &n)?;
            jobs.insert(job.id.clone(), n.clone());
            if job.started_at.is_some() {
                let invocation = g.node(id,"exploration","tool_invocation",("analysis_jobs",&job.id),&job.kind,json!({"status":job.status,"started_at":job.started_at,"finished_at":job.finished_at}))?;
                g.edge(&n, "EXECUTED_BY", &invocation)?;
            }
            if job.status != "completed" {
                continue;
            }
            let result = job.result.as_ref().unwrap_or(&Value::Null);
            let rows = if matches!(job.kind.as_str(), "recon_httpx" | "recon_subfinder") {
                result["observations"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default()
            } else if job.kind == "local_http_inventory" {
                vec![
                    json!({"kind":"http_endpoint","value":result["url"],"facts":result["observation"]}),
                ]
            } else {
                vec![]
            };
            if rows.len() > 1000 {
                return Err(graph_error("Observed-asset budget exceeded"));
            }
            for row in rows {
                let (Some(kind), Some(value)) = (row["kind"].as_str(), row["value"].as_str())
                else {
                    continue;
                };
                if !matches!(kind, "http_endpoint" | "subdomain") || value.len() > 4096 {
                    continue;
                }
                // Resolve a real inventory row, but use this job's immutable result
                // for historical metadata: rediscovery can update assets.job_id.
                let asset:Option<String>=tx.query_row("SELECT id FROM assets a WHERE project_id=?1 AND kind=?3 AND value=?4 AND (target_id=?2 OR (?5 AND target_id IS NULL AND (job_id=?6 OR EXISTS(SELECT 1 FROM research_graph_nodes n WHERE n.session_id=?7 AND n.source_table='assets' AND n.source_id=a.id)))) ORDER BY id LIMIT 1",params![project.id,target_id,kind,value,target.is_none(),job.id,id],|r|r.get(0)).optional().map_err(db_error)?;
                let Some(asset_id) = asset else {
                    continue;
                };
                let facts = &row["facts"];
                let asset_node=g.node(id,"asset",if kind=="subdomain"{"domain"}else{"url"},("assets",&asset_id),value,json!({"kind":kind,"value":safe_label(value),"verification":"observed_unverified"}))?;
                g.edge(
                    &t,
                    if kind == "subdomain" {
                        "DISCOVERED_FROM"
                    } else {
                        "EXPOSES_ENDPOINT"
                    },
                    &asset_node,
                )?;
                let observation=g.node(id,"exploration","observation",("analysis_jobs",&job.id),value,json!({"job_id":job.id,"asset_id":asset_id,"evidence_id":result["evidence_id"],"status":facts["status"],"ip":facts["ip"],"source_tool":job.kind,"integrity":"Not checked by graph; verify linked evidence"}))?;
                g.edge(&n, "PRODUCED", &observation)?;
                g.edge(&observation, "ABOUT", &asset_node)?;
            }
        }
        for job in &s.jobs {
            let dependency: Option<Option<String>> = tx
                .query_row(
                    "SELECT dependency_job_id FROM recon_tool_approvals WHERE job_id=?1",
                    [&job.id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(db_error)?;
            if let Some(Some(d)) = dependency
                && let (Some(a), Some(b)) = (jobs.get(&job.id), jobs.get(&d))
            {
                g.edge(a, "DEPENDS_ON", b)?;
            }
        }
        let mut evidence_nodes = BTreeMap::new();
        for e in &s.evidence {
            if e.project_id != project.id {
                return Err(graph_error("Evidence association crosses projects"));
            }
            let n=g.node(id,"exploration","evidence",("evidence",&e.id),&e.title,json!({"kind":e.kind,"sha256":e.sha256,"verification_state":e.verification_state,"integrity":"Not checked by graph; use evidence verify","created_at":e.created_at}))?;
            g.edge(
                e.job_id
                    .as_ref()
                    .and_then(|j| jobs.get(j))
                    .unwrap_or(&session),
                "PRODUCED",
                &n,
            )?;
            evidence_nodes.insert(e.id.clone(), n);
        }
        let observed_evidence: Vec<(String, String)> = g
            .nodes
            .values()
            .filter(|n| n.kind == "observation")
            .filter_map(|n| {
                n.attributes["evidence_id"]
                    .as_str()
                    .map(|e| (n.id.clone(), e.to_owned()))
            })
            .collect();
        for (observation, evidence) in observed_evidence {
            if let Some(e) = evidence_nodes.get(&evidence) {
                g.edge(&observation, "SUPPORTED_BY", e)?;
            }
        }
        for f in &s.findings {
            let n = g.node(
                id,
                "exploration",
                "finding",
                ("findings", &f.id),
                &f.input.title,
                json!({"status":f.input.status,"severity":f.input.severity,"revision":f.revision}),
            )?;
            g.edge(&session, "HAS_FINDING", &n)?;
            for e in &f.input.evidence_ids {
                if let Some(e) = evidence_nodes.get(e) {
                    g.edge(&n, "SUPPORTED_BY", e)?;
                }
            }
        }
        for report in &s.reports {
            let n = g.node(
                id,
                "exploration",
                "report",
                ("reports", &report.id),
                &report.title,
                json!({"sha256":report.sha256,"created_at":report.created_at}),
            )?;
            g.edge(&session, "HAS_REPORT", &n)?;
        }
        let digest =
            evidence::digest(&serde_json::to_vec(&(&g.nodes, &g.edges)).map_err(json_error)?);
        let previous: Option<String> = tx
            .query_row(
                "SELECT source_sha256 FROM research_graph_snapshots WHERE session_id=?1",
                [id],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        if previous.as_deref() == Some(&digest) {
            return Ok(());
        }
        let stamp = now();
        let ids = serde_json::to_string(&g.nodes.keys().collect::<Vec<_>>()).map_err(json_error)?;
        tx.execute("DELETE FROM research_graph_nodes WHERE session_id=?1 AND id NOT IN(SELECT value FROM json_each(?2))",params![id,ids]).map_err(db_error)?;
        for n in g.nodes.values() {
            tx.execute("INSERT INTO research_graph_nodes VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?9) ON CONFLICT(id) DO UPDATE SET label=excluded.label,attributes_json=excluded.attributes_json,updated_at=excluded.updated_at",params![n.id,id,n.graph,n.kind,n.source_table,n.source_id,n.label,n.attributes.to_string(),stamp]).map_err(db_error)?;
        }
        let ids = serde_json::to_string(&g.edges.keys().collect::<Vec<_>>()).map_err(json_error)?;
        tx.execute("DELETE FROM research_graph_edges WHERE session_id=?1 AND id NOT IN(SELECT value FROM json_each(?2))",params![id,ids]).map_err(db_error)?;
        for e in g.edges.values() {
            tx.execute(
                "INSERT OR IGNORE INTO research_graph_edges VALUES(?1,?2,?3,?4,?5,?6)",
                params![e.id, id, e.source, e.target, e.relation, stamp],
            )
            .map_err(db_error)?;
        }
        tx.execute("INSERT INTO research_graph_snapshots VALUES(?1,1,?2,?3,?4,?5) ON CONFLICT(session_id) DO UPDATE SET revision=revision+1,source_sha256=excluded.source_sha256,updated_at=excluded.updated_at,node_count=excluded.node_count,edge_count=excluded.edge_count",params![id,digest,stamp,g.nodes.len() as i64,g.edges.len() as i64]).map_err(db_error)?;
        Self::audit(
            &tx,
            "research.graph.updated",
            id,
            "materialized_from_records",
        )?;
        tx.commit().map_err(db_error)
    }
    pub fn research_graph(&self, id: &str, q: ResearchGraphQuery) -> Result<ResearchGraphPage> {
        if !matches!(q.graph.as_str(), "" | "asset" | "exploration")
            || q.kind.len() > 40
            || q.search.len() > 128
            || !(1..=200).contains(&q.limit)
            || q.offset > MAX_NODES
        {
            return Err(graph_error(
                "Use asset/exploration filters, at most 128 search bytes and 1–200 nodes per page",
            ));
        }
        self.refresh_research_graph(id)?;
        let mut storage = self.db.connect()?;
        let conn = storage.transaction().map_err(db_error)?;
        let (revision,source_sha256,updated_at,total_nodes,total_edges):(i64,String,String,i64,i64)=conn.query_row("SELECT revision,source_sha256,updated_at,node_count,edge_count FROM research_graph_snapshots WHERE session_id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).map_err(db_error)?;
        let matched_nodes:i64=conn.query_row("SELECT count(*) FROM research_graph_nodes WHERE session_id=?1 AND (?2='' OR graph=?2) AND (?3='' OR kind=?3) AND (?4='' OR instr(lower(label),lower(?4))>0 OR instr(source_id,?4)>0)",params![id,q.graph,q.kind,q.search],|r|r.get(0)).map_err(db_error)?;
        let text:Vec<String>=conn.prepare("SELECT json_object('id',id,'graph',graph,'kind',kind,'source_table',source_table,'source_id',source_id,'label',label,'attributes',json(attributes_json)) FROM research_graph_nodes WHERE session_id=?1 AND (?2='' OR graph=?2) AND (?3='' OR kind=?3) AND (?4='' OR instr(lower(label),lower(?4))>0 OR instr(source_id,?4)>0) ORDER BY graph,kind,id LIMIT ?5 OFFSET ?6").map_err(db_error)?.query_map(params![id,q.graph,q.kind,q.search,q.limit as i64,q.offset as i64],|r|r.get(0)).map_err(db_error)?.collect::<std::result::Result<_,_>>().map_err(db_error)?;
        let total_nodes =
            usize::try_from(total_nodes).map_err(|_| graph_error("Invalid graph count"))?;
        let total_edges =
            usize::try_from(total_edges).map_err(|_| graph_error("Invalid graph count"))?;
        let matched_nodes =
            usize::try_from(matched_nodes).map_err(|_| graph_error("Invalid graph count"))?;
        let nodes: Vec<ResearchGraphNode> = text
            .iter()
            .map(|s| serde_json::from_str(s).map_err(json_error))
            .collect::<Result<_>>()?;
        let ids = serde_json::to_string(&nodes.iter().map(|n| &n.id).collect::<Vec<_>>())
            .map_err(json_error)?;
        let edges:Vec<ResearchGraphEdge>=conn.prepare("SELECT id,source,target,relation FROM research_graph_edges WHERE session_id=?1 AND source IN(SELECT value FROM json_each(?2)) AND target IN(SELECT value FROM json_each(?2)) ORDER BY id LIMIT 1001").map_err(db_error)?.query_map(params![id,ids],|r|Ok(ResearchGraphEdge{id:r.get(0)?,source:r.get(1)?,target:r.get(2)?,relation:r.get(3)?})).map_err(db_error)?.collect::<std::result::Result<_,_>>().map_err(db_error)?;
        let edges_truncated = edges.len() > 1000;
        Ok(ResearchGraphPage {
            session_id: id.into(),
            revision,
            source_sha256,
            updated_at,
            total_nodes,
            total_edges,
            matched_nodes,
            next_offset: (q.offset + nodes.len() < matched_nodes).then_some(q.offset + nodes.len()),
            nodes,
            edges: edges.into_iter().take(1000).collect(),
            edges_truncated,
        })
    }
    pub fn research_graph_export(&self, id: &str, path: &std::path::Path) -> Result<Value> {
        self.refresh_research_graph(id)?;
        let mut storage = self.db.connect()?;
        let conn = storage.transaction().map_err(db_error)?;
        let read = |sql: &str| -> Result<Vec<Value>> {
            conn.prepare(sql)
                .map_err(db_error)?
                .query_map([id], |r| r.get::<_, String>(0))
                .map_err(db_error)?
                .map(|r| serde_json::from_str(&r.map_err(db_error)?).map_err(json_error))
                .collect()
        };
        let nodes = read(
            "SELECT json_object('id',id,'graph',graph,'kind',kind,'source_table',source_table,'source_id',source_id,'label',label,'attributes',json(attributes_json)) FROM research_graph_nodes WHERE session_id=?1 ORDER BY id",
        )?;
        let edges = read(
            "SELECT json_object('id',id,'source',source,'target',target,'relation',relation) FROM research_graph_edges WHERE session_id=?1 ORDER BY id",
        )?;
        let bytes=serde_json::to_vec_pretty(&json!({"schema":1,"session_id":id,"nodes":nodes,"edges":edges,"scope":"Stored provenance only; no AI-generated assets or automatic execution"})).map_err(json_error)?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(graph_error("Graph export exceeds 16 MiB"));
        }
        evidence::export_new(path, &bytes)?;
        Ok(
            json!({"sha256":evidence::digest(&bytes),"bytes":bytes.len(),"nodes":nodes.len(),"edges":edges.len()}),
        )
    }
}
