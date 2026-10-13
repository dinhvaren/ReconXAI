# ReconX AI architecture

The desktop and CLI call the same `reconx-core` service. Business rules live in Rust; the React frontend sends typed operations through a single Tauri command. Local database work runs on blocking workers. AI HTTP requests run asynchronously.

- `reconx-security`: typed errors, input validation, gateway policy, security modes.
- `reconx-storage`: SQLite connections, WAL, busy timeout, transactional versioned migrations.
- `reconx-ai`: provider-specific HTTP adapters and OS secret-store abstraction.
- `reconx-tools`: pinned manifests, archive/executable verification, portable installer, receipts and fixed version checks.
- `reconx-core`: project lifecycle, configuration, audit trail, shared application services.
- `apps/cli`: native `reconx`, Clap, JSON output, exit codes, completions.
- `apps/desktop`: Tauri 2, React, TypeScript, Vite, Tailwind.

Database selection: CLI `--database` override, then `RECONX_DATA_DIR/reconx.db`, then the OS application data directory. Desktop uses the same environment/default rule. No workspace-specific database is created implicitly.

AI replies are unverified text and have no execution channel. Streaming uses Tauri IPC channels and a bounded cancellation registry. Fixed tool version checks are distinct from arbitrary external execution. MCP, recon, findings and reports remain unimplemented. Navigation marks future modules clearly.

Dependencies are sourced from crates.io and the official npm registry. Cargo.lock and package-lock.json preserve resolved versions.

Verified documentation:
- https://v2.tauri.app/start/prerequisites/
- https://v2.tauri.app/develop/calling-rust/
- https://developers.openai.com/api/docs/guides/text?api-mode=responses
- https://platform.claude.com/docs/en/api/messages/create
- https://ai.google.dev/api/generate-content
- https://docs.ollama.com/api/chat
- https://docs.rs/keyring/4.2.0/keyring/
