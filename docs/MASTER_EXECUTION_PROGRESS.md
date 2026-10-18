## Verified Web/API checkpoint - 2026-10-18

Product version remains 0.1.0. Full v1.0 is NOT release-ready. The bounded offline Web/API slice is RUNTIME_VERIFIED on Windows; the complete Web/API module is PARTIAL.

Completed: bounded HAR 1.2/base64 and OpenAPI 3.0/3.1 JSON/YAML ingestion; native picker/drop review; persistent redacted traffic and notes; origin/port/target-aware endpoint correlation; all-page captured identity/role matrix; original-source integrity/provenance; offline header jobs using the existing worker/evidence; separately consented mock AI interpretation; Needs Verification findings; reviewed Markdown/JSON/HTML/PDF output. Original traffic stays Restricted evidence and no imported request is replayed.

The latest hardening fixes concrete-path precedence, bounded one-pass server expansion, total correlation budgets, foreign-target documentation matches, overlapping-target grouping and incomplete first-page exports. A real stale Recon UI failure was reproduced by a deterministic test and fixed by discarding superseded async loads. Build/verify scripts disable incremental compilation for their duration and restore the caller setting; no cache/data deletion was performed.

Verified current source:

- Windows format and warnings-denied Clippy pass; 112 Rust tests pass, zero fail, one explicit OS-store fixture ignored.
- Frontend TypeScript/Vite and 22 tests pass. Actual optimized Tauri and CLI builds pass.
- All 8 native suites pass, totaling 64 checkpoints, including the extended 10-checkpoint HTTP suite. Actual file dialogs, job states/cancellation/failure/recovery, secure synthetic keys, YARA-X, benign MCP, local mock AI, target partitioning, reports and restart are exercised.
- Linux Parrot/WSL shared Core/CLI: 112 pass, zero fail, one ignored. Rust 1.95.0 was restored in a task-local /var/tmp directory from the official checksum-verified rustup installer, without sudo/profile changes. Linux Desktop, Secret Service runtime and Linux packaging are not established by these tests.

Exact matching portable artifacts: `artifacts/windows-web-api-20261018-105742/` (two executables, SHA256SUMS.txt and font notices). Manifest/source hashes: `artifacts/http-current-verification.json`. Current HTTP native record: `artifacts/http-workspace-1792296085396/results.json`. Main logs: artifacts/http-verified-{format,clippy,rust,tauri,cli-release}.log; artifacts/http-verified-frontend-tests.log; artifacts/http-current-linux.log and its separate stderr log. The embedded frontend is index-BNYH02oQ.js.

Actual screenshots were viewed: `artifacts/http-workspace-1792296085396/http-reconx-1920.png`, light-1920, dark-1366 and http-target-partition-matrix.png. HTTP workspace captures cover all three themes at 1920/1366. An independent pypdf parse verified the actual 4-page HTTP report and absence of synthetic credentials: `artifacts/http-workspace-1792296085396/pdf-inspection.json`.

Retained failures: stale Recon UI at artifacts/recon-local-1792294752360/results.json (fixed and retested); a prior Node native-runner abort 3221226505 at artifacts/http-release-verification.json (cause unestablished, fresh current suite passes); initial Linux staging/toolchain failures in http-final-linux*.log. Mixed WSL stdout/stderr capture was corrected; final logs use separate streams. A passing rerun does not establish the abort's root cause.

Source changes: crates/reconx-core/src/http.rs, jobs.rs, lib.rs; storage migration 010/storage lib; CLI main/tests; Tauri typed request bridge; HttpPage/http models/styles/tests; ReconPage async-load guard/regression; App/navigation; fixtures and native HTTP suite; Cargo lock/dependency; build/verify/CI and documentation. Existing SECURITY.md edits are preserved. No user DB, project, credential, sample or evidence was deleted; no Git remote inspection, commit or push occurred.

Migration: 010_http_workspace.sql only; no 011 is needed for derived matrix/correlation fields. Schemas before 10 receive a consistent pre-v10 backup. Preservation/recovery tests are part of the current full suite. Stop older clients before an actual user-workspace upgrade.

