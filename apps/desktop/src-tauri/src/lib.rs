use reconx_core::{
    AppError, ArtifactReview, Core, EvidenceInput, FindingInput, JobEngine, JobInput, JobKind,
    JobReview, McpApproval, McpServerConfig, PlanReview, ProjectExport, ProviderConfig,
    ReportFormat, ReportInput, Result, Settings, TargetExport, TargetInput, ThemePreference,
    ToolProgress,
};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, sync::Mutex};
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::watch;
use zeroize::Zeroizing;

#[derive(Default)]
struct ActiveStreams(Mutex<HashMap<String, watch::Sender<bool>>>);
struct ActiveJobs(Mutex<Option<JobEngine>>);

#[tauri::command]
async fn install_tool(
    core: tauri::State<'_, Core>,
    id: String,
    active: tauri::State<'_, ActiveStreams>,
    updates: tauri::ipc::Channel<ToolProgress>,
) -> Result<Value> {
    let request_id = format!("tool:{id}");
    let (tx, mut rx) = watch::channel(false);
    {
        let mut running = active
            .0
            .lock()
            .map_err(|_| AppError::new("WORKER_ERROR", "Cannot access installation registry"))?;
        if running.contains_key(&request_id) {
            return Err(AppError::new(
                "TOOL_BUSY",
                "An installation of this tool is already running",
            ));
        }
        running.insert(request_id.clone(), tx);
    }
    let result = tokio::select! {
      result=core.tool_install(&id,move |progress| {let _=updates.send(progress);}) => result,
      _=rx.changed()=>Err(AppError::new("CANCELLED","Tool download cancelled before installation")),
    };
    active
        .0
        .lock()
        .map_err(|_| AppError::new("WORKER_ERROR", "Cannot access installation registry"))?
        .remove(&request_id);
    value(result?)
}

