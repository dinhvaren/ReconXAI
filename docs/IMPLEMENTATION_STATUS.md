## 2026-10-17 continuation: local HTTP inventory

Phase 1 Windows optimized runtime acceptance passed, including actual PDF exports
and restart. Phase 2 adds one manually approved HEAD request to owned loopback
URLs through the existing jobs; normalized endpoint assets and private HTTP
evidence are persisted. Desktop preview, target-context navigation, job/result
view, cancellation and CLI preview/approval/assets are implemented. The local adapter passed actual Windows debug/optimized native acceptance; the
optimized application passed six suites (44 checkpoints). Broader reconnaissance
remains PARTIAL. Linux Core/CLI passed 86 tests plus real ELF/output acceptance;
Linux Desktop and installers remain UNVERIFIED.

Migration 006 preserves output records and backs up schemas 1-5 to pre-v6 snapshots.
A report-only JSON header redaction defect was reproduced by native acceptance and
fixed structurally, with regression tests and a safe export refusal for detectable
legacy unsafe snapshots. See LOCAL_INVENTORY.md, RESEARCH_OUTPUTS.md and the master
ledger. No public scanning or automatic model-driven network execution is added.

# Implementation status

Updated 2026-10-16. Verified platform: Windows x64. The full ten-phase platform is unfinished.

## v1.0 Phase 1: evidence, findings and reports

The output pipeline is implemented through the existing Core/SQLite services and
available in Desktop and CLI. Completed jobs now atomically create immutable,
hashed evidence; older results can be collected idempotently. Manual text/JSON,
native file/screenshot imports, bounded previews, source review, integrity checks
and explicit raw/redacted local exports work. Evidence retains source/project/
target/job links and sensitivity; integrity is distinct from verification.

Findings support lifecycle, fields/evidence, stale-edit revisions, comparison and
transactional duplicate merging. Verified requires explicit manual confirmation,
a review note and reviewed non-AI source evidence. CVSS 3.1/4.0 uses supplied
complete vectors; missing/duplicate metrics are rejected rather than defaulted.

Reports are immutable redacted snapshots with exact-preview export approval,
integrity/quality checks, visible omission/redaction warnings and actual Markdown,
JSON, escaped HTML and locally generated Unicode PDF. Seven template labels use a
shared text-first structure with bounty-specific headings. Image/binary contents
remain separate. These are real result records, not fabricated vulnerability data.

Migration 005 preserves earlier tables and creates a consistent pre-v5 backup.
Windows debug verification reached 76 Rust tests, 12 frontend tests and 37 native
checkpoints. Optimized native verification also passed; see TEST_RESULTS.md for the
latest evidence. This is not full v1.0 completion: list pagination, richer report
layouts, Windows custom-workspace ACL review and all remaining research domains/
Linux/installer acceptance remain open. See RESEARCH_OUTPUTS.md and the resumable
MASTER_EXECUTION_PROGRESS.md. The historical sections below are prior checkpoints.

## Milestone 2: reviewed local tools, MCP and AI plans

Implemented and verified on Windows: pinned official YARA-X 1.21.0 portable
installation/version/managed removal/repair/profile review and fixed offline file
signatures through the existing Job Engine. The registry identifies other tools
as pending; no scanner integration is implied by a catalog entry.

MCP Hub now supports actual stdio and bounded Streamable HTTP connections,
secret-free configuration CRUD, OS secret references, discovery/schema inspection,
explicit per-tool allowlists, exact-input manual approvals, timeout/cancellation,
real persisted job results/history and reconnect. The current invocation policy
permits reviewed read-only closed-world tools. A stdio child is trusted user-account
code, not an OS sandbox; arbitrary filesystem/network isolation is not claimed.

The shared capability registry feeds a structured local-file AI planner. Native
consent and one-time plan approval queue real analysis jobs; evidence interpretation
is separately consented, persisted and unverified. Model integration acceptance
uses a deterministic local HTTP mock, not a live cloud account. No autonomous
scanner or generic offensive workflow has been added. Exactly three themes and the
existing typography, target/file services and provider/key storage are preserved.