Current task is complete for the offline acceptance slice. Exact next safe task: extend offline documented-response/schema consistency observations, using actual captures and local references with bounded validation; expose evidence-linked observations, never automatic Verified findings. Start in crates/reconx-core/src/http.rs and tests/http_workspace.rs. Do not repeat provider setup, evidence/report foundations or this import milestone.

Remaining Phase 1 acceptance: full JSON Schema/reference compatibility, request editing/replay and protected authenticated test-identity execution. The current captured matrix does not execute accounts or prove BOLA/BFLA. Project export currently covers the full project rather than the active target/search filter. Later scanner/agent, debugger/mobile/cloud/network/lab modules remain PARTIAL/NOT_STARTED. Windows installer/signing, Linux Desktop/packages, live cloud access and hard OS process isolation remain open. Keep release.ready=false.

Resume: read this checkpoint, inspect git status --short (no remotes), then use the current portable binaries and tests/e2e/http-workspace.mjs for targeted acceptance. scripts/verify.ps1 -Native -Visual -Inputs -Jobs -Integrations -Outputs -LocalInventory -AiProviders -HttpWorkspace runs the full Windows verification. Linux task paths are in artifacts/http-final-linux-{toolchain,stage}.txt; reuse isolated source/build caches, capture stdout/stderr separately, and never claim Desktop support from CLI tests.

Prepared commit summary only (no commit created): feat(web): persist offline HAR/OpenAPI workspaces and evidence-linked capture reviews.

---

Older checkpoint sections below are retained for provenance. The current checkpoint above takes precedence.

## Final verification in progress - 2026-10-18

The source additionally partitions HTTP inventory/matrix/documentation matches by
target association (including the all-project view). New HTTP service tests total
11; latest Windows full suite is running toward 112 passed, one ignored. Frontend
now has 22 passing tests. A new deterministic ReconPage test first reproduced an
older queued load overwriting a completed job; a request-version guard and project
cleanup fixed it. Actual native loopback acceptance then passed all 7 checkpoints.

Keep both earlier records: artifacts/recon-local-1792294752360/results.json is the
reproduced stale UI failure; artifacts/http-release-verification.json records a
later Node native-runner abort (3221226505, no diagnostic) after recon/http/AI passed.
The abort cause is not established. Do not hide either failure or claim the current
new target-partition source passed native acceptance until it is rebuilt/retested.

Linux Rust 1.95.0 was restored in an isolated /var/tmp task directory using the
official rustup-init binary and its verified SHA-256; no sudo or profile edits.
The full Core/CLI suite then passed 111 tests, one ignored. Initial staging missed
manifests and native-sample.rs; those omissions were corrected and failed logs kept.
Do not report the former missing-toolchain blocker as unresolved after restoration.
Capture Linux stdout/stderr inside WSL separately to prevent Windows handle overlap.

Next: finish artifacts/http-verified-{format,clippy,rust,tauri,cli-release}.log;
refresh the current source in the isolated Linux stage and rerun its full suite;
copy exact optimized binaries to a new artifact folder and run all 8 native suites;
inspect current screenshots/PDF, then consolidate all status documents/registry.
The full product and active replay/authenticated execution remain incomplete.

## Earlier continuation - 2026-10-18

The existing schema-010 offline Web/API slice is preserved. On resumption the
working tree still contained all HTTP changes and the earlier SECURITY.md edit.
Baseline HTTP tests passed 6/6 and frontend 20/20. D: now has about 78 GiB free;
no cache deletion was performed by this session. Build/verify scripts now disable
incremental compilation for their duration and restore the caller's preference.

Implemented next: origin-aware OpenAPI correlation (scheme/port, concrete-path
precedence), one-pass bounded server variable expansion and a total pattern-match
budget; a real captured identity/role matrix across all pages; complete bounded
redacted capture export; source-hash provenance and integrity checks on comparisons;
target-filtered imports/history. No migration beyond additive 010 is needed.
HAR import, comparison and analysis remain offline. Replay, identity credential
execution, general scanner automation and the full product remain PARTIAL.

