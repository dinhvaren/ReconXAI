//! Compiled, permission-bounded native extensions. Registry descriptions are never executable.
use super::*;
use serde_json::{Value, json};

const YARA: &str = "reconx.yara-signatures";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionManifest {
    pub manifest_version: u32,
    pub id: String,
    pub name: String,
    pub publisher: String,
    pub version: String,
    pub description: String,
    pub extension_type: String,
    pub category: String,
    pub compatible_reconx: String,
    pub operating_systems: Vec<String>,
    pub architectures: Vec<String>,
    pub required_runtime: String,
    pub required_tools: Vec<String>,
    pub required_mcp_servers: Vec<String>,
    pub install_source: String,
    pub permissions: Vec<String>,
    pub capabilities: Vec<String>,
    pub configuration_schema: Value,
    pub entrypoint: String,
    pub health_check: String,
    pub uninstall_method: String,
}

/// Only a compiled manifest is accepted. Imports cannot add executables, arguments,
/// runtime code, network destinations or permissions to this native adapter.
pub fn validate_extension_manifest(manifest: &ExtensionManifest) -> Result<()> {
    let expected = if manifest.id == MANAGED_MCP_EVERYTHING {
        managed_mcp::manifest(&manifest.version)?
    } else {
        yara_manifest()
    };
    if serde_json::to_value(manifest).map_err(json_error)?
        != serde_json::to_value(expected).map_err(json_error)?
    {
        return Err(AppError::new(
            "EXTENSION_POLICY",
            "This build only executes its compiled, permission-reviewed native extension manifests",
        ));
    }
    Ok(())
}
fn yara_manifest() -> ExtensionManifest {
    ExtensionManifest {
        manifest_version: 1,
        id: YARA.into(), name: "YARA-X File Signatures".into(), publisher: "ReconX AI".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        description: "Run fixed offline PE, ELF and benign research-marker signatures on an explicitly imported file. Results are observations, not verified vulnerabilities.".into(),
        extension_type: "native_tool_adapter".into(), category: "Reverse Engineering".into(),
        compatible_reconx: env!("CARGO_PKG_VERSION").into(),
        operating_systems: vec!["windows".into(), "linux".into()], architectures: vec!["x86_64".into()],
        required_runtime: "ReconX shared Job Engine".into(), required_tools: vec!["yara-x".into()],
        required_mcp_servers: vec![], install_source: "compiled-first-party-catalog".into(),
        permissions: vec!["read:explicitly-imported-file".into(), "execute:pinned-yara-x-fixed-arguments".into(), "write:project-evidence".into()],
        capabilities: vec!["yara-x.signatures".into()],
        configuration_schema: json!({"type":"object","additionalProperties":false}),
        entrypoint: "job:yara_signatures".into(), health_check: "verified-tool-version-and-bounded-signature-scan".into(),
        uninstall_method: "remove-extension-registration-retain-tools-and-evidence".into(),
    }
}
fn extension_manifest(id: &str) -> Result<ExtensionManifest> {
    if id == MANAGED_MCP_EVERYTHING {
        return managed_mcp::manifest("2.0.0");
    }
    if id != YARA {
        return Err(AppError::new(
            "EXTENSION_NOT_FOUND",
            "No compiled extension has this ID",
        ));
    }
    let manifest = yara_manifest();
    validate_extension_manifest(&manifest)?;
    Ok(manifest)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionState {
    pub version: String,
    pub enabled: bool,
    pub status: String,
    pub revision: String,
    pub updated_at: String,
    pub last_health_check: Option<String>,
    pub error_code: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct ExtensionEntry {
    pub managed: Option<ManagedMcpSummary>,
    pub manifest: ExtensionManifest,
    pub provenance: String,
    pub status: String,
    pub supported: bool,
    pub installed: Option<ExtensionState>,
    pub dependencies: Vec<ToolInventory>,
    pub update_available: bool,
    pub available_version: String,
    pub installation_size: Option<u64>,
    pub source_repository: String,
}
#[derive(Debug, Serialize)]
pub struct ExtensionInstallPlan {
    pub package_plan: Option<reconx_tools::packages::NpmPlan>,
    pub extension: ExtensionEntry,
    pub fingerprint: String,
    pub actions: Vec<String>,
    pub requires_admin: bool,
    pub network_access: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionApproval {
    pub id: String,
    pub fingerprint: String,
    pub approved: bool,
}
impl Core {
    pub(super) fn extension_state(&self, id: &str) -> Result<Option<ExtensionState>> {
        let text: Option<String> = self
            .db
            .connect()?
            .query_row(
                "SELECT state_json FROM extension_installations WHERE extension_id=?1",
                [id],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        text.map(|s| serde_json::from_str(&s).map_err(json_error))
            .transpose()
    }
    pub fn extension_entry(&self, id: &str) -> Result<ExtensionEntry> {
        if id == MANAGED_MCP_EVERYTHING {
            return self.managed_entry();
        }
        let manifest = extension_manifest(id)?;
        let installed = self.extension_state(id)?;
        let supported = manifest
            .operating_systems
            .iter()
            .any(|s| s == std::env::consts::OS)
            && manifest
                .architectures
                .iter()
                .any(|s| s == std::env::consts::ARCH);
        let dependencies: Vec<_> = self
            .tool_inventory()?
            .into_iter()
            .filter(|t| manifest.required_tools.contains(&t.manifest.id))
            .collect();
        let healthy = dependencies.len() == manifest.required_tools.len()
            && dependencies
                .iter()
                .all(|t| t.integrity_verified && t.status == "tested");
        let status = if !supported {
            "unsupported"
        } else if let Some(s) = &installed {
            if s.status == "installing" {
                "installing"
            } else if s.status == "failed" {
                "failed"
            } else if !s.enabled {
                "disabled"
            } else if !healthy || s.last_health_check.is_none() {
                "misconfigured"
            } else {
                "ready"
            }
        } else {
            "not_installed"
        };
        Ok(ExtensionEntry {
            managed: None,
            update_available: installed
                .as_ref()
                .is_some_and(|s| s.version != manifest.version),
            available_version: manifest.version.clone(),
            manifest,
            provenance: "first_party_compiled".into(),
            status: status.into(),
            supported,
            installed,
            dependencies,
            installation_size: None,
            source_repository: "https://github.com/dinhvaren/ReconXAI".into(),
        })
    }
    pub fn extension_catalog(&self, query: Option<&str>) -> Result<Vec<ExtensionEntry>> {
        let needle = query.unwrap_or("").trim().to_lowercase();
        if needle.len() > 256 {
            return Err(AppError::new(
                "INVALID_INPUT",
                "Extension search is limited to 256 bytes",
            ));
        }
        let entries = [
            self.extension_entry(YARA)?,
            self.extension_entry(MANAGED_MCP_EVERYTHING)?,
        ];
        Ok(entries
            .into_iter()
            .filter(|e| {
                format!(
                    "{} {} {} {}",
                    e.manifest.id, e.manifest.name, e.manifest.description, e.manifest.category
                )
                .to_lowercase()
                .contains(&needle)
            })
            .collect())
    }
    pub fn extension_install_plan(&self, id: &str) -> Result<ExtensionInstallPlan> {
        if id == MANAGED_MCP_EVERYTHING {
            return self.managed_install_plan(id, "2.0.0", false);
        }
        let extension = self.extension_entry(id)?;
        if !extension.supported {
            return Err(AppError::new(
                "UNSUPPORTED_PLATFORM",
                "No verified extension dependency for this OS/architecture",
            ));
        }
        let fingerprint = reconx_tools::hash(&serde_json::to_vec(&extension).map_err(json_error)?);
        let actions = extension
            .dependencies
            .iter()
            .map(|d| {
                format!(
                    "{} {} {}",
                    if d.integrity_verified {
                        "Reuse and verify"
                    } else {
                        "Download and verify"
                    },
                    d.manifest.name,
                    d.manifest.version
                )
            })
            .chain([
                "Run a real bounded signature health check; register the extension".into(),
                "Uninstall retains managed tools, projects and evidence".into(),
            ])
            .collect();
        Ok(ExtensionInstallPlan { extension, fingerprint, actions, requires_admin: false,
            package_plan:None,
            network_access: "Installation only: pinned GitHub release URLs and SHA-256 hashes shown in dependency manifests. Signature execution is offline.".into() })
    }
    pub async fn extension_install(&self, approval: ExtensionApproval) -> Result<ExtensionEntry> {
        if approval.id == MANAGED_MCP_EVERYTHING {
            return Err(AppError::new(
                "MANAGED_EXTENSION_API",
                "Use the approved managed MCP job/lifecycle API; file-signature operations do not apply",
            ));
        }
        if !approval.approved {
            return Err(AppError::new(
                "APPROVAL_REQUIRED",
                "Approve the exact extension installation plan",
            ));
        }
        let plan = self.extension_install_plan(&approval.id)?;
        if plan.fingerprint != approval.fingerprint {
            return Err(AppError::new(
                "PLAN_CHANGED",
                "Installation state changed; review a fresh plan",
            ));
        }
        let mut state = {
            let mut conn = self.db.connect()?;
            let tx = conn
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(db_error)?;
            let current = self.extension_state(&approval.id)?;
            if current.as_ref().is_some_and(|s| s.status == "installing") {
                return Err(AppError::new(
                    "EXTENSION_BUSY",
                    "Installation is in progress; interrupted installations can be recovered after ten minutes",
                ));
            }
            if current.as_ref().map(|s| &s.revision)
                != plan.extension.installed.as_ref().map(|s| &s.revision)
            {
                return Err(AppError::new(
                    "PLAN_CHANGED",
                    "Extension state changed during approval",
                ));
            }
            let state = ExtensionState {
                version: plan.extension.manifest.version.clone(),
                enabled: false,
                status: "installing".into(),
                revision: Uuid::new_v4().to_string(),
                updated_at: now(),
                last_health_check: None,
                error_code: None,
            };
            tx.execute("INSERT INTO extension_installations(extension_id,state_json) VALUES(?1,?2) ON CONFLICT(extension_id) DO UPDATE SET state_json=excluded.state_json",
            params![approval.id, serde_json::to_string(&state).map_err(json_error)?]).map_err(db_error)?;
            for tool in &plan.extension.manifest.required_tools {
                tx.execute(
                "INSERT OR IGNORE INTO extension_requirements(extension_id,tool_id) VALUES(?1,?2)",
                params![approval.id, tool],
            )
            .map_err(db_error)?;
            }
            Self::audit(&tx, "extension.install", &approval.id, "approved")?;
            tx.commit().map_err(db_error)?;
            state
        };
        let outcome = async {
            for dependency in &plan.extension.dependencies {
                if !dependency.integrity_verified {
                    self.tool_install(&dependency.manifest.id, |_| {}).await?;
                }
                self.tool_doctor(&dependency.manifest.id).await?;
            }
            self.extension_signature_health().await
        }
        .await;
        state.enabled = outcome.is_ok();
        state.status = if outcome.is_ok() {
            "installed"
        } else {
            "failed"
        }
        .into();
        state.error_code = outcome.as_ref().err().map(|e| e.code.clone());
        state.last_health_check = outcome.as_ref().ok().map(|_| now());
        state.updated_at = now();
        self.write_extension_state(&approval.id, &state)?;
        Self::audit(
            &self.db.connect()?,
            "extension.install",
            &approval.id,
            if outcome.is_ok() {
                "completed"
            } else {
                "failed"
            },
        )?;
        outcome?;
        self.extension_entry(&approval.id)
    }
    async fn extension_signature_health(&self) -> Result<()> {
        let manager = self.tool_manager();
        tokio::task::spawn_blocking(move || {
            let result = manager.signatures(b"RECONX_BENIGN_FIXTURE", || Ok(()))?;
            if !result.to_string().contains("ReconX_Test_Marker") {
                return Err(AppError::new(
                    "EXTENSION_HEALTH",
                    "The actual signature health check did not return its benign marker",
                ));
            }
            Ok(())
        })
        .await
        .map_err(|_| AppError::new("WORKER_ERROR", "Extension health worker failed"))?
    }
    pub async fn extension_doctor(&self, id: &str) -> Result<ExtensionEntry> {
        if id == MANAGED_MCP_EVERYTHING {
            return self.managed_doctor().await;
        }
        let entry = self.extension_entry(id)?;
        let mut state = entry.installed.ok_or_else(|| {
            AppError::new("EXTENSION_NOT_INSTALLED", "Install this extension first")
        })?;
        if state.status == "installing" {
            let age = Utc::now()
                .signed_duration_since(
                    chrono::DateTime::parse_from_rfc3339(&state.updated_at).map_err(|_| {
                        AppError::new("INVALID_DATA", "Invalid installation timestamp")
                    })?,
                )
                .num_seconds();
            if age < 600 {
                return Err(AppError::new(
                    "EXTENSION_BUSY",
                    "Installation is still active; recovery waits ten minutes",
                ));
            }
            state.status = "failed".into();
            state.error_code = Some("INSTALL_INTERRUPTED".into());
            state.enabled = false;
            self.write_extension_state(id, &state)?;
        }
        let outcome = async {
            self.tool_doctor("yara-x").await?;
            self.extension_signature_health().await
        }
        .await;
        state.last_health_check = outcome.as_ref().ok().map(|_| now());
        state.error_code = outcome.as_ref().err().map(|e| e.code.clone());
        state.status = if outcome.is_ok() {
            "installed"
        } else {
            "failed"
        }
        .into();
        state.updated_at = now();
        self.write_extension_state(id, &state)?;
        Self::audit(
            &self.db.connect()?,
            "extension.doctor",
            id,
            if outcome.is_ok() {
                "completed"
            } else {
                "failed"
            },
        )?;
        outcome?;
        self.extension_entry(id)
    }
    pub(super) fn write_extension_state(&self, id: &str, state: &ExtensionState) -> Result<()> {
        let changed = self.db.connect()?.execute("UPDATE extension_installations SET state_json=?1 WHERE extension_id=?2 AND json_extract(state_json,'$.revision')=?3", params![serde_json::to_string(state).map_err(json_error)?,id,state.revision]).map_err(db_error)?;
        if changed != 1 {
            return Err(AppError::new(
                "EXTENSION_CHANGED",
                "Extension was changed by another operation",
            ));
        }
        Ok(())
    }
    pub fn extension_set_enabled(&self, id: &str, enabled: bool) -> Result<ExtensionEntry> {
        if id == MANAGED_MCP_EVERYTHING {
            return Err(AppError::new(
                "MANAGED_EXTENSION_API",
                "Use the approved managed MCP job/lifecycle API; file-signature operations do not apply",
            ));
        }
        let entry = self.extension_entry(id)?;
        let mut state = entry.installed.ok_or_else(|| {
            AppError::new("EXTENSION_NOT_INSTALLED", "Install this extension first")
        })?;
        if state.status == "installing" {
            return Err(AppError::new(
                "EXTENSION_BUSY",
                "Wait for installation to finish",
            ));
        }
        if enabled
            && (state.last_health_check.is_none()
                || entry
                    .dependencies
                    .iter()
                    .any(|d| !d.integrity_verified || d.status != "tested")
                || state.status == "failed")
        {
            return Err(AppError::new(
                "EXTENSION_HEALTH_REQUIRED",
                "Run a successful extension health check before enabling",
            ));
        }
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let old = state.revision.clone();
        state.revision = Uuid::new_v4().to_string();
        state.enabled = enabled;
        state.updated_at = now();
        let changed = tx.execute("UPDATE extension_installations SET state_json=?1 WHERE extension_id=?2 AND json_extract(state_json,'$.revision')=?3", params![serde_json::to_string(&state).map_err(json_error)?,id,old]).map_err(db_error)?;
        if changed != 1 {
            return Err(AppError::new(
                "EXTENSION_CHANGED",
                "Extension changed concurrently",
            ));
        }
        if !enabled {
            Self::cancel_extension_jobs(&tx, id)?;
        }
        Self::audit(
            &tx,
            if enabled {
                "extension.enable"
            } else {
                "extension.disable"
            },
            id,
            "completed",
        )?;
        tx.commit().map_err(db_error)?;
        self.extension_entry(id)
    }
    fn cancel_extension_jobs(conn: &Connection, id: &str) -> Result<()> {
        conn.execute("UPDATE analysis_jobs SET cancel_requested=1,status=CASE WHEN status='queued' THEN 'cancelled' ELSE status END,finished_at=CASE WHEN status='queued' THEN ?1 ELSE finished_at END WHERE status IN ('queued','running') AND id IN (SELECT job_id FROM extension_invocations WHERE extension_id=?2)", params![now(),id]).map_err(db_error)?;
        Ok(())
    }
    pub fn extension_remove(&self, id: &str) -> Result<()> {
        if id == MANAGED_MCP_EVERYTHING {
            return Err(AppError::new(
                "MANAGED_EXTENSION_API",
                "Use the approved managed MCP job/lifecycle API; file-signature operations do not apply",
            ));
        }
        extension_manifest(id)?;
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let busy: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM extension_installations WHERE extension_id=?1 AND json_extract(state_json,'$.status')='installing')", [id], |r| r.get(0)).map_err(db_error)?;
        if busy {
            return Err(AppError::new(
                "EXTENSION_BUSY",
                "Wait for installation to finish",
            ));
        }
        Self::cancel_extension_jobs(&tx, id)?;
        tx.execute(
            "DELETE FROM extension_installations WHERE extension_id=?1",
            [id],
        )
        .map_err(db_error)?;
        Self::audit(
            &tx,
            "extension.remove",
            id,
            "completed_tools_and_evidence_retained",
        )?;
        tx.commit().map_err(db_error)
    }
    pub fn extension_invoke(&self, id: &str, file_id: &str) -> Result<AnalysisJob> {
        if id == MANAGED_MCP_EVERYTHING {
            return Err(AppError::new(
                "MANAGED_EXTENSION_API",
                "Use the approved managed MCP job/lifecycle API; file-signature operations do not apply",
            ));
        }
        let entry = self.extension_entry(id)?;
        if entry.status != "ready" {
            return Err(AppError::new(
                "EXTENSION_NOT_READY",
                "Enable and health-check this extension first",
            ));
        }
        let state = entry.installed.ok_or_else(|| {
            AppError::new("EXTENSION_NOT_INSTALLED", "Install this extension first")
        })?;
        self.enqueue_extension_job(
            JobInput {
                kind: JobKind::YaraSignatures,
                file_id: Some(file_id.into()),
                target_id: None,
                timeout_ms: 60_000,
            },
            id,
            &state.revision,
        )
    }
    pub fn extension_history(&self, id: &str) -> Result<Vec<AnalysisJob>> {
        extension_manifest(id)?;
        let conn = self.db.connect()?;
        let ids = conn.prepare("SELECT job_id FROM extension_invocations WHERE extension_id=?1 ORDER BY approved_at DESC LIMIT 100").map_err(db_error)?
            .query_map([id], |r| r.get::<_,String>(0)).map_err(db_error)?.collect::<std::result::Result<Vec<_>,_>>().map_err(db_error)?;
        ids.iter().map(|id| self.job(id)).collect()
    }
    pub(super) fn require_extension_job(conn: &Connection, job_id: &str) -> Result<()> {
        let allowed: Option<bool> = conn.query_row("SELECT EXISTS(SELECT 1 FROM extension_installations e WHERE e.extension_id=i.extension_id AND json_extract(e.state_json,'$.enabled')=1 AND json_extract(e.state_json,'$.revision')=i.revision AND json_extract(e.state_json,'$.status')='installed') FROM extension_invocations i WHERE i.job_id=?1", [job_id], |r| r.get(0)).optional().map_err(db_error)?;
        if allowed == Some(false) {
            return Err(AppError::new(
                "EXTENSION_DISABLED",
                "Extension permission was revoked or its registration changed",
            ));
        }
        Ok(())
    }
}
