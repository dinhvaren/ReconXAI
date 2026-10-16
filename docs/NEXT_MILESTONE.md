# Milestone 2 - Tools, MCP and local AI orchestration

Local HEAD 7587e63 was clean before implementation. Baseline verify.ps1 passed
57 Rust tests (one OS-store fixture ignored) and 11 frontend tests. Existing inputs,
queue, typography and three themes are retained. No remote configuration is printed.

- [x] Verified portable YARA-X installation, health, registry/profile and local invocation.
- [x] MCP SDK gateway, configuration, discovery, approved benign calls and history.
- [x] Desktop/CLI management and unified capability registry.
- [x] Validated AI plans using existing local jobs with explicit artifact consent.
- [x] Migration backup, regression, native runtime/restart acceptance, release build.
- [x] Document platform verification and incomplete lifecycle/integration capabilities.

This milestone does not add offensive scanners or autonomous network operations.

## Verified checkpoint - 2026-10-16

Formatting/Clippy passed; 66 Rust passed, one explicit OS-store fixture ignored;
12 frontend tests and the TypeScript/Vite build passed. Actual Windows debug and
optimized release each passed 31 native checkpoints: regression 8, inputs 9,
existing jobs 8 and Tool/MCP/AI integration 6. Release records:
`artifacts/native-1792144968533`, `inputs-1792145335173`,
`jobs-1792145365667`, `integrations-1792145390620`.
One earlier release input attempt recorded an extra character and failed; the
unchanged rerun passed. Its cause is unestablished, not silently marked successful.
The normal release startup and backed-up default database migration preserved all
existing record digests. See TEST_RESULTS.md for hashes, limitations and evidence.

Actual acceptance includes official YARA-X download plus both integrity checks,
version/repair/removal/profile operations, real stdio MCP discovery/manual call,
HTTP authentication tests, local model JSON planning, approval, actual local tool
jobs and saved unverified AI interpretation. The model is an explicitly labelled
deterministic local fixture. Linux and live cloud accounts remain UNVERIFIED.

## Milestone 2 source map

- Tool lifecycle: `crates/reconx-tools/src/lib.rs`, `src/local.rs`,
  `manifests/tools/yara-x.json`, `SOURCES.md`; shared registry/profile/history in
  `crates/reconx-core/src/tooling.rs`.
- MCP and reviewed plans: `crates/reconx-core/src/mcp.rs`, `mcp_http.rs`,
  `capabilities.rs`, existing `jobs.rs`/`lib.rs` and `Cargo.toml`; `Cargo.lock`.
- Persistence: `crates/reconx-storage/migrations/004_capabilities.sql` and
  `crates/reconx-storage/src/lib.rs`. The additive migration and pre-v4 backup
  retain existing tables; there is no destructive automatic rollback.
- Clients: `apps/cli/src/main.rs`, `apps/desktop/src-tauri/src/lib.rs`,
  `apps/desktop/src/pages/McpPage.tsx`, `OrchestrationPage.tsx`,
  `WorkspacePages.tsx`, `src/App.tsx`, `src/main.tsx`, `src/research.ts`,
  `src/integrations.css`, `src/components/{ActivitySidebar,QuickActions}.tsx`
  and `src/components/navigation.ts`. Existing theme/typography assets are retained.
- Verification: `crates/reconx-core/tests/integrations.rs`, `research_inputs.rs`,
  storage/tool unit tests, `apps/desktop/src/App.test.tsx`,
  `tests/e2e/{mcp-fixture,integrations}.mjs`, `apps/desktop/package.json`,
  `scripts/verify.ps1`, `.github/workflows/ci.yml`, `.gitignore`.
- Persistent records: `README.md`, `docs/{ARCHITECTURE,SECURITY_MODEL,ROADMAP,
  IMPLEMENTATION_STATUS,TEST_RESULTS,NEXT_MILESTONE}.md`, `docs/module-status.json`,
  `docs/{TOOL_MANAGER,MCP_HUB,AI_TOOL_ORCHESTRATION}.md`.

## Next milestone

