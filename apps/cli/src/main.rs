use clap::{CommandFactory, Parser, Subcommand};
use reconx_core::{
    AppError, Core, JobEngine, JobInput, JobKind, ProjectExport, ProviderConfig, Result,
    SecurityMode, Settings, TargetExport, TargetInput, TargetKind,
};
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    path::PathBuf,
};
use zeroize::Zeroizing;

#[derive(Parser)]
#[command(
    name = "reconx",
    version,
    about = "ReconX AI — local research workbench and AI configuration"
)]
struct Cli {
    #[arg(long, global = true, help = "Emit structured JSON output and errors")]
    json: bool,
    #[arg(long, global = true, help = "Override the shared SQLite database path")]
    database: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Init,
    Doctor,
    #[command(subcommand)]
    Project(ProjectCommand),
    #[command(subcommand)]
    Settings(SettingsCommand),
    #[command(subcommand)]
    Ai(AiCommand),
    #[command(subcommand)]
    Tools(ToolsCommand),
    #[command(subcommand)]
    Target(TargetCommand),
    #[command(subcommand)]
    Scope(ScopeCommand),
    #[command(subcommand)]
    Reverse(ReverseCommand),
    #[command(subcommand)]
    Jobs(JobCommand),
    Audit,
    Completions {
        shell: clap_complete::Shell,
    },
}
#[derive(Subcommand)]
enum ProjectCommand {
    Create {
        name: String,
        #[arg(long, default_value = "")]
        description: String,
    },
    List {
        #[arg(long)]
        archived: bool,
    },
    Open {
        project: String,
    },
    Rename {
        project: String,
        name: String,
        #[arg(long)]
        description: Option<String>,
    },
    Archive {
        project: String,
    },
    Restore {
        project: String,
    },
    Export {
        project: String,
        #[arg(long)]
        output: PathBuf,
    },
    Import {
        input: PathBuf,
    },
}
#[derive(Subcommand)]
enum TargetCommand {
    Add {
        #[arg(long)]
        project: String,
        #[arg(long = "type")]
        kind: String,
        #[arg(long)]
        value: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long, default_value = "")]
        description: String,
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
        #[arg(long)]
        allow: Vec<String>,
        #[arg(long)]
        exclude: Vec<String>,
        #[arg(long)]
        authorized: bool,
        #[arg(long, default_value = "passive_auto")]
        mode: String,
    },
    List {
        #[arg(long)]
        project: Option<String>,
    },
    Show {
        id: String,
    },
    /// Replace target configuration using a complete TargetInput JSON file.
    Update {
        id: String,
        #[arg(long)]
        input: PathBuf,
    },
    Remove {
        id: String,
    },
    /// Atomically import a version-1 target export (at most 500 records).
    Import {
        input: PathBuf,
    },
    Export {
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        output: PathBuf,
    },
}
#[derive(Subcommand)]
enum ScopeCommand {
    Show {
        id: String,
    },
    Validate {
        id: String,
        #[arg(long)]
        value: Option<String>,
    },
}
#[derive(Subcommand)]
enum ReverseCommand {
    /// Read local file notes, or replace them from a UTF-8 text file.
    Note {
        id: String,
        #[arg(long)]
        input: Option<PathBuf>,
    },
    Import {
        input: PathBuf,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        authorized: bool,
        #[arg(long)]
        folder: bool,
    },
    List {
        #[arg(long)]
        project: Option<String>,
    },
    Analyze {
        id: String,
    },
    Metadata {
        id: String,
    },
    Strings {
        id: String,
    },
    Remove {
        id: String,
    },
    Compare {
        a: String,
        b: String,
    },
}
#[derive(Subcommand)]
enum JobCommand {
    /// Queue an allowlisted local task. Desktop or `jobs run` executes it.
    Enqueue {
        #[arg(long)]
        kind: String,
        #[arg(long)]
        file: Option<String>,
        #[arg(long)]
        target: Option<String>,
        #[arg(long, default_value_t = 60000)]
        timeout_ms: u64,
    },
    /// Run the durable queue until idle, or continuously with --watch.
    Run {
        #[arg(long, default_value_t = 2)]
        concurrency: usize,
        #[arg(long)]
        watch: bool,
    },
    Result {
        id: String,
    },
    List {
        #[arg(long)]
        project: Option<String>,
    },
    Show {
        id: String,
    },
    Cancel {
        id: String,
    },
}
fn enum_input<T: serde::de::DeserializeOwned>(text: &str) -> Result<T> {
    serde_json::from_value(json!(text)).map_err(|_| {
        AppError::new(
            "INVALID_INPUT",
            "Unknown target type or analysis mode; consult command help and docs",
        )
    })
}
#[derive(Subcommand)]
enum SettingsCommand {
    Show,
    Apply { input: PathBuf },
}
#[derive(Subcommand)]
enum ToolsCommand {
    List,
    Plan {
        tool: String,
    },
    Install {
        tool: String,
    },
    Doctor {
        #[arg(default_value = "ripgrep")]
        tool: String,
    },
    Uninstall {
        tool: String,
    },
}
#[derive(Subcommand)]
enum AiCommand {
    Providers,
    /// Add or update a secret-free provider JSON configuration.
    Configure {
        input: PathBuf,
    },
    Delete {
        provider: String,
    },
    /// Read a credential interactively, or explicitly from standard input.
    Key {
        provider: String,
        #[arg(long)]
        stdin: bool,
    },
    RemoveKey {
        provider: String,
    },
    KeyStatus {
        provider: String,
    },
    Models {
        provider: String,
    },
    Test {
        provider: String,
    },
    /// Send the provided text; AI replies cannot execute actions.
    Chat {
        provider: String,
        #[arg(long)]
        prompt_file: PathBuf,
        #[arg(
            long,
            help = "Stream text incrementally; Ctrl+C cancels. JSON mode returns a single final object"
        )]
        stream: bool,
    },
}
fn read_file(path: &PathBuf) -> Result<String> {
    let metadata =
        std::fs::metadata(path).map_err(|_| AppError::new("FILE_IO", "Cannot read input file"))?;
    if metadata.len() > 1024 * 1024 {
        return Err(AppError::new("INPUT_LIMIT", "Input exceeds 1 MiB"));
    }
    std::fs::read_to_string(path)
        .map_err(|_| AppError::new("FILE_IO", "Cannot read UTF-8 input file"))
}
fn serialize(value: impl serde::Serialize) -> Result<Value> {
    serde_json::to_value(value)
        .map_err(|_| AppError::new("SERIALIZATION_ERROR", "Cannot serialize output"))
}
fn write_new(path: &PathBuf, text: &str) -> Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|_| {
            AppError::new(
                "FILE_IO",
                "Cannot create export; choose a writable path that does not already exist",
            )
        })?;
    file.write_all(text.as_bytes())
        .map_err(|_| AppError::new("FILE_IO", "Cannot write export"))
}
async fn execute(cli: &Cli) -> Result<Value> {
    if let Command::Completions { shell } = cli.command {
        clap_complete::generate(shell, &mut Cli::command(), "reconx", &mut std::io::stdout());
        return Ok(Value::Null);
    }
    let core = Core::open(cli.database.clone())?;
    match &cli.command {
        Command::Target(command) => match command {
            TargetCommand::Add {
                project,
                kind,
                value,
                name,
                description,
                tags,
                allow,
                exclude,
                authorized,
                mode,
            } => serialize(
                core.create_target(TargetInput {
                    project_id: project.clone(),
                    name: name
                        .clone()
                        .unwrap_or_else(|| value.chars().take(120).collect()),
                    kind: enum_input::<TargetKind>(kind)?,
                    value: value.clone(),
                    description: description.clone(),
                    tags: tags.clone(),
                    allowed_scope: allow.clone(),
                    excluded_scope: exclude.clone(),
                    authorized: *authorized,
                    mode: enum_input::<SecurityMode>(mode)?,
                })?,
            ),
            TargetCommand::List { project } => serialize(core.list_targets(project.as_deref())?),
            TargetCommand::Show { id } => serialize(core.target(id)?),
            TargetCommand::Update { id, input } => serialize(
                core.update_target(
                    id,
                    serde_json::from_str(&read_file(input)?)
                        .map_err(|_| AppError::new("INVALID_IMPORT", "Invalid TargetInput JSON"))?,
                )?,
            ),
            TargetCommand::Remove { id } => {
                core.remove_target(id)?;
                Ok(json!({"status":"removed"}))
            }
            TargetCommand::Import { input } => serialize(core.import_targets(
                serde_json::from_str::<TargetExport>(&read_file(input)?).map_err(|_| {
                    AppError::new("INVALID_IMPORT", "Invalid versioned target export")
                })?,
            )?),
            TargetCommand::Export { project, output } => {
                write_new(
                    output,
                    &serde_json::to_string_pretty(&core.export_targets(project.as_deref())?)
                        .map_err(|_| AppError::new("INVALID_DATA", "Cannot serialize targets"))?,
                )?;
                Ok(json!({"status":"exported"}))
            }
        },
        Command::Scope(ScopeCommand::Show { id }) => {
            let target = core.target(id)?;
            Ok(
                json!({"target_id":target.id,"value":target.input.value,"allowed_scope":target.input.allowed_scope,"excluded_scope":target.input.excluded_scope,"authorized":target.input.authorized,"mode":target.input.mode}),
            )
        }
        Command::Scope(ScopeCommand::Validate { id, value }) => {
            serialize(core.validate_scope(id, value.as_deref())?)
        }
        Command::Reverse(command) => match command {
            ReverseCommand::Note { id, input } => {
                if let Some(input) = input {
                    core.save_file_note(id, &read_file(input)?)?;
                }
                Ok(json!({"file_id":id,"note":core.file_note(id)?}))
            }
            ReverseCommand::Import {
                input,
                project,
                authorized,
                folder,
            } => {
                let project = match project {
                    Some(p) => p.clone(),
                    None => {
                        let p = core.list_projects(false)?;
                        if p.len() != 1 {
                            return Err(AppError::new(
                                "PROJECT_REQUIRED",
                                "Use --project when the workspace does not have exactly one active project",
                            ));
                        }
                        p[0].id.clone()
                    }
                };
                if *folder {
                    serialize(core.import_folder(&project, input, *authorized)?)
                } else {
                    serialize(core.import_file(&project, input, *authorized)?)
                }
            }
            ReverseCommand::List { project } => serialize(core.list_files(project.as_deref())?),
            ReverseCommand::Analyze { id } => serialize(core.analyze_file(id)?),
            ReverseCommand::Metadata { id } => serialize(core.file_analysis(id)?),
            ReverseCommand::Strings { id } => serialize(core.file_analysis(id)?.analysis.strings),
            ReverseCommand::Remove { id } => {
                core.remove_file(id)?;
                Ok(json!({"status":"removed","sample_retained":true}))
            }
            ReverseCommand::Compare { a, b } => serialize(core.compare_files(a, b)?),
        },
        Command::Jobs(command) => match command {
            JobCommand::Enqueue {
                kind,
                file,
                target,
                timeout_ms,
            } => serialize(core.enqueue_job(JobInput {
                kind: enum_input::<JobKind>(kind)?,
                file_id: file.clone(),
                target_id: target.clone(),
                timeout_ms: *timeout_ms,
            })?),
            JobCommand::Run { concurrency, watch } => {
                let _engine = JobEngine::start(core.clone(), *concurrency, |_| {})?;
                let interrupted = tokio::signal::ctrl_c();
                tokio::pin!(interrupted);
                loop {
                    if !watch
                        && !core
                            .list_jobs(None)?
                            .iter()
                            .any(|j| matches!(j.status.as_str(), "queued" | "running"))
                    {
                        break;
                    }
                    tokio::select! {_= &mut interrupted => break, _=tokio::time::sleep(std::time::Duration::from_millis(100))=>{}}
                }
                drop(_engine);
                serialize(core.list_jobs(None)?)
            }
            JobCommand::Result { id } => {
                let job = core.job(id)?;
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
                    serialize(core.analysis_for_job(id)?)
                } else {
                    serialize(job.result)
                }
            }
            JobCommand::List { project } => serialize(core.list_jobs(project.as_deref())?),
            JobCommand::Show { id } => serialize(core.job(id)?),
            JobCommand::Cancel { id } => {
                core.cancel_job(id)?;
                serialize(core.job(id)?)
            }
        },
        Command::Init => Ok(
            json!({"database":core.database_path(),"version":env!("CARGO_PKG_VERSION"),"status":"ready"}),
        ),
        Command::Doctor => {
            let mut dependencies = Vec::new();
            for name in ["git", "node", "cargo", "rustc", "docker"] {
                let result = std::process::Command::new(name).arg("--version").output();
                dependencies.push(json!({"name":name,"available":result.as_ref().is_ok_and(|r|r.status.success()),"version":result.ok().and_then(|r|String::from_utf8(r.stdout).ok()).map(|s|s.trim().to_owned())}));
            }
            Ok(
                json!({"os":std::env::consts::OS,"architecture":std::env::consts::ARCH,"database":core.database_path(),"dependencies":dependencies,"execution_capabilities":["local_project_management","ai_text"],"mcp":"not_implemented"}),
            )
        }
        Command::Project(command) => match command {
            ProjectCommand::Create { name, description } => {
                serialize(core.create_project(name, description)?)
            }
            ProjectCommand::List { archived } => serialize(core.list_projects(*archived)?),
            ProjectCommand::Open { project } => serialize(core.project(project)?),
            ProjectCommand::Rename {
                project,
                name,
                description,
            } => {
                let p = core.project(project)?;
                serialize(core.update_project(
                    &p.id,
                    name,
                    description.as_deref().unwrap_or(&p.description),
                )?)
            }
            ProjectCommand::Archive { project } => {
                core.archive_project(project, true)?;
                Ok(json!({"status":"archived"}))
            }
            ProjectCommand::Restore { project } => {
                core.archive_project(project, false)?;
                Ok(json!({"status":"restored"}))
            }
            ProjectCommand::Export { project, output } => {
                let export = core.export_project(project)?;
                write_new(output, &serde_json::to_string_pretty(&export).unwrap())?;
                Ok(json!({"status":"exported","path":output}))
            }
            ProjectCommand::Import { input } => {
                let export: ProjectExport = serde_json::from_str(&read_file(input)?)
                    .map_err(|_| AppError::new("INVALID_DATA", "Invalid project JSON"))?;
                serialize(core.import_project(export)?)
            }
        },
        Command::Settings(command) => match command {
            SettingsCommand::Show => serialize(core.settings()?),
            SettingsCommand::Apply { input } => {
                let settings: Settings = serde_json::from_str(&read_file(input)?)
                    .map_err(|_| AppError::new("INVALID_DATA", "Invalid settings JSON"))?;
                core.save_settings(&settings)?;
                serialize(settings)
            }
        },
        Command::Ai(command) => match command {
            AiCommand::Providers => serialize(core.providers()?),
            AiCommand::Configure { input } => {
                let config: ProviderConfig =
                    serde_json::from_str(&read_file(input)?).map_err(|_| {
                        AppError::new("INVALID_DATA", "Invalid secret-free provider configuration")
                    })?;
                core.save_provider(&config)?;
                serialize(config)
            }
            AiCommand::Delete { provider } => {
                core.delete_provider(provider)?;
                Ok(json!({"status":"deleted"}))
            }
            AiCommand::Key { provider, stdin } => {
                let key = if *stdin {
                    let mut key = Zeroizing::new(String::new());
                    std::io::stdin()
                        .take(8193)
                        .read_to_string(&mut key)
                        .map_err(|_| {
                            AppError::new("SECRET_INPUT", "Cannot read credential from stdin")
                        })?;
                    key
                } else {
                    Zeroizing::new(rpassword::prompt_password("API key (hidden): ").map_err(|_|AppError::new("SECRET_INPUT","Cannot read credential interactively; use --stdin for controlled pipelines"))?)
                };
                core.set_key(provider, key.trim_end_matches(['\r', '\n']))?;
                Ok(json!({"status":"credential_stored"}))
            }
            AiCommand::RemoveKey { provider } => {
                core.remove_key(provider)?;
                Ok(json!({"status":"credential_removed"}))
            }
            AiCommand::KeyStatus { provider } => {
                Ok(json!({"configured":core.credential_status(provider)?}))
            }
            AiCommand::Models { provider } | AiCommand::Test { provider } => {
                serialize(core.models(provider).await?)
            }
            AiCommand::Chat {
                provider,
                prompt_file,
                stream,
            } => {
                let prompt = read_file(prompt_file)?;
                if !stream {
                    serialize(core.chat(provider, &prompt).await?)
                } else {
                    let (tx, rx) = tokio::sync::watch::channel(false);
                    let cancel = tokio::spawn(async move {
                        if tokio::signal::ctrl_c().await.is_ok() {
                            let _ = tx.send(true);
                        }
                    });
                    let mut emitted = 0usize;
                    let result = core
                        .stream_chat(provider, &prompt, rx, |text| {
                            if !cli.json && text.len() >= emitted {
                                print!("{}", &text[emitted..]);
                                let _ = std::io::stdout().flush();
                                emitted = text.len();
                            }
                        })
                        .await;
                    cancel.abort();
                    if cli.json {
                        serialize(result?)
                    } else {
                        println!();
                        result?;
                        Ok(Value::Null)
                    }
                }
            }
        },
        Command::Tools(command) => match command {
            ToolsCommand::List => serialize(core.tool_inventory()?),
            ToolsCommand::Plan { tool } => serialize(core.tool_plan(tool)?),
            ToolsCommand::Install { tool } => serialize(
                core.tool_install(tool, |p| {
                    if !cli.json {
                        eprintln!("{}: {} bytes", p.phase, p.downloaded);
                    }
                })
                .await?,
            ),
            ToolsCommand::Doctor { tool } => serialize(core.tool_doctor(tool).await?),
            ToolsCommand::Uninstall { tool } => {
                core.tool_uninstall(tool)?;
                Ok(json!({"status":"uninstalled"}))
            }
        },
        Command::Audit => serialize(core.audit_events()?),
        Command::Completions { .. } => unreachable!(),
    }
}
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            let code = e.exit_code();
            if code != 0 && std::env::args().any(|a| a == "--json") {
                eprintln!(
                    "{}",
                    json!({"error":{"code":"CLI_USAGE","message":e.to_string()}})
                );
            } else {
                let _ = e.print();
            }
            std::process::exit(code);
        }
    };
    match execute(&cli).await {
        Ok(Value::Null) => {}
        Ok(value) => {
            if cli.json {
                println!("{}", value);
            } else {
                match value {
                    Value::Array(rows) => {
                        if rows.is_empty() {
                            println!("No records.");
                        }
                        for row in rows {
                            println!("{}", serde_json::to_string_pretty(&row).unwrap());
                        }
                    }
                    _ => println!("{}", serde_json::to_string_pretty(&value).unwrap()),
                }
            }
        }
        Err(error) => {
            tracing::warn!(code=%error.code,"Command failed");
            if cli.json {
                eprintln!("{}", json!({"error":error}));
            } else {
                eprintln!("{error}");
            }
            std::process::exit(match error.code.as_str() {
                "INVALID_INPUT" | "INVALID_DATA" | "INVALID_GATEWAY" | "DUPLICATE_NAME" => 2,
                "NOT_FOUND" => 3,
                "BLOCKED_BY_POLICY" => 4,
                _ => 1,
            });
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cli_parses_global_flags_and_help() {
        assert!(Cli::try_parse_from(["reconx", "project", "create", "demo", "--json"]).is_ok());
        assert_eq!(
            Cli::try_parse_from(["reconx", "project", "--help"])
                .err()
                .unwrap()
                .exit_code(),
            0
        );
        assert!(Cli::try_parse_from(["reconx", "ai", "key", "id", "--api-key", "secret"]).is_err());
    }
}
