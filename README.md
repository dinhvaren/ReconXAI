# ReconX AI

The research-output pipeline now links actual jobs to integrity-checked evidence,
manually reviewed findings and immutable redacted reports (Markdown/JSON/HTML/PDF).
Desktop and CLI share these records. See [Research outputs](docs/RESEARCH_OUTPUTS.md)
and the [resumable v1.0 ledger](docs/MASTER_EXECUTION_PROGRESS.md) for current
acceptance, commands and limits. ReconX AI v1.0 is not yet release-ready.

Reconnaissance now has a manually approved loopback HTTP inventory adapter, using
the same scope, jobs, evidence and report services. It performs one HEAD request
and no vulnerability scanning. See [Local inventory](docs/LOCAL_INVENTORY.md) for
requirements, CLI commands and actual platform verification.

Milestone 2 adds a verified YARA-X portable/local-signature adapter, actual MCP Hub
(stdio and bounded Streamable HTTP), and reviewed local AI-to-tool plans using the
existing Job Engine. Windows fixture verification is recorded in docs/TEST_RESULTS.md;
Linux shared Core/CLI now has actual Parrot WSL tests and ELF/report runtime
verification. Linux Desktop, optimized packaging and ARM64 remain unverified.
Other catalog tools are explicitly pending.
See [Tool Manager](docs/TOOL_MANAGER.md), [MCP Hub](docs/MCP_HUB.md) and
[AI tool orchestration](docs/AI_TOOL_ORCHESTRATION.md) for actual commands and limits.

A native desktop and CLI research workbench sharing a Rust core and SQLite database. Project management, settings, secure provider configuration, streaming chat/cancellation, safe connection fallback and a verified portable ripgrep installer work on Windows x64. The full research platform remains under development.

Target/scope management and native local binary import/static analysis now share
the same Desktop/CLI services and persisted records. Start with **+ Add Target** or
**Reverse Engineering → Import File → Analyze File**. See
[Research inputs](docs/RESEARCH_INPUTS.md) and [Functionality audit](docs/FUNCTIONALITY_GAP_AUDIT.md).
Recon, advanced MCP integrations, autonomous agents and the remaining security
domains remain unfinished; reviewed local MCP and output records are described above.

Local work now runs through a shared durable Job Engine: queue/state/progress/logs,
bounded concurrency, timeout/cancellation and interrupted-worker recovery. Open
**Quick actions -> View Jobs** for Job Center, or use `reconx jobs enqueue` and
`reconx jobs run`. File notes persist; optional AI artifact review requires explicit
approval of the preview and destination. See [Local jobs](docs/JOB_ENGINE.md) and
[Current milestone](docs/NEXT_MILESTONE.md). Migration 003 preserves existing data;
stop older clients before upgrading the database.

## Windows development

Verified environment: Rust 1.95.0 MSVC, Node 24.15.0, npm 11.7.0, Visual Studio 2022 C++ Build Tools, WebView2. These were already installed; no privileged installation was performed.

```powershell
cd D:\ReconXAI
cargo build --locked -p reconx
cd apps\desktop
npm ci
npm run tauri -- dev
```

## Embedded native build

```powershell
cd D:\ReconXAI\apps\desktop
npm run build
cd ..\..
cargo build --locked -p reconx-desktop --features custom-protocol
cargo build --locked -p reconx
.\target\debug\reconx-desktop.exe
.\target\debug\reconx.exe --help
```

The custom-protocol build embeds frontend assets and needs no Vite server. Run scripts/build.ps1 for verified optimized Windows portable binaries and SHA-256 digests in artifacts/windows-x64. Both release binaries passed the native integration scenario. Installer configurations exist but installers and Linux Desktop remain unverified; Linux Core/CLI acceptance is recorded in TEST_RESULTS.md.

The desktop offers exactly Light (white/gray), Dark (black/charcoal) and ReconX (original burgundy/lavender/raspberry). Settings and the topbar/sidebar appearance menus share one immediate, automatically saved preference. ReconX is the default; legacy system or missing preferences migrate to reconx. OS appearance does not switch themes. Windows uses Segoe UI Variable Text, with locally bundled Source Sans 3 fallback and IBM Plex Mono for technical content; DM Sans is retained for the wordmark. Metadata starts at 12px and navigation/body use 14px. Ctrl+K searches projects, model configurations and pages. Both side panels can collapse and the AI context pane can resize. Future modules are visibly labeled, without fake research results. See docs/THEMES.md for palettes, controls and migration.

## Shared workspace

Default data is stored under the OS app data directory. Set `RECONX_DATA_DIR` in the same environment for desktop and CLI to choose another workspace. CLI also supports `--database`. Tests use separate temporary data.

```powershell
.\target\debug\reconx.exe project create demo --description "Local research"
.\target\debug\reconx.exe project list --json
.\target\debug\reconx.exe settings show
.\target\debug\reconx.exe ai providers
```

Create provider configurations in desktop Settings. CLI can import secret-free JSON with `ai configure <file>`, set a key via hidden `ai key <provider-id>`, discover models with `ai models <provider-id>`, and send a UTF-8 prompt file with `ai chat <provider-id> --prompt-file <file>`. Add --stream for incremental output; Ctrl+C cancels. JSON mode emits a final structured reply. Never include secrets in configuration files or command-line arguments.

Tool Manager uses a pinned official ripgrep release with verified archive and executable hashes. CLI commands: tools list, tools plan ripgrep, tools install ripgrep, tools doctor and tools uninstall ripgrep. It installs under the selected workspace without administrator access and exposes a fixed version health check; arbitrary commands/security scanning are unavailable.

## Verification

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd apps\desktop
npm run build
npm test
npm run test:native
```

scripts/verify.ps1 -Native performs these checks and the real WebView2 test. Add -ToolInstall to exercise official tool download/verification/health/uninstall in isolated test data. Native tests require a custom-protocol desktop/CLI build and use a loopback mock AI server. Screenshots/results live in ignored artifacts. scripts/dev.ps1 starts normal development; scripts/toolchain.json records the tested compiler/runtime versions. Dependency resolution is locked; CI fixes Rust 1.95.0 and Node 24.15.0.

Add -Visual to capture the dashboard and nine secondary pages at 1920x1080, 1600x900, 1440x900, 1366x768 and 1280x720, check horizontal overflow, and exercise search, panel collapse, notifications, AI pane resizing and masked credential controls. This option also enables the native build/test. Native screenshots emulate content viewports in the actual WebView2.

See docs/IMPLEMENTATION_STATUS.md, docs/TEST_RESULTS.md and docs/ROADMAP.md for exact verified scope. Future module pages are explicitly labeled NOT STARTED.
