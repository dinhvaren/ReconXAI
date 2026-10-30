use clap::{Args, CommandFactory, Parser, Subcommand};
use reconx_core::{
    AppError, Core, EvidenceInput, FindingInput, JobEngine, JobInput, JobKind, JobReview,
    McpApproval, McpServerConfig, PlanReview, ProjectExport, ProviderConfig, ReconConfig,
    ReconPlan, ReconTool, ReportFormat, ReportInput, Result, SecurityMode, Settings, TargetExport,
    TargetInput, TargetKind,
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
    /// Actual offline agent descriptors and persisted runtime state.
    #[command(subcommand)]
    Agents(AgentCommand),
    /// Read-only provenance graphs over stored research results; never executes tools.
    #[command(subcommand)]
    Graph(GraphCommand),
    /// Persistent target-bound sessions over the existing reviewed capabilities.
    #[command(subcommand)]
    Research(ResearchCommand),
    /// Shared active project/target IDs, used by Desktop and CLI.
    #[command(subcommand)]
    Workspace(WorkspaceCommand),
    /// Compiled, permission-bounded native extensions; installation requires a reviewed plan.
    #[command(subcommand)]
    Extensions(ExtensionsCommand),
    /// Saved requests and individually approved loopback replay.
    #[command(subcommand)]
    Http(HttpCommand),
    /// Secure project/target-bound HTTP identities. No plaintext credential output.
    #[command(subcommand)]
    Auth(AuthCommand),
    /// Offline HAR traffic and HTTP capture analysis. Never replays requests.
    #[command(subcommand)]
    Web(WebCommand),
    /// Offline OpenAPI inventory and imported-response comparison.
    #[command(subcommand)]
    Api(ApiCommand),
    #[command(subcommand)]
    Evidence(EvidenceCommand),
    /// Manually approved loopback HTTP inventory; no public target scanning.
    #[command(subcommand)]
    Recon(ReconCommand),
    #[command(subcommand)]
    Findings(FindingCommand),
    #[command(subcommand)]
    Report(ReportCommand),
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
    #[command(subcommand)]
    Mcp(McpCommand),
    #[command(subcommand)]
    Workflow(WorkflowCommand),
    Audit,
    Completions {
        shell: clap_complete::Shell,
    },
}
#[derive(Subcommand)]
enum GraphCommand {
    Nodes {
        session: String,
        #[arg(long, default_value = "")]
        graph: String,
        #[arg(long, default_value = "")]
        kind: String,
        #[arg(long, default_value = "")]
        search: String,
        #[arg(long, default_value_t = 0)]
        offset: usize,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    /// Edges whose endpoints occur in this bounded node page.
    Edges {
        session: String,
        #[arg(long, default_value_t = 0)]
        offset: usize,
    },
    Export {
        session: String,
        #[arg(long)]
        output: PathBuf,
    },
}
#[derive(Subcommand)]
enum HttpCommand {
    Requests {
        #[arg(long)]
        project: String,
        #[arg(long)]
        target: Option<String>,
    },
    Inspect {
        id: String,
    },
    Seed {
        exchange: String,
        #[arg(long)]
        target: Option<String>,
    },
    Save {
        input: PathBuf,
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        revision: Option<i64>,
    },
    Preview {
        id: String,
    },
    /// Queue one exact reviewed request; jobs run executes the durable queue.
    Replay {
        id: String,
        #[arg(long)]
        fingerprint: String,
        #[arg(long)]
        approve: bool,
    },
    History {
        id: String,
    },
    /// Compare actual capture IDs, including results of manual requests.
    Compare {
        first: String,
        second: String,
    },
}
#[derive(Subcommand)]
enum AuthCommand {
    Profiles {
        #[arg(long)]
        project: String,
        #[arg(long)]
        target: Option<String>,
    },
    Add {
        #[arg(long)]
        project: String,
        #[arg(long)]
        target: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        role: String,
        #[arg(long)]
        kind: String,
        #[arg(long, default_value = "")]
        header: String,
        #[arg(long)]
        key_stdin: bool,
    },
    /// Replace credentials and/or metadata. The input file contains metadata only.
    Update {
        id: String,
        input: PathBuf,
        #[arg(long)]
        revision: i64,
        #[arg(long)]
        replace_key: bool,
        #[arg(long)]
        key_stdin: bool,
    },
    Expire {
        id: String,
        #[arg(long)]
        revision: i64,
    },
    Remove {
        id: String,
        #[arg(long)]
        revision: i64,
    },
    Cleanup,
}
#[derive(Args)]
struct HttpImportArgs {
    file: PathBuf,
    #[arg(long)]
    project: String,
    #[arg(long)]
    target: Option<String>,
    #[arg(long)]
    title: Option<String>,
    #[arg(long, default_value = "")]
    identity: String,
    #[arg(long, default_value = "")]
    role: String,
}
#[derive(Subcommand)]
enum WebCommand {
    /// Queue a local captured-header observation job. Desktop or jobs run executes it.
    Analyze {
        id: String,
    },
    ImportHar(HttpImportArgs),
    Traffic {
        #[arg(long)]
        project: String,
        #[arg(long)]
        target: Option<String>,
        #[arg(long, default_value = "")]
        search: String,
        #[arg(long, default_value_t = 0)]
        offset: usize,
    },
    Inspect {
        id: String,
    },
    Note {
        id: String,
        #[arg(long)]
        text: Option<String>,
    },
    Export {
        #[arg(long)]
        project: String,
        output: PathBuf,
    },
}
#[derive(Subcommand)]
enum ApiCommand {
    /// Queue offline schema consistency observations for a capture/operation pair.
    SchemaCheck {
        exchange: String,
        operation: String,
        /// Enable the bounded supported set of syntactic format assertions.
        #[arg(long)]
        assert_formats: bool,
    },
    ImportOpenapi(HttpImportArgs),
    Endpoints {
        #[arg(long)]
        project: String,
        #[arg(long)]
        target: Option<String>,
    },
    Inspect {
        id: String,
    },
    /// Compare existing captures only. This sends no requests.
    Diff {
        left: String,
        right: String,
        #[arg(long, default_value = "")]
        owner_field: String,
    },
    /// Show captured identity/role labels; not an active authorization test.
    AuthzMatrix {
        #[arg(long)]
        project: String,
        #[arg(long)]
        target: Option<String>,
    },
    Finding {
        comparison: String,
    },
}
fn http_import_cli(core: &Core, kind: &str, args: &HttpImportArgs) -> Result<Value> {
    serialize(
        core.http_import(
            kind,
            reconx_core::HttpImportInput {
                project_id: args.project.clone(),
                target_id: args.target.clone(),
                title: args
                    .title
                    .clone()
                    .unwrap_or_else(|| format!("Offline {kind} import")),
                identity: args.identity.clone(),
                role: args.role.clone(),
            },
            &args.file,
        )?,
    )
}
#[derive(Subcommand)]
enum ReconCommand {
    /// Preview fixed managed-tool arguments, scope and data-sharing requirements.
    Plan {
        target: String,
        #[arg(long)]
        tool: String,
        #[arg(long, default_value_t = 4)]
        max_assets: u8,
        #[arg(long, default_value_t = 10)]
        timeout_secs: u32,
    },
    History {
        #[arg(long)]
        target: Option<String>,
    },
    Export {
        #[arg(long)]
        project: String,
        #[arg(long)]
        output: PathBuf,
    },
    /// Preview one HEAD request after checking target authorization and scope.
    Preview { target: String },
    /// Queue the exact approved plan. Desktop or `jobs run` executes the job.
    Run {
        #[arg(required_unless_present = "plan")]
        target: Option<String>,
        #[arg(long)]
        plan: Option<PathBuf>,
        #[arg(long, required_unless_present = "plan")]
        fingerprint: Option<String>,
        #[arg(long)]
        approve: bool,
    },
    /// List actual persisted endpoint observations, including those from Desktop.
    Assets {
        #[arg(long)]
        project: Option<String>,
    },
}
#[derive(Subcommand)]
enum EvidenceCommand {
    List {
        #[arg(long)]
        project: Option<String>,
    },
    Show {
        id: String,
    },
    Add {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        text: PathBuf,
    },
    Import {
        file: PathBuf,
        #[arg(long)]
        project: String,
        #[arg(long)]
        target: Option<String>,
        #[arg(long, default_value = "attachment")]
        kind: String,
        #[arg(long)]
        title: Option<String>,
    },
    FromJob {
        job: String,
    },
    Review {
        id: String,
        #[arg(long)]
        note: PathBuf,
    },
    Verify {
        id: String,
    },
    Export {
        id: String,
        #[arg(long)]
        output: PathBuf,
        #[arg(
            long,
            help = "Export original sensitive bytes; images cannot be automatically redacted"
        )]
        raw_approved: bool,
    },
}
#[derive(Subcommand)]
enum FindingCommand {
    List {
        #[arg(long)]
        project: Option<String>,
    },
    Show {
        id: String,
    },
    Create {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        confirm_verified: bool,
    },
    Update {
        id: String,
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        revision: u32,
        #[arg(long)]
        confirm_verified: bool,
    },
    Remove {
        id: String,
        #[arg(long)]
        revision: u32,
    },
    Compare {
        a: String,
        b: String,
    },
    Merge {
        primary: String,
        duplicate: String,
    },
    Cvss {
        vector: String,
    },
}
#[derive(Subcommand)]
enum ReportCommand {
    List {
        #[arg(long)]
        project: Option<String>,
    },
    Show {
        id: String,
    },
    Prepare {
        #[arg(long)]
        input: PathBuf,
    },
    Export {
        id: String,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        format: String,
        #[arg(long, help = "SHA-256 of the exact saved preview reviewed by the user")]
        reviewed_sha256: String,
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
        #[arg(
            long,
            default_value_t = 8192,
            help = "Streaming import budget per file; deep parser remains independently bounded"
        )]
        max_mib: u64,
        #[arg(
            long,
            help = "Print bounded byte progress to stderr; stdout remains JSON"
        )]
        progress: bool,
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
enum ExtensionsCommand {
    Permissions {
        id: String,
        #[arg(long)]
        tool: Vec<String>,
        #[arg(long)]
        approve: bool,
    },
    Search {
        query: Option<String>,
    },
    List,
    Info {
        id: String,
    },
    Plan {
        id: String,
        #[arg(long)]
        version: Option<String>,
        #[arg(long)]
        managed_runtime: bool,
    },
    Install {
        id: String,
        #[arg(long)]
        version: Option<String>,
        #[arg(long)]
        managed_runtime: bool,
        #[arg(long)]
        wait: bool,
        #[arg(long)]
        fingerprint: String,
        #[arg(long)]
        approve: bool,
    },
    Update {
        id: String,
    },
    Enable {
        id: String,
    },
    Disable {
        id: String,
    },
    Remove {
        id: String,
    },
    Doctor {
        id: String,
    },
    Invoke {
        id: String,
        #[arg(long)]
        file: String,
    },
    History {
        id: String,
    },
}
#[derive(Subcommand)]
enum ToolsCommand {
    Dependencies {
        extension_id: String,
    },
    List,
    Search {
        query: String,
    },
    Info {
        tool: String,
    },
    Host,
    Profiles,
    InstallProfile {
        profile: String,
        #[arg(long)]
        approve: bool,
    },
    Verify {
        tool: String,
    },
    Update {
        tool: String,
    },
    Remove {
        tool: String,
    },
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
enum McpCommand {
    Install {
        id: String,
        #[arg(long, default_value = "2.0.0")]
        version: String,
        #[arg(long)]
        managed_runtime: bool,
        #[arg(long)]
        fingerprint: String,
        #[arg(long)]
        approve: bool,
        #[arg(long)]
        wait: bool,
    },
    Doctor {
        server: Option<String>,
    },
    #[command(subcommand)]
    Catalog(RegistryCommand),
    List,
    /// Register or edit a secret-free server config. Connection executes the reviewed command.
    Add {
        #[arg(long)]
        input: PathBuf,
    },
    Remove {
        server: String,
    },
    Connect {
        server: String,
    },
    Disconnect {
        server: String,
    },
    Test {
        server: String,
    },
    Tools {
        server: String,
    },
    Inspect {
        server: String,
        tool: String,
    },
    Key {
        server: String,
        #[arg(long, default_value = "bearer")]
        name: String,
    },
    /// Preview; supply the returned fingerprint and --approve to enqueue that exact input.
    Invoke {
        server: String,
        tool: String,
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        project: String,
        #[arg(long)]
        fingerprint: Option<String>,
        #[arg(long)]
        approve: bool,
    },
    History {
        server: String,
    },
}
#[derive(Subcommand)]
enum RegistryCommand {
    Search {
        query: Option<String>,
        #[arg(long, value_parser=clap::value_parser!(u16).range(1..=100))]
        limit: Option<u16>,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        runtime: Option<String>,
        #[arg(long)]
        refresh: bool,
        #[arg(long)]
        offline: bool,
        #[arg(long)]
        updated_since: Option<String>,
    },
    Info {
        id: String,
        #[arg(long)]
        version: Option<String>,
        #[arg(long)]
        offline: bool,
    },
    Versions {
        id: String,
        #[arg(long)]
        offline: bool,
    },
    Clear,
}
#[derive(Subcommand)]
enum WorkspaceCommand {
    Show,
    Select {
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        target: Option<String>,
        #[arg(long)]
        revision: Option<i64>,
    },
}
#[derive(Subcommand)]
enum ResearchCommand {
    /// Review a bounded offline DAG over an existing unapproved research plan.
    DagPreview {
        id: String,
        #[arg(long)]
        limits: Option<PathBuf>,
    },
    /// Start the exact fingerprinted offline DAG review; never dispatches network/MCP.
    DagStart {
        input: PathBuf,
    },
    Pause {
        id: String,
    },
    Resume {
        id: String,
    },
    Retry {
        id: String,
    },
    Preview {
        target: String,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long, default_value = "ai_assisted")]
        mode: String,
    },
    Plan {
        input: PathBuf,
    },
    List {
        #[arg(long)]
        project: String,
    },
    Show {
        id: String,
    },
    Approve {
        id: String,
        #[arg(long)]
        fingerprint: String,
        #[arg(long)]
        approve: bool,
        #[arg(long)]
        approve_network: bool,
    },
    Cancel {
        id: String,
    },
    Evidence {
        id: String,
    },
    Interpret {
        input: PathBuf,
    },
    Finding {
        id: String,
        #[arg(long)]
        title: String,
        #[arg(long)]
        details_file: PathBuf,
    },
    Report {
        id: String,
    },
}
#[derive(Subcommand)]
enum WorkflowCommand {
    Tasks {
        session: String,
    },
    Graph {
        session: String,
    },
    Evidence {
        job: String,
    },
    Interpret {
        #[arg(long)]
        input: PathBuf,
    },
    Interpretations {
        job: String,
    },
    Capabilities {
        #[arg(long)]
        project: String,
    },
    Preview {
        file: String,
    },
    /// JSON consent document contains exact metadata preview and approved gateway, never API keys.
    Plan {
        #[arg(long)]
        input: PathBuf,
    },
    List {
        #[arg(long)]
        project: String,
    },
    Approve {
        plan: String,
        #[arg(long)]
        approve: bool,
    },
}
#[derive(Subcommand)]
enum AgentCommand {
    List,
    Status {
        session: String,
        #[arg(long, default_value_t = 0)]
        after: i64,
    },
}
#[derive(Subcommand)]
enum AiCommand {
    Catalog {
        vendor: Option<String>,
    },
    Providers {
        #[command(subcommand)]
        command: Option<AiProfilesCommand>,
    },
    #[command(subcommand)]
    Profiles(AiProfilesCommand),
    #[command(subcommand)]
    Routing(AiRoutingCommand),
    RefreshModels {
        provider: String,
    },
    /// Update secret-free gateway options; custom header values use `ai headers`.
    Gateway {
        provider: String,
        input: PathBuf,
    },
    /// Read custom header JSON through hidden input or stdin; never arguments.
    Headers {
        provider: String,
        #[arg(long)]
        stdin: bool,
    },
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
    /// List model IDs for a profile, or use list / refresh <profile> / select <profile>.
    Models {
        provider: String,
        profile: Option<String>,
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
#[derive(Args)]
struct ProfileCreateArgs {
    #[arg(long)]
    vendor: String,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    model: Option<String>,
    #[arg(long)]
    base_url: Option<String>,
    /// Alibaba Model Studio Singapore workspace ID.
    #[arg(long)]
    workspace: Option<String>,
    #[arg(long)]
    default: bool,
    #[arg(long)]
    key_stdin: bool,
    /// Enable API key auth for an otherwise keyless local server.
    #[arg(long)]
    with_key: bool,
}
#[derive(Subcommand)]
enum AiProfilesCommand {
    List,
    #[command(alias = "add")]
    Create(ProfileCreateArgs),
    SetDefault {
        provider: String,
    },
    Remove {
        provider: String,
    },
    Test {
        provider: String,
    },
}
#[derive(Subcommand)]
enum AiRoutingCommand {
    Show,
    Configure { input: PathBuf },
    Resolve { agent: String },
}
fn read_ai_secret(stdin: bool, prompt: &str) -> Result<Zeroizing<String>> {
    if stdin {
        let mut value = Zeroizing::new(String::new());
        std::io::stdin()
            .take(8193)
            .read_to_string(&mut value)
            .map_err(|_| AppError::new("SECRET_INPUT", "Cannot read secret from stdin"))?;
        Ok(value)
    } else {
        Ok(Zeroizing::new(rpassword::prompt_password(prompt).map_err(
            |_| AppError::new("SECRET_INPUT", "Use an interactive terminal or --key-stdin"),
        )?))
    }
}
fn read_http_secret(stdin: bool) -> Result<Zeroizing<String>> {
    let mut secret = read_ai_secret(
        stdin,
        "HTTP credential (hidden; Basic uses username:password): ",
    )?;
    let length = secret.trim_end_matches(['\r', '\n']).len();
    secret.truncate(length);
    Ok(secret)
}
fn select_ai_default(core: &Core, id: &str) -> Result<Value> {
    let config = core.provider(id)?;
    if !config.enabled || (config.requires_key && !core.credential_status(id)?) {
        return Err(AppError::new(
            "MODEL_UNAVAILABLE",
            "Enable this model and configure its key first",
        ));
    }
    let mut settings = core.settings()?;
    settings.default_provider_id = Some(id.into());
    if settings.fallback_provider_id.as_deref() == Some(id) {
        settings.fallback_provider_id = None;
    }
    core.save_settings(&settings)?;
    serialize(settings)
}
async fn ai_connection_test(core: &Core, id: &str) -> Result<Value> {
    let health = core.test_ai_connection(id).await?;
    if health.state == "failed" {
        return Err(AppError::new(
            health.error_code.as_deref().unwrap_or("AI_TEST_FAILED"),
            &health.message,
        ));
    }
    serialize(health)
}
async fn ai_profiles(core: &Core, command: &AiProfilesCommand) -> Result<Value> {
    match command {
        AiProfilesCommand::List => serialize(core.ai_manager()?.profiles),
        AiProfilesCommand::Remove { provider } => {
            core.delete_provider(provider)?;
            Ok(json!({"status":"deleted"}))
        }
        AiProfilesCommand::Test { provider } => ai_connection_test(core, provider).await,
        AiProfilesCommand::SetDefault { provider } => select_ai_default(core, provider),
        AiProfilesCommand::Create(args) => {
            let preset = core
                .ai_catalog()
                .into_iter()
                .find(|p| p.id == args.vendor)
                .ok_or_else(|| {
                    AppError::new("UNKNOWN_PROVIDER", "Provider is not in the catalog")
                })?;
            let model = args
                .model
                .clone()
                .or(preset.models.first().map(|m| m.id.clone()))
                .ok_or_else(|| {
                    AppError::new(
                        "MODEL_REQUIRED",
                        "Local and custom profiles require --model",
                    )
                })?;
            let name = args.name.clone().unwrap_or_else(|| {
                format!("{} - {}", preset.name, model)
                    .chars()
                    .take(90)
                    .collect()
            });
            let mut config = preset.configuration(&name, &model);
            if let Some(template) = &preset.endpoint_template {
                let workspace = args.workspace.as_deref().ok_or_else(|| {
                    AppError::new(
                        "WORKSPACE_REQUIRED",
                        "Model Studio requires --workspace (Singapore)",
                    )
                })?;
                if workspace.is_empty()
                    || workspace.len() > 100
                    || !workspace
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                {
                    return Err(AppError::new("INVALID_INPUT", "Invalid workspace ID"));
                }
                config.base_url = template.replace("{workspace}", workspace);
            }
            if let Some(url) = &args.base_url {
                config.base_url = url.clone();
            }
            config.requires_key |= args.with_key;
            config.validate()?;
            let key = if config.requires_key {
                Some(read_ai_secret(args.key_stdin, "API key (hidden): ")?)
            } else {
                None
            };
            core.save_ai_profile(
                &config,
                &args.vendor,
                key.as_ref().map(|k| k.trim_end_matches(['\r', '\n'])),
                args.default,
            )?;
            serialize(config)
        }
    }
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
        Command::Graph(command) => match command {
            GraphCommand::Nodes {
                session,
                graph,
                kind,
                search,
                offset,
                limit,
            } => serialize(core.research_graph(
                session,
                reconx_core::ResearchGraphQuery {
                    graph: graph.clone(),
                    kind: kind.clone(),
                    search: search.clone(),
                    offset: *offset,
                    limit: *limit,
                },
            )?),
            GraphCommand::Edges { session, offset } => {
                let page = core.research_graph(
                    session,
                    reconx_core::ResearchGraphQuery {
                        offset: *offset,
                        ..Default::default()
                    },
                )?;
                Ok(
                    json!({"session_id":session,"revision":page.revision,"edges":page.edges,"next_node_offset":page.next_offset,"scope":"Edges between nodes in this page; use graph export for the bounded complete graph"}),
                )
            }
            GraphCommand::Export { session, output } => core.research_graph_export(session, output),
        },
        Command::Http(command) => match command {
            HttpCommand::Requests { project, target } => {
                serialize(core.http_drafts(project, target.as_deref())?)
            }
            HttpCommand::Inspect { id } => serialize(core.http_draft(id)?),
            HttpCommand::Seed { exchange, target } => {
                serialize(core.http_draft_seed(exchange, target.as_deref())?)
            }
            HttpCommand::Save {
                input,
                id,
                revision,
            } => {
                let input = serde_json::from_str(&read_file(input)?).map_err(|_| {
                    AppError::new("INVALID_INPUT", "Invalid HTTP request metadata file")
                })?;
                serialize(core.http_draft_save(id.as_deref(), input, *revision)?)
            }
            HttpCommand::Preview { id } => serialize(core.http_replay_preview(id)?),
            HttpCommand::Replay {
                id,
                fingerprint,
                approve,
            } => serialize(core.http_replay_enqueue(reconx_core::HttpReplayApproval {
                draft_id: id.clone(),
                fingerprint: fingerprint.clone(),
                approved: *approve,
            })?),
            HttpCommand::History { id } => serialize(core.http_replay_history(id)?),
            HttpCommand::Compare { first, second } => {
                core.http_compare(reconx_core::HttpComparisonInput {
                    left: first.clone(),
                    right: second.clone(),
                    owner_field: String::new(),
                })
            }
        },
        Command::Auth(command) => match command {
            AuthCommand::Profiles { project, target } => {
                serialize(core.http_auth_profiles(project, target.as_deref())?)
            }
            AuthCommand::Add {
                project,
                target,
                name,
                role,
                kind,
                header,
                key_stdin,
            } => {
                let kind: reconx_core::HttpAuthKind =
                    serde_json::from_value(json!(kind)).map_err(|_| {
                        AppError::new(
                            "INVALID_INPUT",
                            "Use anonymous, cookie, bearer, api_key, basic or custom_header",
                        )
                    })?;
                let secret = if kind == reconx_core::HttpAuthKind::Anonymous {
                    None
                } else {
                    Some(read_http_secret(*key_stdin)?)
                };
                serialize(core.http_auth_save(
                    None,
                    reconx_core::HttpAuthInput {
                        project_id: project.clone(),
                        target_id: target.clone(),
                        name: name.clone(),
                        role: role.clone(),
                        kind,
                        header_name: header.clone(),
                    },
                    None,
                    secret,
                )?)
            }
            AuthCommand::Update {
                id,
                input,
                revision,
                replace_key,
                key_stdin,
            } => {
                let input = serde_json::from_str(&read_file(input)?).map_err(|_| {
                    AppError::new("INVALID_INPUT", "Invalid HTTP identity metadata file")
                })?;
                let secret = if *replace_key {
                    Some(read_http_secret(*key_stdin)?)
                } else {
                    None
                };
                serialize(core.http_auth_save(Some(id), input, Some(*revision), secret)?)
            }
            AuthCommand::Expire { id, revision } => {
                core.http_auth_expire(id, *revision)?;
                Ok(Value::Null)
            }
            AuthCommand::Remove { id, revision } => {
                core.http_auth_remove(id, *revision)?;
                Ok(Value::Null)
            }
            AuthCommand::Cleanup => {
                core.http_auth_cleanup()?;
                Ok(Value::Null)
            }
        },
        Command::Web(command) => match command {
            WebCommand::Analyze { id } => serialize(core.http_review_enqueue(id)?),
            WebCommand::ImportHar(args) => http_import_cli(&core, "har", args),
            WebCommand::Traffic {
                project,
                target,
                search,
                offset,
            } => serialize(core.http_workspace(project, target.as_deref(), search, *offset)?),
            WebCommand::Inspect { id } => serialize(core.http_exchange(id)?),
            WebCommand::Note { id, text } => serialize(core.http_note(id, text.as_deref())?),
            WebCommand::Export { project, output } => core.http_export(project, output),
        },
        Command::Api(command) => match command {
            ApiCommand::SchemaCheck {
                exchange,
                operation,
                assert_formats,
            } => serialize(core.http_schema_enqueue_with_options(
                exchange,
                operation,
                reconx_core::SchemaReviewOptions {
                    assert_formats: *assert_formats,
                },
            )?),
            ApiCommand::ImportOpenapi(args) => http_import_cli(&core, "openapi", args),
            ApiCommand::Endpoints { project, target } => serialize(
                core.http_workspace(project, target.as_deref(), "", 0)?
                    .operations,
            ),
            ApiCommand::Inspect { id } => serialize(core.api_operation(id)?),
            ApiCommand::Diff {
                left,
                right,
                owner_field,
            } => core.http_compare(reconx_core::HttpComparisonInput {
                left: left.clone(),
                right: right.clone(),
                owner_field: owner_field.clone(),
            }),
            ApiCommand::AuthzMatrix { project, target } => {
                serialize(core.http_workspace(project, target.as_deref(), "", 0)?)
            }
            ApiCommand::Finding { comparison } => {
                serialize(core.http_comparison_finding(comparison)?)
            }
        },
        Command::Recon(command) => match command {
            ReconCommand::Preview { target } => serialize(core.local_http_preview(target)?),
            ReconCommand::Plan {
                target,
                tool,
                max_assets,
                timeout_secs,
            } => {
                let mut config = ReconConfig::for_target(target, enum_input::<ReconTool>(tool)?);
                config.max_assets = *max_assets;
                config.timeout_secs = *timeout_secs;
                serialize(core.recon_plan(config)?)
            }
            ReconCommand::History { target } => serialize(core.recon_history(target.as_deref())?),
            ReconCommand::Export { project, output } => {
                let value = core.recon_export(project)?;
                write_new(
                    output,
                    &serde_json::to_string_pretty(&value).map_err(|_| {
                        AppError::new("SERIALIZATION", "Cannot serialize inventory")
                    })?,
                )?;
                Ok(value)
            }
            ReconCommand::Run {
                target,
                fingerprint,
                plan,
                approve,
            } => {
                if let Some(path) = plan {
                    let p: ReconPlan = serde_json::from_str(&read_file(path)?).map_err(|_| {
                        AppError::new("INVALID_INPUT", "Invalid reviewed recon plan")
                    })?;
                    serialize(core.recon_enqueue(p, *approve)?)
                } else {
                    serialize(
                        core.local_http_enqueue(
                            target
                                .as_deref()
                                .ok_or_else(|| AppError::new("INVALID_INPUT", "Choose a target"))?,
                            fingerprint.as_deref().ok_or_else(|| {
                                AppError::new("INVALID_INPUT", "Review a plan fingerprint")
                            })?,
                            *approve,
                        )?,
                    )
                }
            }
            ReconCommand::Assets { project } => serialize(core.assets(project.as_deref())?),
        },
        Command::Evidence(command) => match command {
            EvidenceCommand::List { project } => serialize(core.evidence_list(project.as_deref())?),
            EvidenceCommand::Show { id } => serialize(core.evidence_show(id)?),
            EvidenceCommand::Add { input, text } => {
                let input: EvidenceInput =
                    serde_json::from_str(&read_file(input)?).map_err(|_| {
                        AppError::new("INVALID_INPUT", "Invalid evidence metadata JSON")
                    })?;
                serialize(core.evidence_add(input, &read_file(text)?)?)
            }
            EvidenceCommand::Import {
                file,
                project,
                target,
                kind,
                title,
            } => serialize(core.evidence_import(
                EvidenceInput {
                    project_id: project.clone(),
                    target_id: target.clone(),
                    kind: enum_input(kind)?,
                    title: title.clone().unwrap_or_else(|| {
                        file.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned()
                    }),
                    source_tool: "manual import".into(),
                    sensitivity: Default::default(),
                },
                file,
            )?),
            EvidenceCommand::FromJob { job } => serialize(core.evidence_from_job(job)?),
            EvidenceCommand::Review { id, note } => {
                serialize(core.evidence_review(id, &read_file(note)?)?)
            }
            EvidenceCommand::Verify { id } => core.evidence_verify(id),
            EvidenceCommand::Export {
                id,
                output,
                raw_approved,
            } => core.evidence_export(id, output, *raw_approved),
        },
        Command::Findings(command) => match command {
            FindingCommand::List { project } => serialize(core.findings(project.as_deref())?),
            FindingCommand::Show { id } => serialize(core.finding(id)?),
            FindingCommand::Create {
                input,
                confirm_verified,
            } => {
                let input: FindingInput = serde_json::from_str(&read_file(input)?)
                    .map_err(|_| AppError::new("INVALID_INPUT", "Invalid finding JSON"))?;
                serialize(core.finding_save(None, input, None, *confirm_verified)?)
            }
            FindingCommand::Update {
                id,
                input,
                revision,
                confirm_verified,
            } => {
                let input: FindingInput = serde_json::from_str(&read_file(input)?)
                    .map_err(|_| AppError::new("INVALID_INPUT", "Invalid finding JSON"))?;
                serialize(core.finding_save(Some(id), input, Some(*revision), *confirm_verified)?)
            }
            FindingCommand::Remove { id, revision } => {
                core.finding_remove(id, *revision)?;
                Ok(Value::Null)
            }
            FindingCommand::Compare { a, b } => core.finding_compare(a, b),
            FindingCommand::Merge { primary, duplicate } => core.finding_merge(primary, duplicate),
            FindingCommand::Cvss { vector } => serialize(reconx_core::calculate_cvss(vector)?),
        },
        Command::Report(command) => match command {
            ReportCommand::List { project } => serialize(core.reports(project.as_deref())?),
            ReportCommand::Show { id } => serialize(core.report_show(id)?),
            ReportCommand::Prepare { input } => {
                let input: ReportInput = serde_json::from_str(&read_file(input)?)
                    .map_err(|_| AppError::new("INVALID_INPUT", "Invalid report JSON"))?;
                serialize(core.report_prepare(input)?)
            }
            ReportCommand::Export {
                id,
                output,
                format,
                reviewed_sha256,
            } => {
                let format: ReportFormat = enum_input(format)?;
                core.report_export(id, format, output, reviewed_sha256)
            }
        },
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
                max_mib,
                progress,
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
                import_cli(
                    core.clone(),
                    project,
                    input.clone(),
                    *authorized,
                    *folder,
                    *max_mib,
                    *progress,
                )
                .await
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
                    if !watch && !core.jobs_pending_work()? {
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
            AiCommand::Catalog { vendor } => {
                if let Some(id) = vendor {
                    serialize(
                        core.ai_catalog()
                            .into_iter()
                            .find(|p| p.id == *id)
                            .ok_or_else(|| AppError::new("UNKNOWN_PROVIDER", "Unknown provider"))?,
                    )
                } else {
                    serialize(core.ai_catalog())
                }
            }
            AiCommand::Providers { command: None } => serialize(core.providers()?),
            AiCommand::Providers {
                command: Some(command),
            }
            | AiCommand::Profiles(command) => ai_profiles(&core, command).await,
            AiCommand::RefreshModels { provider } => {
                serialize(core.refresh_ai_models(provider).await?)
            }
            AiCommand::Routing(command) => match command {
                AiRoutingCommand::Show => serialize(core.ai_routing()?),
                AiRoutingCommand::Configure { input } => {
                    let routes: Vec<reconx_core::AgentRoute> =
                        serde_json::from_str(&read_file(input)?).map_err(|_| {
                            AppError::new(
                                "INVALID_ROUTING",
                                "Expected an array of agent/provider routes",
                            )
                        })?;
                    core.save_ai_routing(&routes)?;
                    serialize(core.ai_routing()?)
                }
                AiRoutingCommand::Resolve { agent } => serialize(core.ai_agent_provider(agent)?),
            },
            AiCommand::Gateway { provider, input } => {
                let options: reconx_core::GatewayOptions = serde_json::from_str(&read_file(input)?)
                    .map_err(|_| {
                        AppError::new("INVALID_OPTIONS", "Expected secret-free gateway options")
                    })?;
                let state = core.ai_manager()?;
                let profile = state
                    .profiles
                    .into_iter()
                    .find(|p| p.provider.id == *provider)
                    .ok_or_else(|| AppError::new("NOT_FOUND", "Provider not found"))?;
                core.save_ai_setup(
                    reconx_core::AiProfileInput {
                        provider: profile.provider,
                        vendor_id: profile.vendor_id,
                        make_default: false,
                        options,
                    },
                    None,
                    None,
                )?;
                Ok(json!({"status":"saved"}))
            }
            AiCommand::Headers { provider, stdin } => {
                let raw = read_ai_secret(*stdin, "Custom header JSON (hidden): ")?;
                let headers: std::collections::BTreeMap<String, String> =
                    serde_json::from_str(&raw).map_err(|_| {
                        AppError::new("INVALID_HEADER", "Expected JSON object with string values")
                    })?;
                let profile = core
                    .ai_manager()?
                    .profiles
                    .into_iter()
                    .find(|p| p.provider.id == *provider)
                    .ok_or_else(|| AppError::new("NOT_FOUND", "Provider not found"))?;
                core.save_ai_setup(
                    reconx_core::AiProfileInput {
                        provider: profile.provider,
                        vendor_id: profile.vendor_id,
                        make_default: false,
                        options: profile.options,
                    },
                    None,
                    Some(&headers),
                )?;
                Ok(json!({"status":"headers_stored"}))
            }
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
            AiCommand::Models { provider, profile } => match provider.as_str() {
                "list" => serialize(core.ai_manager()?.profiles),
                "refresh" => serialize(
                    core.refresh_ai_models(profile.as_deref().ok_or_else(|| {
                        AppError::new("PROFILE_REQUIRED", "Specify a configured profile ID")
                    })?)
                    .await?,
                ),
                "select" => select_ai_default(
                    &core,
                    profile.as_deref().ok_or_else(|| {
                        AppError::new("PROFILE_REQUIRED", "Specify a configured profile ID")
                    })?,
                ),
                _ => serialize(core.models(provider).await?),
            },
            AiCommand::Test { provider } => ai_connection_test(&core, provider).await,
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
        Command::Extensions(command) => match command {
            ExtensionsCommand::Permissions { id, tool, approve } => {
                if id != reconx_core::MANAGED_MCP_EVERYTHING {
                    return Err(AppError::new(
                        "EXTENSION_POLICY",
                        "Only managed MCP permissions use this command",
                    ));
                }
                serialize(core.managed_allow_tools(tool.clone(), *approve).await?)
            }
            ExtensionsCommand::Search { query } => {
                serialize(core.extension_catalog(query.as_deref())?)
            }
            ExtensionsCommand::List => serialize(
                core.extension_catalog(None)?
                    .into_iter()
                    .filter(|e| e.installed.is_some())
                    .collect::<Vec<_>>(),
            ),
            ExtensionsCommand::Info { id } => serialize(core.extension_entry(id)?),
            ExtensionsCommand::Plan {
                id,
                version,
                managed_runtime,
            } => serialize(if id == reconx_core::MANAGED_MCP_EVERYTHING {
                core.managed_install_plan(
                    id,
                    version.as_deref().unwrap_or("2.0.0"),
                    *managed_runtime,
                )?
            } else {
                core.extension_install_plan(id)?
            }),
            ExtensionsCommand::Install {
                id,
                version,
                managed_runtime,
                wait,
                fingerprint,
                approve,
            } if id == reconx_core::MANAGED_MCP_EVERYTHING => {
                install_managed_package(
                    &core,
                    reconx_core::ManagedMcpApproval {
                        id: id.clone(),
                        version: version.clone().unwrap_or("2.0.0".into()),
                        managed_runtime: *managed_runtime,
                        fingerprint: fingerprint.clone(),
                        approved: *approve,
                    },
                    *wait,
                )
                .await
            }
            ExtensionsCommand::Install {
                id,
                fingerprint,
                approve,
                ..
            } => serialize(
                core.extension_install(reconx_core::ExtensionApproval {
                    id: id.clone(),
                    fingerprint: fingerprint.clone(),
                    approved: *approve,
                })
                .await?,
            ),
            ExtensionsCommand::Update { id } => {
                let entry = core.extension_entry(id)?;
                if !entry.update_available {
                    return Err(AppError::new(
                        "NO_EXTENSION_UPDATE",
                        "No newer compatible extension is included in this build",
                    ));
                }
                serialize(core.extension_install_plan(id)?)
            }
            ExtensionsCommand::Enable { id } if id == reconx_core::MANAGED_MCP_EVERYTHING => {
                serialize(core.managed_set_enabled(true).await?)
            }
            ExtensionsCommand::Disable { id } if id == reconx_core::MANAGED_MCP_EVERYTHING => {
                serialize(core.managed_set_enabled(false).await?)
            }
            ExtensionsCommand::Remove { id } if id == reconx_core::MANAGED_MCP_EVERYTHING => {
                core.managed_remove().await?;
                Ok(Value::Null)
            }
            ExtensionsCommand::Enable { id } => serialize(core.extension_set_enabled(id, true)?),
            ExtensionsCommand::Disable { id } => serialize(core.extension_set_enabled(id, false)?),
            ExtensionsCommand::Remove { id } => {
                core.extension_remove(id)?;
                Ok(Value::Null)
            }
            ExtensionsCommand::Doctor { id } => serialize(core.extension_doctor(id).await?),
            ExtensionsCommand::Invoke { id, file } => serialize(core.extension_invoke(id, file)?),
            ExtensionsCommand::History { id } => serialize(core.extension_history(id)?),
        },
        Command::Tools(command) => match command {
            ToolsCommand::Dependencies { extension_id } => {
                serialize(core.extension_install_plan(extension_id)?)
            }
            ToolsCommand::List => serialize(core.tool_inventory()?),
            ToolsCommand::Search { query } => serialize(
                core.tool_catalog()?
                    .into_iter()
                    .filter(|t| t.to_string().to_lowercase().contains(&query.to_lowercase()))
                    .collect::<Vec<_>>(),
            ),
            ToolsCommand::Info { tool } => serialize(
                core.tool_catalog()?
                    .into_iter()
                    .find(|t| t["id"] == *tool)
                    .ok_or_else(|| AppError::new("UNKNOWN_TOOL", "Tool is not in registry"))?,
            ),
            ToolsCommand::Host => serialize(core.tool_host()),
            ToolsCommand::Profiles => serialize(core.tool_profiles()),
            ToolsCommand::InstallProfile { profile, approve } => {
                let plan = core.tool_profile_plan(profile)?;
                if !approve {
                    serialize(json!({"approval_required":true,"plan":plan}))
                } else {
                    let mut installed = Vec::new();
                    for tool in plan {
                        if core
                            .tool_inventory()?
                            .iter()
                            .any(|t| t.manifest.id == tool.id && t.integrity_verified)
                        {
                            installed.push(core.tool_doctor(&tool.id).await?);
                        } else {
                            installed.push(core.tool_install(&tool.id, |_| {}).await?);
                        }
                    }
                    serialize(installed)
                }
            }
            ToolsCommand::Verify { tool } => serialize(core.tool_doctor(tool).await?),
            ToolsCommand::Update { tool } => {
                let inventory = core.tool_inventory()?;
                if inventory
                    .iter()
                    .any(|t| &t.manifest.id == tool && t.integrity_verified)
                {
                    serialize(core.tool_doctor(tool).await?)
                } else {
                    serialize(core.tool_install(tool, |_| {}).await?)
                }
            }
            ToolsCommand::Remove { tool } => {
                core.tool_uninstall(tool)?;
                Ok(json!({"status":"uninstalled"}))
            }
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
        Command::Mcp(command) => match command {
            McpCommand::Install {
                id,
                version,
                managed_runtime,
                fingerprint,
                approve,
                wait,
            } => {
                install_managed_package(
                    &core,
                    reconx_core::ManagedMcpApproval {
                        id: id.clone(),
                        version: version.clone(),
                        managed_runtime: *managed_runtime,
                        fingerprint: fingerprint.clone(),
                        approved: *approve,
                    },
                    *wait,
                )
                .await
            }
            McpCommand::Doctor { server } => {
                let servers = if let Some(id) = server {
                    vec![core.mcp_server(id)?]
                } else {
                    core.mcp_servers()?
                };
                let mut results = vec![];
                for server in servers {
                    if !server.config.enabled {
                        results.push(json!({"id":server.config.id,"status":"disabled"}));
                        continue;
                    }
                    match core.mcp_connect(&server.config.id).await{Ok(checked)=>results.push(json!({"id":checked.config.id,"status":"ready","tools":checked.tools.len()})),Err(error)=>results.push(json!({"id":server.config.id,"status":"failed","error":error}))}
                }
                Ok(json!(results))
            }
            McpCommand::Catalog(command) => match command {
                RegistryCommand::Search {
                    query,
                    limit,
                    cursor,
                    runtime,
                    refresh,
                    offline,
                    updated_since,
                } => serialize(
                    core.registry_search(reconx_core::RegistryQuery {
                        search: query.clone().unwrap_or_default(),
                        limit: *limit,
                        cursor: cursor.clone(),
                        runtime: runtime.clone(),
                        refresh: *refresh,
                        offline: *offline,
                        updated_since: updated_since.clone(),
                        platform: None,
                    })
                    .await?,
                ),
                RegistryCommand::Info {
                    id,
                    version,
                    offline,
                } => serialize(
                    core.registry_detail(id, version.as_deref(), *offline)
                        .await?,
                ),
                RegistryCommand::Versions { id, offline } => {
                    serialize(core.registry_versions(id, *offline).await?)
                }
                RegistryCommand::Clear => {
                    core.registry_clear_cache()?;
                    Ok(Value::Null)
                }
            },
            McpCommand::List => serialize(core.mcp_servers()?),
            McpCommand::Add { input } => {
                let config: McpServerConfig =
                    serde_json::from_str(&read_file(input)?).map_err(|_| {
                        AppError::new("INVALID_INPUT", "Invalid secret-free MCP configuration")
                    })?;
                serialize(core.mcp_save(config).await?)
            }
            McpCommand::Remove { server } => {
                core.mcp_remove(server).await?;
                Ok(json!({"status":"removed"}))
            }
            McpCommand::Connect { server }
            | McpCommand::Test { server }
            | McpCommand::Tools { server } => {
                let s = core.mcp_connect(server).await?;
                core.mcp_disconnect(server).await?;
                serialize(
                    json!({"tested":true,"server":s,"session":"Closed on command completion; Desktop owns its own live sessions"}),
                )
            }
            McpCommand::Disconnect { server } => {
                core.mcp_disconnect(server).await?;
                Ok(
                    json!({"status":"disconnected","note":"Connections belong to the process that opened them"}),
                )
            }
            McpCommand::Inspect { server, tool } => serialize(
                core.mcp_server(server)?
                    .tools
                    .into_iter()
                    .find(|t| t["name"] == *tool)
                    .ok_or_else(|| AppError::new("MCP_TOOL_NOT_FOUND", "Discover tools first"))?,
            ),
            McpCommand::Key { server, name } => {
                let key = Zeroizing::new(
                    rpassword::prompt_password("MCP credential (hidden): ")
                        .map_err(|_| AppError::new("SECRET_INPUT", "Cannot read credential"))?,
                );
                core.mcp_set_secret(server, name, &key)?;
                Ok(json!({"status":"credential_stored"}))
            }
            McpCommand::Invoke {
                server,
                tool,
                input,
                project,
                fingerprint,
                approve,
            } => {
                let input: Value = serde_json::from_str(&read_file(input)?)
                    .map_err(|_| AppError::new("MCP_INPUT", "Invalid JSON object"))?;
                if !approve {
                    serialize(core.mcp_invocation_preview(server, tool, &input)?)
                } else {
                    serialize(core.mcp_enqueue(McpApproval {
                        server_id: server.clone(),
                        tool: tool.clone(),
                        input,
                        project_id: project.clone(),
                        fingerprint: fingerprint.clone().ok_or_else(|| {
                            AppError::new("APPROVAL_REQUIRED", "Supply the reviewed fingerprint")
                        })?,
                        approved: true,
                    })?)
                }
            }
            McpCommand::History { server } => serialize(core.mcp_history(server)?),
        },
        Command::Workspace(command) => match command {
            WorkspaceCommand::Show => serialize(core.workspace_selection()?),
            WorkspaceCommand::Select {
                project,
                target,
                revision,
            } => serialize(core.select_workspace(
                project.as_deref(),
                target.as_deref(),
                revision.unwrap_or(core.workspace_selection()?.revision),
            )?),
        },
        Command::Research(command) => match command {
            ResearchCommand::DagPreview { id, limits } => {
                let limits = limits
                    .as_ref()
                    .map(|p| {
                        serde_json::from_str::<reconx_core::AgentLimits>(&read_file(p)?).map_err(
                            |_| AppError::new("INVALID_INPUT", "Invalid offline resource limits"),
                        )
                    })
                    .transpose()?
                    .unwrap_or_default();
                serialize(core.agent_runtime_preview(id, limits)?)
            }
            ResearchCommand::DagStart { input } => {
                serialize(core.agent_runtime_start(
                    serde_json::from_str(&read_file(input)?).map_err(|_| {
                        AppError::new("INVALID_INPUT", "Invalid offline DAG review")
                    })?,
                )?)
            }
            ResearchCommand::Pause { id } => serialize(core.agent_runtime_pause(id)?),
            ResearchCommand::Resume { id } => serialize(core.agent_runtime_resume(id)?),
            ResearchCommand::Retry { id } => serialize(core.agent_runtime_retry(id)?),
            ResearchCommand::Preview {
                target,
                provider,
                mode,
            } => serialize(core.research_preview(reconx_core::ResearchRequest {
                target_id: target.clone(),
                provider_id: provider.clone(),
                mode: serde_json::from_value(json!(mode)).map_err(|_| {
                    AppError::new(
                        "INVALID_INPUT",
                        "Mode must be manual, ai_assisted or authorized_auto",
                    )
                })?,
            })?),
            ResearchCommand::Plan { input } => serialize(
                core.research_plan(serde_json::from_str(&read_file(input)?).map_err(|_| {
                    AppError::new("INVALID_INPUT", "Expected a reviewed research context")
                })?)
                .await?,
            ),
            ResearchCommand::List { project } => serialize(core.research_list(project)?),
            ResearchCommand::Show { id } => serialize(core.research_session(id)?),
            ResearchCommand::Approve {
                id,
                fingerprint,
                approve,
                approve_network,
            } => serialize(core.research_approve(id, fingerprint, *approve, *approve_network)?),
            ResearchCommand::Cancel { id } => serialize(core.research_cancel(id)?),
            ResearchCommand::Evidence { id } => serialize(core.research_evidence_preview(id)?),
            ResearchCommand::Interpret { input } => serialize(
                core.research_interpret(serde_json::from_str(&read_file(input)?).map_err(
                    |_| {
                        AppError::new(
                            "INVALID_INPUT",
                            "Expected exact evidence and destination consent",
                        )
                    },
                )?)
                .await?,
            ),
            ResearchCommand::Finding {
                id,
                title,
                details_file,
            } => serialize(core.research_finding(id, title, &read_file(details_file)?)?),
            ResearchCommand::Report { id } => serialize(core.research_report(id)?),
        },
        Command::Workflow(command) => match command {
            WorkflowCommand::Tasks { session } => serialize(core.agent_runtime(session, 0)?.tasks),
            WorkflowCommand::Graph { session } => {
                serialize(core.research_graph(session, Default::default())?)
            }
            WorkflowCommand::Evidence { job } => serialize(core.job_evidence_preview(job)?),
            WorkflowCommand::Interpret { input } => {
                let review: JobReview = serde_json::from_str(&read_file(input)?).map_err(|_| {
                    AppError::new("INVALID_INPUT", "Invalid evidence consent document")
                })?;
                serialize(core.interpret_job(review).await?)
            }
            WorkflowCommand::Interpretations { job } => serialize(core.job_interpretations(job)?),
            WorkflowCommand::Capabilities { project } => serialize(core.capabilities(project)?),
            WorkflowCommand::Preview { file } => serialize(core.plan_preview(file)?),
            WorkflowCommand::Plan { input } => {
                let review: PlanReview = serde_json::from_str(&read_file(input)?)
                    .map_err(|_| AppError::new("INVALID_INPUT", "Invalid plan consent document"))?;
                serialize(core.plan_local_tools(review).await?)
            }
            WorkflowCommand::List { project } => serialize(core.tool_plans(project)?),
            WorkflowCommand::Approve { plan, approve } => {
                serialize(core.approve_tool_plan(plan, *approve)?)
            }
        },
        Command::Audit => serialize(core.audit_events()?),
        Command::Agents(command) => match command {
            AgentCommand::List => serialize(reconx_core::research_agents()),
            AgentCommand::Status { session, after } => {
                serialize(core.agent_runtime(session, *after)?)
            }
        },
        Command::Completions { .. } => unreachable!(),
    }
}
async fn import_cli(
    core: Core,
    project: String,
    path: PathBuf,
    authorized: bool,
    folder: bool,
    max_mib: u64,
    progress: bool,
) -> Result<Value> {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    let max_bytes = max_mib
        .checked_mul(1024 * 1024)
        .ok_or_else(|| AppError::new("INPUT_LIMIT", "Import budget is too large"))?;
    let cancelled = Arc::new(AtomicBool::new(false));
    let flag = cancelled.clone();
    let mut worker = tokio::task::spawn_blocking(move || {
        let mut last = std::time::Instant::now();
        let mut stage = String::new();
        let update = |p: reconx_core::FileImportProgress| {
            if flag.load(Ordering::Acquire) {
                return Err(AppError::new(
                    "CANCELLED",
                    "Import cancelled; completed files are retained",
                ));
            }
            if progress
                && (stage != p.stage
                    || p.completed == p.total
                    || last.elapsed() >= std::time::Duration::from_millis(100))
            {
                eprintln!("{}", serde_json::to_string(&p).unwrap_or_default());
                last = std::time::Instant::now();
                stage = p.stage;
            }
            Ok(())
        };
        let limits = reconx_core::FileImportLimits { max_bytes };
        if folder {
            serialize(
                core.import_folder_with_progress(&project, &path, authorized, limits, update)?,
            )
        } else {
            serialize(core.import_file_with_progress(&project, &path, authorized, limits, update)?)
        }
    });
    let result = tokio::select! {result=&mut worker=>result, _=tokio::signal::ctrl_c()=>{cancelled.store(true,Ordering::Release);worker.await}};
    result.map_err(|_| AppError::new("WORKER_ERROR", "Streaming import worker failed"))?
}
async fn install_managed_package(
    core: &Core,
    approval: reconx_core::ManagedMcpApproval,
    wait: bool,
) -> Result<Value> {
    let id = approval.id.clone();
    let job = core.managed_enqueue_install(approval)?;
    if wait {
        let worker = core.clone();
        let job_id = job.id.clone();
        tokio::task::spawn_blocking(move || worker.execute_job(&job_id))
            .await
            .map_err(|_| AppError::new("WORKER_ERROR", "Installation worker stopped"))??;
        loop {
            let state = core.job(&job.id)?;
            if !matches!(state.status.as_str(), "queued" | "running") {
                if state.status != "completed" {
                    return Err(AppError::new(
                        state.error_code.as_deref().unwrap_or("INSTALL_FAILED"),
                        state
                            .error_message
                            .as_deref()
                            .unwrap_or("Managed package installation failed"),
                    ));
                }
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        serialize(core.extension_entry(&id)?)
    } else {
        serialize(job)
    }
}

#[tokio::main]
async fn main() {
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
        Ok(Value::Null) => {
            if cli.json {
                println!("null");
            }
        }
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
                "INVALID_INPUT" | "INVALID_DATA" | "INVALID_GATEWAY" | "DUPLICATE_NAME"
                | "INVALID_CVSS" | "INVALID_IMAGE" | "INVALID_TEXT" | "INPUT_LIMIT"
                | "REPORT_LIMIT" | "PROJECT_MISMATCH" | "STALE_RECORD" | "OUTPUT_EXISTS" => 2,
                "NOT_FOUND" => 3,
                "BLOCKED_BY_POLICY"
                | "PREVIEW_REQUIRED"
                | "REPORT_REDACTION_REQUIRED"
                | "VERIFICATION_REQUIRED"
                | "RAW_APPROVAL_REQUIRED"
                | "UNVERIFIED_AI" => 4,
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
