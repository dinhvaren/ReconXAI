## Current verified checkpoint — 2026-10-29

Version **0.1.0**, SQLite schema **019**, **v1.0 NOT READY**. This supersedes
older checkpoints below. Source base `c920130bab108a55f228b69789d47caeb8e8850f`;
verification ran on the preserved working tree, with source hashes recorded in
[research-provenance-20261029.json](verification/research-provenance-20261029.json).
Local full proof: `artifacts/research-current-verification.json`. No normal user
workspace/credential was reset, sample executed or public target scanned.

Implemented in this slice:
- Pinned real HTTPX/Subfinder adapters, official archive/executable hashes, exact
  pinned PATH reuse, private version/execution config, real local GET/source-list
  health, safe receipt-owned repair and preservation of unexpected destinations.
- Reviewed typed jobs, exact target/scope/tool approval, IP-pinned HTTPX transport,
  bounded output/timeout/cancellation, inventory/evidence, shared Desktop/CLI and
  target research sessions. Migration018 preserves prior session/selection data.
- Read-only persistent asset/exploration provenance over actual sessions, plans,
  approvals, jobs, inventory, evidence, manual findings and reports. Migration019
  adds graph tables, not a queue. Graph review cannot execute tools or AI.
- Research Session graph filters/search/pagination, keyboard/pointer pan, zoom,
  neighborhood focus, source inspection, JSON export and stable restart IDs.
  Local import paths/URL credentials are omitted from graph metadata; source
  evidence integrity remains a separate check. Marketplace search selection and
  duplicate raw/normalized evidence storage were fixed, not hidden by test changes.

| Current verification | Actual result | Evidence |
|---|---|---|
| Windows Rust workspace | 202 passed, 0 failed, 2 explicit opt-in ignores | `artifacts/research-graph-final-rust.log` |
| Parrot/WSL Core + CLI | 202 passed, 0 failed, 2 ignored; 78 Rust/SQL files match | `artifacts/research-graph-linux.log`, source comparison JSON |
| Frontend / TS / Vite | 44 passed; current production build passes | `research-graph-final-frontend-tests.log`, `research-graph-readable-frontend-build.log` |
| Formatting / Clippy | rustfmt check + warning-denied Windows/Linux Clippy pass | proof command ledger; `artifacts/research-final-fmt.log` |
| Windows native research/recon/graph | 12 acceptance groups, 33 actual captures | `artifacts/recon-tools-native-1793259682347/results.json` |
| Native HTTP / outputs regression | 11 / 6 acceptance groups, separately recorded | `artifacts/http-replay-1793259879215/results.json`, `artifacts/outputs-1793259890313/results.json` |
| Legacy manual HEAD regression | 7 groups; current collapsed controls and existing data/exports work | `artifacts/recon-local-1793260674071/results.json` |
| Real tools CLI | 7 groups including repair; Windows/Linux actual owned HTTPX | release CLI fixture records in proof |
| Graph CLI / persistence | Actual schema018 fixture → 019, 16 nodes/17 edges, jobs unchanged, filters/export/backup | Windows/Linux graph acceptance JSON in proof |

Counts are per platform/suite, not summed across repeated runs. Mock planning and
interpretation remain **MOCK_VERIFIED**; no live cloud account claim. Subfinder's
real installation/version/source inventory and parser are tested, but live passive
domain enumeration is **NOT_TESTED** without an owned authorized domain/source.
Remote HTTPX/TLS runtime is unverified; certificate metadata is not a trust verdict.

Accepted development artifacts (unsigned; no installer/release publication):
`artifacts/windows-research-graph-final-20261029/{reconx-desktop.exe,reconx.exe}` and
`artifacts/linux-research-graph-20261029/reconx`. Each folder contains checksums and
a build manifest. Linux GUI remains **UNVERIFIED**; WSL Core/CLI is not GUI proof.

The five-capability request remains **PARTIAL**. No automatic offensive continuation
loop, multi-agent runtime/message bus, general DAG/pause/resume/budgets, universal
invocation envelope, new capture policy/proxy/CA or global asset graph is claimed.
Existing HAR/manual-loopback traffic capture works; a dedicated opt-in recorder is
not implemented. Graphs are bounded session projections refreshed on review, not
an inference system or complete transition-history engine. Refresh after target/
file removal remains a known gap. No hypothesis is automatically Verified.
The existing glib/RustSec, installer/signing/updater, broader domains and Linux GUI
release gates remain open; advisory data was not refreshed in this feature slice.

Guides: [RECON_TOOL_ADAPTERS.md](RECON_TOOL_ADAPTERS.md),
[EXPLORATION_GRAPH.md](EXPLORATION_GRAPH.md),
[UNIFIED_TOOL_EXECUTION.md](UNIFIED_TOOL_EXECUTION.md),
[AUTONOMOUS_RESEARCH.md](AUTONOMOUS_RESEARCH.md),
[MULTI_AGENT_ARCHITECTURE.md](MULTI_AGENT_ARCHITECTURE.md),
[TRAFFIC_RECORDING.md](TRAFFIC_RECORDING.md).

---

Earlier checkpoint history (not current test totals):

## Current product-recovery checkpoint — 2026-10-20

Version **0.1.0**, schema **017**, **v1.0 NOT READY**. This checkpoint supersedes
older results below without combining their test counts. Source base
`b041b667d6c918ce0e22c91f1c65222a2fe91aa2`; changes were uncommitted when verified.
Current evidence: `artifacts/product-recovery-current-verification.json`. No normal user database, credentials or imported
dataset was reset; no sample was executed and no release was published.
Repository verification metadata: [product-recovery-20261020.json](verification/product-recovery-20261020.json).
The local `artifacts/product-recovery-git-delivery.json`, when present, records the
final commit/push and working-tree state; the revision above is the test baseline.

| Current verification | Actual result | Evidence |
|---|---|---|
| Windows workspace Rust | 191 passed, 0 failed, 2 explicitly ignored | `artifacts/large-import-rust-verified.log` |
| Parrot/WSL Core + CLI Rust | 191 passed, 0 failed, 2 explicitly ignored; 71 Rust/SQL files match staged source | `artifacts/large-import-linux-final.log` |
| Clippy and rustfmt | Warning-denied Windows/Linux Clippy; Windows rustfmt pass | `large-import-clippy-verified.log`, Linux log, `recovery-combined-fmt.log` in artifacts |
| Frontend / TypeScript / Vite | 39 passed; current production frontend build passes | `artifacts/recovery-final-frontend-tests.log`, `recovery-final-frontend-build.log` |
| Windows optimized Desktop + CLI | Actual portable executables, both NotSigned | `artifacts/windows-product-recovery-20261020-ux-final`, `artifacts/product-recovery-final-assets.log` |
| Linux optimized CLI | Actual ELF; Core/CLI acceptance only | `artifacts/linux-cli-large-import-final-20261020/reconx` |
| Native integrated research | 9 acceptance checks | `artifacts/research-session-1792506323444/results.json` |
| Native large-file import | 6 acceptance checks | `artifacts/large-import-native-1792506310716/results.json` |
| Native HTTP regression | 11 acceptance checks | `artifacts/http-replay-1792506338490/results.json` |
| Native evidence/findings/reports | 6 acceptance checks; independent PDF text/redaction inspection | `artifacts/outputs-1792506349760/results.json` |
| Native shell UX | 216 page/theme/size observations; 3 shell/theme checks | `artifacts/ux-after-1792506362426/results.json` |
| Real research CLI | 36 commands on each host; mock planning + real HEAD/static jobs + four reports | `artifacts/research-cli-{windows,linux}-combined-20261020/results.json` |
| Large CLI fixtures | 80/256/512/1024 MiB; independent source/blob hashes; current largest metadata job; Linux running SIGINT | `artifacts/large-import-cli-windows-20261020/results.json`, `large-import-cli-linux-final-proof-20261020/results.json` |
| Actual schema-16 → 17 CLI upgrade | 14 commands each host; old queued job/evidence preserved; valid exact backup; old client rejects newer DB without byte changes | `artifacts/research-upgrade-{windows,linux}-20261020/results.json` |

### Diagnostics and visual evidence

The first expanded native research assertion read the top-bar target before the
independent CLI snapshot refreshed. Its failure screenshot/log remain retained;
the corrected test waits for both displayed and SQLite-persisted IDs and passes.
A worker-start timeout run executed zero frontend tests under concurrent heavy
linking; bounded two-worker reruns pass all39. A source-edit/compile race diagnostic
is superseded by a fresh full191-test run. None of these failed attempts is counted
as a pass. Exact retained paths are in the verification JSON.

