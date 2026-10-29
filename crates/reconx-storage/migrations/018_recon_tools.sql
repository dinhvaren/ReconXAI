CREATE TABLE recon_tool_approvals (
 job_id TEXT PRIMARY KEY REFERENCES analysis_jobs(id),
 plan_json TEXT NOT NULL CHECK(json_valid(plan_json)),
 dependency_job_id TEXT REFERENCES analysis_jobs(id),
 approved_at TEXT NOT NULL
);
CREATE INDEX recon_dependency ON recon_tool_approvals(dependency_job_id);
