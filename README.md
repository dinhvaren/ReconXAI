# ReconX AI

A native desktop and CLI research workbench sharing a Rust core and SQLite database. Project management, settings, secure provider configuration, streaming chat/cancellation, safe connection fallback and a verified portable ripgrep installer work on Windows x64. The full research platform remains under development.

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

The custom-protocol build embeds frontend assets and needs no Vite server. Run scripts/build.ps1 for verified optimized Windows portable binaries and SHA-256 digests in artifacts/windows-x64. Both release binaries passed the native integration scenario. Installer configurations exist but installers and Linux remain unverified.

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

See docs/IMPLEMENTATION_STATUS.md, docs/TEST_RESULTS.md and docs/ROADMAP.md for exact verified scope. Future module pages are explicitly labeled NOT STARTED.