Current verification: 10 HTTP service tests passed, TypeScript/Vite build passed,
21 frontend tests passed, formatting and Clippy passed. Full Rust tests, optimized
Tauri/CLI, native regression and Linux shared Core/CLI are running or next.
Logs use artifacts/http-final-*.log. Exact next action: finish these checks,
inspect clean three-theme screenshots, update every status document and record
the matching binary hashes. Never substitute historical native results for the
current source. No Git remote access, user DB opening, push or cloud test.

## Historical runtime and disk-space checkpoint

Current Web/API source passed 107 Rust tests on Windows and 107 on Linux shared
Core/CLI (one ignored each), frontend 20 tests, TypeScript/Vite, formatting and
Clippy. Debug native acceptance passed 8 actual checkpoints:
`artifacts/http-workspace-1792241586062/results.json`. HAR/native picker, OpenAPI
JSON/YAML, actual offline jobs/evidence, separately consented mock AI, capture
comparison, unverified finding, four CLI report exports and restart are verified.
No production release or full Phase 1/v1.0 completion is claimed.

User next requested a disk-usage diagnosis. Read-only measurement found ~69.7 GiB
logical project size: target ~64.0 GiB (debug incremental ~25.6 GiB, debug deps
~29.8 GiB) and artifacts ~5.5 GiB. D: has ~24.9 GiB free of 238.5 GiB. NTFS hard
links can cause logical sums to exceed unique allocation. No cleanup was performed.
See artifacts/disk-usage-latest.json. Avoid more large builds until cache growth is
addressed. Preserve source, SECURITY.md changes, raw evidence and source snapshots.

Next: agree/perform appropriate generated-cache cleanup, reduce future debug/test
cache growth, finish optimized Windows build/native regression, inspect clean
workspace screenshots (existing ones include the deliberately tested error banner),
harden large server-template correlation budgets, then update all status docs.
Active replay/authenticated identity execution and remaining product phases are
still incomplete. Linux Desktop/signing/installers/live cloud remain unverified.

## Web/API offline workspace - current work (2026-10-17)

Base HEAD: `70bc08d`. The pre-existing SECURITY.md edit is preserved. No Git remote
configuration, credentials, user DB, imported samples or evidence were inspected
or reset. Baseline current-source Rust tests (99 passed, one ignored) and frontend
17 tests passed again before implementation.

Implemented in source (native acceptance IN PROGRESS): HAR 1.2 bounded import,
base64 content decoding, redacted request/response inventory, native picker/drop
review, OpenAPI 3.0/3.1 JSON/YAML with same-document pointers and parser budgets,
endpoint correlation, captured-role diff/ownership indicators, notes, restricted
original evidence, review-only findings and exports. A fixed offline captured-header
handler uses the existing Job Engine, evidence and explicit selected-result AI
consent. CLI shares every backend service. Schema 010 is additive and snapshots
older databases to pre-v10 backups. Existing AI routes and schema 009 remain intact.

This is an OFFLINE analysis slice, not full Phase 1 or v1.0 acceptance. Active
replay/request editing, authenticated test identities, automatic BOLA/BFLA testing,
full JSON Schema validation and external/multi-file references remain unavailable.
No imported request is sent to any endpoint. AI review is a separate explicit
preview/destination approval and is tested only against an owned loopback mock.

Current tasks: finish Rust/Clippy/frontend checks; build actual Tauri/CLI; run
`tests/e2e/http-workspace.mjs` through the native file picker and restart; inspect
screenshots; run native regression; test Linux shared Core/CLI if WSL is available.
Source before changes: artifacts/web-api-source-1792240236/before.tar.gz.
Do not mark the new slice RUNTIME_VERIFIED until the native result exists.

## AI Provider Manager - current verified source (2026-10-17)

Implemented on HEAD `d6af47e`, as explicitly requested after the external working-tree
replacement. No reset, checkout, remote inspection, commit or push. This section
supersedes historical counts and next-task descriptions below; product version
remains 0.1.0 and full v1.0 release acceptance is incomplete.

