# AI Provider Manager - verified 2026-10-17

Current source: HEAD `d6af47e` plus preserved local changes. This section supersedes
historical counts below. Full v1.0 is not complete; no live cloud account claim.

| Check | Actual result | Local record |
| --- | --- | --- |
| `cargo fmt --all -- --check` | exit 0 | `artifacts/ai-manager-final-format.log` |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | exit 0 | `artifacts/ai-manager-final-clippy.log` |
| `cargo test --locked --workspace` | 99 passed, 0 failed, 1 ignored | `artifacts/ai-manager-final-rust.log` |
| `npm run build` (Desktop) | TypeScript + Vite passed | `artifacts/ai-manager-verified-tauri.log` |
| `npm test` (Desktop) | 17 passed, 0 failed, 2 files | final run at 18:53 local time |
| `npm run tauri -- build --no-bundle` | optimized native Desktop built | `artifacts/ai-manager-verified-tauri.log` |
| `cargo build --locked --release -p reconx` | exit 0, optimized CLI built | final incremental build: 51.33 seconds |
| Linux `cargo test --locked --offline --workspace --exclude reconx-desktop -j2` | 99 passed, 0 failed, 1 ignored | `artifacts/ai-manager-final-linux.log` |

The ignored Rust case is the explicit OS-store fixture. Native Windows tests
separately save, replace, reopen and remove synthetic Credential Manager entries.
All databases used by this acceptance are task-owned temporary/artifact workspaces;
the normal user DB was not opened or migrated. No unrelated credential was read.

## Actual Windows Tauri/WebView2 acceptance

Matching optimized binaries: `artifacts/windows-ai-providers-20261017-185028/`.
Their SHA-256 values and all result paths are saved in
`artifacts/ai-manager-verification.json` and the binary directory's `checksums.json`.
The two delivered binaries had zero matches for the limited known GitHub-token
and private-key PEM patterns: `artifacts/ai-manager-binary-pattern-review.json`.
This is a limited pattern check, not a complete credential/security release audit.
Scripts use these exact binaries via `RECONX_DESKTOP_EXE` / `RECONX_CLI_EXE`, with
isolated `RECONX_DATA_DIR` and WebView2 profile directories.

| Native suite | Passed checkpoints | Result directory |
| --- | --- | --- |
| `tests/e2e/ai-providers.mjs` | 10 | `artifacts/native-ai-providers-1792237828887/` |
| `tests/e2e/native.mjs` (visual enabled) | 8 | `artifacts/native-1792238059810/` |
| `tests/e2e/inputs.mjs` | 9 | `artifacts/inputs-1792238103145/` |
| `tests/e2e/jobs.mjs` | 8 | `artifacts/jobs-1792238116432/` |
| `tests/e2e/integrations.mjs` | 6 | `artifacts/integrations-1792238145137/` |
| `tests/e2e/outputs.mjs` | 6 | `artifacts/outputs-1792238163896/` |
| `tests/e2e/recon-local.mjs` | 7 | `artifacts/recon-local-1792238177437/` |

Total: **54 passed checkpoints**, all 7 suites exit 0. The integration suite also
installed/verified actual pinned YARA-X, discovered/invoked benign stdio MCP tools,
ran real local jobs and restarted the app. Network assessment used owned loopback
fixtures only. AI responses came from deterministic loopback protocol mocks.

AI-specific native checks cover six vendor Save flows with no automatic network
calls, actual Responses/Messages/Gemini inference shapes, model discovery and
selection reaching the backend, custom auth/private headers, failed authentication
state/retry, Desktop/CLI routing parity, restart/secure keys, keyless local discovery,
profile removal and missing-key model disabling. Other registry vendors share the
validated compatible adapter where documented; no real cloud quota or model access
has been verified for any vendor.

Screenshots inspected visually:
`artifacts/native-ai-providers-1792237828887/ai-settings-light.png`,
`ai-settings-dark.png`, `ai-settings-reconx.png`, and
`ai-workspace-selected-model.png`. Native visual regression also captures nine
pages at 1920/1600/1440/1366/1280 widths, typography, dialogs and theme menus.

## Additional deterministic coverage / corrections

New service tests exercise atomic provider/key/header/default persistence, OS-store
failure compensation, no plaintext secrets in SQLite/WAL or profile serialization,
unchanged project data, credential destination binding/deletion, five protocol
inference/discovery forms, invalid model IDs, duplicate removal, selected-model
retention, safe 401/403/404/429 errors and stale concurrent key changes. Gateway
fixtures test explicit proxy use, header authentication and echo/stream redaction.
Artifact review refuses an unreviewed proxy. Migration tests preserve a simulated
schema-8 workspace with unknown tables and a pre-upgrade backup. CLI tests reopen
the same profile/default/routes through Core. Frontend tests exercise native-style
setup, filtering, keyless models, recoverable save errors and workspace URLs.

