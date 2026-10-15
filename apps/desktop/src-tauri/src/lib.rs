use reconx_core::{
    AppError, ArtifactReview, Core, JobEngine, JobInput, JobKind, ProjectExport, ProviderConfig,
    Result, Settings, TargetExport, TargetInput, ThemePreference, ToolProgress,
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
    updates: tauri::ipc::Channel<ToolProgress>,
) -> Result<Value> {
    value(
        core.tool_install(&id, move |progress| {
            let _ = updates.send(progress);
        })
        .await?,
    )
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
        Request::FileExplain { review } => value(core.explain_file_artifact(review).await?),
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
) -> Result<Vec<std::path::PathBuf>> {
    tauri::async_runtime::spawn_blocking(move || {
        let dialog = window.app_handle().dialog().file().set_parent(&window);
        let paths = if save {
            dialog
                .set_file_name("reconx-targets.json")
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
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
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
