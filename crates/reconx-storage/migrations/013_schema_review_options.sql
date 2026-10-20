-- Legacy queued/completed schema jobs keep their original fingerprint and have
-- no options row. New jobs bind explicit options into their existing fingerprint.
CREATE TABLE http_schema_job_options (
 job_id TEXT PRIMARY KEY REFERENCES http_schema_jobs(job_id) ON DELETE CASCADE,
 options_json TEXT NOT NULL CHECK(json_valid(options_json))
);
