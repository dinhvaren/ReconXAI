CREATE TABLE targets (
 id TEXT PRIMARY KEY, project_id TEXT NOT NULL REFERENCES projects(id),
 name TEXT NOT NULL, kind TEXT NOT NULL, value TEXT NOT NULL,
 json TEXT NOT NULL CHECK(json_valid(json)), created_at TEXT NOT NULL,
 updated_at TEXT NOT NULL, last_analyzed_at TEXT,
 UNIQUE(project_id, kind, value)
);
CREATE INDEX targets_project_type ON targets(project_id, kind);
CREATE TABLE imported_files (
 id TEXT PRIMARY KEY, project_id TEXT NOT NULL REFERENCES projects(id),
 target_id TEXT REFERENCES targets(id) ON DELETE SET NULL,
 name TEXT NOT NULL, source_value TEXT NOT NULL, sha256 TEXT NOT NULL CHECK(length(sha256)=64),
 size INTEGER NOT NULL CHECK(size>=0), format TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE INDEX imported_files_project ON imported_files(project_id, created_at);
CREATE TABLE analysis_jobs (
 id TEXT PRIMARY KEY, project_id TEXT NOT NULL REFERENCES projects(id),
 target_id TEXT REFERENCES targets(id) ON DELETE SET NULL,
 file_id TEXT REFERENCES imported_files(id) ON DELETE SET NULL,
 kind TEXT NOT NULL, status TEXT NOT NULL CHECK(status IN ('running','completed','failed','cancelled')),
 created_at TEXT NOT NULL, finished_at TEXT, error_code TEXT,
 logs_json TEXT NOT NULL CHECK(json_valid(logs_json))
);
CREATE INDEX analysis_jobs_project ON analysis_jobs(project_id, created_at);
CREATE INDEX analysis_jobs_target ON analysis_jobs(target_id);
CREATE TABLE binary_analyses (
 id TEXT PRIMARY KEY, file_id TEXT NOT NULL REFERENCES imported_files(id) ON DELETE CASCADE,
 job_id TEXT NOT NULL UNIQUE REFERENCES analysis_jobs(id),
 created_at TEXT NOT NULL, artifact_name TEXT NOT NULL, artifact_sha256 TEXT NOT NULL
);
CREATE INDEX binary_analyses_file ON binary_analyses(file_id, created_at);