Current research and large-file views each have 12 actual captures (three themes,
1280×720/1366×768/1440×900/1920×1080). The shell UX run adds 216 observations over
18 pages at those same themes/sizes, checks sampled control heights30/34/38,
disabled explanations and overflow, plus theme keyboard/restart and emulated
raster scaling. Five representative final images (large Light1280, research
Dark1366 and ReconX1920, AI Workspace Dark1366 and Targets Light1280) were opened
and inspected; not every capture was manually
inspected. No user screenshot attachment was available for pixel comparison.
The existing typography and exactly three themes are preserved.

These passes do not complete the broad workbench. Live cloud AI and actual local
inference engines are unverified (research AI tests use deterministic owned mocks).
General remote replay, active authorization matrix, stateful APIs, full schema,
DNS/RDAP/CT/scanners, advanced analyzers/mobile/cloud/network/lab, general DAG and
standing auto-policy, pause/resume, budgets/billing/OAuth, two useful Marketplace
analyzer categories, first-run dependency setup, signed updater, clean installer
lifecycle and Linux GUI remain PARTIAL/BLOCKED/UNVERIFIED. The known glib unsound
and maintenance security gate remains unresolved; advisory data was not refreshed
in this functional slice. Windows binaries are unsigned.

Large-file limits remain explicit: forced termination may leave private import
staging; no automatic crash-staging GC/resume, general workspace quota, complete
large-file decompilation or OS process sandbox. Custom Windows ACL hardening is
still open. The configurable 64 GiB ceiling is not runtime verification at 64 GiB.
Authorized Auto currently means reviewed offline steps, not reusable standing
authorization or uncontrolled scanning. Missing usage/cost is unavailable, never
fabricated as zero. MCP same-user subprocesses are not sandboxed.

---

Historical checkpoints below retain their original scope and dates.

## Product recovery: target-bound research — 2026-10-20

Version **0.1.0**, schema **017**, **v1.0 NOT READY**. This checkpoint supersedes
older counts below; historical sections remain historical. Source base b041b66;
changes are uncommitted. No reset, normal user-data cleanup, commit, push or release.

Implemented and runtime verified: shared persisted project/target IDs with revision
checks; Start AI Research entry points; one integrated session view; metadata-only
AI planning consent; strict capability-ID plans; exact one-shot approval and atomic
job batch; separately approved owned-loopback HEAD; real local metadata/strings;
existing YARA-X availability detection; actual progress/logs; evidence; separately
reviewed unverified AI interpretation; manual Needs Verification finding; report
preview and four existing export formats. Desktop and CLI use the same Core and DB.
No arbitrary shell/MCP/scanner execution or automatic Verified finding was added.

Proof: `artifacts/research-current-verification.json`. Accepted portable Windows
pair: `artifacts/windows-research-current-20261020`; Linux CLI:
`artifacts/linux-cli-research-20261020/reconx`. Both Windows executables are
actually NotSigned. Native tests assert the embedded frontend asset matches current
Vite output; earlier candidates are preserved. Native screenshots are genuine
WebView2 captures; no user attachment was available for pixel comparison.

Current results: Windows Rust **186 passed / 0 failed / 2 ignored**; Linux Core/CLI
**186/0/2**; frontend **39 passed**. Warning-denied Clippy both hosts; rustfmt and
TypeScript/Vite pass. Native current build: research **8**, HTTP regression **11**,
output regression **6** acceptance checks. Research captures cover exactly three
themes × four sizes (1280/1366/1440/1920), no document overflow; Light1280, Dark1366
and ReconX1920 captures were opened and inspected. Real CLI research acceptance is
**36 commands per host**, including approved fixture HEAD, mock AI plan/interpretation,
hash verification, real PE/ELF static jobs, review finding and four redacted reports.
These are separate suites, not a claim of broad domain coverage.

The previously pending cached-MCP-runtime fix also passes actual Windows negative
acceptance: **37 commands**, running install cancellation, previous Ready retention,
cached package binding to the newly approved managed Node path/hash, actual MCP
health and failed corrupted-package update rollback. Evidence:
`artifacts/mcp-cli-research-negative-20261020/negative-results.json`.

Limits: only listed bounded local capabilities; domain registration alone is not
recon/scanning. Remote replay, active authz matrix, stateful APIs, multi-domain DAG,
pause/resume, reusable standing auto-policy, accounting/cost budgets, live cloud
accounts, actual Ollama/LM Studio/vLLM engines, advanced decompiler/mobile/cloud/network
adapters, two useful marketplace analyzer categories, setup/installer/signed updater
and Linux GUI acceptance remain incomplete or unverified. AI tests use owned local
deterministic protocol fixtures only. The known glib/dependency security release
gate remains unresolved; this checkpoint did not rerun the advisory audit.

**Next P0:** replace the 64 MiB ingestion architecture with bounded streaming copy
and SHA-256, cancellable progress, disk-space checks, atomic publication and u64
storage. Preserve a separate bounded deep-parser budget. Current `reverse.rs`
`import_file` still reads/inspects the entire <=64 MiB input, `list_files` still
decodes size through u32, and `prepare_file_analysis` calls the bounded sample
reader. No 80/256/512 MiB or 1 GiB import pass is claimed. Do not merely enlarge
MAX_FILE_BYTES or pass raw file bytes through React/IPC.

### Changes and continuation

New Core `research_sessions.rs`, migration017, shared-context `workspace.tsx`,
ResearchPage and StartResearchButton. Updated App/navigation/target dialogs and
project-aware existing workspaces; existing Agent Workflows, chat, imports, MCP,
Job Engine, evidence and reporting remain available. Existing queue/cancel/HEAD
helpers now accept an enclosing transaction; no parallel scheduler was introduced.
New CLI `workspace` and `research` groups and matching Tauri operations. Architecture
and JSON review formats: `docs/AI_RESEARCH_SESSIONS.md`.

Regression failures fixed: schema upgrade fixtures initially retained new017 tables
while simulating old versions; fixtures now remove only their isolated newer layout,
without making production migrations permissive. Native restart initially depended
on browser-local session hints; restore now reads SQLite history. Async selection/list
responses are fenced; deleted/archived selections invalidate IDs. Quick Create Finding
remains usable. History status now follows actual jobs and planning context closes
after plan creation. Native first attempt's failed restart proof is retained under
`artifacts/research-session-1792487208696/results.json`. A subsequent fixture compile
failure used a helper module as a program; test now compiles an explicit benign main.
An early full-test invocation overlapped source edits and failed rustdoc; only the
final complete log counts above are accepted. All 69 staged Rust/SQL source hashes
match the Linux test snapshot.

Exact reproducible commands (use isolated data and explicit accepted binaries):

```powershell
$env:CARGO_INCREMENTAL='0'
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -j2 -- -D warnings
cargo test --locked --workspace -j2
cd apps/desktop
npm run build
npm test
cd ../..
cargo build --locked --release -p reconx -p reconx-desktop --features reconx-desktop/custom-protocol -j2
$env:RECONX_CLI_EXE='D:\ReconXAI\artifacts\windows-research-current-20261020\reconx.exe'
$env:RECONX_DESKTOP_EXE='D:\ReconXAI\artifacts\windows-research-current-20261020\reconx-desktop.exe'
$env:RECONX_VERIFY_ASSET='1'
node tests/e2e/research-session.mjs
node tests/e2e/http-replay.mjs
node tests/e2e/outputs.mjs
```

Desktop source asset changes require a fresh embedded-asset build; the native test
can assert the actual script path, preventing a stale executable from being accepted.
Build logs: recovery-release-build-final.log and recovery-desktop-current-assets.log.
Linux stage/run: `artifacts/stage-recovery-linux.py`, `artifacts/recovery-linux-run.sh`;
the latter uses the retained1.95 toolchain and native Linux temp workspace, excludes
Desktop and builds Core/CLI offline. CLI acceptance command:
`python scripts/research_session_smoke.py --binary <explicit binary> --sample <benign PE/ELF> --output <new isolated directory>`.
Each CLI call is a separate process; logs carry actual exit codes/timestamps. Never
execute the imported sample. `scripts/verify.ps1 -Research` and `npm run test:research`
provide reproducible native entry points.