Earlier retained failures were corrected, not counted as passes: closed native
select options required attached-state waits; Default badges changed exact heading
selectors; old Credential-menu tests were migrated to Manage/API-key controls;
a stale embedded frontend was detected and a fresh optimized build was retested.
The final binary embeds `index-CFD9ZsHX.js`. Earlier intermediate artifacts are
not the verified delivery. No production error was hidden by synthetic output.

## Limits and next verification

Linux tests ran in actual Parrot x86_64 on WSL with the task-local Rust 1.95.0
installation. Linux Desktop lacks webkit2gtk-4.1 development SDK; Secret Service
runtime, optimized Linux distribution and installers remain UNVERIFIED. Windows
installer packaging was not run by this slice. Live cloud testing requires separate
explicit opt-in; local mocks do not establish hosted-model availability. The saved
agent slots are not a claim of fully implemented multi-domain agents. Advanced
capability enforcement, pricing/account metadata and fallback budgets remain work.

The prior verification record follows for historical provenance.

# Current verified checkpoint - 2026-10-17

Product version remains 0.1.0. This is not full v1.0 release acceptance.

- Windows: cargo fmt --all -- --check, Clippy (workspace/all targets, warnings
  denied), cargo test --locked --workspace passed: **86 passed, 0 failed,
  1 explicit OS-store fixture ignored**. Frontend build and **13 tests passed**.
- Final scripts/verify.ps1 -Outputs -LocalInventory passed. After the Linux-only
  fixture correction, Windows CLI 5 / research-input 8 tests and Clippy passed
  again. Production limits and binaries were unchanged by that test correction.
- Tauri optimized no-bundle build and optimized CLI build passed. The exact copied
  binaries in artifacts/windows-x64 passed **44 native checkpoints**:

| Suite | Checkpoints | Actual result record |
| --- | --- | --- |
| Regression / appearance / typography | 8 | artifacts/native-1792217533338/results.json |
| Targets / PE imports | 9 | artifacts/inputs-1792217578167/results.json |
| Jobs / cancellation / recovery | 8 | artifacts/jobs-1792217588488/results.json |
| YARA-X / MCP / local mock AI | 6 | artifacts/integrations-1792217614199/results.json |
| Evidence / findings / reports | 6 | artifacts/outputs-1792217631931/results.json |
| Approved loopback HEAD / outputs | 7 | artifacts/recon-local-1792217644790/results.json |

Latest debug output/local records: outputs-1792217139477 and
recon-local-1792217154231. Other debug suites passed earlier in this phase.
Actual three-theme captures at 1920/1366 and a rasterized optimized PDF were
visually inspected; shell regression covers five desktop sizes. PDFs were
independently parsed with pypdf, including Vietnamese, redaction and intact URL
assertions. Models and services were explicitly owned local fixtures.

Normal Desktop is open from the tested executable: recorded owned PID 10840,
title ReconX AI, zero startup stderr. Default schema 4 to 6 preserved hashes/counts
of **all 13 original tables**, with a matching schema-4 backup and zero foreign-key
errors. No fixture records were inserted in the normal database. Record:
artifacts/release-upgrade-1792216876468/after.json.

Known Git-token/private-key pattern scan returned zero matches in the two current
portable binaries. Scope/limitations and hashes are in
artifacts/windows-x64/credential-pattern-review.json. This is not a complete
credential, dependency or release audit. No Git remote configuration was read.

## Actual Linux shared Core and CLI

Parrot x86_64 on WSL2 became available on 2026-10-17. Earlier no-runtime findings
below are historical. GCC 14.2.0 and Node 20.19.2 were already installed. Official
Rust 1.95.0 was installed only in a task-owned /tmp directory after verifying the
published installer SHA-256. No profile or privileged system package changed.
Provenance: https://rust-lang.github.io/rustup/installation/other.html;
artifacts/linux-verify-1792217270990/installer.json records the actual checksum.

- Actual command: cargo test --locked --workspace --exclude reconx-desktop.
  **86 passed, 0 failed, 1 ignored**. Full stdout/stderr and exit code are retained
  in cargo-test-final.log and test-result-final.json.
- Standalone debug CLI smoke created a project/domain, imported /usr/bin/true ELF,
  independently matched SHA-256, extracted 29 sections, verified evidence, saved
  a Needs Verification observation (no vulnerability assertion), exported all four
  report formats and reopened records from new CLI processes. Windows pypdf
  independently extracted Vietnamese from the Linux-generated PDF.
