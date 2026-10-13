use chrono::Utc;
use directories::ProjectDirs;
pub use reconx_ai::{AiReply, OsSecretStore, ProviderConfig, ProviderKind, SecretStore};
use reconx_security::validate_text;
pub use reconx_security::{AppError, Result, SecurityMode};
use reconx_storage::{
    Database, db_error,
    rusqlite::{Connection, OptionalExtension, TransactionBehavior, params},
};
pub use reconx_tools::{Inventory as ToolInventory, ToolManifest, ToolProgress};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{path::PathBuf, sync::Arc};
use uuid::Uuid;
use zeroize::Zeroizing;

fn credential_reference(config: &ProviderConfig) -> String {
    let digest = Sha256::digest(format!("{}|{:?}", config.base_url, config.kind).as_bytes());
    let fingerprint: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    format!("{}:{fingerprint}", config.id)
}

fn now() -> String {
    Utc::now().to_rfc3339()
}
fn json_error(_: serde_json::Error) -> AppError {
    AppError::new("INVALID_DATA", "Stored or imported data is invalid")
}
fn duplicate_error(error: reconx_storage::rusqlite::Error) -> AppError {
    if matches!(&error, reconx_storage::rusqlite::Error::SqliteFailure(e, _) if e.code == reconx_storage::rusqlite::ErrorCode::ConstraintViolation)
    {
        AppError::new(
            "DUPLICATE_NAME",
            "A record with this name already exists, including archived records",
        )
    } else {
        db_error(error)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub security_mode: SecurityMode,
    pub default_provider_id: Option<String>,
    #[serde(default)]
    pub fallback_provider_id: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct AuditEvent {
    pub id: i64,
    pub action: String,
    pub entity_id: String,
    pub outcome: String,
    pub timestamp: String,
}
#[derive(Debug, Serialize)]
pub struct Snapshot {
    pub projects: Vec<Project>,
    pub providers: Vec<ProviderConfig>,
    pub settings: Settings,
    pub audit: Vec<AuditEvent>,
    pub database_path: String,
    pub version: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectExport {
    pub format_version: u32,
    pub name: String,
    pub description: String,
}
#[derive(Clone)]
pub struct Core {
    db: Database,
    secrets: Arc<dyn SecretStore>,
}
impl Core {
    pub fn open(path: Option<PathBuf>) -> Result<Self> {
        Self::with_secrets(path, Arc::new(OsSecretStore))
    }
    pub fn with_secrets(path: Option<PathBuf>, secrets: Arc<dyn SecretStore>) -> Result<Self> {
        let path = path
            .or_else(|| {
                std::env::var_os("RECONX_DATA_DIR").map(|p| PathBuf::from(p).join("reconx.db"))
            })
            .or_else(|| {
                ProjectDirs::from("org", "ReconX", "ReconXAI")
                    .map(|p| p.data_local_dir().join("reconx.db"))
            })
            .ok_or_else(|| {
                AppError::new(
                    "WORKSPACE_IO",
                    "Unable to locate application data directory",
                )
            })?;
        let db = Database::open(path)?;
        Ok(Self { db, secrets })
    }
    pub fn database_path(&self) -> &std::path::Path {
        self.db.path()
    }
    fn audit(conn: &Connection, action: &str, entity: &str, outcome: &str) -> Result<()> {
        conn.execute(
            "INSERT INTO audit_events(action,entity_id,outcome,timestamp) VALUES (?1,?2,?3,?4)",
            params![action, entity, outcome, now()],
        )
        .map_err(db_error)?;
        Ok(())
    }
    pub fn list_projects(&self, include_archived: bool) -> Result<Vec<Project>> {
        let conn = self.db.connect()?;
        let mut query = conn.prepare("SELECT id,name,description,created_at,updated_at,archived FROM projects WHERE archived=0 OR ?1 ORDER BY updated_at DESC,id").map_err(db_error)?;
        query
            .query_map([include_archived], |r| {
                Ok(Project {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    description: r.get(2)?,
                    created_at: r.get(3)?,
                    updated_at: r.get(4)?,
                    archived: r.get(5)?,
                })
            })
            .map_err(db_error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(db_error)
    }
    pub fn project(&self, id_or_name: &str) -> Result<Project> {
        self.list_projects(true)?
            .into_iter()
            .find(|p| p.id == id_or_name || p.name.eq_ignore_ascii_case(id_or_name))
            .ok_or_else(|| AppError::new("NOT_FOUND", "Project does not exist"))
    }
    pub fn create_project(&self, name: &str, description: &str) -> Result<Project> {
        let name = validate_text(name, 120, "Project name")?;
        if name.contains(['\n', '\t']) {
            return Err(AppError::new(
                "INVALID_INPUT",
                "Project name must occupy one line",
            ));
        }
        if description.len() > 8192 {
            return Err(AppError::new(
                "INVALID_INPUT",
                "Description exceeds 8192 bytes",
            ));
        }
        let p = Project {
            id: Uuid::new_v4().to_string(),
            name,
            description: description.trim().into(),
            created_at: now(),
            updated_at: now(),
            archived: false,
        };
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute("INSERT INTO projects(id,name,description,created_at,updated_at) VALUES (?1,?2,?3,?4,?5)", params![p.id,p.name,p.description,p.created_at,p.updated_at]).map_err(duplicate_error)?;
        Self::audit(&tx, "project.created", &p.id, "completed")?;
        tx.commit().map_err(db_error)?;
        Ok(p)
    }
    pub fn update_project(&self, id: &str, name: &str, description: &str) -> Result<Project> {
        let p = self.project(id)?;
        let name = validate_text(name, 120, "Project name")?;
        if name.contains(['\n', '\t']) || description.len() > 8192 {
            return Err(AppError::new(
                "INVALID_INPUT",
                "Invalid project name or description",
            ));
        }
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute(
            "UPDATE projects SET name=?1,description=?2,updated_at=?3 WHERE id=?4",
            params![name, description.trim(), now(), p.id],
        )
        .map_err(duplicate_error)?;
        Self::audit(&tx, "project.updated", &p.id, "completed")?;
        tx.commit().map_err(db_error)?;
        self.project(&p.id)
    }
    pub fn archive_project(&self, id: &str, archived: bool) -> Result<()> {
        let p = self.project(id)?;
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute(
            "UPDATE projects SET archived=?1,updated_at=?2 WHERE id=?3",
            params![archived, now(), p.id],
        )
        .map_err(db_error)?;
        Self::audit(
            &tx,
            if archived {
                "project.archived"
            } else {
                "project.restored"
            },
            &p.id,
            "completed",
        )?;
        tx.commit().map_err(db_error)
    }
    pub fn export_project(&self, id: &str) -> Result<ProjectExport> {
        let p = self.project(id)?;
        Ok(ProjectExport {
            format_version: 1,
            name: p.name,
            description: p.description,
        })
    }
    pub fn import_project(&self, export: ProjectExport) -> Result<Project> {
        if export.format_version != 1 {
            return Err(AppError::new(
                "UNSUPPORTED_FORMAT",
                "Unsupported project export version",
            ));
        }
        self.create_project(&export.name, &export.description)
    }
    pub fn settings(&self) -> Result<Settings> {
        let conn = self.db.connect()?;
        let text: Option<String> = conn
            .query_row("SELECT json FROM settings WHERE id=1", [], |r| r.get(0))
            .optional()
            .map_err(db_error)?;
        text.map(|t| serde_json::from_str(&t).map_err(json_error))
            .unwrap_or_else(|| Ok(Settings::default()))
    }
    pub fn save_settings(&self, settings: &Settings) -> Result<()> {
        if let Some(id) = &settings.default_provider_id {
            self.provider(id)?;
        }
        if let Some(id) = &settings.fallback_provider_id {
            self.provider(id)?;
            if settings.default_provider_id.as_deref() == Some(id) {
                return Err(AppError::new(
                    "INVALID_INPUT",
                    "Primary and fallback providers must differ",
                ));
            }
        }
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute("INSERT INTO settings(id,json) VALUES (1,?1) ON CONFLICT(id) DO UPDATE SET json=excluded.json",[serde_json::to_string(settings).map_err(json_error)?]).map_err(db_error)?;
        Self::audit(&tx, "settings.updated", "workspace", "completed")?;
        tx.commit().map_err(db_error)
    }
    pub fn providers(&self) -> Result<Vec<ProviderConfig>> {
        let conn = self.db.connect()?;
        let mut q = conn
            .prepare("SELECT json FROM providers ORDER BY name")
            .map_err(db_error)?;
        let rows = q
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(db_error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(db_error)?;
        rows.into_iter()
            .map(|r| serde_json::from_str(&r).map_err(json_error))
            .collect()
    }
    pub fn provider(&self, id: &str) -> Result<ProviderConfig> {
        self.providers()?
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| AppError::new("NOT_FOUND", "Provider does not exist"))
    }
    pub fn save_provider(&self, config: &ProviderConfig) -> Result<()> {
        config.validate()?;
        if let Ok(old) = self.provider(&config.id)
            && (old.base_url != config.base_url || old.kind != config.kind)
            && self.secrets.get(&credential_reference(&old))?.is_some()
        {
            return Err(AppError::new(
                "GATEWAY_CREDENTIAL_BOUND",
                "Remove the existing credential before changing the gateway or protocol, then explicitly set a credential for the new destination",
            ));
        }
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute("INSERT INTO providers(id,name,json) VALUES (?1,?2,?3) ON CONFLICT(id) DO UPDATE SET name=excluded.name,json=excluded.json",params![config.id,config.name,serde_json::to_string(config).map_err(json_error)?]).map_err(duplicate_error)?;
        Self::audit(&tx, "provider.saved", &config.id, "completed")?;
        tx.commit().map_err(db_error)
    }
    pub fn delete_provider(&self, id: &str) -> Result<()> {
        let config = self.provider(id)?;
        self.secrets.delete(&credential_reference(&config))?;
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        tx.execute("DELETE FROM providers WHERE id=?1", [id])
            .map_err(db_error)?;
        let mut settings = self.settings()?;
        if settings.default_provider_id.as_deref() == Some(id) {
            settings.default_provider_id = None;
        }
        if settings.fallback_provider_id.as_deref() == Some(id) {
            settings.fallback_provider_id = None;
        }
        tx.execute("INSERT INTO settings(id,json) VALUES (1,?1) ON CONFLICT(id) DO UPDATE SET json=excluded.json",[serde_json::to_string(&settings).map_err(json_error)?]).map_err(db_error)?;
        Self::audit(&tx, "provider.deleted", id, "completed")?;
        tx.commit().map_err(db_error)
    }
    pub fn set_key(&self, id: &str, secret: &str) -> Result<()> {
        let config = self.provider(id)?;
        self.secrets.set(&credential_reference(&config), secret)?;
        Self::audit(&self.db.connect()?, "credential.updated", id, "completed")
    }
    pub fn remove_key(&self, id: &str) -> Result<()> {
        let config = self.provider(id)?;
        self.secrets.delete(&credential_reference(&config))?;
        Self::audit(&self.db.connect()?, "credential.removed", id, "completed")
    }
    pub fn credential_status(&self, id: &str) -> Result<bool> {
        let config = self.provider(id)?;
        Ok(self.secrets.get(&credential_reference(&config))?.is_some())
    }
    pub async fn models(&self, id: &str) -> Result<Vec<String>> {
        let config = self.provider(id)?;
        let key = if config.requires_key {
            self.secrets.get(&credential_reference(&config))?
        } else {
            None
        };
        let gateway = reconx_ai::Gateway::new(config)?;
        let result = gateway.models(key.as_ref().map(|k| k.as_str())).await;
        Self::audit(
            &self.db.connect()?,
            "ai.models",
            id,
            if result.is_ok() {
                "completed"
            } else {
                "failed"
            },
        )?;
        result
    }
    pub async fn chat(&self, id: &str, prompt: &str) -> Result<AiReply> {
        let config = self.provider(id)?;
        let key: Option<Zeroizing<String>> = if config.requires_key {
            self.secrets.get(&credential_reference(&config))?
        } else {
            None
        };
        let gateway = reconx_ai::Gateway::new(config)?;
        let result = gateway.chat(prompt, key.as_ref().map(|k| k.as_str())).await;
        Self::audit(
            &self.db.connect()?,
            "ai.chat",
            id,
            if result.is_ok() {
                "completed"
            } else {
                "failed"
            },
        )?;
        if let Some(fallback) = self.fallback_for(id, &result)? {
            let config = self.provider(&fallback)?;
            let key = if config.requires_key {
                self.secrets.get(&credential_reference(&config))?
            } else {
                None
            };
            let mut result = reconx_ai::Gateway::new(config)?
                .chat(prompt, key.as_ref().map(|k| k.as_str()))
                .await;
            Self::audit(
                &self.db.connect()?,
                "ai.fallback",
                &fallback,
                if result.is_ok() {
                    "completed"
                } else {
                    "failed"
                },
            )?;
            if let Ok(reply) = &mut result {
                reply.fallback_used = true;
            }
            return result;
        }
        result
    }
    fn fallback_for<T>(&self, id: &str, result: &Result<T>) -> Result<Option<String>> {
        if matches!(result, Err(error) if error.code == "AI_CONNECT_ERROR") {
            let settings = self.settings()?;
            if settings.default_provider_id.as_deref() == Some(id) {
                return Ok(settings.fallback_provider_id);
            }
        }
        Ok(None)
    }
    pub async fn stream_chat<F>(
        &self,
        id: &str,
        prompt: &str,
        cancellation: tokio::sync::watch::Receiver<bool>,
        mut update: F,
    ) -> Result<AiReply>
    where
        F: FnMut(&str) + Send,
    {
        let config = self.provider(id)?;
        let key = if config.requires_key {
            self.secrets.get(&credential_reference(&config))?
        } else {
            None
        };
        let gateway = reconx_ai::Gateway::new(config)?;
        let result = gateway
            .stream_chat(
                prompt,
                key.as_ref().map(|k| k.as_str()),
                cancellation.clone(),
                &mut update,
            )
            .await;
        let outcome = match &result {
            Ok(_) => "completed",
            Err(e) if e.code == "CANCELLED" => "cancelled",
            _ => "failed",
        };
        Self::audit(&self.db.connect()?, "ai.stream", id, outcome)?;
        if let Some(fallback) = self.fallback_for(id, &result)? {
            let config = self.provider(&fallback)?;
            let key = if config.requires_key {
                self.secrets.get(&credential_reference(&config))?
            } else {
                None
            };
            let mut result = reconx_ai::Gateway::new(config)?
                .stream_chat(
                    prompt,
                    key.as_ref().map(|k| k.as_str()),
                    cancellation,
                    &mut update,
                )
                .await;
            Self::audit(
                &self.db.connect()?,
                "ai.stream_fallback",
                &fallback,
                match &result {
                    Ok(_) => "completed",
                    Err(e) if e.code == "CANCELLED" => "cancelled",
                    _ => "failed",
                },
            )?;
            if let Ok(reply) = &mut result {
                reply.fallback_used = true;
            }
            return result;
        }
        result
    }
    fn tool_manager(&self) -> reconx_tools::ToolManager {
        reconx_tools::ToolManager::new(
            self.db
                .path()
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .join("tools"),
        )
    }
    pub fn tool_inventory(&self) -> Result<Vec<ToolInventory>> {
        self.tool_manager().inventory()
    }
    pub fn tool_plan(&self, id: &str) -> Result<ToolManifest> {
        self.tool_manager().plan(id)
    }
    pub async fn tool_install<F>(&self, id: &str, progress: F) -> Result<ToolInventory>
    where
        F: FnMut(ToolProgress) + Send,
    {
        Self::audit(&self.db.connect()?, "tool.install", id, "running")?;
        let result = self.tool_manager().install(id, progress).await;
        Self::audit(
            &self.db.connect()?,
            "tool.install",
            id,
            if result.is_ok() {
                "completed"
            } else {
                "failed"
            },
        )?;
        result
    }
    pub async fn tool_doctor(&self, id: &str) -> Result<ToolInventory> {
        let result = self.tool_manager().doctor(id).await;
        Self::audit(
            &self.db.connect()?,
            "tool.doctor",
            id,
            if result.is_ok() {
                "completed"
            } else {
                "failed"
            },
        )?;
        result
    }
    pub fn tool_uninstall(&self, id: &str) -> Result<()> {
        let result = self.tool_manager().uninstall(id);
        Self::audit(
            &self.db.connect()?,
            "tool.uninstall",
            id,
            if result.is_ok() {
                "completed"
            } else {
                "failed"
            },
        )?;
        result
    }
    pub fn audit_events(&self) -> Result<Vec<AuditEvent>> {
        let conn = self.db.connect()?;
        let mut q=conn.prepare("SELECT id,action,entity_id,outcome,timestamp FROM audit_events ORDER BY id DESC LIMIT 100").map_err(db_error)?;
        q.query_map([], |r| {
            Ok(AuditEvent {
                id: r.get(0)?,
                action: r.get(1)?,
                entity_id: r.get(2)?,
                outcome: r.get(3)?,
                timestamp: r.get(4)?,
            })
        })
        .map_err(db_error)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(db_error)
    }
    pub fn snapshot(&self) -> Result<Snapshot> {
        Ok(Snapshot {
            projects: self.list_projects(true)?,
            providers: self.providers()?,
            settings: self.settings()?,
            audit: self.audit_events()?,
            database_path: self.db.path().display().to_string(),
            version: env!("CARGO_PKG_VERSION").into(),
        })
    }
}
