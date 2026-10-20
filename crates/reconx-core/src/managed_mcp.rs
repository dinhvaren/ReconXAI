//! Reviewed MCP package lifecycle on the existing durable Job Engine.
use super::*;
use reconx_tools::packages::{NpmPlan, PackageManager, PackageReceipt};
use serde_json::{Value, json};
pub const MANAGED_MCP_EVERYTHING: &str = "reconx.mcp-everything";
const ENV_PROJECT: &str = "00000000-0000-4000-8000-000000000001";
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedMcpSummary {
    pub server_id: Option<String>,
    pub active_job: Option<AnalysisJob>,
    pub tools: Vec<Value>,
    pub allowed_tools: Vec<String>,
    pub package_name: String,
    pub runtime_requirement: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedMcpApproval {
    pub id: String,
    pub version: String,
    #[serde(default)]
    pub managed_runtime: bool,
    pub fingerprint: String,
    pub approved: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Descriptor {
    receipt: PackageReceipt,
    config: McpServerConfig,
    config_revision: String,
    tools: Vec<Value>,
}
pub(super) fn manifest(version: &str) -> Result<ExtensionManifest> {
    if !["1.0.0", "2.0.0"].contains(&version) {
        return Err(AppError::new(
            "PACKAGE_VERSION",
            "No reviewed package lock exists for this version",
        ));
    }
    Ok(ExtensionManifest {manifest_version:1,id:MANAGED_MCP_EVERYTHING.into(),name:"MCP Everything (reviewed npm package)".into(),publisher:"modelcontextprotocol — integration by ReconX AI".into(),version:version.into(),description:"Official MCP test server installed from pinned npm dependencies. Tool calls require separate permission and exact-input approval.".into(),extension_type:"managed_mcp_npm".into(),category:"MCP".into(),compatible_reconx:env!("CARGO_PKG_VERSION").into(),operating_systems:vec!["windows".into(),"linux".into()],architectures:vec!["x86_64".into()],required_runtime:"Node.js >=24; managed portable runtime available".into(),required_tools:vec![],required_mcp_servers:vec![],install_source:"reviewed-npm-lock".into(),permissions:vec!["download:pinned-npm-and-node-artifacts".into(),"execute:reviewed-package-at-user-privileges".into(),"mcp:initialize-and-list-tools-only".into(),"no-os-sandbox-available".into(),"no-automatic-tool-invocation".into()],capabilities:vec!["mcp.discovered-tools".into()],configuration_schema:json!({"type":"object","properties":{"allowed_tools":{"type":"array","items":{"type":"string"}}},"additionalProperties":false}),entrypoint:"node:node_modules/@modelcontextprotocol/server-everything/dist/index.js stdio".into(),health_check:"actual-mcp-initialize-list-tools-schema-validation".into(),uninstall_method:"owned-package-removal-retain-shared-runtime-and-download-cache".into()})
}
fn revision(config: &McpServerConfig) -> Result<String> {
    Ok(reconx_tools::hash(
        &serde_json::to_vec(config).map_err(json_error)?,
    ))
}
impl Core {
    fn package_manager(&self) -> PackageManager {
        PackageManager::new(
            self.db
                .path()
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .join("extensions"),
        )
    }
    fn managed_descriptor(&self, id: &str) -> Result<Option<Descriptor>> {
        let text: Option<String> = self
            .db
            .connect()?
            .query_row(
                "SELECT descriptor_json FROM managed_mcp_extensions WHERE extension_id=?1",
                [id],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        text.map(|s| serde_json::from_str(&s).map_err(json_error))
            .transpose()
    }
    pub fn managed_install_history(&self, id: &str) -> Result<Vec<AnalysisJob>> {
        let conn = self.db.connect()?;
        let ids=conn.prepare("SELECT job_id FROM extension_install_jobs WHERE extension_id=?1 ORDER BY rowid DESC LIMIT 30").map_err(db_error)?.query_map([id],|r|r.get::<_,String>(0)).map_err(db_error)?.collect::<std::result::Result<Vec<_>,_>>().map_err(db_error)?;
        ids.iter().map(|id| self.job(id)).collect()
    }
    pub(super) fn managed_entry(&self) -> Result<ExtensionEntry> {
        self.managed_reconcile()?;
        let manifest = manifest("2.0.0")?;
        let installed = self.extension_state(MANAGED_MCP_EVERYTHING)?;
        let descriptor = self.managed_descriptor(MANAGED_MCP_EVERYTHING)?;
        let active_job = self
            .managed_install_history(MANAGED_MCP_EVERYTHING)?
            .into_iter()
            .find(|j| matches!(j.status.as_str(), "queued" | "running"));
        let healthy = descriptor.as_ref().is_some_and(|d| {
            self.package_manager().verify(&d.receipt).is_ok()
                && self
                    .mcp_server(&d.config.id)
                    .is_ok_and(|s| s.revision == d.config_revision)
        });
        let supported = std::env::consts::ARCH == "x86_64"
            && ["windows", "linux"].contains(&std::env::consts::OS);
        let status = if !supported {
            "unsupported"
        } else if active_job.is_some() {
            "installing"
        } else if let Some(state) = &installed {
            if state.status == "failed" {
                "failed"
            } else if !state.enabled {
                "disabled"
            } else if healthy && state.last_health_check.is_some() {
                "ready"
            } else {
                "misconfigured"
            }
        } else {
            "not_installed"
        };
        Ok(ExtensionEntry {
            available_version: manifest.version.clone(),
            update_available: installed.as_ref().is_some_and(|s| s.version != "2.0.0"),
            manifest,
            provenance: "reviewed_package_manifest".into(),
            status: status.into(),
            supported,
            installed,
            dependencies: vec![],
            installation_size: None,
            source_repository: "https://github.com/modelcontextprotocol/servers".into(),
            managed: Some(ManagedMcpSummary {
                server_id: descriptor.as_ref().map(|d| d.config.id.clone()),
                active_job,
                allowed_tools: descriptor
                    .as_ref()
                    .map(|d| d.config.allowed_tools.clone())
                    .unwrap_or_default(),
                tools: descriptor.map(|d| d.tools).unwrap_or_default(),
                package_name: "@modelcontextprotocol/server-everything".into(),
                runtime_requirement: "Node.js >=24".into(),
            }),
        })
    }
    pub fn managed_install_plan(
        &self,
        id: &str,
        version: &str,
        managed_runtime: bool,
    ) -> Result<ExtensionInstallPlan> {
        if id != MANAGED_MCP_EVERYTHING {
            return Err(AppError::new(
                "EXTENSION_NOT_FOUND",
                "No reviewed installer adapter for this MCP package",
            ));
        }
        let mut entry = self.managed_entry()?;
        entry.manifest = manifest(version)?;
        let package = self.package_manager().plan(version, managed_runtime)?;
        let descriptor = self.managed_descriptor(id)?;
        if let Some(old) = &descriptor
            && self.mcp_server(&old.config.id)?.revision != old.config_revision
        {
            return Err(AppError::new(
                "MANAGED_CONFIG_CHANGED",
                "MCP configuration was edited outside this extension; reconcile it before changing installed packages",
            ));
        }
        let fingerprint=reconx_tools::hash(&serde_json::to_vec(&json!({"manifest":entry.manifest,"package":package,"installed":entry.installed,"config_revision":descriptor.as_ref().map(|d|&d.config_revision)})).map_err(json_error)?);
        let actions = vec![
            format!(
                "{} Node {}",
                if package.runtime.managed {
                    "Prepare verified managed"
                } else {
                    "Reuse approved existing"
                },
                package.runtime.version
            ),
            format!(
                "Verify and extract {} exact npm tarballs; run no install scripts",
                package.dependencies.len()
            ),
            "Run MCP initialize and list_tools; validate actual tool schemas; do not invoke tools"
                .into(),
            "Commit only after health verification; retain previous version on update failure"
                .into(),
        ];
        Ok(ExtensionInstallPlan{extension:entry,fingerprint,actions,requires_admin:false,network_access:"HTTPS downloads limited to pinned registry.npmjs.org and nodejs.org artifacts. The approved MCP package runs at user privileges without an OS sandbox; all tools initially denied.".into(),package_plan:Some(package)})
    }
    pub fn managed_enqueue_install(&self, approval: ManagedMcpApproval) -> Result<AnalysisJob> {
        if !approval.approved {
            return Err(AppError::new(
                "APPROVAL_REQUIRED",
                "Approve the exact package, dependency and process permission plan",
            ));
        }
        let plan =
            self.managed_install_plan(&approval.id, &approval.version, approval.managed_runtime)?;
        if approval.fingerprint != plan.fingerprint {
            return Err(AppError::new(
                "PLAN_CHANGED",
                "Dependencies, runtime or MCP configuration changed; review a new plan",
            ));
        }
        let descriptor = self.managed_descriptor(&approval.id)?;
        let previous = self.extension_state(&approval.id)?;
        let job = Uuid::new_v4().to_string();
        let reservation = Uuid::new_v4().to_string();
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let busy:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM extension_install_jobs o JOIN analysis_jobs j ON j.id=o.job_id WHERE extension_id=?1 AND status IN ('queued','running'))",[&approval.id],|r|r.get(0)).map_err(db_error)?;
        if busy {
            return Err(AppError::new(
                "EXTENSION_BUSY",
                "An installation is already queued or running",
            ));
        }
        if let Some(old) = &descriptor {
            let busy:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM capability_invocations i JOIN analysis_jobs j ON i.job_id=j.id WHERE server_id=?1 AND j.status IN ('queued','running'))",[&old.config.id],|r|r.get(0)).map_err(db_error)?;
            if busy {
                return Err(AppError::new(
                    "MCP_BUSY",
                    "Finish or cancel this server's active jobs before upgrading",
                ));
            }
        }
        let current: Option<String> = tx
            .query_row(
                "SELECT state_json FROM extension_installations WHERE extension_id=?1",
                [&approval.id],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        if current
            .as_ref()
            .map(|s| {
                serde_json::from_str::<ExtensionState>(s)
                    .map(|s| s.revision)
                    .map_err(json_error)
            })
            .transpose()?
            != previous.as_ref().map(|s| s.revision.clone())
        {
            return Err(AppError::new(
                "PLAN_CHANGED",
                "Extension changed during approval",
            ));
        }
        tx.execute("INSERT OR IGNORE INTO projects(id,name,description,created_at,updated_at,archived) VALUES(?1,'ReconX AI Environment','Internal approved installation history; no research targets',?2,?2,1)",params![ENV_PROJECT,now()]).map_err(db_error)?;
        tx.execute("INSERT INTO analysis_jobs(id,project_id,kind,status,created_at,timeout_ms,logs_json) VALUES(?1,?2,'extension_install','queued',?3,600000,'[\"Exact dependency plan approved\"]')",params![job,ENV_PROJECT,now()]).map_err(db_error)?;
        let mut state = previous.clone().unwrap_or(ExtensionState {
            version: approval.version.clone(),
            enabled: false,
            status: "installing".into(),
            revision: reservation.clone(),
            updated_at: now(),
            last_health_check: None,
            error_code: None,
        });
        state.revision = reservation.clone();
        state.status = "installing".into();
        state.updated_at = now();
        tx.execute("INSERT INTO extension_installations VALUES(?1,?2) ON CONFLICT(extension_id) DO UPDATE SET state_json=excluded.state_json",params![approval.id,serde_json::to_string(&state).map_err(json_error)?]).map_err(db_error)?;
        tx.execute("INSERT INTO extension_install_jobs(job_id,extension_id,plan_json,previous_state_json,previous_config_revision,reservation_revision) VALUES(?1,?2,?3,?4,?5,?6)",params![job,approval.id,serde_json::to_string(&plan).map_err(json_error)?,current,descriptor.as_ref().map(|d|&d.config_revision),reservation]).map_err(db_error)?;
        Self::audit(
            &tx,
            "extension.install.queued",
            &approval.id,
            "exact_plan_approved",
        )?;
        tx.commit().map_err(db_error)?;
        self.job(&job)
    }
    pub(super) fn prepare_managed_install(
        &self,
        job: &AnalysisJob,
        update: &mut impl FnMut(u8, &str) -> Result<()>,
    ) -> Result<jobs::PreparedJob> {
        let text: String = self
            .db
            .connect()?
            .query_row(
                "SELECT plan_json FROM extension_install_jobs WHERE job_id=?1",
                [&job.id],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        let plan: Value = serde_json::from_str(&text).map_err(json_error)?;
        let package: NpmPlan =
            serde_json::from_value(plan.get("package_plan").cloned().ok_or_else(|| {
                AppError::new(
                    "PACKAGE_POLICY",
                    "Queued installation has no reviewed package plan",
                )
            })?)
            .map_err(json_error)?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| AppError::new("WORKER_ERROR", "Cannot start installation runtime"))?;
        let progress = std::cell::RefCell::new(update);
        runtime.block_on(async {
            (*progress.borrow_mut())(5,"Preparing approved runtime and dependency graph")?;
            let manager=self.package_manager();
            let last_progress=std::cell::Cell::new(std::time::Instant::now()-std::time::Duration::from_secs(1));
            let install=manager.install(&package,|stage,bytes,total|{if last_progress.get().elapsed()<std::time::Duration::from_millis(250){return Ok(());}last_progress.set(std::time::Instant::now());let detail=match total{Some(n)=>format!("{stage}: {bytes}/{n} bytes"),None=>format!("{stage}: {bytes} bytes")};(*progress.borrow_mut())(5,&detail)});tokio::pin!(install);
            let receipt=loop{tokio::select!{result=&mut install=>break result?,_=tokio::time::sleep(std::time::Duration::from_millis(100))=>{if last_progress.get().elapsed()>=std::time::Duration::from_secs(1){last_progress.set(std::time::Instant::now());(*progress.borrow_mut())(5,"Waiting for approved dependency I/O")?;}}}};
            manager.verify(&receipt)?;(*progress.borrow_mut())(60,"Verified dependency tree; negotiating real MCP health check")?;
            let health_id=format!("rxhealth-{}",job.id);let server_id=format!("rxmcp-{}",reconx_tools::hash(MANAGED_MCP_EVERYTHING.as_bytes())[..24].to_owned());
            let old=self.managed_descriptor(MANAGED_MCP_EVERYTHING)?;
            let config=McpServerConfig{id:health_id.clone(),name:"MCP Everything health check".into(),transport:"stdio".into(),executable:Some(receipt.runtime.executable.clone()),args:vec![std::path::Path::new(&receipt.directory).join("node_modules/@modelcontextprotocol/server-everything/dist/index.js").to_string_lossy().into(),"stdio".into()],working_directory:Some(receipt.directory.clone()),endpoint:None,environment:vec![],bearer_auth:false,enabled:true,trusted:true,timeout_ms:60000,allowed_tools:vec![]};
            let isolated=Core{db:self.db.clone(),secrets:self.secrets.clone(),mcp_gateway:Arc::new(mcp::McpGateway::default())};
            let health=async{isolated.mcp_save(config.clone()).await?;let connected=isolated.mcp_connect(&health_id);tokio::pin!(connected);loop{tokio::select!{value=&mut connected=>break value,_=tokio::time::sleep(std::time::Duration::from_millis(100))=>{(*progress.borrow_mut())(60,"Waiting for MCP initialize and list_tools")?;}}}}.await;
            let _=isolated.mcp_disconnect(&health_id).await;let _=isolated.mcp_remove(&health_id).await;
            let health=health?;if health.tools.is_empty(){return Err(AppError::new("MCP_HEALTH","Package connected but returned no tools"));}
            let mut config=config;config.id=server_id;config.name="MCP Everything (managed)".into();config.allowed_tools=old.as_ref().map(|d|d.config.allowed_tools.clone()).unwrap_or_default();config.enabled=old.as_ref().map(|d|d.config.enabled).unwrap_or(true);
            if config.allowed_tools.iter().any(|name|!health.tools.iter().any(|t|t["name"]==name.as_str())){return Err(AppError::new("MCP_TOOL_CHANGED","A previously allowed tool disappeared; review the update permissions"));}
            let descriptor=Descriptor{config_revision:revision(&config)?,config:config.clone(),receipt,tools:health.tools};
            self.db.connect()?.execute("UPDATE extension_install_jobs SET staged_json=?1 WHERE job_id=?2",params![serde_json::to_string(&descriptor).map_err(json_error)?,job.id]).map_err(db_error)?;
            (*progress.borrow_mut())(90,"Actual MCP discovery passed; ready for atomic activation")?;
            Ok(jobs::PreparedJob{result:json!({"extension_id":MANAGED_MCP_EVERYTHING,"package":"@modelcontextprotocol/server-everything","version":package.version,"dependencies_verified":package.dependencies.len(),"server_id":config.id,"tools_discovered":descriptor.tools.len(),"tools_invoked_during_health":0,"verification":"Actual MCP initialize/list_tools and schema validation"}),analysis:None,http:None})
        })
    }
    pub(super) fn commit_managed_install(
        &self,
        conn: &Connection,
        job: &AnalysisJob,
    ) -> Result<()> {
        let (id,reservation,old_revision,staged):(String,String,Option<String>,String)=conn.query_row("SELECT extension_id,reservation_revision,previous_config_revision,staged_json FROM extension_install_jobs WHERE job_id=?1",[&job.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).map_err(db_error)?;
        let current:String=conn.query_row("SELECT json_extract(state_json,'$.revision') FROM extension_installations WHERE extension_id=?1",[&id],|r|r.get(0)).map_err(db_error)?;
        if current != reservation {
            return Err(AppError::new(
                "EXTENSION_CHANGED",
                "Registration changed before activation",
            ));
        }
        let descriptor: Descriptor = serde_json::from_str(&staged).map_err(json_error)?;
        let actual: Option<String> = conn
            .query_row(
                "SELECT revision FROM mcp_servers WHERE id=?1",
                [&descriptor.config.id],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_error)?;
        if actual != old_revision {
            return Err(AppError::new(
                "MANAGED_CONFIG_CHANGED",
                "User-edited MCP configuration was preserved; update activation rejected",
            ));
        }
        conn.execute("INSERT INTO mcp_servers(id,config_json,revision,tools_json,last_connected_at,updated_at) VALUES(?1,?2,?3,?4,?5,?5) ON CONFLICT(id) DO UPDATE SET config_json=excluded.config_json,revision=excluded.revision,tools_json=excluded.tools_json,last_connected_at=excluded.last_connected_at,updated_at=excluded.updated_at",params![descriptor.config.id,serde_json::to_string(&descriptor.config).map_err(json_error)?,descriptor.config_revision,serde_json::to_string(&descriptor.tools).map_err(json_error)?,now()]).map_err(db_error)?;
        conn.execute("INSERT INTO managed_mcp_extensions VALUES(?1,?2,?3) ON CONFLICT(extension_id) DO UPDATE SET server_id=excluded.server_id,descriptor_json=excluded.descriptor_json",params![id,descriptor.config.id,staged]).map_err(db_error)?;
        let state = ExtensionState {
            version: descriptor.receipt.version,
            enabled: descriptor.config.enabled,
            status: "installed".into(),
            revision: reservation,
            updated_at: now(),
            last_health_check: Some(now()),
            error_code: None,
        };
        conn.execute(
            "UPDATE extension_installations SET state_json=?1 WHERE extension_id=?2",
            params![serde_json::to_string(&state).map_err(json_error)?, id],
        )
        .map_err(db_error)?;
        conn.execute(
            "UPDATE extension_install_jobs SET reconciled=1 WHERE job_id=?1",
            [&job.id],
        )
        .map_err(db_error)?;
        Self::audit(conn, "extension.package.activated", &id, "health_verified")
    }
    pub(super) fn managed_reconcile(&self) -> Result<()> {
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let rows=tx.prepare("SELECT o.job_id,o.extension_id,o.previous_state_json,o.reservation_revision,j.error_code FROM extension_install_jobs o JOIN analysis_jobs j ON j.id=o.job_id WHERE o.reconciled=0 AND j.status IN ('failed','cancelled')").map_err(db_error)?.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,Option<String>>(2)?,r.get::<_,String>(3)?,r.get::<_,Option<String>>(4)?))).map_err(db_error)?.collect::<std::result::Result<Vec<_>,_>>().map_err(db_error)?;
        for (job, id, previous, reservation, error) in rows {
            let state = if let Some(previous) = previous {
                previous
            } else {
                serde_json::to_string(&ExtensionState {
                    version: "2.0.0".into(),
                    enabled: false,
                    status: "failed".into(),
                    revision: reservation.clone(),
                    updated_at: now(),
                    last_health_check: None,
                    error_code: error,
                })
                .map_err(json_error)?
            };
            tx.execute("UPDATE extension_installations SET state_json=?1 WHERE extension_id=?2 AND json_extract(state_json,'$.revision')=?3",params![state,id,reservation]).map_err(db_error)?;
            tx.execute(
                "UPDATE extension_install_jobs SET reconciled=1 WHERE job_id=?1",
                [job],
            )
            .map_err(db_error)?;
            Self::audit(
                &tx,
                "extension.install.recovered",
                &id,
                "previous_configuration_retained",
            )?;
        }
        tx.commit().map_err(db_error)
    }
    pub(super) fn managed_verify_server(&self, id: &str) -> Result<()> {
        let text:Option<(String,String)>=self.db.connect()?.query_row("SELECT extension_id,descriptor_json FROM managed_mcp_extensions WHERE server_id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(db_error)?;
        if let Some((extension, text)) = text {
            let state = self.extension_state(&extension)?.ok_or_else(|| {
                AppError::new(
                    "EXTENSION_DISABLED",
                    "Managed server has no enabled extension",
                )
            })?;
            if !state.enabled {
                return Err(AppError::new(
                    "EXTENSION_DISABLED",
                    "Managed MCP extension is disabled",
                ));
            }
            if self
                .managed_install_history(&extension)?
                .iter()
                .any(|j| matches!(j.status.as_str(), "queued" | "running"))
            {
                return Err(AppError::new(
                    "EXTENSION_BUSY",
                    "Wait for the reviewed installation to complete",
                ));
            }
            let descriptor: Descriptor = serde_json::from_str(&text).map_err(json_error)?;
            if self.mcp_server(id)?.revision != descriptor.config_revision {
                return Err(AppError::new(
                    "MANAGED_CONFIG_CHANGED",
                    "Managed MCP configuration changed outside this extension",
                ));
            }
            self.package_manager().verify(&descriptor.receipt)?;
        }
        Ok(())
    }
    pub async fn managed_set_enabled(&self, enabled: bool) -> Result<ExtensionEntry> {
        self.managed_change_config(Some(enabled), None).await?;
        self.managed_entry()
    }
    pub async fn managed_allow_tools(
        &self,
        tools: Vec<String>,
        approved: bool,
    ) -> Result<ExtensionEntry> {
        if !approved {
            return Err(AppError::new(
                "APPROVAL_REQUIRED",
                "Approve the explicitly selected MCP tools",
            ));
        }
        self.managed_change_config(None, Some(tools)).await?;
        self.managed_entry()
    }
    async fn managed_change_config(
        &self,
        enabled: Option<bool>,
        tools: Option<Vec<String>>,
    ) -> Result<()> {
        let entry = self.managed_entry()?;
        if entry
            .managed
            .as_ref()
            .is_some_and(|m| m.active_job.is_some())
        {
            return Err(AppError::new(
                "EXTENSION_BUSY",
                "Cancel or finish the active installation first",
            ));
        }
        let mut descriptor = self
            .managed_descriptor(MANAGED_MCP_EVERYTHING)?
            .ok_or_else(|| {
                AppError::new("EXTENSION_NOT_INSTALLED", "Install the MCP package first")
            })?;
        if self.mcp_server(&descriptor.config.id)?.revision != descriptor.config_revision {
            return Err(AppError::new(
                "MANAGED_CONFIG_CHANGED",
                "User-edited MCP config requires reconciliation",
            ));
        }
        if enabled == Some(true) {
            self.package_manager().verify(&descriptor.receipt)?;
        }
        if let Some(tools) = tools {
            if tools.len() > 64
                || tools.iter().any(|name| {
                    !descriptor.tools.iter().any(|t| {
                        t["name"] == name.as_str()
                            && t["annotations"]["readOnlyHint"] == true
                            && t["annotations"]["openWorldHint"] == false
                            && t["annotations"]["destructiveHint"] != true
                    })
                })
            {
                return Err(AppError::new(
                    "BLOCKED_BY_POLICY",
                    "Only discovered read-only closed-world tools may be allowed",
                ));
            }
            descriptor.config.allowed_tools = tools;
        }
        if let Some(enabled) = enabled {
            descriptor.config.enabled = enabled;
        }
        self.mcp_disconnect(&descriptor.config.id).await?;
        let old = descriptor.config_revision.clone();
        descriptor.config_revision = revision(&descriptor.config)?;
        let mut conn = self.db.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let changed=tx.execute("UPDATE mcp_servers SET config_json=?1,revision=?2,updated_at=?3 WHERE id=?4 AND revision=?5",params![serde_json::to_string(&descriptor.config).map_err(json_error)?,descriptor.config_revision,now(),descriptor.config.id,old]).map_err(db_error)?;
        if changed != 1 {
            return Err(AppError::new(
                "MANAGED_CONFIG_CHANGED",
                "Configuration changed concurrently",
            ));
        }
        tx.execute(
            "UPDATE managed_mcp_extensions SET descriptor_json=?1 WHERE extension_id=?2",
            params![
                serde_json::to_string(&descriptor).map_err(json_error)?,
                MANAGED_MCP_EVERYTHING
            ],
        )
        .map_err(db_error)?;
        if let Some(enabled) = enabled {
            let mut state = self
                .extension_state(MANAGED_MCP_EVERYTHING)?
                .ok_or_else(json_error_missing)?;
            state.enabled = enabled;
            state.revision = Uuid::new_v4().to_string();
            state.updated_at = now();
            tx.execute(
                "UPDATE extension_installations SET state_json=?1 WHERE extension_id=?2",
                params![
                    serde_json::to_string(&state).map_err(json_error)?,
                    MANAGED_MCP_EVERYTHING
                ],
            )
            .map_err(db_error)?;
            if !enabled {
                tx.execute("UPDATE analysis_jobs SET cancel_requested=1,status=CASE WHEN status='queued' THEN 'cancelled' ELSE status END WHERE status IN ('queued','running') AND id IN (SELECT job_id FROM capability_invocations WHERE server_id=?1)",[&descriptor.config.id]).map_err(db_error)?;
            }
        }
        Self::audit(
            &tx,
            "extension.mcp.permissions",
            MANAGED_MCP_EVERYTHING,
            "explicit_change",
        )?;
        tx.commit().map_err(db_error)
    }
    pub async fn managed_doctor(&self) -> Result<ExtensionEntry> {
        let descriptor = self
            .managed_descriptor(MANAGED_MCP_EVERYTHING)?
            .ok_or_else(json_error_missing)?;
        self.managed_verify_server(&descriptor.config.id)?;
        let checked = self.mcp_connect(&descriptor.config.id).await;
        let mut state = self
            .extension_state(MANAGED_MCP_EVERYTHING)?
            .ok_or_else(json_error_missing)?;
        if let Err(error) = checked {
            state.last_health_check = None;
            state.error_code = Some(error.code.clone());
            self.write_extension_state(MANAGED_MCP_EVERYTHING, &state)?;
            return Err(error);
        }
        state.last_health_check = Some(now());
        state.error_code = None;
        self.write_extension_state(MANAGED_MCP_EVERYTHING, &state)?;
        self.managed_entry()
    }
    pub async fn managed_remove(&self) -> Result<()> {
        let entry = self.managed_entry()?;
        if entry
            .managed
            .as_ref()
            .is_some_and(|m| m.active_job.is_some())
        {
            return Err(AppError::new(
                "EXTENSION_BUSY",
                "Cancel or finish installation before uninstall",
            ));
        }
        let Some(descriptor) = self.managed_descriptor(MANAGED_MCP_EVERYTHING)? else {
            self.db
                .connect()?
                .execute(
                    "DELETE FROM extension_installations WHERE extension_id=?1",
                    [MANAGED_MCP_EVERYTHING],
                )
                .map_err(db_error)?;
            return Ok(());
        };
        if self.mcp_server(&descriptor.config.id)?.revision != descriptor.config_revision {
            return Err(AppError::new(
                "MANAGED_CONFIG_CHANGED",
                "User-edited MCP config was preserved; reconcile before uninstall",
            ));
        }
        self.package_manager().verify(&descriptor.receipt)?;
        self.managed_set_enabled(false).await?;
        self.mcp_remove(&descriptor.config.id).await?;
        self.package_manager().remove(&descriptor.receipt)?;
        self.db
            .connect()?
            .execute(
                "DELETE FROM extension_installations WHERE extension_id=?1",
                [MANAGED_MCP_EVERYTHING],
            )
            .map_err(db_error)?;
        Self::audit(
            &self.db.connect()?,
            "extension.remove",
            MANAGED_MCP_EVERYTHING,
            "owned_package_removed_shared_runtime_retained",
        )
    }
}
fn json_error_missing() -> AppError {
    AppError::new(
        "EXTENSION_NOT_INSTALLED",
        "Managed MCP installation record is unavailable",
    )
}
