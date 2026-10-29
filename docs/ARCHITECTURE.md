# ReconX AI architecture

The desktop and CLI call the same `reconx-core` service. Business rules live in Rust; the React frontend sends typed operations through a single Tauri command. Local database work runs on blocking workers. AI HTTP requests run asynchronously.

- `reconx-security`: typed errors, input validation, gateway policy, security modes.
- `reconx-storage`: SQLite connections, WAL, busy timeout, transactional versioned migrations.
- `reconx-ai`: provider-specific HTTP adapters and OS secret-store abstraction.
- `reconx-tools`: pinned manifests, archive/executable verification, portable installer, receipts and fixed version checks.
- `reconx-analysis`: bounded read-only object/ZIP parsing, hashes, entropy and local strings; no sample execution.
- `reconx-core`: project lifecycle, configuration, audit trail, shared application services.
- `apps/cli`: native `reconx`, Clap, JSON output, exit codes, completions.
- `apps/desktop`: Tauri 2, React, TypeScript, Vite, Tailwind.

Database selection: CLI `--database` override, then `RECONX_DATA_DIR/reconx.db`, then the OS application data directory. Desktop uses the same environment/default rule. No workspace-specific database is created implicitly.

AI replies remain unverified text. Chat does not execute commands; a separate,
reviewed local-file planner can queue typed allowlisted jobs. Streaming uses Tauri
IPC channels and a bounded cancellation registry. Fixed version checks, managed
YARA-X signatures, manually approved benign MCP calls, evidence/findings/reports,
offline HAR/OpenAPI review and individually reviewed loopback HTTP have real core
services. Reviewed HTTPX metadata and passive Subfinder adapters now use the same
Tool Manager, typed jobs, inventory and evidence pipeline; target-session plans
require separate network/data-sharing approval. General scanners, remote replay and broad autonomous agents are not
implemented. Navigation marks future modules clearly.

Migration 002 extends existing projects with targets, imported sample metadata,
analysis jobs and hashed binary-analysis artifact references. Target/scope validation
is deterministic Rust code. Both clients use the same typed CRUD/import/analysis
services; Tauri's file dialogs supply native paths, never fabricated browser records.
Static analysis runs on workers with persistent stage logs and cooperative cancellation.
Policy is rechecked at each stage. Raw samples/strings/hex stay in local managed files;
SQLite stores references and metadata. See RESEARCH_INPUTS.md for limits and removals.

Dependencies are sourced from crates.io and the official npm registry. Cargo.lock and package-lock.json preserve resolved versions.

Schema version is 019. Opening an existing workspace checks its version with
a read-only SQLite connection before enabling WAL, then rechecks under the write
lock. Newer/corrupt files are rejected; additive upgrade backups and preservation
tests remain in storage. This is not a complete workspace backup/restore feature;
binary/evidence bytes and OS credentials have separate lifecycles.

Dependency/release readiness is separate from functional test success. The pinned
Python PDF parser is test-only; glib/GTK still blocks Linux GUI safety acceptance.
See DEPENDENCY_SECURITY.md and RELEASE_READINESS.md. Source/release artifacts and
current acceptance hashes are local under artifacts/, excluded from Git.

Migration 003 adds the durable local Job Engine in reconx-core/src/jobs.rs. Atomic
SQLite claims enforce a two-job ceiling across processes; worker leases, fenced
terminal transitions and current-policy checkpoints protect result publication.
Native worker events and persisted reads feed the shared Job Center. CLI can drain
or watch the same queue. Synchronous CLI analysis waits for an already claimed job
rather than creating an inconsistent second execution. Local notes are independent
of provider settings. Selected-artifact AI review binds consent to its exact preview
and destination and has no implicit fallback. See JOB_ENGINE.md for limits/interfaces.

Desktop presentation uses a four-region shell in App.tsx. Grouped navigation and activity/shared controls live in src/components; Dashboard and existing business-bearing pages live in src/pages. The token system supports exactly Light/Dark/ReconX, native Segoe UI Variable Text, locally bundled Source Sans 3/IBM Plex Mono and the original DM Sans wordmark and original SVG assets without runtime font/image downloads. Active project/target IDs now live in SQLite, shared with CLI through the revision-checked WorkspaceProvider. Selection alone transmits no data to AI. Shared settings remain in SQLite. Search indexes existing project/provider metadata and routes; future module views expose no execution channel.

## Target-bound sessions and streaming imports (schema 017)

