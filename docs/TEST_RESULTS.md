# Verification record

Windows x64, Rust 1.95.0 MSVC, Node 24.15.0, npm 11.7.0. Updated 2026-10-13.

## Current three-theme verification

Command: scripts/verify.ps1 -Native -Visual -ToolInstall.

- Rust formatting and Clippy with warnings denied: passed.
- Rust workspace: 31 passed, 0 failed, 1 ignored by default.
- Frontend TypeScript/Vite build: passed.
- Frontend tests: 9 passed, 0 failed.
- Native embedded Tauri/CLI debug build and run: passed, nine checkpoints.
- Optimized Windows Tauri/CLI build and run: passed, nine checkpoints.
- Explicit Windows OS credential round trip previously passed with an ephemeral entry removed; native scenarios also verify masked credential set/status/removal.

Debug record: artifacts/native-1791906428229/results.json. Release record: artifacts/native-1791906555280/results.json. Release tests use target/release executables with RECONX_E2E_VISUAL=1 and RECONX_E2E_TOOL_INSTALL=1.

scripts/build.ps1 generated final portable binaries, SHA256SUMS.txt and bundled font notices in artifacts/windows-x64. The old normal window closed gracefully and the new executable opened with a valid ReconX AI window handle. Normal saved system preference migrated to reconx; security mode and default/fallback provider settings compared equal before/after. No project database was reset. A window-only Win32 capture is saved at artifacts/windows-x64/desktop-preview.png.

## Runtime checkpoints

1. Desktop project creation is visible to CLI in the shared SQLite database.
2. CLI project creation is visible to desktop. Explicit Local workspace context works.
3. Provider discovery and CLI/native text/streaming work against an owned loopback fixture.
4. Native cancellation interrupts the response and records a cancelled audit outcome.
5. Projects, provider configuration and settings survive complete process restart.
6. Exactly Light/Dark/ReconX apply immediately from synchronized controls, preserve unsaved Settings edits and share persistence with CLI. OS appearance changes do not select another theme.
7. Search, panel collapse, notifications/Escape, context resizing/tabs and masked credential operations work. Dashboard and nine pages fit all five requested desktop viewports.
8. Actual official pinned ripgrep install, archive/executable SHA-256 verification, CLI/native inventory, version health check and uninstall work.
9. DM Sans/IBM Plex Mono load where used. Dashboard/Settings, menus and dialogs render in all three themes at 1366x768 and 1920x1080. Titles are 26px, navigation 14px, no visible text/control below 12px, and navigation labels do not clip. Keyboard arrows, selection, Escape and Tab work. Each theme survives restart; the exact menu/dropdown choices and preserved ReconX color tokens are asserted.

viewport-matrix.json has 50 page/size observations across 1920x1080, 1600x900, 1440x900, 1366x768 and 1280x720. appearance-matrix.json has 12 observations (three themes, two pages, two sizes). Screenshots are beside each record, including theme-light-dashboard-1920x1080.png, theme-dark-dashboard-1920x1080.png and theme-reconx-dashboard-1920x1080.png. Light/charcoal Dark/burgundy ReconX Dashboard, Settings and menu screenshots were captured and visually inspected. Original ReconX tokens are frozen in docs/palettes/reconx-original.json from revision ac18d15 and verified against native computed values.

## Regression and migration coverage

Rust tests include transactional theme-only updates, all old light/dark/system/missing migration cases, canonical persisted values, preservation of project/provider/mode/routing records and idempotent reopening. Frontend tests cover menu/dropdown synchronization, immediate saving, draft protection, write failure rollback, rapid write serialization and absence of an OS appearance listener, alongside existing project/error/navigation checks.

Existing coverage remains for SQLite migrations, CLI parsing/errors, project archive/import/export, concurrent writers, credentials/gateway isolation, provider protocol parsing/redaction, streaming/cancellation, fallback without ambiguous replay and installer integrity/safe uninstall.

## Limits and history

Captures are emulated content viewports inside actual native WebView2. Multiple physical monitors/DPI, Linux/ARM64, live cloud accounts, CI execution and installer generation/installation remain unverified. Future analysis/MCP/workflow/reporting services remain unimplemented. Fixture responses are protocol test data, not research findings. No user API keys were captured and no public targets were tested.

The preceding typography repair used Light/Dark/System; that mode is now removed. Its records remain under artifacts/native-1791904765297 and artifacts/native-1791905144941. Earlier original redesign records remain under artifacts/native-1791901611396. Native profile contention was resolved with an isolated WEBVIEW2_USER_DATA_FOLDER. No known failing introduced build/runtime checks remain.
