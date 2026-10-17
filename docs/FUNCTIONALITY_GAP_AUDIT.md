# Current feature inventory - 2026-10-17

Inspected shared Rust Core, typed Tauri requests, React routes, CLI, migrations
and actual Windows fixtures. This table supersedes the historical audit below.
No Git remotes/credentials were inspected and no local changes were reset.

| Product area | Current state | Verified behavior / remaining gap |
| --- | --- | --- |
| Projects / targets / scope | TESTED | Existing CRUD/import/export/restart and typed CLI |
| Typography / three themes | TESTED | Retained Segoe/Source Sans/Plex and Light/Dark/ReconX |
| Shared jobs | TESTED | Queue/claims/cancel/timeouts/recovery; no general DAG/pause |
| Native binary inspection | TESTED | PE native, ELF/Mach-O/APK fixtures; no decompiler |
| Tool lifecycle | PARTIAL | Tested ripgrep/YARA-X only; catalog entries are pending |
| MCP gateway | PARTIAL | Real stdio/HTTP benign fixtures; no OS sandbox/OAuth |
| AI provider protocols | PARTIAL | Local protocol fixtures; live cloud accounts unverified |
| Reviewed local AI plans | TESTED | Mock model, exact consent, real local jobs and saved output |
| General agents/routing/budgets | NOT_STARTED | No multi-domain autonomous agent system |
| Evidence management | TESTED | Immutable artifacts/hash/review, native imports, shared records |
| Manual findings | TESTED | Lifecycle/revisions/evidence/duplicates/CVSS supplied vectors |
| Research report exports | TESTED | Reviewed Markdown/JSON/HTML/PDF; actual Unicode PDF inspection |
| Dedicated report visual layouts | PARTIAL | Seven labels share text-first structure |
| Local HTTP inventory | TESTED | Windows actual approved HEAD, job/cancel/failure, asset/evidence/report and restart parity |
| Passive public intelligence | NOT_STARTED | No DNS/RDAP/CT/history service |
| Crawling/scanner adapters | NOT_STARTED | No external scanner workflow or vulnerability claims |
| Web/API offline imports | NOT_STARTED | HAR/OpenAPI/Burp/schema inventory next |
| Auth identity matrix | NOT_STARTED | No multi-identity request execution |
| Network assessment | NOT_STARTED | No network inventory/import service |
| Ghidra/debugger adapters | NOT_STARTED | Generic MCP discovery does not prove interoperability |
| Android analysis | PARTIAL | APK content identification only; manifest/JADX not implemented |
| iOS research | PARTIAL | Thin Mach-O static parser only; IPA/entitlements unsupported |
| Cloud/IaC | NOT_STARTED | No analyzer |
| Containers/SBOM | NOT_STARTED | No analyzer or vulnerability DB correlation |
| CI/CD source review | NOT_STARTED | No YAML analyzer |
| Red team lab | NOT_STARTED | No lab inventory/trust model |
| Windows portable build | TESTED | Current optimized binaries passed 44 native checkpoints |
| Windows installer | PARTIAL | Configuration only, generation/install unverified |
| Linux core/CLI | TESTED | Parrot WSL: 86 tests and actual ELF/hash/evidence/report acceptance |
| Linux Desktop | BLOCKED | webkit2gtk-4.1 development package missing; no privileged installation performed |
| Linux/ARM packages | UNSUPPORTED | Current runtime acceptance not established |
| v1.0 release | BLOCKED | Remaining domains/runtime/packaging/security review incomplete |

BLOCKED here is a feature inventory label, not a claim that no independent work
can continue. Windows custom shared-directory ACLs, parser process isolation,
large-workspace pagination, live AI verification and release signing remain open.
See MASTER_EXECUTION_PROGRESS.md and TEST_RESULTS.md for current exact evidence.

---

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
