# Verification record

Windows x64, Rust 1.95.0 MSVC, Node 24.15.0, npm 11.7.0. Updated 2026-10-14.

## Current typography verification

- Baseline: TypeScript/Vite build and 11 frontend tests passed before changes.
- After: npm test **11 passed, 0 failed**; npm run build passed (TypeScript + Vite).
- cargo check --locked --workspace: passed.
- cargo fmt --all -- --check and cargo clippy --locked --workspace --all-targets
  -- -D warnings: passed. cargo test --locked --workspace: **45 passed, 0 failed,
  1 ignored** (explicit OS credential-store fixture).
- npm run tauri -- build --no-bundle: optimized Windows desktop build passed.
  scripts/build.ps1 also built/copied desktop and CLI, checksums and font notices
  to artifacts/windows-x64. This does not claim an installer was generated.
- Native typography before and after: **57 observations per phase**, including
  Dashboard, Projects, Targets & Scope, Reverse Engineering, AI Workspace and
  Settings at 1920x1080, 1440x900 and 1366x768 in all three themes (54), plus three
  Settings device-scale captures. Actual Tauri/WebView2 screenshots were inspected.
- Evidence: artifacts/typography-refinement/{before,after}/results.json,
  computed-styles.json and PNGs. Interactive comparison: compare.html in their
  parent directory. After checks verify 24/600 titles, 400 navigation, 500 active
  navigation, no sampled text below 12px or above 600, actual platform fonts,
  readable 13px technical text, navigation bounds and workspace overflow.
- Platform-font trials found the unsuffixed Segoe UI Variable family falling back
  to Arial; the Text family resolves to native Segoe. Bundled Source Sans 3 was
  separately loaded from the application origin and inspected in WebView2.
- Existing native debug regression: **8 checkpoints passed**, including 50
  secondary-page/viewport and 12 theme/page/viewport observations, synchronized
  appearance menus/settings/restarts, project/CLI sharing, local AI and key masking.
  Record: artifacts/native-1791966981632/results.json.
- Copied optimized release input regression: **9 checkpoints passed**. Record:
  artifacts/inputs-1791968144226/results.json. Native file picker, scope editing,
  import/export, shared target/PE results, persistent jobs and restart were tested.
- Copied optimized release existing regression: **8 checkpoints passed**, including
  the same 50 page/viewport and 12 appearance observations, all three theme restarts
  and both synchronized menus. Record: artifacts/native-1791968095176/results.json.

All fixtures use isolated databases and local owned samples/test gateways. No user
project or stored credential was reset. Device scales 1.25/1.5/2 are CDP emulation
of the native renderer, not physical Windows monitor-setting tests. Linux/ARM64
rendering, live cloud providers and unimplemented research modules are unverified.

## Current functional-input verification

Baseline before changes: scripts/verify.ps1, 31 Rust / 9 frontend passed.
Final command: scripts/verify.ps1 -Inputs -Native.

- Rust formatting and Clippy, warnings denied: passed.
- Rust workspace: **45 passed, 0 failed, 1 ignored** (explicit OS credential fixture).
- TypeScript/Vite production build: passed; frontend **11 passed, 0 failed**.
- Native embedded debug application/CLI: passed, **6 existing regression checkpoints**
  and **9 research-input checkpoints**, each using an isolated shared database.
- Records: artifacts/native-1791946811208/results.json and
  artifacts/inputs-1791946818102/results.json.
- Native Open and Save As selected actual owned PE/JSON inputs: target create/edit,
  scope decisions, native target import/export, quick actions, PE hash/sections/imports/
  strings/job persistence, native restart, and document bounds at 1366×768.
- ReconX/Light/Dark screenshots were captured from the actual renderer and inspected;
  these illustrate real fixture results, not fabricated security findings.
- Parser fixtures verify ELF64, thin Mach-O64, APK content identification, malformed
  signatures, extraction limits and entropy. Core tests verify all 17 target kinds,
  atomic imports, scope boundaries/exclusions, input failures, version-1 migration
  preservation, archive policy, cancellation, changed authorization/identity,
  modified sample/result rejection, comparison and retained originals/history.
- npm run tauri -- build --no-bundle: optimized Windows desktop build passed.
- CI YAML parsed; Windows native input step added. CI execution is still unverified.

The first native attempts exposed selector/modal initialization issues and Windows
picker automation differences; these were corrected before the final passing run.
Earlier failed artifacts are retained. Physical OS drag/drop and folder-picker
interaction are unverified; folder import/comparison/error handling have Rust tests.
Linux/ARM64 runtime and installer installation are unverified. Cloud AI providers,
MCP, recon, agents, findings and report scenarios have no new acceptance evidence.

Release verification: scripts/build.ps1 passed and produced desktop/CLI plus
SHA256SUMS.txt and bundled-font notices in artifacts/windows-x64. The actual copied
release executables passed the same 9 input and 6 existing regression checkpoints:
artifacts/inputs-1791947186174/results.json and
artifacts/native-1791947198252/results.json. Release reconx --version returned 0.1.0.
Release native screenshots were generated; these are separate from debug evidence.

## Previous three-theme verification — 2026-10-13

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