Migration 004 is additive and automatically snapshots existing schema 1-3 databases
before upgrade. Tool/MCP/plan state and history use the existing shared database.
See TOOL_MANAGER.md, MCP_HUB.md and AI_TOOL_ORCHESTRATION.md. TEST_RESULTS.md records
actual native evidence; full platform phases remain IN_PROGRESS where incomplete.

The following sections retain earlier milestone evidence and limits; this section
supersedes historical statements that MCP/local AI tool plans had not started.

## Durable local execution milestone

Target CRUD/import/export and native binary import are retained and extended with
scope-review jobs and CLI scope show. The shared Rust Job Engine adds a durable
queue, five allowlisted local handlers, a global two-job concurrency ceiling,
configurable timeout, cooperative cancellation, persisted progress/logs/results and
worker lease recovery after process termination. Desktop and CLI use identical
services. Migration 003 preserves version-2 job IDs/results/settings and adds notes.

The functional Job Center queues, filters, cancels and opens exact results; reverse
analysis uses the same engine. Local research notes persist. Optional AI artifact
review requires backend-validated consent bound to the preview/gateway, excludes
paths/notes and never silently falls back to another provider. AI explanations remain unverified; reviewed local tool plans are now described above. Existing provider/key storage, project services and all three themes
are preserved. See JOB_ENGINE.md and NEXT_MILESTONE.md for commands, source map and
the current verification checklist; TEST_RESULTS.md records final build evidence.

## Typography refinement: verified Windows slice

The shared type system now uses Windows Segoe UI Variable Text, a locally bundled
OFL Source Sans 3 fallback, and IBM Plex Mono for technical content. Native computed
styles and platform font inspection confirm actual font selection. Titles are
24px/600, body and navigation 14px/400, active navigation and controls 500;
secondary copy is 13px and metadata 12px. Conflicting style patches, excessive bold
and normal-text tracking were removed. Existing wordmarks, palettes, shell, routes
and backend services remain intact. Appearance still has exactly Light/Dark/ReconX.

Native before/after captures cover six pages, three themes and three desktop sizes
(54 observations plus three device-scale captures per phase). All after checks pass;
Windows monitor scaling and Linux rendering were not physically tested. See
TYPOGRAPHY_OVERHAUL.md and artifacts/typography-refinement/compare.html for the
measured causes, actual screenshots and limitations.

## Functional inputs / local analysis milestone

Shared Rust target CRUD supports 17 types, tags, descriptions, scope exclusions,
authorization and modes. Native Add Target works from Dashboard, Projects, inventory
and global quick actions. Inventory supports search/type/project filters, edit,
confirmed delete, versioned native JSON import/export, scope validation and actual
linked file/job history. CLI target and scope commands use the same records.

Reverse Engineering imports real local files through native dialogs, direct folder
import or a registered native drop handler. Read-only parsing identifies PE/ELF/thin
Mach-O, APK/ZIP directories or raw data; exposes hashes, size, architecture, entry,
sections/segments, imports/exports, bounded strings, entropy and hex preview. Results
persist separately from configuration. Search, file selection, tabs, resizable file
pane, removal, comparison and job history have real services. CLI reverse/jobs share
them. Native file picker/import/export, PE analysis and restart sharing are verified;
physical OS drag/drop and folder-picker interaction remain unverified.

Migration 002 preserves existing projects/settings/providers. Job policy checks bind
the import identity and recheck authorization/scope between stages. Job cancellation
is cooperative; local queues/recovery now use migration 003. DAG/retries/pause remain pending. No scanner,
MCP connection, agent orchestration, normalized findings or report export was added.
See FUNCTIONALITY_GAP_AUDIT.md and RESEARCH_INPUTS.md for exact limits.

## Phase 1: verified milestone

Native Tauri desktop and reconx CLI share a Rust core and SQLite database. Project create/open/rename, archive/restore, search and metadata import/export work. Settings, audit history, dashboard, collapsible navigation, CLI help/version/doctor, JSON errors/exit codes and completions are implemented. Native tests verify sharing in both directions and persistence across restart.

