CREATE TABLE http_auth_profiles (
 id TEXT PRIMARY KEY,
 project_id TEXT NOT NULL REFERENCES projects(id),
 target_id TEXT REFERENCES targets(id) ON DELETE SET NULL,
 revision INTEGER NOT NULL CHECK(revision>0),
 data_json TEXT NOT NULL CHECK(json_valid(data_json)),
 secret_ref TEXT,
 expired INTEGER NOT NULL DEFAULT 0 CHECK(expired IN (0,1)),
 last_check_json TEXT CHECK(last_check_json IS NULL OR json_valid(last_check_json))
);
CREATE INDEX http_auth_project ON http_auth_profiles(project_id,target_id);
CREATE TABLE http_auth_retired_secrets (
 secret_ref TEXT PRIMARY KEY
);
CREATE TABLE http_request_drafts (
 id TEXT PRIMARY KEY,
 project_id TEXT NOT NULL REFERENCES projects(id),
 target_id TEXT REFERENCES targets(id) ON DELETE SET NULL,
 source_exchange_id TEXT REFERENCES http_exchanges(id),
 revision INTEGER NOT NULL CHECK(revision>0),
 data_json TEXT NOT NULL CHECK(json_valid(data_json)),
 updated_at TEXT NOT NULL
);
CREATE INDEX http_draft_target ON http_request_drafts(project_id,target_id,updated_at);
CREATE TABLE http_replay_jobs (
 job_id TEXT PRIMARY KEY REFERENCES analysis_jobs(id) ON DELETE CASCADE,
 draft_id TEXT NOT NULL REFERENCES http_request_drafts(id),
 source_exchange_id TEXT REFERENCES http_exchanges(id),
 fingerprint TEXT NOT NULL CHECK(length(fingerprint)=64),
 approved_at TEXT NOT NULL,
 exchange_id TEXT REFERENCES http_exchanges(id)
);
CREATE INDEX http_replay_draft ON http_replay_jobs(draft_id,approved_at);
CREATE TABLE http_replay_dispatch (
 project_id TEXT PRIMARY KEY REFERENCES projects(id),
 started_at_ms INTEGER NOT NULL
);