- TESTED bounded AI settings slice: typed catalog of 24 providers with documented
  defaults/presets, searchable setup dialog, model dropdown, secure key entry,
  atomic save, enable/default/remove profiles, actual model inference tests,
  bounded model discovery/cache, immediate AI Workspace selection and agent routes.
- Custom protocols, authentication headers, private headers, explicit proxies,
  discovery paths and streaming preferences use the shared Rust service. Stored
  keys/header values remain in the OS secret store, never ordinary SQLite fields.
- Saving does not initiate inference or mark Connected. Health records the last
  explicit test, latency and error code. Key/config changes invalidate health;
  concurrent changes cannot publish an obsolete successful test.
- Additive migration `009_ai_provider_manager.sql` reserves interrupted 007/008
  numbers, snapshots older databases and preserves existing/unknown tables. Existing
  credential references, projects, settings, typography and three themes are retained.
- Windows: formatting, Clippy with warnings denied, 99 Rust tests passed (one
  explicit OS-store fixture ignored), frontend type/build and 17 tests passed.
  Actual Tauri optimized build passed, plus 7 native suites / 54 checkpoints.
- Linux Parrot/WSL: 99 shared Core/CLI Rust tests passed, one ignored. Linux Desktop,
  Secret Service runtime, optimized packaging and installers remain UNVERIFIED.
- Cloud account/model access is UNVERIFIED. Native inference used actual loopback
  mock servers, synthetic OS-stored keys and the running Tauri/WebView2 app.
  No personal evidence or provider credentials were uploaded. User DB was untouched.

Verified portable Desktop/CLI: `artifacts/windows-ai-providers-20261017-185028/`.
Machine-readable results/hashes: `artifacts/ai-manager-verification.json`.
Settings screenshots: `artifacts/native-ai-providers-1792237828887/` (three themes).
See `AI_PROVIDER_MANAGER.md` and `TEST_RESULTS.md` for setup, commands and evidence.

Remaining AI work: account-specific capability validation, richer pricing/usage,
rate/cost budgets, per-account discovery metadata, multi-hop data-sharing-aware
fallback and live opt-in provider verification. Current agents use text-based
plans; saved routes do not imply every future security agent is implemented.
Selected-artifact review rejects configured proxies until its consent explicitly
covers the intermediary. Existing destination-bound artifact consent stays enforced.

Next exact product task: implement bounded OFFLINE HAR import/endpoint inventory on
this current schema, followed by offline OpenAPI. Audit current assets/evidence/jobs
first. Do not restore interrupted source or repeat completed settings work.
No imported request replay, credentials upload or scanner automation is implied.

Resume checks: inspect this ledger and `git status --short`, then
`powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify.ps1 -AiProviders`
for the AI slice. Full native regression also uses `-Native -Visual -Inputs -Jobs
-Integrations -Outputs -LocalInventory`. Do not print credential-bearing remotes.
Suggested commit summary only (no commit created):
`feat(ai): add secure provider/model setup, discovery, health and shared routing`.

# ReconX AI v1.0 execution ledger

Updated: 2026-10-17. Baseline a29c5db was clean before this work. Local changes
are preserved; no Git remote configuration, credential rotation, commit or push.
Existing layout/fonts and exactly Light/Dark/ReconX remain unchanged.

## Current checkpoint

- TESTED bounded output slice: actual jobs -> immutable evidence -> manual findings
  with verification gates/CVSS -> reviewed redacted Markdown/JSON/HTML/PDF reports.
- TESTED bounded inventory slice: one manually approved HEAD to a scoped loopback
  URL through the existing worker, with assets/private evidence, cancel/failure,
  restart and CLI parity. Full reconnaissance is PARTIAL, not complete.
- Windows optimized Desktop/CLI are built and all six native suites passed (44
  checkpoints). Normal Desktop is open from artifacts/windows-x64; recorded owned
  PID 10840, title ReconX AI, zero stderr at verification. Do not kill it merely
  to rebuild; check identity and active jobs before any normal shutdown.
