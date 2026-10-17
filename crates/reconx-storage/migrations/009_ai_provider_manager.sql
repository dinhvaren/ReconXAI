-- 007/008 are reserved for interrupted local development snapshots. Never reuse
-- their version numbers or remove unknown tables from existing workspaces.
CREATE TABLE IF NOT EXISTS ai_profile_metadata (
    provider_id TEXT PRIMARY KEY REFERENCES providers(id) ON DELETE CASCADE,
    vendor_id TEXT NOT NULL DEFAULT 'custom',
    revision INTEGER NOT NULL DEFAULT 0,
    models_json TEXT NOT NULL DEFAULT '[]',
    models_fingerprint TEXT,
    models_refreshed_at TEXT,
    connection_json TEXT,
    options_json TEXT NOT NULL DEFAULT '{}',
    header_names_json TEXT NOT NULL DEFAULT '[]'
);
CREATE TABLE IF NOT EXISTS ai_agent_routing (
    agent_id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL REFERENCES providers(id) ON DELETE CASCADE
);