## Desktop UI redesign: verified Windows slice

Replaced the former dashboard presentation with the supplied reference's four-region composition, original illustrated banner/project cards and integrated activity panel. DM Sans/IBM Plex Mono and readable typography now replace the earlier Manrope system. All implemented pages retain their operations. Exactly Light/Dark/ReconX appearance remains shared with CLI settings, with the signature burgundy/lavender/raspberry palette preserved under ReconX.

Keyboard workspace search, project selector, rail shortcuts, collapsible groups/panels, focus/hover/error/loading/empty states, notifications and resizable AI context work. Dashboard displays persisted projects/audit events. Future pages have task-specific empty states and NOT_STARTED labels. New UI tests and native screenshots verify common desktop dimensions; see UI_REDESIGN.md for the reference comparison and exact limits.

## Phase 2: working AI slice, phase in progress

Provider CRUD, enable/disable, custom gateway, protocol, model, timeout and output budgets work. Eight provider kinds have local request-protocol tests; five protocol families have stream event fixture coverage. Desktop/CLI support streaming, cancellation, model discovery and single-turn text requests.

OS credentials are separate from ordinary configuration. Credential references bind provider ID, protocol and gateway fingerprint. Named configurations, a default configuration and an explicitly configured fallback are available. Fallback handles only definite pre-transmission connection errors; HTTP errors, ambiguous timeouts and partial output are not replayed. Replies include tokens, latency, provider ID and fallback usage.

Remaining: persisted provider health/usage/cost, per-minute rate limits and budget accounting, multi-turn context, model-specific parameters, custom authentication/headers/proxies, per-agent routing, provider plugins and live cloud verification.

## Phase 3: verified Windows installation slice, phase in progress

Tool Manager and CLI provide inventory, plans, download progress, pinned official downloads, archive/executable SHA-256 verification, safe entry copying, receipts, fixed version checks and uninstall. Native UI/CLI verified a real ripgrep 15.2.0 Windows installation and removal without administrator access.

Windows/Linux manifest checksums were verified against official checksum files and actual downloaded archives. Linux binary hash was computed; Linux execution remains unverified. Health checks prove pinned integrity/version, not security scanner integration.

Remaining: general dependency graphs, unmanaged inventory, package manager/WSL execution, other adapters, unreviewed upstream updates and multi-version rollback. Core/Reverse profiles, pre-publication download cancellation and fixed YARA-X signatures are implemented.

## Packaging and future modules

Optimized Windows portable desktop/CLI binaries are generated by scripts/build.ps1 with SHA256SUMS.txt. Pinned Windows/Linux CI and NSIS/Debian configurations exist. CI runs, installers and Linux/ARM64 are unverified.

General autonomous agents/DAG workflow scheduling, recon, normalized findings/evidence,
offline HAR/API analysis, reporting and extended research modules are NOT STARTED.
Target/scope and local static-binary inputs are the tested subset described above.
Future navigation pages still state their unavailable capabilities explicitly.

Project archive is reversible; permanent project deletion is not exposed. Target/file
inventory removal is separate and preserves retained job/cache data as documented.
Project export contains metadata only. Notes/exports are not automatically redacted.
Chat is transient and single-turn. SQLite is not encrypted. Existing machine
provider/MCP credentials were not imported. No public targets were tested.

Machine-readable registry: docs/module-status.json.

## Current appearance and typography

Exactly Light, Dark and ReconX are implemented and native-tested. Light is white/gray; Dark is near-black/charcoal; ReconX preserves the extracted existing burgundy palette and original artwork. Both appearance menus and the Settings dropdown synchronize immediately and persist automatically. All three restore after native restart. System was removed; old system/missing settings migrate transactionally to reconx without changing other records. New installations default to ReconX.

DM Sans/IBM Plex Mono remain bundled locally with 12px minimum metadata, 14px navigation/body and 26px page titles. Original navigation structure, project/provider/credential settings, gateways, shared storage and CLI remain accessible. See THEMES.md and THEME_TYPOGRAPHY_FIX.md for current controls and validation. Earlier references to Dark/Light/System describe superseded milestones.
