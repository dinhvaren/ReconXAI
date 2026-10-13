use reconx_core::{
    AppError, Core, ProjectExport, ProviderConfig, Result, Settings, ThemePreference, ToolProgress,
};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, sync::Mutex};
use tokio::sync::watch;
use zeroize::Zeroizing;

#[derive(Default)]
struct ActiveStreams(Mutex<HashMap<String, watch::Sender<bool>>>);

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
            tauri::window::Color(247, 244, 247, 255),
        ),
        ThemePreference::Dark => (tauri::Theme::Dark, tauri::window::Color(49, 0, 25, 255)),
        ThemePreference::System => {
            return Err(AppError::new(
                "INVALID_INPUT",
                "Window theme must be resolved to light or dark",
            ));
        }
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
    window.show().map_err(map_error)
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
        .manage(core)
        .manage(ActiveStreams::default())
        .invoke_handler(tauri::generate_handler![
            service,
            stream_chat,
            cancel_stream,
            install_tool,
            apply_window_theme
        ])
        .run(tauri::generate_context!())
        .expect("Desktop runtime failed");
}
