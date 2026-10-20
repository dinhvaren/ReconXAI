-- Registrations never contain credentials or downloaded runtime code.
CREATE TABLE extension_installations (
 extension_id TEXT PRIMARY KEY,
 state_json TEXT NOT NULL CHECK(json_valid(state_json))
);
CREATE TABLE extension_requirements (
 extension_id TEXT NOT NULL REFERENCES extension_installations(extension_id) ON DELETE CASCADE,
 tool_id TEXT NOT NULL,
 PRIMARY KEY(extension_id,tool_id)
);
-- Invocation history survives extension uninstall; jobs remain the execution authority.
CREATE TABLE extension_invocations (
 job_id TEXT PRIMARY KEY REFERENCES analysis_jobs(id) ON DELETE CASCADE,
 extension_id TEXT NOT NULL,
 revision TEXT NOT NULL,
 approved_at TEXT NOT NULL
);
CREATE INDEX extension_invocations_history ON extension_invocations(extension_id,approved_at);
