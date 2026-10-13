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

Desktop presentation uses a four-region shell in App.tsx. Grouped navigation and activity/shared controls live in src/components; Dashboard and existing business-bearing pages live in src/pages. The token system supports exactly Light/Dark/ReconX, locally bundled DM Sans/IBM Plex Mono and original SVG assets without runtime font/image downloads. Selected project context is local presentation state; it does not send project data to AI or change CLI operations. Shared settings remain in SQLite. Search indexes existing project/provider metadata and routes; future module views expose no execution channel.

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
