-- Shared IDs and a session ledger reuse existing projects, jobs and evidence.
CREATE TABLE workspace_selection (
 id INTEGER PRIMARY KEY CHECK(id=1),
 revision INTEGER NOT NULL DEFAULT 0,
 project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
 target_id TEXT REFERENCES targets(id) ON DELETE SET NULL
);
INSERT INTO workspace_selection(id) VALUES(1);
CREATE TRIGGER workspace_target_removed BEFORE DELETE ON targets BEGIN
 UPDATE workspace_selection SET target_id=NULL,revision=revision+1 WHERE target_id=OLD.id;
END;
CREATE TRIGGER workspace_project_removed BEFORE DELETE ON projects BEGIN
 UPDATE workspace_selection SET project_id=NULL,target_id=NULL,revision=revision+1 WHERE project_id=OLD.id;
END;
CREATE TRIGGER workspace_project_archived AFTER UPDATE OF archived ON projects WHEN NEW.archived=1 BEGIN
 UPDATE workspace_selection SET project_id=NULL,target_id=NULL,revision=revision+1 WHERE project_id=NEW.id;
END;
CREATE TABLE research_sessions (
 id TEXT PRIMARY KEY,
 project_id TEXT NOT NULL REFERENCES projects(id),
 target_id TEXT REFERENCES targets(id) ON DELETE SET NULL,
 provider_id TEXT,
 created_at TEXT NOT NULL,
 approved_at TEXT,
 cancelled_at TEXT,
 plan_json TEXT NOT NULL CHECK(json_valid(plan_json)),
 reply_json TEXT CHECK(reply_json IS NULL OR json_valid(reply_json))
);
CREATE INDEX research_project ON research_sessions(project_id,created_at);
CREATE TABLE research_session_jobs (
 session_id TEXT NOT NULL REFERENCES research_sessions(id),
 step INTEGER NOT NULL,
 job_id TEXT NOT NULL UNIQUE REFERENCES analysis_jobs(id),
 PRIMARY KEY(session_id,step)
);
CREATE TABLE research_session_records (
 session_id TEXT NOT NULL REFERENCES research_sessions(id),
 record_id TEXT NOT NULL,
 kind TEXT NOT NULL CHECK(kind IN ('finding','report','interpretation')),
 created_at TEXT NOT NULL,
 PRIMARY KEY(session_id,record_id,kind)
);