Current work after this checkpoint: large-file import design/audit; no implementation
yet. First next edit must separate ingestion from `reconx-analysis::MAX_FILE_BYTES`,
not change the parser cap. Add cancellation and partial-file cleanup tests before
advertising large imports. Preserve existing databases and accepted binaries. Source
protection: `git status --short` before resuming; keep all uncommitted changes.

---

## MCP Registry / reviewed npm runtime checkpoint — 2026-10-19

Version **0.1.0**, schema **016**; **v1.0 NOT READY**. A bounded Registry + npm
installation slice is runtime verified. A subsequent cached-runtime selection fix
is currently rebuilding/retesting; the accepted binaries below remain preserved.
No commit/push/publication. Normal user data/credentials were not opened for tests.

Authoritative bounded proof: artifacts/mcp-current-verification.json. Windows pair:
artifacts/windows-mcp-candidate-1792415560; Linux CLI:
artifacts/linux-cli-mcp-1792415142/reconx. Windows binaries are actually NotSigned.
The OneDrive Desktop ReconX AI.lnk points to this tested Windows pair; record:
artifacts/desktop-shortcut-mcp.json. Earlier schema014 candidates are retained.

Implemented: actual official v0.1 Registry search/details/version listing, encoded
IDs/cursors, pagination, runtime filters, bounded HTTP/schema/retry policies and
SQLite offline/stale cache. Actual public Core CLI search(2/page), encoded detail,
version lookup, next cursor, offline and clear all pass (6 calls):
artifacts/registry-core-live-1792415328. Native Registry live/offline also passes:
artifacts/registry-native-1792415690918. Public metadata is untrusted; no package
execution is authorized by registry presence. Unknown OS/schema/install methods
stay explicit. Version-history UI currently displays one page.

Reviewed npm adapter reconx.mcp-everything installs official MCP Everything1/2
from compiled locks (108/94 tarballs), validates SHA-512, safely extracts without
npm scripts, reuses compatible Node>=24 or automatically installs pinned portable
Node24.22.0 with archive/executable SHA-256, checks real free space, and uses the
existing persistent Job Engine. Actual initialize/list_tools/schema health precedes
atomic activation. Generated MCP config has no secrets and starts with no allowed
tools. Desktop permission approval and exact-input invocation approval are separate.
Actual Echo output persists; no automatic tool calls or fabricated findings.
Everything is a protocol test server, not a scanner or Ghidra/Burp integration.

Schema015 adds bounded metadata cache;016 adds managed package descriptors and
installation operations/history. Previous configurations survive failed/cancelled
activation; user-edited configs fail reconciliation rather than being overwritten.
Enable/disable/remove, integrity checks, reviewed updates/rollback and shared-runtime
retention reuse Core state. Legacy YARA APIs explicitly reject managed MCP IDs.
CLI mcp install/doctor, catalog, tools dependencies and extension permissions share
the same backend. JSON null operations now emit machine-readable null.

Verified before the last cache-runtime fix: Windows Rust180/0/2 and Linux Core/CLI
180/0/2; warning-denied Clippy both hosts; rustfmt, TypeScript/Vite; frontend36/0.
76 Rust/SQL/manifest/lock source files match the Linux snapshot. Native current
Windows acceptance30 checks: managed MCP4 + Registry1 + preserved YARA5 + HTTP14 +
outputs6. Actual CLI lifecycle27 commands per host covers real package installation,
Echo, restart, disable/enable, health,2→1→2, file-tamper rejection, managed removal,
retained history and actual schema14→16 backup. Windows native separately forces
managed Node installation; Linux detects Node20 as incompatible and auto-installs24.
Native screenshots of all three themes and the installation dialog were viewed.
Frontend and both reviewed npm lock audits report zero vulnerabilities; historical
glib/maintenance backlog still blocks release (no new cargo audit claim).

Failures/fixes: Registry upstream timeouts reproduced with Rust/curl/Python; bounded
30s attempts and retries eventually passed actual API acceptance. Throttled excessive
per-chunk logs and corrected total-byte reporting. Windows shared-load15s MCP timeout
was real; removed duplicate tree hashing and set managed MCP60s bounded timeout,
then real CLI/native Echo passed. Native test corrected Connect/Discover capitalization.
Actual screenshot exposed missing dialog padding; fixed shared-body spacing/width.
Linux native filesystem testing avoids Windows-mounted-filesystem metadata overhead.
The last discovered issue was cached package receipts retaining a previously selected
Node; source now rebinds receipt to the exact approved verified runtime atomically.

Current unfinished task: finish acceptance of that cached-runtime fix. Running logs:
artifacts/mcp-runtime-fix-{check,test,release}.log and the log named by
artifacts/mcp-linux-current-log.txt. Once built, run
scripts/managed_mcp_failure_smoke.py --binary <new-cli> --data <isolated prior-smoke>
--approve-install on Windows and Linux; it tests actual nonzero-byte download
cancellation, exact managed runtime rebinding and failed-update preservation.
Then repeat native managed MCP acceptance using the new pair, update proof and
shortcut only after acceptance, and replace this in-progress paragraph.

