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
        if (1..6).contains(&previous) {
            // VACUUM INTO takes a consistent SQLite snapshot, including committed WAL data.
            // It refuses to overwrite a file. Backups live beside the private database.
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| AppError::new("BACKUP_FAILED", "System clock is invalid"))?
                .as_nanos();
            let backup = path.with_extension(format!("pre-v6-{stamp}.bak"));
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
        if version > 6 {
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
            6
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
            6
        );
        let backup = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .find(|e| e.file_name().to_string_lossy().contains("pre-v6"))
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
            6
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
