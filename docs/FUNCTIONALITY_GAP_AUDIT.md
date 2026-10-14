# Functionality gap audit — 2026-10-14

Audited local commit `e355fd4`, Rust services, SQLite migration 001, CLI dispatch,
Tauri IPC, navigation, and React pages. Existing code and user data are retained.
A page or menu entry is not evidence of a working service.

| Capability | Observed behavior before this milestone | Status |
| --- | --- | --- |
| Projects | Create, rename, archive/restore, search, metadata import/export; shared SQLite | TESTED |
| Appearance | Exactly Light, neutral Dark, original ReconX; persisted shared settings | TESTED |
| AI | Provider CRUD, OS secret store, models, text chat/stream/cancel, conservative fallback | Implemented subset |
| Tool manager | Verified portable ripgrep install, health, removal; no security tool execution | Implemented subset |
| Targets / scope | Navigation placeholder; no table, migration, IPC, CLI | NOT_STARTED |
| Reverse engineering | Placeholder; no native picker, parser, imports, analysis records | NOT_STARTED |
| Recon / network / web / API | Informational placeholders; no target-based workflow | NOT_STARTED |
| MCP | Placeholder; no server registry or gateway | NOT_STARTED |
| Agents / scheduler | No plans, target agents, persistent analysis jobs, approvals | NOT_STARTED |
| Findings / evidence / reports | Placeholders; no normalized records or exports | NOT_STARTED |
| Mobile / cloud / containers / CI/CD / lab | No working analyzers | NOT_STARTED |
| Windows | Existing verification script passed on this host | TESTED subset |
| Linux / ARM64 | CI definitions only; not executed here | UNVERIFIED |

Baseline: `scripts/verify.ps1` passed: Rust check/Clippy/format, 31 Rust tests,
one intentionally ignored OS credential-store test, frontend type/build and 9
frontend tests. Native interaction is not inferred from that run.

Implementation priority: Milestone A target/scope inputs and bounded local static
binary imports, shared CLI/Desktop services, migrations and restart tests.
Target registration grants no execution capability. Later modules remain
unimplemented until their own acceptance evidence exists.

## Architecture to preserve

- `Core` owns SQLite and OS credentials; both clients reuse its services.
- Typed Tauri requests dispatch blocking local operations to worker threads.
- React shares snapshot refresh and a centralized three-theme manager.
- Migration 001 contains projects, settings, providers and audit events.
- Native tests use isolated application and CLI children with one temporary database.

## Milestone evidence

Implemented target/scope CRUD, all 17 registrations, grouping tags, native JSON
import/export, CLI parity, SQLite migration 002 and real file-linked job details.
Implemented native file import and bounded local PE/ELF/thin Mach-O/ZIP/APK/raw
analysis with hashes, strings, sections, import/export tables, comparison, persistent
artifacts and cooperative cancellation. Native tests verify owned PE file selection,
target save/open dialogs, shared records and restart. Exact test records live in
TEST_RESULTS.md. Advanced reverse integrations, recon, MCP, agent orchestration,
normalized evidence/findings/reporting and expanded tool profiles remain gaps.
Drag/drop registration and folder analysis exist; physical OS drop/folder-picker
interaction is not yet verified. Never infer completion of a full module from this
tested input/static-analysis slice.
