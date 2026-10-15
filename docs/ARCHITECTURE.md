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

AI replies are unverified text and have no execution channel. Streaming uses Tauri IPC channels and a bounded cancellation registry. Fixed tool version checks are distinct from arbitrary external execution. MCP, recon, findings and reports remain unimplemented. Navigation marks future modules clearly.

Migration 002 extends existing projects with targets, imported sample metadata,
analysis jobs and hashed binary-analysis artifact references. Target/scope validation
is deterministic Rust code. Both clients use the same typed CRUD/import/analysis
services; Tauri's file dialogs supply native paths, never fabricated browser records.
Static analysis runs on workers with persistent stage logs and cooperative cancellation.
Policy is rechecked at each stage. Raw samples/strings/hex stay in local managed files;
SQLite stores references and metadata. See RESEARCH_INPUTS.md for limits and removals.

Dependencies are sourced from crates.io and the official npm registry. Cargo.lock and package-lock.json preserve resolved versions.

Migration 003 adds the durable local Job Engine in reconx-core/src/jobs.rs. Atomic
SQLite claims enforce a two-job ceiling across processes; worker leases, fenced
terminal transitions and current-policy checkpoints protect result publication.
Native worker events and persisted reads feed the shared Job Center. CLI can drain
or watch the same queue. Synchronous CLI analysis waits for an already claimed job
rather than creating an inconsistent second execution. Local notes are independent
of provider settings. Selected-artifact AI review binds consent to its exact preview
and destination and has no implicit fallback. See JOB_ENGINE.md for limits/interfaces.

Desktop presentation uses a four-region shell in App.tsx. Grouped navigation and activity/shared controls live in src/components; Dashboard and existing business-bearing pages live in src/pages. The token system supports exactly Light/Dark/ReconX, native Segoe UI Variable Text, locally bundled Source Sans 3/IBM Plex Mono and the original DM Sans wordmark and original SVG assets without runtime font/image downloads. Selected project context is local presentation state; it does not send project data to AI or change CLI operations. Shared settings remain in SQLite. Search indexes existing project/provider metadata and routes; future module views expose no execution channel.

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
