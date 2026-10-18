CREATE TABLE http_imports (
 id TEXT PRIMARY KEY,
 project_id TEXT NOT NULL REFERENCES projects(id),
 target_id TEXT REFERENCES targets(id) ON DELETE SET NULL,
 kind TEXT NOT NULL CHECK(kind IN ('har','openapi')),
 title TEXT NOT NULL,
 sha256 TEXT NOT NULL CHECK(length(sha256)=64),
 created_at TEXT NOT NULL,
 evidence_id TEXT NOT NULL REFERENCES evidence(id),
 summary_json TEXT NOT NULL CHECK(json_valid(summary_json))
);
CREATE INDEX http_import_project ON http_imports(project_id,created_at);
CREATE TABLE http_exchanges (
 id TEXT PRIMARY KEY,
 import_id TEXT NOT NULL REFERENCES http_imports(id) ON DELETE CASCADE,
 project_id TEXT NOT NULL REFERENCES projects(id),
 target_id TEXT REFERENCES targets(id) ON DELETE SET NULL,
 host TEXT NOT NULL,
 method TEXT NOT NULL,
 path TEXT NOT NULL,
 status INTEGER NOT NULL,
 data_json TEXT NOT NULL CHECK(json_valid(data_json))
);
CREATE INDEX http_exchange_project ON http_exchanges(project_id,host,method,path);
CREATE INDEX http_exchange_target ON http_exchanges(target_id);
CREATE TABLE api_operations (
 id TEXT PRIMARY KEY,
 import_id TEXT NOT NULL REFERENCES http_imports(id) ON DELETE CASCADE,
 project_id TEXT NOT NULL REFERENCES projects(id),
 target_id TEXT REFERENCES targets(id) ON DELETE SET NULL,
 method TEXT NOT NULL,
 path TEXT NOT NULL,
 data_json TEXT NOT NULL CHECK(json_valid(data_json))
);
CREATE INDEX api_operation_project ON api_operations(project_id,method,path);
CREATE INDEX api_operation_target ON api_operations(target_id);
CREATE TABLE http_notes (
 exchange_id TEXT PRIMARY KEY REFERENCES http_exchanges(id) ON DELETE CASCADE,
 note TEXT NOT NULL,
 updated_at TEXT NOT NULL
);
CREATE TABLE http_comparisons (
 id TEXT PRIMARY KEY,
 project_id TEXT NOT NULL REFERENCES projects(id),
 evidence_id TEXT NOT NULL REFERENCES evidence(id),
 created_at TEXT NOT NULL,
 data_json TEXT NOT NULL CHECK(json_valid(data_json))
);
CREATE TABLE http_review_jobs (
 job_id TEXT PRIMARY KEY REFERENCES analysis_jobs(id) ON DELETE CASCADE,
 exchange_id TEXT NOT NULL REFERENCES http_exchanges(id),
 fingerprint TEXT NOT NULL
);
CREATE INDEX http_review_exchange ON http_review_jobs(exchange_id);