- Evidence permissions were checked: file 0600 and directory 0700. Acceptance:
  artifacts/linux-verify-1792217270990/acceptance.json.
- Initial Linux tests correctly rejected an oversized DWARF test executable. The
  shared test-only native-sample.rs now strips debug sections from a private copy
  using binutils; original executable and production limit remain intact. Initial
  smoke input used invalid severity info; fixture corrected to informational.
- Linux Desktop is **UNVERIFIED**: pkg-config cannot find webkit2gtk-4.1. Linux
  frontend rendering, OS-store round-trip, optimized CLI packaging and installers
  remain unverified. Core tests do not establish Linux Desktop compatibility.

## Defects reproduced and fixed in this continuation

Real native preview exposed cookie leakage from embedded JSON header pairs.
Structured redaction now runs before embedding, also covering plain HTTP JSON,
base64 values and bounded nested JSON. Detectable unsafe old snapshots are retained
but blocked from export until a new preview is prepared. Actual PDF raster review
also caught drive-path matching p:/ inside http://; path boundaries were corrected
and URL preservation is asserted. Other fixes: visible busy-dialog dismissal state
with a frontend regression, exact accessible mode labels, accepted Windows sockets
explicitly leaving nonblocking mode, native locators and global Add Target usage.

Failed local-inventory artifacts are retained (1792152575394, 1792152721168,
1792216059876, 1792216071764, 1792216524155, 1792216580750). Both product defects
and harness assumptions were corrected; failed runs were never reported as passes.
Full recon, Web/API, general agents/mobile/cloud/CI/lab, installers and production
hardening remain incomplete. No user datasets or real provider keys were uploaded.

---

# Historical verification record

Windows x64, Rust 1.95.0 MSVC, Node 24.15.0, npm 11.7.0. Updated 2026-10-16.

## v1.0 Phase 1: actual research-output pipeline

- Baseline a29c5db: clean working tree; existing verify.ps1 passed 66 Rust / 12
  frontend tests, with one explicit OS-store fixture ignored.
- Current workspace: **76 Rust passed, 0 failed, 1 ignored**, including eight new
  output-service tests, one migration-004-to-005 preservation/backup test and one
  real CLI/Core evidence/finding/report test. Formatting and Clippy with warnings
  denied passed. TypeScript/Vite production build and **12 frontend tests** passed.
- `scripts/verify.ps1 -Native -Visual -Inputs -Jobs -Integrations -Outputs` passed
  **37 actual native checkpoints**: regression 8, input 9, jobs 8, tools/MCP/AI 6,
  outputs 6. Debug records:
  `artifacts/native-1792148709941`, `inputs-1792148767745`, `jobs-1792148781944`,
  `integrations-1792148821112`, `outputs-1792148864027`.
- `npm run tauri -- build --no-bundle` passed. `scripts/build.ps1` built/copied
  optimized Desktop/CLI binaries and font notices/checksums. Release regression:
  `artifacts/native-1792150150622`, `inputs-1792150193087`, `jobs-1792150202938`,
  `integrations-1792150227775`, `outputs-1792150245675`.
  After full JSON preview and format-reapproval were added, the final release
  output scenario passed again: **`artifacts/outputs-1792150784430/results.json`**.
- Actual local file parsing automatically publishes immutable source/job evidence.
  Native PNG import uses a real Windows picker; the browser renders the stored
  image. HTTP text is retained privately; integrity and manual review persist.
  A native Verified finding is rejected without supporting review, then an actual
  fixture observation saves with explicit confirmation. No vulnerability is invented.
- Native report preview exposes redactions and omitted images. JSON mode shows
  every structured field; changing format clears approval. The Windows save dialog
  default filename is checked against the chosen PDF format. A real multi-page
  PDF is independently parsed by installed pypdf 6.16.1, including Vietnamese text
  and redaction assertions. CLI exports valid JSON, safe escaped HTML and Markdown
  from the same immutable snapshot. Restart reopens the same records.
- Outputs captures cover Evidence/Findings/Reports in all three themes at 1920x1080
  and 1366x768. Actual captures and a rasterized PDF page were visually inspected;
  spacing and explicit PDF font weight were corrected from those observations.
- Failures fixed during development: initial Rust/TypeScript type/ownership issues;
  library CVSS defaults for incomplete vectors; discarded temporary test-directory
  ownership; missing dropdown accessible labels; native harness expecting a code
  instead of a UI error message; a locator bound to the pre-restart page; and a
  cross-process Edit read incorrectly using GetWindowText rather than WM_GETTEXT.
  A real redaction regression for JSON `name: Api-Key / value` pairs was reproduced
  and fixed. Failed native records remain under outputs-1792147772431,
  1792147922248, 1792148096942, 1792148332827 and 1792149695212.
