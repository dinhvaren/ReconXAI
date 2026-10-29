-- Materialized audit graphs over existing records. This is not an execution queue.
CREATE TABLE research_graph_nodes (
 id TEXT PRIMARY KEY,
 session_id TEXT NOT NULL REFERENCES research_sessions(id) ON DELETE CASCADE,
 graph TEXT NOT NULL CHECK(graph IN ('asset','exploration')),
 kind TEXT NOT NULL,
 source_table TEXT NOT NULL,
 source_id TEXT NOT NULL,
 label TEXT NOT NULL,
 attributes_json TEXT NOT NULL CHECK(json_valid(attributes_json)),
 created_at TEXT NOT NULL,
 updated_at TEXT NOT NULL,
 UNIQUE(session_id,id)
);
CREATE TABLE research_graph_edges (
 id TEXT PRIMARY KEY,
 session_id TEXT NOT NULL REFERENCES research_sessions(id) ON DELETE CASCADE,
 source TEXT NOT NULL,
 target TEXT NOT NULL,
 relation TEXT NOT NULL,
 created_at TEXT NOT NULL,
 FOREIGN KEY(session_id,source) REFERENCES research_graph_nodes(session_id,id) ON DELETE CASCADE,
 FOREIGN KEY(session_id,target) REFERENCES research_graph_nodes(session_id,id) ON DELETE CASCADE,
 UNIQUE(session_id,source,target,relation)
);
CREATE TABLE research_graph_snapshots (
 session_id TEXT PRIMARY KEY REFERENCES research_sessions(id) ON DELETE CASCADE,
 revision INTEGER NOT NULL CHECK(revision>0),
 source_sha256 TEXT NOT NULL,
 updated_at TEXT NOT NULL,
 node_count INTEGER NOT NULL,
 edge_count INTEGER NOT NULL
);
CREATE INDEX research_graph_filter ON research_graph_nodes(session_id,graph,kind,id);
CREATE INDEX research_graph_edge_page ON research_graph_edges(session_id,source,target);
