-- Preserve stable job/result IDs while replacing the old four-state constraint.
-- The migration runs in one IMMEDIATE transaction with foreign keys enabled.
CREATE TEMP TABLE retained_binary_analyses AS SELECT * FROM binary_analyses;
DROP TABLE binary_analyses;
CREATE TABLE analysis_jobs_next (
 id TEXT PRIMARY KEY, project_id TEXT NOT NULL REFERENCES projects(id),
 target_id TEXT REFERENCES targets(id) ON DELETE SET NULL,
 file_id TEXT REFERENCES imported_files(id) ON DELETE SET NULL,
 kind TEXT NOT NULL,
 status TEXT NOT NULL CHECK(status IN ('queued','running','completed','failed','cancelled')),
 created_at TEXT NOT NULL, started_at TEXT, finished_at TEXT,
 timeout_ms INTEGER NOT NULL DEFAULT 60000 CHECK(timeout_ms BETWEEN 1 AND 3600000),
 progress INTEGER NOT NULL DEFAULT 0 CHECK(progress BETWEEN 0 AND 100),
 stage TEXT NOT NULL DEFAULT 'Queued', error_code TEXT, error_message TEXT,
 worker_id TEXT, lease_until INTEGER,
 cancel_requested INTEGER NOT NULL DEFAULT 0 CHECK(cancel_requested IN (0,1)),
 result_json TEXT CHECK(result_json IS NULL OR json_valid(result_json)),
 logs_json TEXT NOT NULL CHECK(json_valid(logs_json))
);
INSERT INTO analysis_jobs_next(id,project_id,target_id,file_id,kind,status,created_at,started_at,
 finished_at,error_code,logs_json,progress,stage)
 SELECT id,project_id,target_id,file_id,kind,status,created_at,created_at,
 finished_at,error_code,logs_json,CASE WHEN status='completed' THEN 100 ELSE 0 END,
 CASE WHEN status='completed' THEN 'Completed' ELSE 'Imported legacy job' END FROM analysis_jobs;
DROP TABLE analysis_jobs;
ALTER TABLE analysis_jobs_next RENAME TO analysis_jobs;
CREATE INDEX analysis_jobs_project ON analysis_jobs(project_id,created_at);
CREATE INDEX analysis_jobs_target ON analysis_jobs(target_id);
CREATE INDEX analysis_jobs_queue ON analysis_jobs(status,created_at);
CREATE INDEX analysis_jobs_lease ON analysis_jobs(status,lease_until);
CREATE TABLE binary_analyses (
 id TEXT PRIMARY KEY, file_id TEXT NOT NULL REFERENCES imported_files(id) ON DELETE CASCADE,
 job_id TEXT NOT NULL UNIQUE REFERENCES analysis_jobs(id),
 created_at TEXT NOT NULL, artifact_name TEXT NOT NULL, artifact_sha256 TEXT NOT NULL
);
INSERT INTO binary_analyses SELECT * FROM retained_binary_analyses;
DROP TABLE retained_binary_analyses;
CREATE INDEX binary_analyses_file ON binary_analyses(file_id,created_at);
CREATE TABLE file_notes (
 file_id TEXT PRIMARY KEY REFERENCES imported_files(id) ON DELETE CASCADE,
 note TEXT NOT NULL CHECK(length(note)<=32768), updated_at TEXT NOT NULL
);