- One release copy attempt failed because the previously opened owned Desktop
  locked its executable. Its exact PID/path and zero active workspace jobs were
  checked; it was closed normally, then the build/copy passed. No forced termination
  of a normal user application or deletion of user data was used.
- Existing default-workspace preservation baseline:
  `artifacts/outputs-upgrade-1792148765308/before.json`. Normal startup upgrade smoke
  is recorded separately when performed. Linux runtime remains **UNVERIFIED**;
  WSL lists no installed distribution. No cloud provider/user data upload occurred.

The earlier sections below retain their original milestone counts and limitations.

## Milestone 2 verification (Windows x64)

- Baseline before implementation: 57 Rust passed, one explicit OS-store fixture
  ignored; 11 frontend passed. Local HEAD was 7587e63 and the working tree was clean.
- Current Rust workspace: **66 passed, 0 failed, 1 ignored**. Seven new real MCP/
  mock-model integration tests cover stdio/HTTP authentication, discovery/schema,
  exact approval, persisted jobs/restart, denied permissions, timeout/cancellation,
  unavailable executable/server, oversized output, escaped credential redaction,
  structured AI policy rejection, actual local jobs and saved interpretation.
  New storage/tool unit tests verify pre-upgrade SQLite backup and manifest safety.
- Formatting/Clippy (workspace/all targets, warnings denied): passed.
- Frontend TypeScript/Vite build: passed; **12 frontend tests passed**, including
  secret-free MCP configuration and disabled untrusted connection controls.
- Debug native verification: **31 checkpoints passed** (8 regression/visual,
  9 target/import, 8 existing job engine, 6 tools/MCP/AI integration):
  `artifacts/native-1792081858453/results.json`,
  `artifacts/inputs-1792081901057/results.json`,
  `artifacts/jobs-1792081914450/results.json`,
  `artifacts/integrations-1792081951499/results.json`.
- Native Tool Manager downloads official YARA-X, verifies archive/executable hashes,
  runs its actual version, shares registry with CLI, executes fixed signatures,
  updates to the pinned manifest, removes and repairs the installation, displays
  real operation logs, and installs the reviewed Reverse profile.
- Native MCP Hub adds a real Node stdio fixture, discovers tools/schemas, permits
  only an approved benign echo call, persists actual output/logs, reconnects after
  restart and proves a fixture host-secret environment marker was not inherited.
  HTTP auth/invalid credentials are additionally covered by the Rust local server
  fixture with an isolated in-memory secret store.
- The **deterministic local HTTP model fixture** produces a structured capability
  plan, then genuine metadata/YARA-X jobs run through the shared engine. Native
  consent, actual output SHA-256/signature, events, persisted interpretation and
  CLI reads/restart are verified. No live cloud model account is claimed.
- Actual screenshots for MCP, reviewed plans, evidence interpretation, tool logs
  and all three themes were captured and visually inspected. New pages also have
  1366x768 bounds assertions; existing native visual regression covers five sizes.
- Initial native harness runs failed on separator/accessible-name mismatches and
  a completed-event arrival race. The corrected harness waits for actual events;
  failed artifacts remain locally available. No production tool output was faked.
- Linux runtime is **UNVERIFIED**: WSL lists no available distro; Docker reports
  its Linux engine pipe unavailable. Official Linux YARA-X archive/executable
  hashes were checked, but the executable was not run. CI was configured, not run.
- Optimized Windows release: `npm run tauri -- build --no-bundle` and
  `scripts/build.ps1` passed. The copied portable Desktop/CLI binaries passed the
  same **31 native checkpoints**:
  `artifacts/native-1792144968533/results.json`,
  `artifacts/inputs-1792145335173/results.json`,
  `artifacts/jobs-1792145365667/results.json`,
  `artifacts/integrations-1792145390620/results.json`.
- A preceding release input run failed before its first checkpoint because the
  saved domain was `example.testo` instead of the fixture's `example.test`:
  `artifacts/inputs-1792145013664/results.json`. The unchanged test and application
  passed all nine input checks on rerun. Source inspection found no code appending
  this character; the cause is not established and the failed record is retained.
- The ordinary release Desktop started against the existing default workspace;
  schema 3 upgraded to 4 after an automatic private SQLite backup. Before/after
  record counts and digests for projects/settings/providers/targets/imports/jobs/
  analyses/notes are identical. Evidence:
  `artifacts/milestone-2-upgrade-1792082372052/before.json` and `after.json`.
  This default workspace had one settings row and no research records; migration
  fixtures separately verify upgrades containing existing project/file/job data.
  No acceptance fixtures were added to the default workspace.