Exact source files for this milestone: crates/reconx-core/src/{registry,managed_mcp,
extensions,jobs,mcp,lib}.rs; tests/managed_mcp.rs; crates/reconx-tools/src/{packages,
lib}.rs and Cargo.toml; Cargo.lock; crates/reconx-storage/src/lib.rs and migrations
015_mcp_registry.sql/016_managed_mcp.sql; apps/cli/src/main.rs;
apps/desktop/src-tauri/src/lib.rs; apps/desktop/src/{extensions.ts,extensions.css,
ManagedMcp.test.tsx,App.tsx,pages/ExtensionsPage.tsx,components/RegistryBrowser.tsx,
components/ManagedMcpDetails.tsx}; manifests/mcp/*; scripts/managed_mcp*.py;
schema16 fixture updates in scripts/{extension,http_schema}_smoke.py and
tests/e2e/extensions.mjs; new tests/e2e/{managed-mcp,registry}.mjs. Prior changes remain.

Commands: cargo fmt --all -- --check; cargo clippy --locked --workspace --all-targets
-j2 -- -D warnings; cargo test --locked --workspace -j2; cargo build --locked
--release -p reconx -p reconx-desktop --features reconx-desktop/custom-protocol -j2;
Linux --offline equivalents excluding Desktop; npm test/build/audit; npm audit
--package-lock-only for each reviewed package; scripts/managed_mcp_smoke.py;
tests/e2e/{managed-mcp,registry,extensions,http-workspace,outputs}.mjs. Exact artifacts
and hashes are in the proof. No fake pass is assigned to earlier failed iterations.

Remaining: generalized npm/PyPI/Cargo/OCI/MCPB installers and runtime constraints,
more reviewed integration adapters, large/paginated MCP catalogs/schema refs,
remote authentication/OAuth, OS sandbox, disk-full/permissions/crash recovery
acceptance, first-run profile wizard, installer lifecycle, Linux GUI/Secret Service,
signed updater/signing, billing/OAuth/local model runtimes and original research
backlog. No universal one-click installation or Linux Desktop claim.

## UX and extension test results — 2026-10-19

Active priority: user product milestones A-H. A's bounded Windows UX and B's
first-party YARA-X installation/invocation slice are accepted. Full A cross-platform
visual acceptance and broader B/C marketplace/MCP setup remain PARTIAL. Preserve
all earlier schema/security work. Version remains **0.1.0**, schema **014**;
**v1.0 is NOT READY**. No commit, push, installer publication or release.

Authoritative current proof: artifacts/ux-extensions-current-verification.json.
Windows portable pair: D:\ReconXAI\artifacts\windows-extensions-verified-20261019-172615.
Linux optimized CLI: artifacts/linux-cli-extensions/reconx. Actual hashes/signature
status are beside the binaries. Windows Desktop and CLI are actually NotSigned.
The OneDrive Desktop ReconX AI.lnk now points to the new accepted Windows pair;
artifacts/desktop-shortcut-extensions.json verifies its target and binary hash.
The normal user workspace/credentials were not opened or migrated by this work.

Implemented UX: shared 30/34/38 px controls, stable Button DOM and disabled reasons;
actual Lucide icons; preserved Segoe UI Variable/Source Sans 3/IBM Plex Mono;
exactly one rail Appearance menu (Light/Dark/ReconX), informational Settings;
48px rail, collapsible navigation and optional activity drawer; real compact
project/jobs/audit dashboard, useful recon empty state, direct provider navigation,
collapsed provider catalog and build-derived version. Native visual inspection
fixed ReconX token specificity, inconsistent default HTTP project binding,
provider/reverse density and extension detail/install-dialog padding/alignment.
No original user-referenced screenshot files were attached, so no comparison with
those missing files is claimed. Actual baseline: artifacts/ux-before-1792391426265.
Current native matrix: artifacts/ux-after-1792405586148: **120 captures**, ten pages ×
three themes × four sizes (1280×720/1366×768/1440×900/1920×1080). Screenshots were
actually viewed. Extra native checks cover rail width, disabled explanations,
one theme menu, panel/navigation collapse, keyboard/persistence/restart and HTTP
default project binding. 125/150/200% are CDP raster scaling, not physical OS DPI.
Reviewable before/after viewer: artifacts/ux-comparison.html.

Implemented bounded Marketplace: one compiled YARA-X native adapter; strict
versioned manifest, real dependency/source/hash inspection, exact approved plan,
portable dependency installation/reuse, actual version and benign signature health
check before Ready, safe failure state, enable/disable/remove, shared-tool retention,
input-file selection, real Job Engine execution/evidence and persistent history.
CLI and Desktop share the same Rust methods/state. Migration014 adds registrations,
tool requirements and atomic invocation permission bindings. Workers recheck the
enabled revision at checkpoints and final commit. Disable/removal cancels queued
or running bound work. Uninstall preserves tools, artifacts and history; tools
needed by any installed registration cannot be removed. Native runtime directly
tests queued cancellation; concurrent running-extension revocation is not separately
claimed as runtime verified. No arbitrary native library/plugin or shell execution.

Verified: Windows Rust **171 passed/0 failed/1 ignored**, Linux Core/CLI **171/0/1**;
extension tests3/storage12; frontend **33/0**; TypeScript/Vite, rustfmt and warning-
denied Clippy on Windows and Linux Core/CLI. Actual source hashes match for 64 Rust,
SQL and manifest files in the Linux snapshot: artifacts/extensions-source-hashes.json.
Optimized Windows pair and Linux CLI built. Final native extension acceptance:
artifacts/extensions-1792405575406 (5 checkpoints); fresh final HTTP/schema14 and outputs:
20 checkpoints. Current binary's UX matrix120 passes. Earlier same-checkpoint
existing suites pass77; schema14 tools/MCP/AI-plan6 and HTTP replay11 also pass.
Provider inference fixtures are not live account verification. Actual CLI extension
lifecycle: **28 commands per host**, real pinned YARA-X, evidence hashes, integrity,
disable/enable, rejected stale/unapproved plans, retained tools/history and reinstall.
Both hosts use the actual schema13 binary to create queued work, then verify the
intact pre-v14 backup and successful migrated execution. Database integrity and
foreign-key checks pass. npm audit: zero vulnerabilities. Cargo.lock is unchanged;
the previously verified glib unsoundness plus maintenance backlog still blocks v1.

Commands/logs: npm test, npm run build, npm audit; cargo fmt --all -- --check;
cargo clippy --locked --workspace --all-targets -j2 -- -D warnings; cargo test
--locked --workspace -j2; cargo build --locked --release -p reconx -p reconx-desktop
--features reconx-desktop/custom-protocol -j2; Linux offline equivalents excluding
Desktop; scripts/extension_smoke.py --legacy-binary <schema13> --approve-install;
tests/e2e/extensions.mjs, ux.mjs, http-workspace.mjs, outputs.mjs and existing native
suites. Final logs: artifacts/extensions-verified-tests.log,
extensions-linux-final-tests.log, extensions-final-clippy.log,
extensions-dialog-refined-tests.log, extensions-dialog-refined-frontend-build.log,
extensions-final-native-build.log, extensions-final-native-*.log and
extensions-final-regression-*.log. Final YAML workflow parsing and py_compile pass.

Retained failures: early Button conditional wrapper lost pointer/ref continuity
(19 frontend pass/7 fail), fixed by a stable native button and regression test;
extension UI DOM shim/API/Send compile errors fixed before acceptance; native
reinstall test held a locator from the prior WebView, fixed and rerun. Native
screenshots caught/fixed actual padding defects. Linux staging first omitted
fixture/workflow files, then shared Cargo cache retained an old compile-time
CARGO_MANIFEST_DIR; staged complete source and touched workspace source to force
recompile. Combined Windows WSL stdout/stderr handles overwrote log segments;
final test logging is redirected inside Linux, with complete171/0/1 results.
No guard was relaxed to make any test pass.


---
## Current conditional-schema checkpoint - 2026-10-19

Bounded OpenAPI 3.1 if/then/else now uses real captured predicates, selected-branch
provenance and the existing shared budgets/format policy. Windows and Parrot/WSL
Core+CLI Rust: 167/0/1 each; schema 41/storage 11; frontend 26; native HTTP/schema
and outputs: 20 fresh checkpoints. Optimized binaries, real schema-12 upgrades,
evidence/four-format reports, PDF inspection and restart pass on both CLI hosts.
Proof: artifacts/schema-conditional-current-verification.json; exact commands,
files, candidates and next multi-file-import action: MASTER_EXECUTION_PROGRESS.md.
No new migration; schema 013/version 0.1.0. v1.0 NOT READY: glib, Linux GUI,
installer/signing, full schema coverage and unfinished research modules remain.
No commit, push or release publication.

---

## Current opt-in format checkpoint - 2026-10-19

Default-off per-job syntactic formats, durable fingerprint policy and additive
schema-013 migration are runtime-verified. Windows and Parrot/WSL Core+CLI Rust:
163/0/1 each; schema 37/storage 11; frontend 26; native HTTP/schema+outputs: 19
fresh checkpoints. Real old-binary queued-job upgrades, consistent backups,
evidence, four report formats, independent PDF parsing and restart pass on both
hosts. Proof: artifacts/schema-format-current-verification.json; exact commands,
files, candidates, limitations and next action: MASTER_EXECUTION_PROGRESS.md.
Version 0.1.0; v1.0 NOT READY. glib/maintenance, Linux GUI, installer/signing and
unfinished modules remain unresolved. No commit, push or release publication.

---

## Current offline regex checkpoint - 2026-10-19

Bounded pattern and 3.1 patternProperties now run through shared jobs, evidence,
Desktop and CLI. Specialized/unsupported regex syntax and limits stay explicit.
Final Windows and Parrot/WSL Core-CLI Rust: 157/0/1 each; frontend 26; native
HTTP/schema+outputs: 18 fresh checkpoints; optimized CLI/evidence/four-format
reports pass on both hosts. Exact scope/failures/commands/next action:
MASTER_EXECUTION_PROGRESS.md and artifacts/schema-pattern-current-verification.json.
No migration; schema 012/version 0.1.0. v1.0 remains NOT READY. Next: explicit,
default-off durable format assertion policy; Linux GUI/installers/signing and
glib/maintenance review remain unresolved. No push or release publication.

---

## Current checkpoint: bounded offline tuples, uniqueness and dependencies - 2026-10-18

Phase 1.5 advanced safely through the existing shared Core/Job Engine/Desktop/CLI.
uniqueItems now uses numeric/structural equality; OpenAPI 3.1 prefixItems/items
and dependentRequired/dependentSchemas retain exact source locations. Redacted,
invalid, cyclic and budget-limited cases remain explicit/inconclusive. No network
resolver, external replay, automatic finding verification or migration was added.
The schema remains 012, version 0.1.0, and v1.0 is NOT READY.

Current proof: `artifacts/schema-arrays-current-verification.json`. Windows Rust
150 passed/0 failed/1 ignored; Parrot/WSL Core+CLI 150/0/1; frontend 26 passed;
format, warning-denied Clippy, TypeScript/Vite and optimized Windows Desktop/CLI
and Linux CLI builds pass. Fresh native HTTP/schema, output and replay acceptance
passes 29 checkpoints across three suites. Other suites retain their earlier
historical proof and were not reclassified as freshly rerun. Native screenshots
of actual uniqueness/tuple observations were inspected. Separate optimized CLI
processes on Windows and Linux verified jobs, source evidence, persistence and
all four exports; pypdf 6.19.0 independently parsed both five-page PDFs/redaction.
Linux Desktop/Secret Service/packages and Windows installer lifecycle/signing
remain unverified; this checkpoint produced portable binaries, no new installer.

Fresh cargo-audit still reports one glib unsound warning and six maintenance
warnings; npm audit reports zero advisories. The explicit deny-unsound check exits
1 and dependency readiness stays BLOCKED. Earlier GitHub alert export is retained
as historical input, not a new remote verification. No advisory was suppressed.

Current unfinished task: complete remaining safe offline schema coverage
(patterns/patternProperties, formats with explicit assertion policy, resource/
multi-file resolution) with bounded CPU/memory and original evidence provenance.
Next exact action: inspect the existing JSON Schema library's regex limits and
dialect semantics, then implement the next bounded offline validator slice with
positive/negative/redaction/budget cases. Broader Web/API, recon, AI, tool/MCP,
reverse/mobile/cloud/network and release requirements remain PARTIAL/NOT_STARTED
as recorded in module-status.json and RELEASE_READINESS.md.

Exact changed files, commands, retained failures and next action: MASTER_EXECUTION_PROGRESS.md.

---

## Current followup: CI fixture reliability and complete project DOCX — 2026-10-18

Preserved the tested production source, schema 012, user DBs, credentials, typography
and exactly Light/Dark/ReconX. No new production adapter or migration. v1.0 is NOT
COMPLETE; glib remains a blocking unsound advisory. Eight High pypdf alerts remain
confirmed fixed; no warning or policy has been suppressed.

Actual GitHub CI for c22f44c: CodeQL Advanced succeeded (run 39125535750), but
verification failed (39125535775). Bounded downloaded job logs reproduced two test
assumptions: Linux keyless-provider removal correctly fails closed without Secret
Service (optional headers may still exist), and a Windows MCP cancellation test
reused a 500ms timeout and could terminate before the call started. CLI regression
now asserts retained configuration/default/routing on SECRET_STORE_UNAVAILABLE,
while native Windows still covers successful OS-backed deletion. Cancellation
gets a separate long-running benign echo budget and waits for actual progress,
failing immediately with the job state if terminal early. Production timeout,
permissions and credential-deletion behavior are unchanged.

Fresh tests: Windows Rust 143 passed/0 failed/1 ignored; Parrot/WSL Core+CLI
143/0/1 with an explicitly absent DBUS session bus; format, warning-denied Clippy,
26 frontend tests and TypeScript/Vite pass. DOCX generator tests 3 pass and
dependency-readiness tests 4 pass. Current evidence:
artifacts/documentation-current-verification.json, documentation-windows-rust.log,
documentation-linux-{stdout,stderr}.log, documentation-clippy.log and
documentation-frontend-{tests,build}.log. Earlier failed document-format checks
are retained separately; subsequent final structural and Word rendering checks pass.
Native 9 suites/77 checkpoints and optimized binaries remain the unchanged
production pair in artifacts/security-current-verification.json; they were not
unnecessarily rebuilt/retested for test/docs-only changes. Remote CI fixes require
the subsequent source revision's actual run; the glib gate remains expected red.

Delivered ReconX_AI_Tai_lieu_du_an_day_du.docx in the user's requested Desktop/report
directory: 64 pages, populated TOC/page numbers, 25 chapters plus source-derived
provider/migration/module/CLI appendices. Every text style/run including table,
heading, technical examples, header/footer and TOC is Times New Roman/black.
Microsoft Word pagination and local PDF inspection verified only TimesNewRoman
font variants, all text color 0, Vietnamese, no replacement glyphs, blank pages or
page-edge overflow. Internal PDF/screenshots are verification artifacts, not a
substitute for the requested editable DOCX. Proof: project-document-delivery.json,
project-document-word-validation.json, project-document-pdf-inspection.json under
artifacts/. No user data or credentials embedded; no cloud document service used.

Reusable source: docs/PROJECT_DOCUMENTATION.vi.md and scripts/build_project_document.py;
optional dependency scripts/documentation-requirements.txt; tests in
scripts/test_project_document.py. The generator refuses an existing destination,
uses no remote template/network/normal DB and invokes an optional CLI only with
--help. README records generation. Existing Word files were not overwritten.

Next: inspect the new remote CI run after authorized push; keep glib/GTK and Linux
GUI/Secret Service/clean installers blocked until independently resolved/tested.
Continue the safe offline/data-integrity backlog, not placeholder modules or
claims of general external replay/active authorization/scanner automation. Latest
source push proof remains artifacts/source-push-result.json. No force-push,
release publication, privileged install or user-workspace migration performed.

---

## Verified remote-security / CI / Linux CLI followup — 2026-10-18

GitHub confirmed all eight pypdf High alerts (#2-#9) FIXED after authorized push
34421a8; actual fresh export: artifacts/dependabot-after-second-check.json. Glib
Medium #1 remains OPEN and the dependency gate remains BLOCKED; no advisory was
dismissed. Six RustSec maintenance warnings remain. Full v1.0 is NOT COMPLETE.

Remote CI initially rejected a pre-existing unquoted --only-binary=:all: pip run
line as invalid YAML. A new Rust workflow-parser regression reproduced the exact
line-103 error, then passed with a folded run string. Both shipped YAML workflows
and string-valued run steps are checked. Before/fixed logs: security-workflow-
before-fix.log, security-workflow-final.log and security-workflow-linux-final.log
under artifacts/. This distinguishes syntax acceptance from successful CI jobs;
the glib gate is still expected to fail. Remote full CI is not claimed successful.

Latest regression: 143 Rust tests passed on Windows and 143 on Parrot/WSL Core+CLI,
one ignored each; frontend 26 and native 9 suites/77 checkpoints remain verified.
The followup only changes CI/test/scripts/docs, not production Rust/frontend logic;
the same final Windows binaries/installer remain valid. Current aggregate proof:
artifacts/security-current-verification.json. Exact latest logs:
artifacts/security-ci-followup-{rust,clippy,linux-stdout,linux-stderr}.log.

New optimized Linux CLI built offline and executed from artifacts/linux-cli-security-20261018-185625/reconx.
Actual separate CLI processes imported /usr/bin/true, independently matched SHA-256,
parsed Elf/X86_64 (29 sections, 581 strings), completed/persisted a real analysis
job, verified evidence, created an explicitly synthetic Needs Verification record,
and exported Markdown/JSON/HTML/PDF from a reviewed snapshot. Restart/process
persistence checks passed. Independent pypdf 6.19.0 inspection of the Linux-generated
3-page PDF confirmed Vietnamese text/redaction. Evidence: artifacts/linux-cli-security-20261018-185625/smoke/acceptance.json,
pdf-inspection.json, build-manifest.json, compiler.txt and SHA256SUMS.txt. Linux
Desktop, Secret Service, packages and other distributions remain UNVERIFIED.

New reusable command: python3 scripts/linux_release_smoke.py --binary PATH --output
NEW_DIRECTORY. It refuses existing output, retains its isolated test workspace,
uses a benign system ELF without executing it, and never calls a network/AI
service or credential manager. CI includes this acceptance and report artifacts.
No migration added; schema remains 012. Exact next task remains compatible glib
remediation/isolated Linux GUI prerequisites, then remaining safe offline/product
requirements. Source commit/push uses the user's existing authorization; no release
publication, force-push, normal user DB/key changes or privileged package installs.

---

## Current security/schema verification — 2026-10-18

Functional verification passes; dependency/release acceptance remains BLOCKED.
All commands used isolated test databases and local fixtures; no personal cloud
credentials, public target scans or normal user data were used.

| Command/check | Result | Evidence |
|---|---|---|
| cargo fmt --all -- --check | pass | final local check |
| cargo clippy --locked --workspace --all-targets -j2 -- -D warnings | pass | artifacts/security-final-clippy.log |
| cargo test --locked --workspace -j2 | 142 passed, 1 ignored | artifacts/security-final-rust.log |
| Linux cargo test --locked --offline --workspace --exclude reconx-desktop -j2 | 142 passed, 1 ignored | artifacts/security-final-linux-stdout.log |
| npm run build / npm test | pass / 26 passed | artifacts/security-expanded-frontend-*.log |
| python -m unittest discover -s scripts -p test_dependency_readiness.py -v | 4 passed | artifacts/security-readiness-tests.log |
| npm audit --json --registry https://registry.npmjs.org | 0 findings, exit 0 | artifacts/security-npm-before.json |
| cargo audit --deny unsound --json | **BLOCKED, exit 1**: glib; 6 maintenance warnings | artifacts/security-rust-gate.json |
| CLI future/corrupt database rejection | 2 actual CLI cases; original bytes unchanged | artifacts/security-db-preflight.json |
| Optimized shared CLI/Desktop build | pass | artifacts/security-release-build.log |
| npm run tauri -- build --config ../../packaging/tauri.windows.json | NSIS built; NotSigned; clean installation unverified | artifacts/security-tauri-bundle.log |
| Native http-replay | 11 passed | artifacts/http-replay-1792323486749/results.json |
| Native http-workspace | 12 passed | artifacts/http-workspace-1792323498002/results.json |
| Native native | 8 passed | artifacts/native-1792323515422/results.json |
| Native inputs | 9 passed | artifacts/inputs-1792323558147/results.json |
| Native jobs | 8 passed | artifacts/jobs-1792323568765/results.json |
| Native integrations | 6 passed | artifacts/integrations-1792323595665/results.json |
| Native outputs | 6 passed | artifacts/outputs-1792323613641/results.json |
| Native recon-local | 7 passed | artifacts/recon-local-1792323626517/results.json |
| Native ai-providers | 10 passed | artifacts/native-ai-providers-1792323638564/results.json |
| Independent PDF extraction/redaction | pypdf 6.19.0, passed | artifacts/http-replay-1792323486749/pdf-inspection.json |
| Negative HTTPS certificate rejection | Windows + Linux CLI, TCP observed and zero HTTP requests | artifacts/security-tls-windows.log; security-tls-linux.log |

Nine native suites: 77 checkpoints. Final matched binaries and installer are in
artifacts/windows-security-20261018-183806; hashes/source fingerprints: artifacts/security-current-verification.json.
New schema runtime evidence includes allOf/minProperties with exact OpenAPI pointer
and preserved evidence. All three themes and shared CLI persistence still pass.
The retained failed first cancellation run is explained in the master ledger;
it is not counted as a pass. Linux GUI probe: artifacts/security-linux-desktop-probe.json.
Artifact log pattern scan: 1,794 files, no matches in the documented limited scope.
No comprehensive secret audit, trusted-CA HTTPS success, clean installer, live
cloud-provider or Linux Desktop claim is made. Dependency resolution details and
remaining glib risk: DEPENDENCY_SECURITY.md.

---

# Current manual HTTP and secure identity verification - 2026-10-18

This verifies a bounded manual loopback fixture slice plus existing product regressions, not autonomous authorization testing or all v1.0. Tests used isolated synthetic workspaces/credentials and owned local fixtures; normal user data and cloud providers were not used. Exact binaries: artifacts/windows-replay-final-20261018-160538. Source/binary hashes and machine-readable proof: artifacts/replay-current-verification.json.

| Command/check | Actual result | Record |
|---|---|---|
| cargo fmt --all -- --check | pass | artifacts/replay-current-format.log |
| cargo clippy --locked --workspace --all-targets -j2 -- -D warnings | pass | artifacts/replay-current-clippy.log |
| cargo test --locked --workspace -j2 | 133 passed, 0 failed, 1 ignored | artifacts/replay-current-rust.log |
| Linux cargo test --locked --offline --workspace --exclude reconx-desktop -j2 | 133 passed, 0 failed, 1 ignored | artifacts/replay-linux-current-stdout.log; separate stderr log |
| npm run build (TypeScript and Vite) | pass | artifacts/replay-current-frontend-build.log |
| npm test | 26 passed, 0 failed | artifacts/replay-current-frontend-tests.log |
| npm run tauri -- build --no-bundle | pass; subsequent custom-protocol build includes final guard/target preservation | artifacts/replay-verified-tauri-build.log |
| cargo build --locked --release -p reconx -p reconx-desktop --features reconx-desktop/custom-protocol -j2 | pass, final matched binary pair | artifacts/replay-current-release-build.log |
| Native http-replay | 11 passed | artifacts/http-replay-1792314338414/results.json |
| Native http-workspace | 12 passed | artifacts/http-workspace-1792314349444/results.json |
| Native native | 8 passed | artifacts/native-1792314366857/results.json |
| Native inputs | 9 passed | artifacts/inputs-1792314409622/results.json |
| Native jobs | 8 passed | artifacts/jobs-1792314420262/results.json |
| Native integrations | 6 passed | artifacts/integrations-1792314447326/results.json |
| Native outputs | 6 passed | artifacts/outputs-1792314471536/results.json |
| Native recon-local | 7 passed | artifacts/recon-local-1792314487988/results.json |
| Native ai-providers | 10 passed | artifacts/native-ai-providers-1792314502191/results.json |

Nine native suites total 77 checkpoints, all passed. Actual new runtime proof includes native HAR picker, no import/registration autosend, masked OS-backed Alice/Bob keys, actual POST/GET and identity-dependent output, credential-echo redaction, job logs/history, CLI approval handled by Desktop worker, external redirect not followed, real pending cancellation, restart reuse without secret reentry, four report formats and all three theme dialogs at 1920/1440/1366. Existing target/file/job/MCP/mock-AI/provider/report workflows also pass. The ignored generic OS-store unit test remains ignored by default; new Windows native credential tests genuinely use the OS store.

Separate negative TLS acceptance also passed on native Windows CLI and Linux Core/CLI: artifacts/replay-tls-windows.log and artifacts/replay-tls-linux.log. Each contacted the owned local TLS fixture, rejected its self-signed certificate with HTTP_CONNECT, and sent zero HTTP requests. No trust store was changed; ephemeral key/certificate files were removed. Trusted-CA HTTPS success is not established. The new target-deletion acceptance preserves history and disables detached identities.

Independent PDF inspection: artifacts/http-replay-1792314338414/pdf-inspection.json. Report text decoded by pypdf; tested credential markers absent from PDF/JSON/HTML/Markdown and default traffic export, and ordinary DB/WAL. Final screenshots in the HTTP replay result directory were opened and inspected, including request review, actual redacted response and three themes. Dialog spacing/width changed only within the two new forms. No live cloud API or arbitrary public-target testing was performed. Linux Core/CLI is not Linux Desktop, HTTPS fixture or Secret Service runtime verification.

New core tests cover actual network execution, five credential methods, echo redaction, secure rotation/expiry/deletion and deferred cleanup, stale approvals, project/origin isolation, forbidden destinations/headers/proxies/redirect policies, oversized declared/chunked responses, timeout and cancellation. Tests also check no plaintext keys in SQLite and reject selected credentials in public payloads. Migration011 history survives012 with backup/FK checks. Frontend tests reject extra returned metadata in save payloads and stale choices during editor reopen.

Retained failed attempts: artifacts/replay-full-rust-first.log (accepted Windows fixture sockets now explicitly blocking); the two early native results http-replay-1792312481422 and http-replay-1792312573558 (uncaptured review failure/stale selector risk, then empty successful CLI output parsed as JSON). The selector now waits for fresh revisions with regression coverage; the runner accepts empty mutation output. Do not assert that the first uncaptured error's exact cause was proven. Failed Linux staging logs remain under replay-linux/replay-linux-final; current stage includes MCP fixtures and recompiles the embedded manifest path correctly. Synthetic OS keys from failed native fixtures were removed separately; user keys were untouched. Earlier passing native run before the final loading guard is retained separately from the final matched run.

Remaining unverified/incomplete: automatic BOLA/BFLA/stateful testing, general remote replay, proxy/followed redirects, multipart/binary upload, full JSON Schema, trusted-CA HTTPS success and abrupt HTTP termination fixture, live providers, Linux Desktop/Secret Service, installer/signing and full release acceptance. No such feature is promoted based on compilation.

---

Older checkpoints below are retained for provenance; the current checkpoint above takes precedence.

# Current manual HTTP and secure identity verification - 2026-10-18

This verifies a bounded manual loopback fixture slice plus existing product regressions, not autonomous authorization testing or all v1.0. Tests used isolated synthetic workspaces/credentials and owned local fixtures; normal user data and cloud providers were not used. Exact binaries: artifacts/windows-replay-final-20261018-155056. Source/binary hashes and machine-readable proof: artifacts/replay-current-verification.json.

| Command/check | Actual result | Record |
|---|---|---|
| cargo fmt --all -- --check | pass | artifacts/replay-current-format.log |
| cargo clippy --locked --workspace --all-targets -j2 -- -D warnings | pass | artifacts/replay-current-clippy.log |
| cargo test --locked --workspace -j2 | 133 passed, 0 failed, 1 ignored | artifacts/replay-current-rust.log |
| Linux cargo test --locked --offline --workspace --exclude reconx-desktop -j2 | 133 passed, 0 failed, 1 ignored | artifacts/replay-linux-current-stdout.log; separate stderr log |
| npm run build (TypeScript and Vite) | pass | artifacts/replay-current-frontend-build.log |
| npm test | 26 passed, 0 failed | artifacts/replay-current-frontend-tests.log |
| npm run tauri -- build --no-bundle | pass; subsequent custom-protocol build includes final guard/target preservation | artifacts/replay-verified-tauri-build.log |
| cargo build --locked --release -p reconx -p reconx-desktop --features reconx-desktop/custom-protocol -j2 | pass, final matched binary pair | artifacts/replay-current-release-build.log |
| Native http-replay | 10 passed | artifacts/http-replay-1792313456492/results.json |
| Native http-workspace | 12 passed | artifacts/http-workspace-1792313466894/results.json |
| Native native | 8 passed | artifacts/native-1792313484353/results.json |
| Native inputs | 9 passed | artifacts/inputs-1792313527088/results.json |
| Native jobs | 8 passed | artifacts/jobs-1792313537586/results.json |
| Native integrations | 6 passed | artifacts/integrations-1792313564297/results.json |
| Native outputs | 6 passed | artifacts/outputs-1792313581730/results.json |
| Native recon-local | 7 passed | artifacts/recon-local-1792313594680/results.json |
| Native ai-providers | 10 passed | artifacts/native-ai-providers-1792313606719/results.json |

Nine native suites total 76 checkpoints, all passed. Actual new runtime proof includes native HAR picker, no import/registration autosend, masked OS-backed Alice/Bob keys, actual POST/GET and identity-dependent output, credential-echo redaction, job logs/history, CLI approval handled by Desktop worker, external redirect not followed, real pending cancellation, restart reuse without secret reentry, four report formats and all three theme dialogs at 1920/1440/1366. Existing target/file/job/MCP/mock-AI/provider/report workflows also pass. The ignored generic OS-store unit test remains ignored by default; new Windows native credential tests genuinely use the OS store.

Independent PDF inspection: artifacts/http-replay-1792313456492/pdf-inspection.json. Report text decoded by pypdf; tested credential markers absent from PDF/JSON/HTML/Markdown and default traffic export, and ordinary DB/WAL. Final screenshots in the HTTP replay result directory were opened and inspected, including request review, actual redacted response and three themes. Dialog spacing/width changed only within the two new forms. No live cloud API or arbitrary public-target testing was performed. Linux Core/CLI is not Linux Desktop, HTTPS fixture or Secret Service runtime verification.

New core tests cover actual network execution, five credential methods, echo redaction, secure rotation/expiry/deletion and deferred cleanup, stale approvals, project/origin isolation, forbidden destinations/headers/proxies/redirect policies, oversized declared/chunked responses, timeout and cancellation. Tests also check no plaintext keys in SQLite and reject selected credentials in public payloads. Migration011 history survives012 with backup/FK checks. Frontend tests reject extra returned metadata in save payloads and stale choices during editor reopen.

Retained failed attempts: artifacts/replay-full-rust-first.log (accepted Windows fixture sockets now explicitly blocking); the two early native results http-replay-1792312481422 and http-replay-1792312573558 (uncaptured review failure/stale selector risk, then empty successful CLI output parsed as JSON). The selector now waits for fresh revisions with regression coverage; the runner accepts empty mutation output. Do not assert that the first uncaptured error's exact cause was proven. Failed Linux staging logs remain under replay-linux/replay-linux-final; current stage includes MCP fixtures and recompiles the embedded manifest path correctly. Synthetic OS keys from failed native fixtures were removed separately; user keys were untouched. Earlier passing native run before the final loading guard is retained separately from the final matched run.

Remaining unverified/incomplete: automatic BOLA/BFLA/stateful testing, general remote replay, proxy/followed redirects, multipart/binary upload, full JSON Schema, dedicated HTTPS and abrupt HTTP termination fixture, live providers, Linux Desktop/Secret Service, installer/signing and full release acceptance. No such feature is promoted based on compilation.

---

Older checkpoints below are retained for provenance; the current checkpoint above takes precedence.

# Current offline schema verification - 2026-10-18

Preserved source at HEAD fe76e79, product 0.1.0. This verifies a bounded offline
schema observation slice; it does not verify active Web/API testing or all v1.0.
Exact immutable Desktop/CLI: artifacts/windows-schema-20261018-113252.
Source hashes, binary hashes, native paths, PDF inspection and logs are recorded in
artifacts/schema-current-verification.json. No normal user database or real cloud
account was used; no imported request was replayed.

| Command/check | Actual result | Log |
| --- | --- | --- |
| Baseline cargo test --locked --workspace -j2 | 112 passed, 0 failed, 1 ignored | artifacts/schema-baseline-rust.log |
| Baseline npm test | 22 passed | artifacts/schema-baseline-frontend.log |
| cargo fmt --all -- --check | exit 0 | artifacts/schema-verified-format.log |
| cargo clippy --locked --workspace --all-targets -j2 -- -D warnings | exit 0 | artifacts/schema-verified-clippy.log |
| cargo test --locked --workspace -j2 | 124 passed, 0 failed, 1 ignored | artifacts/schema-current-rust.log |
| npm run build; npm test | TypeScript/Vite passed; 23 frontend tests passed | artifacts/schema-final-frontend.log |
| npm run tauri -- build --no-bundle | optimized Windows Desktop built | artifacts/schema-final-tauri.log |
| cargo build --locked --release -p reconx -j2 | optimized Windows CLI built | artifacts/schema-verified-cli.log |
| Linux cargo test --locked --offline --workspace --exclude reconx-desktop -j2 | 124 passed, 0 failed, 1 ignored | artifacts/schema-current-linux.log; separate stderr |

The ignored case is the explicit OS-secret-store fixture. Actual native provider
acceptance separately used synthetic Credential Manager entries. Current Linux
verification is Parrot/WSL Core+CLI; Linux Desktop/Secret Service runtime, installers,
signing and live cloud interoperability remain UNVERIFIED.

## Windows native execution

All eight suites ran sequentially against the exact binaries above. The HTTP visual
follow-up uses those same binaries; its earlier passing record is retained rather
than double counted. Total **66 checkpoints**, zero failures in these final runs.

| Suite | Checkpoints | Actual result record |
| --- | --- | --- |
| http-workspace | 12 passed | artifacts/http-workspace-1792298219547/results.json |
| native | 8 passed | artifacts/native-1792298075709/results.json |
| inputs | 9 passed | artifacts/inputs-1792298119022/results.json |
| jobs | 8 passed | artifacts/jobs-1792298129766/results.json |
| integrations | 6 passed | artifacts/integrations-1792298156667/results.json |
| outputs | 6 passed | artifacts/outputs-1792298174808/results.json |
| recon-local | 7 passed | artifacts/recon-local-1792298187711/results.json |
| ai-providers | 10 passed | artifacts/native-ai-providers-1792298199307/results.json |

HTTP coverage: native picker HAR/JSON/YAML import, exact target/origin/schema pairing,
real existing-worker jobs, explicit partial coverage, intentional JSON type/required
mismatches, zero-network results, source artifact integrity/pointers, no automatic
vulnerability creation, evidence, reviewed four-format reports, restart and shared
CLI state. The remaining suites rechecked projects/targets, PE import/static
analysis, cancellation/timeout/recovery, synthetic secure AI settings, real benign
MCP calls/local AI plans, managed tool lifecycle and approved loopback HEAD inventory.
No general scanner or active authorization-matrix acceptance is claimed.

## Independent visual/output inspection

Actual WebView2 schema-detail images were viewed in artifacts/http-workspace-1792298219547:
schema-reconx-1920.png, schema-light-1440.png, schema-dark-1366.png.
The suite captures the schema card in all three themes at 1920x1080, 1440x900 and
1366x768. Provenance wraps at narrower widths; document overflow assertions pass.
Existing layout/fonts/colors remain intact. Narrow panels still scroll vertically;
no new high-DPI or Linux GUI claim is made.

Independent pypdf parsing read the actual six-page report, found the schema job
evidence and none of seven synthetic credential markers; pdf-inspection.json is
beside the report. All four existing report exports were executed with reviewed
snapshot hashes. Limited known GitHub-token/private-key patterns were checked in
the exact executables; schema-binary-pattern-review.json states that limited scope.
This is not a full security or secret audit.

## Retained failures and fixes

- First new schema tests: duplicate HAR entries made helper ordering select the
  untouched entry. Each validation test now has one distinct entry; no production
  checks were weakened. Raw schema-core-first-tests.log is retained.
- Initial new frontend result guard treated unknown JSON as a ReactNode; an explicit
  null check corrected it. Clippy caught an array/Vec test mismatch; corrected.
- The new CLI test initially used jobs run <id>, but the existing command drains
  the queue with jobs run. An old migration test expected 10 after migration 011;
  it now expects 11 while still checking retained project/settings/provider data.
  Earlier schema-final-rust.log/schema-verified-rust.log retain these failures.
- WSL inline shell arguments expanded incorrectly; the same isolated toolchain
  worked when invoked through an LF script file. Streams remain separate.
- Native artifacts/http-workspace-1792297973145/results.json expected fields from
  the deliberately minimal YAML schema instead of the JSON schema. The backend
  correctly reported no such mismatch. The test selects the explicit JSON operation
  and final native runs pass. This failed result/screenshot is retained.

## Remaining acceptance

The overall Web/API module is PARTIAL. Full JSON Schema, serialized parameter-value
validation, general relative/multi-file references, request replay, secure active
identities, stateful BOLA/BFLA research and scanner orchestration are not implemented
by this slice. Schema observations never establish authorization or a vulnerability.
Migration 011 snapshots older schemas with pre-v11 backups and reuses jobs/evidence;
only isolated acceptance databases were upgraded. Stop older clients before any
real workspace upgrade. See HTTP_SCHEMA_ANALYSIS.md for exact limits and commands.

---

Older checkpoint sections below are retained for provenance. This checkpoint takes precedence.

# Current Web/API verification - 2026-10-18

Current source is the preserved working tree on HEAD 70bc08d; product 0.1.0. This is an offline Web/API acceptance milestone, not full v1.0. All native suites below used the exact optimized binaries in `artifacts/windows-web-api-20261018-105742`; hashes and source fingerprints are in `artifacts/http-current-verification.json`.

| Command/check | Actual result | Record |
| --- | --- | --- |
| cargo fmt --all -- --check | exit 0 | artifacts/http-verified-format.log |
| cargo clippy --locked --workspace --all-targets -j2 -- -D warnings | exit 0 | artifacts/http-verified-clippy.log |
| cargo test --locked --workspace -j2 | 112 passed, 0 failed, 1 ignored | artifacts/http-verified-rust.log |
| npm run build / Tauri frontend prebuild | TypeScript/Vite passed | artifacts/http-verified-tauri.log |
| npm test | 22 passed, 0 failed, 4 files | artifacts/http-verified-frontend-tests.log |
| npm run tauri -- build --no-bundle | optimized Windows Desktop built | artifacts/http-verified-tauri.log |
| cargo build --locked --release -p reconx -j2 | optimized Windows CLI built | artifacts/http-verified-cli-release.log |
| Linux cargo test --locked --offline --workspace --exclude reconx-desktop -j2 | 112 passed, 0 failed, 1 ignored | artifacts/http-current-linux.log; separate stderr log |
| PowerShell build/verify AST parse | no syntax errors | actual console check; scripts restore the caller incremental setting |

The ignored case is the explicit OS-secret-store fixture. Native provider acceptance separately exercises synthetic Credential Manager values. All test DBs/workspaces and sample accounts are task-owned. No normal user DB or real cloud account was used.

## Current Windows native execution

| Suite | Passed checkpoints | Result |
| --- | --- | --- |
| native | 8 | `artifacts/native-1792295863172/results.json` |
| http-workspace | 10 | `artifacts/http-workspace-1792296085396/results.json` |
| recon-local | 7 | `artifacts/recon-local-1792295916549/results.json` |
| ai-providers | 10 | `artifacts/native-ai-providers-1792295928577/results.json` |
| inputs | 9 | `artifacts/inputs-1792295937679/results.json` |
| jobs | 8 | `artifacts/jobs-1792295947866/results.json` |
| integrations | 6 | `artifacts/integrations-1792295974285/results.json` |
| outputs | 6 | `artifacts/outputs-1792295992032/results.json` |

Total **64 passed checkpoints**, 8 suites. The extra HTTP pass extends the same matched binaries to overlapping-target partitioning; its previous 9-checkpoint record is retained in the manifest, not double-counted.

Actual HTTP coverage: native HAR/JSON/YAML dialogs, redacted decoded bodies, independent SHA-256, original integrity, local existing-worker job/evidence, destination-approved mock AI review, captured ownership/status comparison, Needs Verification finding, all four report formats, malformed-input retry, persistent notes, restart, Desktop/CLI parity and cross-target documentation/inventory isolation. No imported request was replayed. A model's interpretation never verifies a finding.

The broader suites recheck real PE import/analysis, target/scope, jobs and actual cancellation/timeout/termination recovery, three themes, chat streaming/cancel, secure synthetic provider setup, verified official YARA-X lifecycle, real benign MCP calls, explicit local AI plans, evidence/finding/CVSS/report gates and manually approved owned-loopback HEAD inventory.

## Visual and independent output inspection

Actual current screenshots were viewed in `artifacts/http-workspace-1792296085396`: http-reconx-1920.png, http-light-1920.png, http-dark-1366.png and http-target-partition-matrix.png. The suite captures both 1920 and 1366 in all three themes; no document horizontal overflow. These are real WebView2 captures, not generated mockups. Narrow layouts require vertical scrolling; no native high-DPI or Linux Desktop claim is added.

Independent pypdf parsing read 4 actual report pages, found the comparison finding and none of the six synthetic credential markers: `artifacts/http-workspace-1792296085396/pdf-inspection.json`. This is separate from checking the PDF magic bytes. Existing outputs acceptance also independently verifies Vietnamese text/redaction. A limited known GitHub-token/private-key pattern scan found zero matches in both delivered executables: artifacts/http-current-binary-pattern-review.json. It is not a complete secret/security audit.

## Retained failures and corrections

- Native Recon showed a completed DB job with a stale UI: artifacts/recon-local-1792294752360/results.json. A deterministic frontend test failed before the request-version guard and passed after; final native recon passes.
- A prior native Node process exited 3221226505 without a diagnostic: artifacts/http-release-verification.json / http-release-native-native.log. Cause remains unestablished. The fresh current full suite passed; the earlier abort is not erased or explained away.
- Linux initially lacked its former /tmp toolchain; checksum-verified official rustup restored Rust 1.95.0 into a task-local /var/tmp directory, with no sudo or profile changes. Staging initially omitted manifests and native-sample.rs, then was corrected. Earlier failed logs remain http-final-linux*.log.
- Mixed WSL stdout/stderr handles produced overlapping log text. Final verification captures inside Linux and forwards streams separately; complete final counts are from the clean log.
- TypeScript caught an incorrect origin field and an incomplete typed test fixture; Clippy caught a complex grouping tuple. Both were fixed; final checks pass with no warning suppression.

## Source changes and remaining acceptance

Implemented core service/models in crates/reconx-core/src/http.rs, existing jobs/lib bindings, storage migration 010/storage lib, shared CLI dispatch/tests and Tauri request handlers. Desktop adds HttpPage/http.ts/http.css/tests and wires existing routes; ReconPage gains the stale-load guard/test. Deterministic fixtures and native HTTP acceptance, Cargo dependency/lock, scripts/build.ps1, scripts/verify.ps1, CI, README and status docs are updated. Pre-existing SECURITY.md changes are preserved. No commit/push occurred.

The complete Web/API module remains PARTIAL: no live identity replay engine, full schema validation, general scanners or verified external Burp/ZAP adapter. Captured labels do not establish authorization. Redacted HTTP export covers the entire project. Original Restricted attachments remain sensitive; redaction is heuristic and sharing needs review. Custom-workspace Windows ACLs and strong OS sandboxing remain open. Linux results cover actual Parrot/WSL Core/CLI, not Desktop/Secret Service/installers. Live cloud accounts, signing and Windows/Linux installer acceptance remain unverified. Full v1.0 release remains blocked.

---

Older checkpoint sections below are retained for provenance. The current checkpoint above takes precedence.

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