- Linux shared Core/CLI: 86 Rust tests passed, one explicit OS-store test ignored.
  Actual CLI imported /usr/bin/true ELF, independently matched SHA-256, retained
  29 sections, saved evidence/unverified observation and exported all four report
  formats. Independent pypdf extracted Vietnamese from the Linux-generated PDF.
- Linux Desktop is UNVERIFIED: Parrot lacks webkit2gtk-4.1 development SDK. No
  privileged package or profile changes. Optimized Linux CLI/installers unverified.
- Full v1.0 is NOT ready. Do not bump the product version or claim all domains work.

## Current/next task

Next product vertical slice: bounded OFFLINE HAR ingestion and endpoint inventory,
then OpenAPI import and deterministic header/cookie observations. Reuse existing
assets/evidence/jobs and project/target boundaries. Keep raw authentication and
cookies only in private artifacts; typed normalized records omit secret values.
Use local synthetic traffic, hard input/depth/entry limits and real import/native
picker/CLI/restart tests. Import must never replay requests or execute code. Design
one migration only after auditing schema 006; do not duplicate target/job tables.

Before resuming: inspect Git status and this ledger. Do not rerun a completed
phase merely because another document's historical section lists it as planned.
Current Windows verification:
  powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify.ps1 -Outputs -LocalInventory
Full native regression flags also include -Native -Visual -Inputs -Jobs -Integrations.
Build: scripts/build.ps1; do not overwrite a running normal executable. Tauri
build --no-bundle was also executed successfully on the current product source.

## Feature inventory

| Area | State | Scope / remaining acceptance |
| --- | --- | --- |
| Projects / targets / scope | TESTED | Existing Windows native/CLI and Linux Core/CLI fixtures |
| Fonts / three themes | TESTED | Preserved; Linux Desktop rendering unverified |
| Shared jobs | TESTED | Claims/cancel/timeout/recovery; no general DAG/pause/retry |
| Local binary workspace | PARTIAL | Tested PE/ELF/thin Mach-O/ZIP/APK metadata; no decompiler |
| Evidence | TESTED | Immutable artifacts/hash/review/import/export and job provenance |
| Manual findings/CVSS | TESTED | Lifecycle/revisions/links/duplicates, complete supplied 3.1/4 vectors |
| Reports | PARTIAL | Four tested formats; seven labels share text-first layouts |
| Loopback HTTP inventory | TESTED | Exact manual approval only; no scanners or AI selection |
| Full passive/active recon | PARTIAL | DNS/RDAP/CT/history/crawling/scanner adapters unimplemented |
| Web/API analysis | NOT_STARTED | Offline HAR/OpenAPI next; no identity matrix/replay service |
| Network assessment | NOT_STARTED | No network import/service inventory module |
| Tool manager | PARTIAL | Windows ripgrep/YARA-X lifecycle; remaining catalog integrations pending |
| MCP | PARTIAL | Real stdio/HTTP benign fixtures; no OS sandbox/OAuth/security adapters |
| AI provider protocols | PARTIAL | Local fixtures; live accounts/rate/cost/budget/routing unverified |
| Reviewed local AI plans | TESTED | Mock planner, real local jobs, consented unverified interpretation |
| General multi-agent orchestration | NOT_STARTED | No multi-domain DAG/agents/Authorized Auto execution |
| Ghidra/debugger integrations | NOT_STARTED | Generic discovery is not validated interoperability |
| Android/iOS | PARTIAL | APK identification/thin Mach-O only; advanced/device work unsupported |
| Cloud/container/CI/CD | NOT_STARTED | No real analyzers yet |
| Red team lab | NOT_STARTED | No lab inventory/trust model |
| Windows portable release | TESTED | Current exact optimized artifacts passed native acceptance |
| Windows installer | PARTIAL | Configuration only; generation/install/signing unverified |
| Linux Core/CLI | TESTED | Parrot WSL2 debug runtime; not a Desktop/package claim |
| Linux Desktop | BLOCKED | Missing WebKit SDK; no privileged automatic installation |
| Linux/ARM release packaging | PARTIAL | Configuration only; runtime/package acceptance still required |
| v1.0 release | BLOCKED | Remaining domains, hardening and installer acceptance open |

