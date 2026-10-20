-- Cache is public, untrusted metadata; it never grants package execution rights.
CREATE TABLE mcp_registry_cache (
 cache_key TEXT PRIMARY KEY, cache_schema INTEGER NOT NULL,
 api_version TEXT NOT NULL, body_json TEXT NOT NULL CHECK(json_valid(body_json)),
 fetched_at TEXT NOT NULL, expires_at TEXT NOT NULL
);
CREATE TABLE mcp_registry_entries (
 server_id TEXT NOT NULL, version TEXT NOT NULL,
 server_json TEXT NOT NULL CHECK(json_valid(server_json)),
 registry_metadata TEXT NOT NULL CHECK(json_valid(registry_metadata)),
 fetched_at TEXT NOT NULL, expires_at TEXT NOT NULL,
 PRIMARY KEY(server_id,version)
);
