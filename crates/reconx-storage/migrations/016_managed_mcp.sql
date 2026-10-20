CREATE TABLE managed_mcp_extensions (
 extension_id TEXT PRIMARY KEY REFERENCES extension_installations(extension_id) ON DELETE CASCADE,
 server_id TEXT NOT NULL UNIQUE,
 descriptor_json TEXT NOT NULL CHECK(json_valid(descriptor_json))
);
CREATE TABLE extension_install_jobs (
 job_id TEXT PRIMARY KEY REFERENCES analysis_jobs(id) ON DELETE CASCADE,
 extension_id TEXT NOT NULL,
 plan_json TEXT NOT NULL CHECK(json_valid(plan_json)),
 previous_state_json TEXT CHECK(previous_state_json IS NULL OR json_valid(previous_state_json)),
 previous_config_revision TEXT,
 reservation_revision TEXT NOT NULL,
 staged_json TEXT CHECK(staged_json IS NULL OR json_valid(staged_json)),
 reconciled INTEGER NOT NULL DEFAULT 0 CHECK(reconciled IN(0,1))
);
CREATE INDEX extension_install_jobs_extension ON extension_install_jobs(extension_id);