`research_sessions.rs` adds persisted typed plans and links to existing jobs,
evidence, manual findings and report snapshots. It discovers actual compiled/local
capabilities, validates strict capability-ID proposals, binds approval to current
scope/model/settings and queues approved work atomically through `jobs.rs`.
Metadata-only AI planning and evidence interpretation have separate consent steps.
Owned-loopback HEAD needs additional explicit approval and displays its destination
and limits. This is a bounded local workflow, not general autonomous pentesting or
a DAG scheduler. See [AI_RESEARCH_SESSIONS.md](AI_RESEARCH_SESSIONS.md).

`file_ingest.rs` streams path-based imports on blocking workers, hashes/copies in
256 KiB chunks, checks disk space/source mutation, and atomically publishes complete
artifacts with transactional target/file metadata. Import limits are separate from
the 64 MiB deep-parser budget. Large metadata jobs stream full integrity/entropy,
retain only a bounded prefix for strings/hex and explicitly defer deep structure.
No sample bytes cross React/Tauri IPC. Native progress/cancellation and CLI use the
same Core; file sizes use checked SQLite i64/u64 conversion. See
[STREAMING_FILE_IMPORT.md](STREAMING_FILE_IMPORT.md), including crash-staging limits.

Appearance is centralized in src/theme.tsx and shared ThemeMenu components. Saved appearance is read before the native window is shown. The core's transaction-scoped set_theme operation updates only that preference; Settings drafts exclude theme and use the central write queue. CSS roles cover the neutral Light/Dark palettes and preserved ReconX palette. Legacy system or missing preference migrates to reconx on open; new installations default to ReconX. No OS appearance listener is installed. See THEMES.md.

Additional verified appearance/runtime documentation:
- https://v2.tauri.app/reference/javascript/api/namespacewindow/
- https://github.com/microsoft/playwright/blob/main/docs/src/webview2.md

Verified documentation:
- https://v2.tauri.app/start/prerequisites/
- https://v2.tauri.app/develop/calling-rust/
- https://developers.openai.com/api/docs/guides/text?api-mode=responses
- https://platform.claude.com/docs/en/api/messages/create
- https://ai.google.dev/api/generate-content
- https://docs.ollama.com/api/chat
- https://docs.rs/keyring/4.2.0/keyring/


## Reviewed local tool/MCP bridge (Milestone 2)

`reconx-tools` owns pinned portable manifests and the fixed YARA-X process adapter.
Core `tooling.rs` exposes supported vs pending catalog/profile metadata and shared
installation audit snapshots. Core `mcp.rs` owns config, disjoint secret references,
SDK sessions, validated exact-input approvals and job-backed benign calls;
`mcp_http.rs` bounds HTTP JSON/SSE bytes before parsing. `capabilities.rs` combines
actual local/MCP capabilities and validates persisted local AI plans. It queues
typed existing handlers; `jobs.rs` remains the only analysis scheduler. Management
health/connection checks are bounded immediate operations, not separate schedulers.
SQLite migration 004 adds configuration/cache/approval/plan/interpretation tables
without duplicating projects, targets or jobs. Desktop and CLI call these services.

## Research-output pipeline and local availability (2026-10-17)

Core evidence.rs/findings.rs/reporting.rs own normalized outputs. Migration 005
adds linked evidence/finding/report metadata; artifact bytes are immutable private
UUID-named files outside SQLite. Hash verification precedes preview/export. Job
completion publishes observed evidence within its existing transaction. Finding
verification requires explicit manual source review; AI confidence is not evidence.
Reports are immutable redacted snapshots with exact-preview export approval and
locally generated Unicode PDF. Structured JSON content is redacted before text
embedding; detectable unsafe older snapshots require regeneration before export.

Migration 006 adds assets and exact local HTTP approvals. recon.rs previews one
HEAD to an authorized loopback URL, fingerprints policy/input and queues the existing
worker. Every checkpoint rechecks authorization. No LLM execution route is added.
Completion atomically links the observed endpoint, private response headers and job
result evidence. ReconPage/RecordsPage and the CLI call these same services.
Current migrations snapshot versions 1-5 before an additive schema-6 transaction;
there is no destructive automatic rollback. See LOCAL_INVENTORY.md and
RESEARCH_OUTPUTS.md for byte limits and platform-dependent guarantees.


Schema019 adds a bounded, read-only materialized provenance graph over existing
session/job/evidence/asset records. It is not an execution queue, agent runtime or
network capability. Desktop and CLI call the same Core projection/page/export
services; source identities and scope associations remain authoritative. See
EXPLORATION_GRAPH.md for limits, privacy, consistency and acceptance boundaries.