#[tauri::command]
async fn stream_chat(
    core: tauri::State<'_, Core>,
    active: tauri::State<'_, ActiveStreams>,
    id: String,
    prompt: String,
    request_id: String,
    updates: tauri::ipc::Channel<String>,
) -> Result<Value> {
    uuid::Uuid::parse_str(&request_id)
        .map_err(|_| AppError::new("INVALID_INPUT", "Stream request ID must be a UUID"))?;
    let (tx, rx) = watch::channel(false);
    {
        let mut running = active
            .0
            .lock()
            .map_err(|_| AppError::new("WORKER_ERROR", "Cannot access stream registry"))?;
        if running.len() >= 4 || running.contains_key(&request_id) {
            return Err(AppError::new(
                "AI_BUSY",
                "At most four streams may run; request IDs must be unique",
            ));
        }
        running.insert(request_id.clone(), tx.clone());
    }
    let result = core
        .stream_chat(&id, &prompt, rx, move |text| {
            if updates.send(text.to_owned()).is_err() {
                let _ = tx.send(true);
            }
        })
        .await;
    active
        .0
        .lock()
        .map_err(|_| AppError::new("WORKER_ERROR", "Cannot access stream registry"))?
        .remove(&request_id);
    value(result?)
}
#[tauri::command]
fn cancel_stream(active: tauri::State<'_, ActiveStreams>, request_id: String) -> Result<()> {
    let running = active
        .0
        .lock()
        .map_err(|_| AppError::new("WORKER_ERROR", "Cannot access stream registry"))?;
    let tx = running
        .get(&request_id)
        .ok_or_else(|| AppError::new("NOT_FOUND", "Stream is no longer running"))?;
    tx.send(true)
        .map_err(|_| AppError::new("NOT_FOUND", "Stream is no longer running"))
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    ToolsCatalog,
    ToolsHistory {
        id: String,
    },
    ToolsHost,
    ToolsProfiles,
    ToolsProfilePlan {
        id: String,
    },
    McpList,
    McpHistory {
        id: String,
    },
    JobEvidencePreview {
        id: String,
    },
    JobInterpret {
        review: JobReview,
    },
    JobInterpretations {
        id: String,
    },
    McpSave {
        server: McpServerConfig,
    },
    McpRemove {
        id: String,
    },
    McpConnect {
        id: String,
    },
    McpDisconnect {
        id: String,
    },
    McpState {
        id: String,
    },
    McpSetSecret {
        id: String,
        name: String,
        key: String,
    },
    McpPreview {
        id: String,
        tool: String,
        input: Value,
    },
    McpInvoke {
        approval: McpApproval,
    },
    Capabilities {
        project: String,
    },
    PlanPreview {
        file_id: String,
    },
    PlanCreate {
        review: PlanReview,
    },
    PlansList {
        project: String,
    },
    PlanApprove {
        id: String,
        approved: bool,
    },
    TargetsList {
        project: Option<String>,
    },
    TargetShow {
        id: String,
    },
    TargetCreate {
        input: TargetInput,
    },
    TargetUpdate {
        id: String,
        input: TargetInput,
    },
    TargetRemove {
        id: String,
    },
    TargetsImport {
        export: TargetExport,
    },
    TargetsImportFile {
        path: std::path::PathBuf,
    },
    TargetsExport {
        project: Option<String>,
    },
    TargetsExportFile {
        project: Option<String>,
        path: std::path::PathBuf,
    },
    ScopeValidate {
        id: String,
        candidate: Option<String>,
    },
    FilesList {
        project: Option<String>,
    },
    FileImport {
        project: String,
        path: std::path::PathBuf,
        authorized: bool,
        folder: bool,
    },
    FileImportTarget {
        id: String,
    },
    FileRemove {
        id: String,
    },
    FileAnalysis {
        id: String,
    },
    FilesCompare {
        a: String,
        b: String,
    },
    JobsList {
        project: Option<String>,
    },
    JobShow {
        id: String,
    },
    JobCancel {
        id: String,
    },
    FileAnalyze {
        id: String,
        timeout_ms: Option<u64>,
        kind: Option<JobKind>,
    },
    JobEnqueue {
        input: JobInput,
    },
    JobResult {
        id: String,
    },
    FileNote {
        id: String,
    },
    SaveFileNote {
        id: String,
        note: String,
    },
    FileArtifactPreview {
        id: String,
        section: String,
    },
    FileExplain {
        review: ArtifactReview,
    },
    ReconPreview {
        id: String,
    },
    ReconEnqueue {
        id: String,
        fingerprint: String,
        approved: bool,
    },
    AssetsList {
        project: Option<String>,
    },
    EvidenceList {
        project: Option<String>,
    },
    EvidenceShow {
        id: String,
    },
    EvidenceAdd {
        input: EvidenceInput,
        text: String,
    },
    EvidenceImport {
        input: EvidenceInput,
        path: std::path::PathBuf,
    },
    EvidenceFromJob {
        id: String,
    },
    EvidenceReview {
        id: String,
        note: String,
    },
    EvidenceVerify {
        id: String,
    },
    EvidenceExport {
        id: String,
        path: std::path::PathBuf,
        raw_approved: bool,
    },
    FindingsList {
        project: Option<String>,
    },
    FindingShow {
        id: String,
    },
    FindingSave {
        id: Option<String>,
        input: Box<FindingInput>,
        revision: Option<u32>,
        confirm_verified: bool,
    },
    FindingRemove {
        id: String,
        revision: u32,
    },
    FindingCompare {
        a: String,
        b: String,
    },
    FindingMerge {
        primary: String,
        duplicate: String,
    },
    CvssCalculate {
        vector: String,
    },
    ReportsList {
        project: Option<String>,
    },
    ReportShow {
        id: String,
    },
    ReportPrepare {
        input: ReportInput,
    },
    ReportExport {
        id: String,
        format: ReportFormat,
        path: std::path::PathBuf,
        reviewed_sha256: String,
    },
    Snapshot,
    GetTheme,
    SetTheme {
        theme: ThemePreference,
    },
    ToolsList,
    ToolsPlan {
        id: String,
    },
    ToolsDoctor {
        id: String,
    },
    ToolsUninstall {
        id: String,
    },
    CreateProject {
        name: String,
        description: String,
    },
    UpdateProject {
        id: String,
        name: String,
        description: String,
    },
    ArchiveProject {
        id: String,
        archived: bool,
    },
    ExportProject {
        id: String,
    },
    ImportProject {
        project: ProjectExport,
    },
    SaveSettings {
        settings: Settings,
    },
    AiCatalog,
    AiManager,
    AiRoutingSave {
        routes: Vec<reconx_core::AgentRoute>,
    },
    AiAgentProvider {
        agent: String,
    },
    AiProfileSave {
        provider: ProviderConfig,
        vendor_id: String,
        key: Option<String>,
        make_default: bool,
        #[serde(default)]
        options: reconx_core::GatewayOptions,
        headers: Option<std::collections::BTreeMap<String, String>>,
    },
    AiConnectionTest {
        id: String,
    },
    AiModelsRefresh {
        id: String,
    },
    AiModelsDiscover {
        provider: ProviderConfig,
        key: Option<String>,
        #[serde(default)]
        options: reconx_core::GatewayOptions,
    },
    SaveProvider {
        provider: ProviderConfig,
    },
    DeleteProvider {
        id: String,
    },
    SetKey {
        id: String,
        key: String,
    },
    RemoveKey {
        id: String,
    },
    CredentialStatus {
        id: String,
    },
    Models {
        id: String,
    },
    Chat {
        id: String,
        prompt: String,
    },
}
fn value(item: impl serde::Serialize) -> Result<Value> {
    serde_json::to_value(item)
        .map_err(|_| AppError::new("SERIALIZATION_ERROR", "Cannot serialize response"))
}
fn local(core: Core, request: Request) -> Result<Value> {
    match request {
        Request::AiCatalog => value(core.ai_catalog()),
        Request::AiManager => value(core.ai_manager()?),
        Request::AiRoutingSave { routes } => {
            core.save_ai_routing(&routes)?;
            Ok(Value::Null)
        }
        Request::AiAgentProvider { agent } => value(core.ai_agent_provider(&agent)?),
        Request::AiProfileSave {
            provider,
            vendor_id,
            key,
            make_default,
            options,
            headers,
        } => {
            let key = key.map(Zeroizing::new);
            core.save_ai_setup(
                reconx_core::AiProfileInput {
                    provider,
                    vendor_id,
                    make_default,
                    options,
                },
                key.as_ref().map(|s| s.as_str()),
                headers.as_ref(),
            )?;
            Ok(Value::Null)
        }
        Request::ReconPreview { id } => value(core.local_http_preview(&id)?),
        Request::ReconEnqueue {
            id,
            fingerprint,
            approved,
        } => value(core.local_http_enqueue(&id, &fingerprint, approved)?),
        Request::AssetsList { project } => value(core.assets(project.as_deref())?),
        Request::EvidenceList { project } => value(core.evidence_list(project.as_deref())?),
        Request::EvidenceShow { id } => value(core.evidence_show(&id)?),
        Request::EvidenceAdd { input, text } => value(core.evidence_add(input, &text)?),
        Request::EvidenceImport { input, path } => value(core.evidence_import(input, &path)?),
        Request::EvidenceFromJob { id } => value(core.evidence_from_job(&id)?),
        Request::EvidenceReview { id, note } => value(core.evidence_review(&id, &note)?),
        Request::EvidenceVerify { id } => core.evidence_verify(&id),
        Request::EvidenceExport {
            id,
            path,
            raw_approved,
        } => core.evidence_export(&id, &path, raw_approved),
        Request::FindingsList { project } => value(core.findings(project.as_deref())?),
        Request::FindingShow { id } => value(core.finding(&id)?),
        Request::FindingSave {
            id,
            input,
            revision,
            confirm_verified,
        } => value(core.finding_save(id.as_deref(), *input, revision, confirm_verified)?),
        Request::FindingRemove { id, revision } => {
            core.finding_remove(&id, revision)?;
            Ok(Value::Null)
        }
        Request::FindingCompare { a, b } => core.finding_compare(&a, &b),
        Request::FindingMerge { primary, duplicate } => core.finding_merge(&primary, &duplicate),
        Request::CvssCalculate { vector } => value(reconx_core::calculate_cvss(&vector)?),
        Request::ReportsList { project } => value(core.reports(project.as_deref())?),
        Request::ReportShow { id } => value(core.report_show(&id)?),
        Request::ReportPrepare { input } => value(core.report_prepare(input)?),
        Request::ReportExport {
            id,
            format,
            path,
            reviewed_sha256,
        } => core.report_export(&id, format, &path, &reviewed_sha256),
        Request::TargetsList { project } => value(core.list_targets(project.as_deref())?),
        Request::TargetShow { id } => value(core.target(&id)?),
        Request::TargetCreate { input } => value(core.create_target(input)?),
        Request::TargetUpdate { id, input } => value(core.update_target(&id, input)?),
        Request::TargetRemove { id } => {
            core.remove_target(&id)?;
            Ok(Value::Null)
        }
        Request::TargetsImport { export } => value(core.import_targets(export)?),
        Request::TargetsImportFile { path } => {
            use std::io::Read;
            let mut bytes = Vec::new();
            let file = std::fs::File::open(path)
                .map_err(|_| AppError::new("FILE_IO", "Cannot read target list"))?;
            file.take(1024 * 1024 + 1)
                .read_to_end(&mut bytes)
                .map_err(|_| AppError::new("FILE_IO", "Cannot read target list"))?;
            if bytes.len() > 1024 * 1024 {
                return Err(AppError::new("INPUT_LIMIT", "Target list exceeds 1 MiB"));
            }
            let export = serde_json::from_slice(&bytes).map_err(|_| {
                AppError::new("INVALID_IMPORT", "Expected a version-1 target export JSON")
            })?;
            value(core.import_targets(export)?)
        }
        Request::TargetsExport { project } => value(core.export_targets(project.as_deref())?),
        Request::TargetsExportFile { project, path } => {
            use std::io::Write;
            let bytes = serde_json::to_vec_pretty(&core.export_targets(project.as_deref())?)
                .map_err(|_| AppError::new("INVALID_DATA", "Cannot serialize targets"))?;
            let mut file = std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(path)
                .map_err(|_| {
                    AppError::new(
                        "FILE_IO",
                        "Choose a writable export path that does not exist",
                    )
                })?;
            file.write_all(&bytes)
                .map_err(|_| AppError::new("FILE_IO", "Cannot write target export"))?;
            Ok(Value::Null)
        }
        Request::ScopeValidate { id, candidate } => {
            value(core.validate_scope(&id, candidate.as_deref())?)
        }
        Request::FilesList { project } => value(core.list_files(project.as_deref())?),
        Request::FileImport {
            project,
            path,
            authorized,
            folder,
        } => {
            if folder {
                value(core.import_folder(&project, &path, authorized)?)
            } else {
                value(core.import_file(&project, &path, authorized)?)
            }
        }
        Request::FileImportTarget { id } => {
            let target = core.target(&id)?;
            let decision = core.validate_scope(&id, None)?;
            if decision.status != "allowed" {
                return Err(AppError::new("BLOCKED_BY_POLICY", &decision.reason));
            }
            if !target.input.kind.local_file() {
                return Err(AppError::new(
                    "UNSUPPORTED_TARGET",
                    "Only local binary, library, firmware or APK targets can be imported here",
                ));
            }
            value(core.import_file(
                &target.input.project_id,
                std::path::Path::new(&target.input.value),
                target.input.authorized,
            )?)
        }
        Request::FileRemove { id } => {
            core.remove_file(&id)?;
            Ok(Value::Null)
        }
        Request::FileAnalysis { id } => value(core.file_analysis(&id)?),
        Request::FilesCompare { a, b } => value(core.compare_files(&a, &b)?),
        Request::JobsList { project } => value(core.list_jobs(project.as_deref())?),
        Request::JobEnqueue { input } => value(core.enqueue_job(input)?),
        Request::FileAnalyze {
            id,
            timeout_ms,
            kind,
        } => value(core.enqueue_job(JobInput {
            kind: kind.unwrap_or(JobKind::LocalStaticBinary),
            file_id: Some(id),
            target_id: None,
            timeout_ms: timeout_ms.unwrap_or(60_000),
        })?),
        Request::JobResult { id } => {
            let job = core.job(&id)?;
            if job.status != "completed" {
                return Err(AppError::new(
                    "JOB_NOT_COMPLETED",
                    "This job has no committed result",
                ));
            }
            if job.file_id.is_some()
                && matches!(
                    job.kind.as_str(),
                    "local_static_binary" | "binary_metadata" | "strings_extraction"
                )
            {
                value(core.analysis_for_job(&id)?)
            } else {
                value(job.result)
            }
        }
        Request::FileNote { id } => value(core.file_note(&id)?),
        Request::SaveFileNote { id, note } => {
            core.save_file_note(&id, &note)?;
            Ok(Value::Null)
        }
        Request::FileArtifactPreview { id, section } => {
            value(core.file_artifact_preview(&id, &section)?)
        }
        Request::JobShow { id } => value(core.job(&id)?),
        Request::JobCancel { id } => {
            core.cancel_job(&id)?;
            Ok(Value::Null)
        }
        Request::Snapshot => value(core.snapshot()?),
        Request::GetTheme => value(core.settings()?.theme),
        Request::SetTheme { theme } => value(core.set_theme(theme)?),
        Request::ToolsCatalog => value(core.tool_catalog()?),
        Request::ToolsHistory { id } => value(core.tool_history(&id)?),
        Request::ToolsHost => value(core.tool_host()),
        Request::ToolsProfiles => value(core.tool_profiles()),
        Request::ToolsProfilePlan { id } => value(core.tool_profile_plan(&id)?),
        Request::McpHistory { id } => value(core.mcp_history(&id)?),
        Request::JobEvidencePreview { id } => value(core.job_evidence_preview(&id)?),
        Request::JobInterpretations { id } => value(core.job_interpretations(&id)?),
        Request::McpList => value(core.mcp_servers()?),
        Request::McpPreview { id, tool, input } => {
            value(core.mcp_invocation_preview(&id, &tool, &input)?)
        }
        Request::McpInvoke { approval } => value(core.mcp_enqueue(approval)?),
        Request::McpSetSecret { id, name, key } => {
            let key = Zeroizing::new(key);
            core.mcp_set_secret(&id, &name, &key)?;
            Ok(Value::Null)
        }
        Request::Capabilities { project } => value(core.capabilities(&project)?),
        Request::PlanPreview { file_id } => value(core.plan_preview(&file_id)?),
        Request::PlansList { project } => value(core.tool_plans(&project)?),
        Request::PlanApprove { id, approved } => value(core.approve_tool_plan(&id, approved)?),
        Request::ToolsList => value(core.tool_inventory()?),
        Request::ToolsPlan { id } => value(core.tool_plan(&id)?),
        Request::ToolsUninstall { id } => {
            core.tool_uninstall(&id)?;
            Ok(Value::Null)
        }
        Request::CreateProject { name, description } => {
            value(core.create_project(&name, &description)?)
        }
        Request::UpdateProject {
            id,
            name,
            description,
        } => value(core.update_project(&id, &name, &description)?),
        Request::ArchiveProject { id, archived } => {
            core.archive_project(&id, archived)?;
            Ok(Value::Null)
        }
        Request::ExportProject { id } => value(core.export_project(&id)?),
        Request::ImportProject { project } => value(core.import_project(project)?),
        Request::SaveSettings { settings } => {
            core.save_settings(&settings)?;
            Ok(Value::Null)
        }
        Request::SaveProvider { provider } => {
            core.save_provider(&provider)?;
            Ok(Value::Null)
        }
        Request::DeleteProvider { id } => {
            core.delete_provider(&id)?;
            Ok(Value::Null)
        }
        Request::SetKey { id, key } => {
            let key = Zeroizing::new(key);
            core.set_key(&id, &key)?;
            Ok(Value::Null)
        }
        Request::RemoveKey { id } => {
            core.remove_key(&id)?;
            Ok(Value::Null)
        }
        Request::CredentialStatus { id } => value(core.credential_status(&id)?),
        _ => Err(AppError::new("INVALID_REQUEST", "Invalid local operation")),
    }
}
#[tauri::command]
async fn service(core: tauri::State<'_, Core>, request: Request) -> Result<Value> {
    let core = core.inner().clone();
    match request {
        Request::McpSave { server } => value(core.mcp_save(server).await?),
        Request::McpRemove { id } => {
            core.mcp_remove(&id).await?;
            Ok(Value::Null)
        }
        Request::McpConnect { id } => value(core.mcp_connect(&id).await?),
        Request::McpDisconnect { id } => {
            core.mcp_disconnect(&id).await?;
            Ok(Value::Null)
        }
        Request::McpState { id } => value(core.mcp_state(&id).await),
        Request::JobInterpret { review } => value(core.interpret_job(review).await?),
        Request::PlanCreate { review } => value(core.plan_local_tools(review).await?),
        Request::FileExplain { review } => value(core.explain_file_artifact(review).await?),
        Request::AiConnectionTest { id } => value(core.test_ai_connection(&id).await?),
        Request::AiModelsRefresh { id } => value(core.refresh_ai_models(&id).await?),
        Request::AiModelsDiscover {
            provider,
            key,
            options,
        } => {
            let key = key.map(Zeroizing::new);
            value(
                core.discover_ai_models(provider, key.as_ref().map(|s| s.as_str()), options)
                    .await?,
            )
        }
        Request::Models { id } => value(core.models(&id).await?),
        Request::ToolsDoctor { id } => value(core.tool_doctor(&id).await?),
        Request::Chat { id, prompt } => value(core.chat(&id, &prompt).await?),
        other => tauri::async_runtime::spawn_blocking(move || local(core, other))
            .await
            .map_err(|_| AppError::new("WORKER_ERROR", "Background operation failed"))?,
    }
}
#[tauri::command]
fn apply_window_theme(window: tauri::WebviewWindow, theme: ThemePreference) -> Result<()> {
    let (native_theme, background) = match theme {
        ThemePreference::Light => (
            tauri::Theme::Light,
            tauri::window::Color(245, 246, 248, 255),
        ),
        ThemePreference::Dark => (tauri::Theme::Dark, tauri::window::Color(17, 19, 24, 255)),
        ThemePreference::Reconx => (tauri::Theme::Dark, tauri::window::Color(49, 0, 25, 255)),
    };
    let map_error = |_| {
        AppError::new(
            "WINDOW_APPEARANCE_ERROR",
            "Could not update native window appearance",
        )
    };
    window.set_theme(Some(native_theme)).map_err(map_error)?;
    window
        .set_background_color(Some(background))
        .map_err(map_error)?;
    // Native E2E instances still run the real WebView2/IPC stack. Hide only the
    // owned test window so physical desktop input cannot race CDP interactions.
    if std::env::var("RECONX_NATIVE_TEST_HIDDEN").as_deref() == Ok("1") {
        window.hide().map_err(map_error)
    } else {
        window.show().map_err(map_error)
    }
}
#[tauri::command]
async fn select_local_paths(
    window: tauri::WebviewWindow,
    folder: bool,
    save: bool,
    file_name: Option<String>,
) -> Result<Vec<std::path::PathBuf>> {
    let file_name = file_name.unwrap_or_else(|| "reconx-targets.json".into());
    if file_name.len() > 128
        || file_name.contains(['/', '\\'])
        || file_name.chars().any(char::is_control)
    {
        return Err(AppError::new(
            "INVALID_INPUT",
            "Default export name must be a simple filename",
        ));
    }
    tauri::async_runtime::spawn_blocking(move || {
        let dialog = window.app_handle().dialog().file().set_parent(&window);
        let paths = if save {
            dialog
                .set_file_name(&file_name)
                .blocking_save_file()
                .into_iter()
                .collect()
        } else if folder {
            dialog.blocking_pick_folder().into_iter().collect()
        } else {
            dialog.blocking_pick_files().unwrap_or_default()
        };
        paths
            .into_iter()
            .map(|p| {
                p.into_path().map_err(|_| {
                    AppError::new("INVALID_FILE", "Only native local paths are supported")
                })
            })
            .collect()
    })
    .await
    .map_err(|_| AppError::new("WORKER_ERROR", "Native file dialog failed"))?
}
pub fn run() {
    use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};
    // SDK wire diagnostics can contain untrusted payloads or echoed credentials.
    // This filter cannot be overridden by a more-specific RUST_LOG directive.
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
                    !metadata.target().starts_with("rmcp")
                })),
        )
        .init();
    let core = Core::open(None).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1)
    });
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(core.clone())
        .manage(ActiveJobs(Mutex::new(None)))
        .setup(move |app| {
            let handle = app.handle().clone();
            let engine = JobEngine::start(core.clone(), 2, move |job| {
                let _ = handle.emit("research-job", job);
            })?;
            *app.state::<ActiveJobs>()
                .0
                .lock()
                .map_err(|_| std::io::Error::other("Job engine lock unavailable"))? = Some(engine);
            Ok(())
        })
        .manage(ActiveStreams::default())
        .invoke_handler(tauri::generate_handler![
            service,
            select_local_paths,
            stream_chat,
            cancel_stream,
            install_tool,
            apply_window_theme
        ])
        .build(tauri::generate_context!())
        .expect("Desktop runtime failed")
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::Exit)
                && let Ok(mut jobs) = app.state::<ActiveJobs>().0.lock()
            {
                drop(jobs.take());
            }
        });
}
