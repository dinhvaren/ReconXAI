-- Reuses analysis_jobs, its worker leases, logs, cancellation and result evidence.
CREATE TABLE http_schema_jobs (
 job_id TEXT PRIMARY KEY REFERENCES analysis_jobs(id) ON DELETE CASCADE,
 exchange_id TEXT NOT NULL REFERENCES http_exchanges(id),
 operation_id TEXT NOT NULL REFERENCES api_operations(id),
 fingerprint TEXT NOT NULL CHECK(length(fingerprint)=64)
);
CREATE INDEX http_schema_exchange ON http_schema_jobs(exchange_id,operation_id);
