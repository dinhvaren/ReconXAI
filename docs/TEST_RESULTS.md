# Verification record

Windows x64, Rust 1.95.0 MSVC, Node 24.15.0, npm 11.7.0. Updated 2026-10-13.

## Latest checks

- scripts/verify.ps1: Rust format, Clippy with warnings denied, workspace tests, frontend TypeScript/Vite build and frontend tests passed.
- Rust workspace: 29 passed, 0 failed, 1 ignored by default.
- cargo test --locked -p reconx-ai --test gateway real_os_secret_round_trip -- --ignored: 1 passed; ephemeral Windows credential entry removed.
- npm test after the added keyboard-search regression: 6 passed, 0 failed.
- Final npm run build after correcting the test's ByRoleOptions types: passed (TypeScript and Vite).
- scripts/build.ps1: optimized Windows CLI and embedded Tauri desktop built successfully. Artifacts and SHA256SUMS.txt are in artifacts/windows-x64.
- scripts/verify.ps1 PowerShell parser check: passed after adding -Visual.
- git diff --check: passed (line-ending notices are informational).

## Native debug and release scenarios

Debug: artifacts/native-1791901117905/results.json, eight passing checkpoints. Release: artifacts/native-1791901462246/results.json, eight passing checkpoints. Final expanded release matrix: artifacts/native-1791901611396/results.json, eight passing checkpoints. Executed with RECONX_E2E_VISUAL=1 and RECONX_E2E_TOOL_INSTALL=1; release tests select the generated release executable paths.

Checkpoints:

1. Desktop project creation is visible to CLI through the shared database.
2. CLI project creation is visible to desktop.
3. Provider discovery, text and streaming requests work through both interfaces against an owned loopback HTTP fixture.
4. Native cancellation interrupts streaming and persists a cancelled audit outcome.
5. Projects, provider configuration and settings survive full native restart.
6. Dark/Light preferences persist and System responds to emulated OS appearance changes.
7. Search, panel collapse, notification dialog/Escape, context resizing/tabs and masked OS credential store/status/removal work. Dashboard plus nine secondary pages fit all five desktop viewports without document/workspace horizontal overflow.
8. Actual pinned official ripgrep download, archive/executable SHA-256 verification, desktop/CLI inventory, fixed version health checks and uninstall work.

Final viewport-matrix.json contains 50 page/viewport observations: 1920x1080, 1600x900, 1440x900, 1366x768 and 1280x720 for Dashboard, Projects, AI Workspace, Settings, MCP Hub, Tool Manager, Reconnaissance, Findings, Reverse Engineering and Reports. Screenshots are beside the record. Dashboard, light appearance, secondary pages, narrow layouts and collapsed shell were visually inspected against the downloaded reference.

These are real native WebView2 captures with emulated content viewports. Physical monitor scaling, Windows chrome, Linux rendering and future module execution are not verified by them. Fixture projects, model replies and credentials are clearly test data and are isolated from normal user data. No stored user API keys were captured.

The normal release application was also opened visibly with the existing default database and Dark appearance. A window-only Win32 capture was inspected at artifacts/windows-x64/desktop-preview.png, showing the actual Windows title bar and native content. This establishes this local window's rendering, not multiple physical monitors or DPI scales. The running release process had title ReconX AI and a valid main window handle.

## Regression coverage and resolved failures

Rust coverage includes migrations, CLI parsing/errors, concurrent writers, project persistence/archive/import/export, appearance compatibility, gateway policy, credential binding, provider payloads/model lists/redaction, streaming/cancellation, safe fallback without HTTP replay and installer integrity/preservation of unexpected files. Frontend coverage verifies project/settings IPC, errors, theme persistence/OS updates and keyboard navigation with an unconfigured provider.

The hero initially failed to display because of an unquoted inline SVG data URL; bundled CSS assets fixed it and native tests assert a rendered background. Planned marker titles initially changed accessible navigation names; explicit labels fixed native selection. Screenshots wait for fonts/repaint. A new unit test expected an outdated empty-state phrase; it now checks the disabled send control and passes. The same test incorrectly used Playwright's exact option in Testing Library ByRoleOptions; removing that unsupported option fixed the final TypeScript build. No known failing introduced checks remain.

## Unverified scope

Cloud accounts, Linux/ARM64, installer generation/installation, CI execution and future analysis/MCP/workflow/reporting services remain unverified or unimplemented. A portable binary build does not establish installer operation. No public infrastructure was actively tested. Previous pre-redesign foundation records remain under artifacts/native-1791894959180 and artifacts/native-1791895084172.