1. Normalize completed local job observations into evidence records with source,
   hashes and exact result references; preserve original results separately.
2. Add manually editable findings with explicit confidence/verification status.
   AI interpretations remain unverified, separate from deterministic evidence.
3. Export redacted Markdown/JSON/HTML reports, then independently verify PDF.
4. Add offline HAR/OpenAPI ingestion using local fixtures and the same services.

The broader tool manager/MCP/workflow phases remain IN_PROGRESS: package managers,
WSL, unmanaged executable adoption, general dependency graphs, multi-version
rollback, OAuth, sandboxing, validated Ghidra/debugger adapters, general agents and
DAG scheduling have not been implemented. Linux CI is configured but not executed.

## Prepared Milestone 2 commit summary

feat(core): integrate reviewed local tools, MCP and AI capability plans

- Add pinned YARA-X portable lifecycle and fixed offline signatures to shared jobs.
- Add bounded MCP stdio/HTTP discovery, secret isolation, approvals and history.
- Add explicit local AI plan/evidence consent and one-time typed job approvals.
- Preserve existing state with additive migration 004 and automatic backups.
- Add Desktop/CLI integrations and real Windows native/fixture acceptance tests.

No commit, push or upload has been performed.

---

# Functional research milestone — 2026-10-15

At initial inspection, local HEAD 4ef7e4d matched the public repository HEAD. Working tree was clean at
inspection; the committed typography, palettes and business services are retained.
Baseline scripts/verify.ps1 passed: 45 Rust tests, 1 intentionally ignored OS-store
fixture, 11 frontend tests, formatting, Clippy and TypeScript/Vite build.

## Implementation checklist

- [x] Inspect actual target/file services, migration 002, native commands and CLI.
- [x] Preserve Segoe UI Variable Text, local Source Sans 3/IBM Plex Mono and exactly
  Light/Dark/ReconX. Do not duplicate existing target/import implementations.
- [x] Add scope show and verify target CRUD/duplicate policy/restart in both clients.
- [x] Migration 003: persistent queued jobs, start/deadline/progress/result metadata,
  worker leases and local research notes; preserve all version-2 records/artifacts.
- [x] Shared allowlisted local job engine with atomic claims, bounded concurrency,
  cancellation, configurable timeout, error/log persistence and expired-lease recovery.
- [x] Real job types: scope/registration review, static binary, metadata, strings,
  local artifact indexing. No shell strings, sample execution or network scanners.
- [x] Desktop Job Center with filters, state/progress/logs/results/cancellation;
  native job events and CLI worker/enqueue/result commands.
- [x] Connect target detail and reverse workspace to the engine; local notes,
  source path and explicitly consented selected-artifact AI handoff.
- [x] Deterministic tests for migrations, concurrency, failure, timeout,
  cancellation, interruption and CLI/Desktop consistency.
- [x] Native acceptance: Research Demo + example.test; owned PE import/hash/
  sections/strings/restart; real running/completed/failed/cancelled jobs/screenshots.
- [x] Final release build/runtime regression, data-preservation check and status/report.

## Existing limits to retain explicitly

Static analysis is read-only and capped at 64 MiB; parsers never execute samples.
No decompiler, scanner, MCP execution or findings/report engine is claimed here.
Linux/ARM64 and physical OS drag/drop remain unverified until actually tested.
No user data, credentials or Git remote configuration is reset; no push is authorized.

## Verified debug checkpoint

Final scripts/verify.ps1 -Native -Inputs -Jobs -Visual passed: 57 Rust tests,
11 frontend tests and 25 native checkpoints. Records:
artifacts/native-1792077037276, inputs-1792077081584 and jobs-1792077094197.
This includes the CLI/already claimed Desktop worker race regression. Native
captures were inspected and compact form/pane bounds asserted. Optimized release also passed the same 25 native checkpoints; records:
artifacts/native-1792077312439, inputs-1792077355999 and jobs-1792077366043.
Normal Desktop was reopened after a backed-up, verified schema-3 upgrade; original
record counts and settings/providers/projects/files/results digests are preserved.