## Migrations and decisions

005_research_outputs.sql adds evidence/findings/links/reports/exports. Artifacts
are private UUID-named files outside SQLite. Completed job transactions publish
result references plus observed evidence. Original bytes are immutable; integrity
is distinct from vulnerability verification. Verified findings require explicit
manual confirmation, a review note and reviewed non-AI source evidence.

006_local_inventory.sql adds exact HTTP approvals and normalized assets. Approval
fingerprints bind target input and security policy; validation runs while waiting
and before commit. No second scheduler, unrestricted shell or autonomous network
path was added. One HEAD, no redirect/proxy/auth/cookie jar/body download/retry,
five-second deadline, 20 approvals/project/minute, existing global two-job limit.

Current migration code snapshots schemas 1-5 to pre-v6 backups before a single
additive transaction. Default schema 4 -> 6 upgrade preserved hashes/counts of
all 13 original tables, snapshot matches before state, no foreign-key errors.
Record: artifacts/release-upgrade-1792216876468/after.json. No test data was seeded
into the normal user database. Recovery is forward repair/restore a private backup
with clients stopped, not an automatic destructive downgrade.

Reports are immutable snapshots requiring exact-preview approval. Raw images and
binaries stay separate. Structural redaction covers credential fields/header
pairs/base64 values and nested JSON; deeper serialized JSON is visibly withheld.
Detectable unsafe older snapshots require regeneration, never silent rewriting.
PDF uses bundled licensed Source Sans 3, explicit weight 400 and glyph/line limits.

## Verified commands and records

Windows: cargo fmt --all -- --check; cargo clippy --locked --workspace --all-targets
-- -D warnings; cargo test --locked --workspace: 86 passed, 0 failed, 1 ignored.
Frontend npm run build / npm test: 13 passed. Modified native-fixture tests were
also rerun individually after the Linux fixture correction (CLI 5 + inputs 8).
Tauri optimized no-bundle build and optimized CLI build passed. Exact artifact
hashes: artifacts/windows-x64/SHA256SUMS.txt. Known Git-token/private-key pattern
scan found zero matches in these two binaries; this is not a complete secret audit.

Optimized native acceptance (all passed):
- artifacts/native-1792217533338/results.json (8)
- artifacts/inputs-1792217578167/results.json (9)
- artifacts/jobs-1792217588488/results.json (8)
- artifacts/integrations-1792217614199/results.json (6)
- artifacts/outputs-1792217631931/results.json (6)
- artifacts/recon-local-1792217644790/results.json (7)

Latest debug output/local runs:
- artifacts/outputs-1792217139477/results.json
- artifacts/recon-local-1792217154231/results.json
Other debug suites passed earlier in this phase; TEST_RESULTS.md keeps records.
Actual screenshots for all three themes and a rasterized release PDF were viewed.
Native report PDFs were independently parsed with installed pypdf; no fake result
or vulnerability was generated. Owned fixtures and mock models are labelled.

Linux: artifacts/linux-verify-1792217270990/acceptance.json, cargo-test-final.log,
test-result-final.json and installer.json. Official rustup-init SHA-256 checked
before execution. Isolated Rust 1.95.0 and source live under
/tmp/reconx-linux-1792217270990 inside Parrot; CARGO_HOME=cargo and RUSTUP_HOME=rustup
below that directory. No profile/global package changes. Source snapshot excludes
.git, credentials, user datasets, node_modules and output artifacts.
Command: cargo test --locked --workspace --exclude reconx-desktop with task-local
Rust environment, Node 20.19.2 and two compiler jobs. CLI process/restart consistency,
actual ELF parsing, loopback/MCP/mock-model and all output fixtures ran on Linux.
Evidence Unix mode 600 and directory mode 700 were independently checked.

Linux initially rejected oversized DWARF test executables. Shared test-only helper
now strips debug sections from a private executable copy using binutils, retaining
the original; production limits unchanged. Initial CLI smoke used an invalid
severity alias info; fixture corrected to informational, product enum unchanged.