- Copied release SHA-256:
  CLI `713809688da0c0787c580dd7a0350bc53b16989eb8be1d3c84771090f69ea543`;
  Desktop `a563bb62c5759ba4ade6ff684281d59b6879111000bf9c989a6d535747a2341a`.
  Native release MCP captures (`mcp-light.png`, `mcp-dark.png`, `mcp-reconx.png`),
  the 1366x768 workflow and operation/evidence dialogs were visually inspected in
  `artifacts/integrations-1792145390620`.

The older sections below retain historical evidence; their counts do not describe
Milestone 2. Native checkpoint counts are assertions in grouped scenarios, not
independent unique products or vulnerabilities.

## Current durable local-job verification

Command: scripts/verify.ps1 -Native -Inputs -Jobs -Visual.

- Baseline before changes: scripts/verify.ps1 passed, 45 Rust / 11 frontend tests.
- Final formatting and Clippy (workspace/all targets, warnings denied): passed.
- Rust workspace: **57 passed, 0 failed, 1 ignored** (explicit OS-store fixture).
  Eleven new core job tests cover typed handlers, corruption/timeout, queued/running
  cancellation, cross-engine concurrency, live/expired lease ownership, migration
  with retained binary artifacts, notes/restart, explicit AI consent/destination,
  no fallback and a synchronous CLI/already claimed Desktop worker race.
  CLI integration also verifies actual enqueue/drain/result/cancel/note commands.
- TypeScript/Vite production build: passed; frontend **11 passed, 0 failed**.
- Actual embedded Tauri/WebView2 debug: **25 checkpoints passed**:
  existing regression 8, target/file input scenario 9, new job scenario 8.
- Debug records: artifacts/native-1792077037276/results.json,
  artifacts/inputs-1792077081584/results.json,
  artifacts/jobs-1792077094197/results.json and events.json.
- Research Demo/example.test is created through Desktop in an isolated workspace,
  reopened after process restart and read by CLI. No public target traffic occurs.
- Actual native Windows picker imports the owned PE executable; independent Node
  SHA-256 matches persisted Rust data. Sections/strings and exact job results reopen.
- A real 64 MiB raw fixture enters Running, is cancelled through native UI and
  commits no result. A configured timeout genuinely fails; killing the owned native
  process during work and restarting produces JOB_INTERRUPTED after lease expiry.
- The local AI fixture receives zero calls without consent, then one request after
  explicit native approval. The payload excludes source paths and local notes.
- Native captures verify Light/Dark/ReconX, Job Center compact control bounds and
  document bounds at 1366/1920. Existing regression covers 50 secondary page/viewport
  and 12 appearance observations. Actual PNGs were inspected, not just generated.
- Existing version-2 default workspace was backed up before release upgrade at
  artifacts/upgrade-1792077147853; only counts/digests are recorded in preservation
  metadata. Existing OS credentials and Git remote settings were not changed; native
  regression creates/removes only its own ephemeral credential entry.
- scripts/build.ps1: optimized Windows desktop and CLI builds passed; copied
  executables and SHA256SUMS.txt/font notices in artifacts/windows-x64 were verified.
  The copied release executables passed **the same 25 native checkpoints**:
  artifacts/native-1792077312439/results.json,
  artifacts/inputs-1792077355999/results.json,
  artifacts/jobs-1792077366043/results.json. Release PNGs were captured and inspected.
- Normal release Desktop was opened on the existing default workspace. Schema 2
  upgraded to 3, all previous record counts and settings/provider/project/file/result
  digests matched, and foreign_key_check was empty. Preservation metadata is in
  artifacts/upgrade-1792077147853/after-upgrade.json.
- Updated CI YAML parsed successfully; no remote CI run or push was performed.
- Normal startup smoke also verified a visible ReconX AI window and zero startup
  error-log bytes: artifacts/windows-x64/normal-startup.json. This check is separate
  from the isolated hidden native automation instances.

Intermediate development failures (integer progress overflow and accessible labels
containing loaded control text) were fixed; their failed native artifacts remain
available. A Windows loopback refusal fixture uses a sufficient timeout to exercise
connection failure rather than an unrelated request deadline.

All development uses isolated owned samples/services and real parser output.
Timeout/cancellation are cooperative, not process-level hard termination. Physical
OS drop/folder-picker, Linux/ARM64, installer execution, live cloud AI and advanced
scanner/MCP/findings/report workflows remain unverified or unimplemented as stated.

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
