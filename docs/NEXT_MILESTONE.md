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
