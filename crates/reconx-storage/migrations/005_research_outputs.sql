CREATE TABLE evidence (
 id TEXT PRIMARY KEY,
 project_id TEXT NOT NULL REFERENCES projects(id),
 target_id TEXT REFERENCES targets(id) ON DELETE SET NULL,
 job_id TEXT REFERENCES analysis_jobs(id) ON DELETE SET NULL,
 kind TEXT NOT NULL,
 title TEXT NOT NULL,
 created_at TEXT NOT NULL,
 metadata_json TEXT NOT NULL CHECK(json_valid(metadata_json)),
 UNIQUE(job_id,kind)
);
CREATE INDEX evidence_project_kind ON evidence(project_id,kind,created_at);
CREATE INDEX evidence_target ON evidence(target_id);
CREATE TABLE findings (
 id TEXT PRIMARY KEY,
 project_id TEXT NOT NULL REFERENCES projects(id),
 target_id TEXT REFERENCES targets(id) ON DELETE SET NULL,
 title TEXT NOT NULL,
 status TEXT NOT NULL,
 severity TEXT NOT NULL,
 revision INTEGER NOT NULL CHECK(revision>0),
 created_at TEXT NOT NULL,
 updated_at TEXT NOT NULL,
 duplicate_of TEXT REFERENCES findings(id) ON DELETE SET NULL,
 data_json TEXT NOT NULL CHECK(json_valid(data_json))
);
CREATE INDEX findings_project_status ON findings(project_id,status,severity);
CREATE TABLE finding_evidence (
 finding_id TEXT NOT NULL REFERENCES findings(id) ON DELETE CASCADE,
 evidence_id TEXT NOT NULL REFERENCES evidence(id),
 PRIMARY KEY(finding_id,evidence_id)
);
CREATE TABLE reports (
 id TEXT PRIMARY KEY,
 project_id TEXT NOT NULL REFERENCES projects(id),
 title TEXT NOT NULL,
 template TEXT NOT NULL,
 created_at TEXT NOT NULL,
 sha256 TEXT NOT NULL CHECK(length(sha256)=64),
 size INTEGER NOT NULL CHECK(size>=0)
);
CREATE INDEX reports_project ON reports(project_id,created_at);
CREATE TABLE report_exports (
 id TEXT PRIMARY KEY,
 report_id TEXT NOT NULL REFERENCES reports(id),
 format TEXT NOT NULL,
 sha256 TEXT NOT NULL,
 created_at TEXT NOT NULL
);
