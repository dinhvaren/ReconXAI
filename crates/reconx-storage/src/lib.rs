use reconx_security::{AppError, Result};
pub use rusqlite;
use rusqlite::{Connection, TransactionBehavior};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Debug, Clone)]
pub struct Database {
    path: PathBuf,
}
pub fn db_error(_: rusqlite::Error) -> AppError {
    AppError::new(
        "DATABASE_ERROR",
        "Database operation failed; check workspace permissions and integrity",
    )
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)
                .map_err(|_| AppError::new("WORKSPACE_IO", "Cannot create workspace directory"))?;
        }
        let database = Self {
            path: path.to_owned(),
        };
        let mut conn = database.connect()?;
        let previous: i64 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(db_error)?;
        if (1..10).contains(&previous) {
            // VACUUM INTO takes a consistent SQLite snapshot, including committed WAL data.
            // It refuses to overwrite a file. Backups live beside the private database.
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| AppError::new("BACKUP_FAILED", "System clock is invalid"))?
                .as_nanos();
            let backup = path.with_extension(format!("pre-v10-{stamp}.bak"));
            conn.execute("VACUUM INTO ?1", [backup.to_string_lossy().as_ref()])
                .map_err(|_| {
                    AppError::new(
                        "BACKUP_FAILED",
                        "Cannot back up database before migration; no schema changes applied",
                    )
                })?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&backup, std::fs::Permissions::from_mode(0o600)).map_err(
                    |_| AppError::new("BACKUP_FAILED", "Cannot protect migration backup"),
                )?;
            }
        }
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let version: i64 = tx
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(db_error)?;
        if version > 10 {
            return Err(AppError::new(
                "SCHEMA_TOO_NEW",
                "Database was created by a newer ReconX version; no migration was performed",
            ));
        }
        if version == 0 {
            tx.execute_batch(include_str!("../migrations/001_foundation.sql"))
                .map_err(db_error)?;
            tx.pragma_update(None, "user_version", 1)
                .map_err(db_error)?;
        }
        if version < 2 {
            tx.execute_batch(include_str!("../migrations/002_research_inputs.sql"))
                .map_err(db_error)?;
            tx.pragma_update(None, "user_version", 2)
                .map_err(db_error)?;
        }
        if version < 3 {
            tx.execute_batch(include_str!("../migrations/003_job_engine.sql"))
                .map_err(db_error)?;
            tx.pragma_update(None, "user_version", 3)
                .map_err(db_error)?;
        }
        if version < 4 {
            tx.execute_batch(include_str!("../migrations/004_capabilities.sql"))
                .map_err(db_error)?;
            tx.pragma_update(None, "user_version", 4)
                .map_err(db_error)?;
        }
        if version < 5 {
            tx.execute_batch(include_str!("../migrations/005_research_outputs.sql"))
                .map_err(db_error)?;
            tx.pragma_update(None, "user_version", 5)
                .map_err(db_error)?;
        }
        if version < 6 {
            tx.execute_batch(include_str!("../migrations/006_local_inventory.sql"))
                .map_err(db_error)?;
            tx.pragma_update(None, "user_version", 6)
                .map_err(db_error)?;
        }
        if version < 9 {
            tx.execute_batch(include_str!("../migrations/009_ai_provider_manager.sql"))
                .map_err(db_error)?;
            // Earlier interrupted snapshots may contain the first metadata table.
            let columns: Vec<String> = {
                let mut q = tx
                    .prepare("PRAGMA table_info(ai_profile_metadata)")
                    .map_err(db_error)?;
                q.query_map([], |r| r.get(1))
                    .map_err(db_error)?
                    .collect::<std::result::Result<_, _>>()
                    .map_err(db_error)?
            };
            for (name, definition) in [
                ("revision", "INTEGER NOT NULL DEFAULT 0"),
                ("options_json", "TEXT NOT NULL DEFAULT '{}'"),
                ("header_names_json", "TEXT NOT NULL DEFAULT '[]'"),
            ] {
                if !columns.iter().any(|c| c == name) {
                    tx.execute_batch(&format!(
                        "ALTER TABLE ai_profile_metadata ADD COLUMN {name} {definition}"
                    ))
                    .map_err(db_error)?;
                }
            }
            tx.pragma_update(None, "user_version", 9)
                .map_err(db_error)?;
        }
        if version < 10 {
            tx.execute_batch(include_str!("../migrations/010_http_workspace.sql"))
                .map_err(db_error)?;
            tx.pragma_update(None, "user_version", 10)
                .map_err(db_error)?;
        }
        tx.commit().map_err(db_error)?;
        Ok(database)
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn connect(&self) -> Result<Connection> {
        let conn = Connection::open(&self.path).map_err(db_error)?;
        conn.busy_timeout(Duration::from_secs(5))
            .map_err(db_error)?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(db_error)?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(db_error)?;
        Ok(conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn version_nine_http_migration_preserves_projects_providers_and_agent_routes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("v9.db");
        let db = Database::open(&path).unwrap();
        let conn = db.connect().unwrap();
        conn.execute_batch("DROP TABLE http_review_jobs; DROP TABLE http_comparisons; DROP TABLE http_notes; DROP TABLE api_operations; DROP TABLE http_exchanges; DROP TABLE http_imports; INSERT INTO projects(id,name,description,created_at,updated_at) VALUES('retained','Retained project','Notes stay intact','before','before'); INSERT INTO providers(id,name,json) VALUES('profile','Retained model','{\"model\":\"fixture\"}'); INSERT INTO ai_profile_metadata(provider_id,vendor_id,options_json) VALUES('profile','custom','{\"streaming\":false}'); INSERT INTO ai_agent_routing VALUES('web','profile');").unwrap();
        conn.pragma_update(None, "user_version", 9).unwrap();
        drop(conn);
        drop(db);
        let db = Database::open(&path).unwrap();
        let conn = db.connect().unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT description FROM projects WHERE id='retained'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "Notes stay intact"
        );
        assert_eq!(
            conn.query_row(
                "SELECT options_json FROM ai_profile_metadata WHERE provider_id='profile'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "{\"streaming\":false}"
        );
        assert_eq!(
            conn.query_row(
                "SELECT provider_id FROM ai_agent_routing WHERE agent_id='web'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "profile"
        );
        assert_eq!(
            conn.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
                .unwrap(),
            10
        );
        assert_eq!(
            conn.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r
                .get::<_, i64>(
                0
            ))
            .unwrap(),
            0
        );
        assert!(
            std::fs::read_dir(dir.path())
                .unwrap()
                .flatten()
                .any(|e| e.file_name().to_string_lossy().contains("pre-v10"))
        );
    }
    #[test]
    fn additive_ai_upgrade_preserves_interrupted_snapshot_tables_and_creates_backup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("snapshot.db");
        let db = Database::open(&path).unwrap();
        let conn = db.connect().unwrap();
        conn.execute_batch("CREATE TABLE retained_http_artifact(id TEXT PRIMARY KEY,sha256 TEXT);INSERT INTO retained_http_artifact VALUES('retained','unchanged');DROP TABLE ai_agent_routing;DROP TABLE ai_profile_metadata;CREATE TABLE ai_profile_metadata(provider_id TEXT PRIMARY KEY REFERENCES providers(id),vendor_id TEXT NOT NULL DEFAULT 'custom',models_json TEXT NOT NULL DEFAULT '[]',models_fingerprint TEXT,models_refreshed_at TEXT,connection_json TEXT);").unwrap();
        conn.execute_batch("DROP TABLE http_review_jobs; DROP TABLE http_comparisons; DROP TABLE http_notes; DROP TABLE api_operations; DROP TABLE http_exchanges; DROP TABLE http_imports;").unwrap();
        conn.pragma_update(None, "user_version", 8).unwrap();
        drop(conn);
        drop(db);
        let db = Database::open(&path).unwrap();
        let conn = db.connect().unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT sha256 FROM retained_http_artifact WHERE id='retained'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "unchanged"
        );
        assert_eq!(
            conn.pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
                .unwrap(),
            10
        );
        for column in ["options_json", "header_names_json", "revision"] {
            assert!(
                conn.prepare(&format!("SELECT {column} FROM ai_profile_metadata"))
                    .is_ok()
            );
        }
        assert!(
            std::fs::read_dir(dir.path())
                .unwrap()
                .flatten()
                .any(|p| p.file_name().to_string_lossy().contains("pre-v10"))
        );
    }
    #[test]
    fn version_five_outputs_survive_local_inventory_migration_and_backup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("outputs.db");
        let conn = Connection::open(&path).unwrap();
        for sql in [
            include_str!("../migrations/001_foundation.sql"),
            include_str!("../migrations/002_research_inputs.sql"),
            include_str!("../migrations/003_job_engine.sql"),
            include_str!("../migrations/004_capabilities.sql"),
            include_str!("../migrations/005_research_outputs.sql"),
        ] {
            conn.execute_batch(sql).unwrap();
        }
        conn.execute("INSERT INTO projects(id,name,description,created_at,updated_at,archived) VALUES('owned','Keep outputs','','2026-10-16','2026-10-16',0)", []).unwrap();
        conn.execute("INSERT INTO evidence(id,project_id,kind,title,created_at,metadata_json) VALUES('source','owned','user_note','Retained source','2026-10-16','{\"sha256\":\"retained\"}')", []).unwrap();
        conn.pragma_update(None, "user_version", 5).unwrap();
        drop(conn);
        let db = Database::open(&path).unwrap();
        let backup = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|r| r.ok())
            .find(|e| e.file_name().to_string_lossy().ends_with(".bak"))
            .unwrap();
        let snapshot =
            Connection::open_with_flags(backup.path(), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .unwrap();
        assert_eq!(
            snapshot
                .pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
                .unwrap(),
            5
        );
        for conn in [snapshot, db.connect().unwrap()] {
            assert_eq!(
                conn.query_row(
                    "SELECT metadata_json FROM evidence WHERE id='source'",
                    [],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
                "{\"sha256\":\"retained\"}"
            );
            assert!(
                conn.prepare("PRAGMA foreign_key_check")
                    .unwrap()
                    .query([])
                    .unwrap()
                    .next()
                    .unwrap()
                    .is_none()
            );
        }
        assert_eq!(
            db.connect()
                .unwrap()
                .query_row("SELECT count(*) FROM assets", [], |r| r.get::<_, u32>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            db.connect()
                .unwrap()
                .pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
                .unwrap(),
            10
        );
    }
    #[test]
    fn version_four_research_output_migration_backs_up_and_preserves_capabilities() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("upgrade.db");
        let conn = Connection::open(&path).unwrap();
        for sql in [
            include_str!("../migrations/001_foundation.sql"),
            include_str!("../migrations/002_research_inputs.sql"),
            include_str!("../migrations/003_job_engine.sql"),
            include_str!("../migrations/004_capabilities.sql"),
        ] {
            conn.execute_batch(sql).unwrap();
        }
        conn.execute("INSERT INTO tool_installations(tool_id,state_json,updated_at) VALUES('owned','{\"state\":\"retained\"}','2026-10-16')",[]).unwrap();
        conn.pragma_update(None, "user_version", 4).unwrap();
        drop(conn);
        let db = Database::open(&path).unwrap();
        assert_eq!(
            db.connect()
                .unwrap()
                .query_row(
                    "SELECT state_json FROM tool_installations WHERE tool_id='owned'",
                    [],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            "{\"state\":\"retained\"}"
        );
        assert_eq!(
            db.connect()
                .unwrap()
                .pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
                .unwrap(),
            10
        );
        let backup = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .find(|e| e.file_name().to_string_lossy().contains("pre-v10"))
            .unwrap();
        let snapshot =
            Connection::open_with_flags(backup.path(), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .unwrap();
        assert_eq!(
            snapshot
                .pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
                .unwrap(),
            4
        );
        Database::open(&path).unwrap();
    }
    #[test]
    fn version_three_is_backed_up_before_additive_capability_migration() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("private.db");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(include_str!("../migrations/001_foundation.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/002_research_inputs.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../migrations/003_job_engine.sql"))
            .unwrap();
        conn.execute("INSERT INTO projects(id,name,description,created_at,updated_at,archived) VALUES('owned','Keep existing data','private note','2026-10-15','2026-10-15',0)",[]).unwrap();
        conn.pragma_update(None, "user_version", 3).unwrap();
        drop(conn);
        let db = Database::open(&path).unwrap();
        assert_eq!(
            db.connect()
                .unwrap()
                .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
                .unwrap(),
            10
        );
        let backup = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .find(|e| e.file_name().to_string_lossy().ends_with(".bak"))
            .unwrap();
        let snapshot =
            Connection::open_with_flags(backup.path(), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .unwrap();
        assert_eq!(
            snapshot
                .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
                .unwrap(),
            3
        );
        for conn in [snapshot, db.connect().unwrap()] {
            assert_eq!(
                conn.query_row(
                    "SELECT description FROM projects WHERE id='owned'",
                    [],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
                "private note"
            );
            assert!(
                conn.prepare("PRAGMA foreign_key_check")
                    .unwrap()
                    .query([])
                    .unwrap()
                    .next()
                    .unwrap()
                    .is_none()
            );
        }
    }
    #[test]
    fn migrations_are_idempotent_and_refuse_future_schema() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let db = Database::open(&path).unwrap();
        Database::open(&path).unwrap();
        db.connect()
            .unwrap()
            .pragma_update(None, "user_version", 99)
            .unwrap();
        assert_eq!(Database::open(&path).unwrap_err().code, "SCHEMA_TOO_NEW");
    }
}