## Prepared commit summary

feat(core): add durable local research jobs and integrated file workspace

- Preserve version-2 records with migration 003; add fenced queue/leases and notes.
- Share typed local analysis, scope review, progress, cancellation and recovery
  across Desktop and CLI; retain existing typography/themes/providers.
- Add Job Center, exact result links and explicit destination-bound artifact review.
- Include previously ignored native input fixtures and add real job acceptance tests.

No commit or push has been performed.

## Files created or modified

Existing source/user data are preserved; previously ignored deterministic input
fixtures are included for reproducible clean-checkout verification after commit.

- [.github/workflows/ci.yml](../.github/workflows/ci.yml)
- [.gitignore](../.gitignore)
- [README.md](../README.md)
- [apps/cli/src/main.rs](../apps/cli/src/main.rs)
- [apps/cli/tests/cli.rs](../apps/cli/tests/cli.rs)
- [apps/desktop/package.json](../apps/desktop/package.json)
- [apps/desktop/src-tauri/src/lib.rs](../apps/desktop/src-tauri/src/lib.rs)
- [apps/desktop/src/App.tsx](../apps/desktop/src/App.tsx)
- [apps/desktop/src/components/FileResearchPanel.tsx](../apps/desktop/src/components/FileResearchPanel.tsx)
- [apps/desktop/src/components/JobHistory.tsx](../apps/desktop/src/components/JobHistory.tsx)
- [apps/desktop/src/components/ui.tsx](../apps/desktop/src/components/ui.tsx)
- [apps/desktop/src/pages/JobsPage.tsx](../apps/desktop/src/pages/JobsPage.tsx)
- [apps/desktop/src/pages/ReversePage.tsx](../apps/desktop/src/pages/ReversePage.tsx)
- [apps/desktop/src/pages/TargetsPage.tsx](../apps/desktop/src/pages/TargetsPage.tsx)
- [apps/desktop/src/research.ts](../apps/desktop/src/research.ts)
- [apps/desktop/src/styles.css](../apps/desktop/src/styles.css)
- [crates/reconx-analysis/src/lib.rs](../crates/reconx-analysis/src/lib.rs)
- [crates/reconx-core/src/jobs.rs](../crates/reconx-core/src/jobs.rs)
- [crates/reconx-core/src/lib.rs](../crates/reconx-core/src/lib.rs)
- [crates/reconx-core/src/reverse.rs](../crates/reconx-core/src/reverse.rs)
- [crates/reconx-core/tests/jobs.rs](../crates/reconx-core/tests/jobs.rs)
- [crates/reconx-core/tests/research_inputs.rs](../crates/reconx-core/tests/research_inputs.rs)
- [crates/reconx-storage/migrations/003_job_engine.sql](../crates/reconx-storage/migrations/003_job_engine.sql)
- [crates/reconx-storage/src/lib.rs](../crates/reconx-storage/src/lib.rs)
- [docs/ARCHITECTURE.md](../docs/ARCHITECTURE.md)
- [docs/DECISIONS.md](../docs/DECISIONS.md)
- [docs/IMPLEMENTATION_STATUS.md](../docs/IMPLEMENTATION_STATUS.md)
- [docs/JOB_ENGINE.md](../docs/JOB_ENGINE.md)
- [docs/NEXT_MILESTONE.md](../docs/NEXT_MILESTONE.md)
- [docs/RESEARCH_INPUTS.md](../docs/RESEARCH_INPUTS.md)
- [docs/ROADMAP.md](../docs/ROADMAP.md)
- [docs/SECURITY_MODEL.md](../docs/SECURITY_MODEL.md)
- [docs/TEST_RESULTS.md](../docs/TEST_RESULTS.md)
- [docs/module-status.json](../docs/module-status.json)
- [scripts/verify.ps1](../scripts/verify.ps1)
- [tests/e2e/inputs.mjs](../tests/e2e/inputs.mjs)
- [tests/e2e/jobs.mjs](../tests/e2e/jobs.mjs)
- [tests/e2e/select-file.ps1](../tests/e2e/select-file.ps1)
