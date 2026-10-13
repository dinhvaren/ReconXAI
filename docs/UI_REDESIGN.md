# ReconX AI UI redesign

Current appearance has exactly Light/Dark/ReconX with bundled DM Sans/IBM Plex Mono. The original visual composition/artwork is retained. Early audit/verification below documents the previous redesign; THEMES.md describes the active palette split, removal of OS-following mode, safe migration and latest runtime checks.

## Reference audit

Downloaded and visually inspected the supplied Figma reference on 2026-10-13. Local reference: artifacts/design-reference/reference.png. It is a design reference only; reference branding, community content and artwork will not be shipped.

Four contiguous regions: 72 px identity rail, 226 px lavender navigation, burgundy main workspace, 252 px raspberry activity panel. Reference uses a narrow top search/toolbar, a wide illustrated banner, two image-led cards and compact horizontal cards. The outer pink presentation frame is omitted for native desktop space.

## Existing application audit

Before redesign, App.tsx owns page navigation, SQLite snapshot refresh, busy/error/notice state and persisted appearance selection. Projects, Settings, AI Workspace and Tool Manager are functional inline pages. Their API bindings call typed Rust operations through service, stream_chat, cancel_stream and install_tool. CLI uses the same core/database.

Rust settings contain security_mode, default/fallback provider IDs and Dark/Light/System appearance. Theme work from the preceding turn is retained. Existing project/provider records and secure OS credentials are preserved. The schema, API contracts and CLI behavior are unchanged by this redesign.

MCP, recon, findings, reverse engineering and reports are explicit NOT STARTED pages. No MCP server, job scheduler or findings service exists; the new UI must not imply otherwise.

## Design foundation

Manrope variable font is bundled locally with its SIL OFL license from google/fonts. Compact 12–14 px navigation/body, 20–24 px headings, moderate 600–750 weights. Burgundies #39001f / #52002e, lavender navigation #7e6b84, raspberry activity #780b46, pink #e83c89 and selective gold #f2aa38.

Light appearance remains available as a coordinated pale rose/lavender variant. System follows OS appearance. Palette tokens control all pages and native HTML controls.

Artwork is original SVG abstract circuitry, orbital geometry and facets. No copied community photography or branding. Assets are bundled through CSS URLs; the native screenshot check asserts that the hero background actually renders.

## Navigation and presentation

Extract business-bearing pages without changing their requests. Introduce shared Icon, Heading, badges/status, art cards, activity rows, tabs, modal and terminal surfaces. App shell manages rail, collapsible navigation groups, current-project selection, centered search, notifications, right-panel toggle and theme.

Search indexes real projects, provider names and implemented/future module routes. Selecting results navigates; Ctrl+K focuses search and Escape closes it. Project selection is presentation context and never silently sends data to an AI provider.

Dashboard uses an illustrated workspace banner, two project/action cards, research shortcuts and persisted audit activity. No repeated metric grid. Right panel shows selected workspace, configured model and real AI audit activity; unsupported jobs/MCP/findings are explicitly labeled.

Existing CRUD, import/export/archive, provider credentials/routing/tests, streaming/cancellation and verified portable tool installation remain accessible. AI page receives a practical conversation + research-context split. Future technical pages receive honest empty states, without fake decompiler output or clickable controls that cannot work.

## Desktop behavior and verification plan

Check 1920×1080, 1600×900, 1440×900, 1366×768 and 1280×720 in real WebView2. Right panel collapses on narrower layouts; secondary nav can collapse while rail remains. Content/panels scroll independently. No horizontal clipping. Resizable split for AI context where space permits.

Run TypeScript/Vite/frontend tests, Rust/CLI regression, Tauri build and native IPC interaction tests. Capture actual dark/light screenshots, compare against reference and iterate. Do not equate mock data with real findings.

## Delivered implementation

All six UI phases are implemented and verified on Windows x64. App.tsx now composes the shared shell; components/ui.tsx provides line icons, headings, status indicators, activity rows, modal and audit table. components/navigation.ts defines grouped routes. pages/Dashboard.tsx owns the illustrated overview; pages/WorkspacePages.tsx retains the existing project, provider, chat and tool operations and provides explicit future module views. styles.css supplies shared tokens, controls and responsive layouts.

Dashboard uses actual persisted projects and audit records. The gold project bar measures metadata completeness (name/description), not invented job progress. Scope is explicitly unconfigured. Provider configuration is labeled separately from connection verification. There are no fabricated agents, findings, MCP servers or system usage figures.

Working interactions: Ctrl+K and keyboard search selection; real project context selection; grouped navigation and rail shortcuts; collapsible navigation/activity; notifications in an accessible native dialog; persisted Dark/Light/System; AI context pointer/keyboard resizing and History/Evidence tabs; response metadata details; existing project/provider/tool forms and native streaming/cancellation. Notifications come from actual action messages/errors/audit failures. Reasoning summaries are response metadata, not hidden chain-of-thought.

MCP, Findings and Reverse Engineering have task-specific presentation and honest empty states. Reverse view tabs change the reserved workspace; no load/debug/export controls pretend to invoke absent services. Those backend modules remain NOT_STARTED. Agent task execution, evidence attachments and approval workflows remain unavailable.

## Visual verification

Real Tauri WebView2 screenshots were captured and inspected beside the downloaded reference. Matching elements: four-region silhouette, contiguous lavender/burgundy/raspberry surfaces, dark rounded navigation selection, centered slim search, wide artwork banner, two illustrated cards with selective gold borders, compact horizontal research cards and stacked right-panel activities. Light mode is a coordinated alternate palette rather than a separate component layout.

Debug native record: artifacts/native-1791901117905/results.json. Optimized release record: artifacts/native-1791901462246/results.json. Both passed eight end-to-end checkpoints, including provider model discovery, native/CLI text and streaming, cancellation, database sharing/restart, masked OS credential storage/status/removal, real pinned ripgrep installation/integrity/health/uninstall and new UI interactions.

Final expanded release record: artifacts/native-1791901611396/results.json, eight passing checkpoints and 50 page/viewport observations in viewport-matrix.json. The nine secondary pages were checked at all five requested sizes, alongside Dashboard. Screenshots at each size are preserved beside this record.

The screenshots use viewport emulation inside the real native WebView2, not a browser-only substitute. They verify content at the requested pixel dimensions; physical monitor scaling, Windows chrome and Linux rendering are not established by these captures. Dashboard was checked at all five sizes; the expanded final release matrix also checks each of the nine major secondary pages at all five sizes.

The normal release is opened visibly using the existing default database. artifacts/windows-x64/desktop-preview.png is an inspected window-only Win32 capture including the actual Windows title bar; it confirms this machine's native window separately from the emulated viewport matrix. Appearance was set to Dark for the requested reference preview; all other saved settings and project/provider records were retained.

Resolved visual defects: inline unquoted SVG data URLs produced a missing background and were replaced with bundled CSS assets; theme screenshots now wait for fonts and repaint; planned indicators no longer contaminate navigation accessible names; selected search queries clear on navigation. An added frontend test initially expected an obsolete empty-state label; it now verifies the disabled send action when no provider is configured.

Remaining: implement future backend modules before enabling their actions, verify Linux/ARM64 and installers, and add external integration fixtures as those services are delivered. Current functionality and limitations are recorded in IMPLEMENTATION_STATUS.md and module-status.json.
