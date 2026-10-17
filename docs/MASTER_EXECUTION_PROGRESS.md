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
