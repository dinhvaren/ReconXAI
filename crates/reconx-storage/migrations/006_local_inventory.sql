CREATE TABLE local_http_approvals (
 job_id TEXT PRIMARY KEY REFERENCES analysis_jobs(id) ON DELETE CASCADE,
 fingerprint TEXT NOT NULL,
 approved_at TEXT NOT NULL
);
CREATE INDEX local_http_approval_budget ON local_http_approvals(approved_at);
CREATE TABLE assets (
 id TEXT PRIMARY KEY,
 project_id TEXT NOT NULL REFERENCES projects(id),
 target_id TEXT REFERENCES targets(id) ON DELETE SET NULL,
 kind TEXT NOT NULL,
 value TEXT NOT NULL,
 first_seen TEXT NOT NULL,
 last_seen TEXT NOT NULL,
 job_id TEXT NOT NULL REFERENCES analysis_jobs(id),
 facts_json TEXT NOT NULL CHECK(json_valid(facts_json)),
 UNIQUE(project_id,target_id,kind,value)
);
CREATE INDEX assets_project_target ON assets(project_id,target_id,last_seen);
