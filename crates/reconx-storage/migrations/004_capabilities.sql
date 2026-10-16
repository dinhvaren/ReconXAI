-- Additive migration. Existing projects, credentials references and jobs are untouched.
CREATE TABLE mcp_servers (
 id TEXT PRIMARY KEY,
 config_json TEXT NOT NULL CHECK(json_valid(config_json)),
 revision TEXT NOT NULL,
 tools_json TEXT NOT NULL DEFAULT '[]' CHECK(json_valid(tools_json)),
 last_connected_at TEXT,
 updated_at TEXT NOT NULL
);
CREATE TABLE capability_invocations (
 job_id TEXT PRIMARY KEY REFERENCES analysis_jobs(id) ON DELETE CASCADE,
 server_id TEXT NOT NULL,
 tool_name TEXT NOT NULL,
 input_json TEXT NOT NULL CHECK(json_valid(input_json)),
 approval_fingerprint TEXT NOT NULL,
 approved_at TEXT NOT NULL
);
CREATE INDEX capability_invocations_server ON capability_invocations(server_id,approved_at);
CREATE TABLE tool_installations (
 tool_id TEXT PRIMARY KEY,
 state_json TEXT NOT NULL CHECK(json_valid(state_json)),
 updated_at TEXT NOT NULL
);
CREATE TABLE ai_tool_plans (
 id TEXT PRIMARY KEY,
 project_id TEXT NOT NULL REFERENCES projects(id),
 file_id TEXT REFERENCES imported_files(id) ON DELETE SET NULL,
 plan_json TEXT NOT NULL CHECK(json_valid(plan_json)),
 created_at TEXT NOT NULL,
 executed_at TEXT,
 jobs_json TEXT NOT NULL DEFAULT '[]' CHECK(json_valid(jobs_json))
);
CREATE TABLE ai_tool_interpretations (
 id TEXT PRIMARY KEY,
 job_id TEXT NOT NULL REFERENCES analysis_jobs(id) ON DELETE CASCADE,
 provider_id TEXT NOT NULL,
 response_json TEXT NOT NULL CHECK(json_valid(response_json)),
 created_at TEXT NOT NULL
);