## Fixed defects and remaining risks

Fixed during execution: incomplete CVSS vectors silently defaulted by the library;
HAR name/value secret redaction; embedded JSON/base64 report leak; Windows path
pattern incorrectly redacting HTTP URLs; close buttons silently ignoring busy
clicks; native accessibility labels; test locator/restart/cross-process picker
issues; Winsock accepted sockets inheriting nonblocking mode. Failed artifacts
are retained and the actual fixes/acceptance are recorded in TEST_RESULTS.md.

Remaining: latest-1000 list limits, richer taxonomy/report layouts and attachment
presentation; custom Windows workspace ACL review, no hard process sandbox, no
same-account race-free filesystem guarantee, unencrypted private artifacts, orphan
artifact recovery/retention, unknown-secret manual review, live provider accounts,
agent budgets/routing, remaining research modules, Linux Desktop/packages, Windows
installer/signing and complete dependency/license/release review. No arbitrary
exploitation or autonomous offensive execution is implemented.

## Modified files

- .github/workflows/ci.yml
- .gitignore
- Cargo.lock
- README.md
- apps/cli/src/main.rs
- apps/cli/tests/cli.rs
- apps/desktop/package-lock.json
- apps/desktop/package.json
- apps/desktop/src-tauri/src/lib.rs
- apps/desktop/src/App.test.tsx
- apps/desktop/src/App.tsx
- apps/desktop/src/components/QuickActions.tsx
- apps/desktop/src/components/TargetDialog.tsx
- apps/desktop/src/components/navigation.ts
- apps/desktop/src/components/ui.tsx
- apps/desktop/src/main.tsx
- apps/desktop/src/pages/TargetsPage.tsx
- apps/desktop/src/pages/WorkspacePages.tsx
- apps/desktop/src/research.ts
- crates/reconx-core/Cargo.toml
- crates/reconx-core/src/capabilities.rs
- crates/reconx-core/src/jobs.rs
- crates/reconx-core/src/lib.rs
- crates/reconx-core/src/mcp.rs
- crates/reconx-core/src/reverse.rs
- crates/reconx-core/tests/research_inputs.rs
- crates/reconx-storage/src/lib.rs
- docs/ARCHITECTURE.md
- docs/DECISIONS.md
- docs/FUNCTIONALITY_GAP_AUDIT.md
- docs/IMPLEMENTATION_STATUS.md
- docs/NEXT_MILESTONE.md
- docs/ROADMAP.md
- docs/SECURITY_MODEL.md
- docs/TEST_RESULTS.md
- docs/module-status.json
- scripts/verify.ps1
- tests/e2e/select-file.ps1
- apps/desktop/src/pages/ReconPage.tsx
- apps/desktop/src/pages/RecordsPage.tsx
- apps/desktop/src/records.css
- crates/reconx-core/src/evidence.rs
- crates/reconx-core/src/findings.rs
- crates/reconx-core/src/recon.rs
- crates/reconx-core/src/reporting.rs
- crates/reconx-core/tests/local_inventory.rs
- crates/reconx-core/tests/outputs.rs
- crates/reconx-storage/migrations/005_research_outputs.sql
- crates/reconx-storage/migrations/006_local_inventory.sql
- docs/LOCAL_INVENTORY.md
- docs/MASTER_EXECUTION_PROGRESS.md
- docs/RESEARCH_OUTPUTS.md
- tests/e2e/native-sample.rs
- tests/e2e/outputs.mjs
- tests/e2e/recon-local.mjs
- tests/e2e/requirements.txt

## Prepared commit summary

feat(core): persist reviewed research outputs and approved loopback inventory

- Add evidence/finding/CVSS services and immutable redacted four-format reports.
- Reuse jobs for exact manually approved local HTTP inventory and asset evidence.
- Add Desktop/CLI workflows, safe migration backups and actual restart acceptance.
- Preserve typography/themes/providers; verify Windows native and Linux Core/CLI.

No commit or push was performed. Do not publish releases without explicit approval.
