## Current followup: CI fixture reliability and complete project DOCX — 2026-10-18

Preserved the tested production source, schema 012, user DBs, credentials, typography
and exactly Light/Dark/ReconX. No new production adapter or migration. v1.0 is NOT
COMPLETE; glib remains a blocking unsound advisory. Eight High pypdf alerts remain
confirmed fixed; no warning or policy has been suppressed.

Actual GitHub CI for c22f44c: CodeQL Advanced succeeded (run 39125535750), but
verification failed (39125535775). Bounded downloaded job logs reproduced two test
assumptions: Linux keyless-provider removal correctly fails closed without Secret
Service (optional headers may still exist), and a Windows MCP cancellation test
reused a 500ms timeout and could terminate before the call started. CLI regression
now asserts retained configuration/default/routing on SECRET_STORE_UNAVAILABLE,
while native Windows still covers successful OS-backed deletion. Cancellation
gets a separate long-running benign echo budget and waits for actual progress,
failing immediately with the job state if terminal early. Production timeout,
permissions and credential-deletion behavior are unchanged.

Fresh tests: Windows Rust 143 passed/0 failed/1 ignored; Parrot/WSL Core+CLI
143/0/1 with an explicitly absent DBUS session bus; format, warning-denied Clippy,
26 frontend tests and TypeScript/Vite pass. DOCX generator tests 3 pass and
dependency-readiness tests 4 pass. Current evidence:
artifacts/documentation-current-verification.json, documentation-windows-rust.log,
documentation-linux-{stdout,stderr}.log, documentation-clippy.log and
documentation-frontend-{tests,build}.log. Earlier failed document-format checks
are retained separately; subsequent final structural and Word rendering checks pass.
Native 9 suites/77 checkpoints and optimized binaries remain the unchanged
production pair in artifacts/security-current-verification.json; they were not
unnecessarily rebuilt/retested for test/docs-only changes. Remote CI fixes require
the subsequent source revision's actual run; the glib gate remains expected red.

Delivered ReconX_AI_Tai_lieu_du_an_day_du.docx in the user's requested Desktop/report
directory: 64 pages, populated TOC/page numbers, 25 chapters plus source-derived
provider/migration/module/CLI appendices. Every text style/run including table,
heading, technical examples, header/footer and TOC is Times New Roman/black.
Microsoft Word pagination and local PDF inspection verified only TimesNewRoman
font variants, all text color 0, Vietnamese, no replacement glyphs, blank pages or
page-edge overflow. Internal PDF/screenshots are verification artifacts, not a
substitute for the requested editable DOCX. Proof: project-document-delivery.json,
project-document-word-validation.json, project-document-pdf-inspection.json under
artifacts/. No user data or credentials embedded; no cloud document service used.

Reusable source: docs/PROJECT_DOCUMENTATION.vi.md and scripts/build_project_document.py;
optional dependency scripts/documentation-requirements.txt; tests in
scripts/test_project_document.py. The generator refuses an existing destination,
uses no remote template/network/normal DB and invokes an optional CLI only with
--help. README records generation. Existing Word files were not overwritten.

Next: inspect the new remote CI run after authorized push; keep glib/GTK and Linux
GUI/Secret Service/clean installers blocked until independently resolved/tested.
Continue the safe offline/data-integrity backlog, not placeholder modules or
claims of general external replay/active authorization/scanner automation. Latest
source push proof remains artifacts/source-push-result.json. No force-push,
release publication, privileged install or user-workspace migration performed.

---

## Verified remote-security / CI / Linux CLI followup — 2026-10-18

GitHub confirmed all eight pypdf High alerts (#2-#9) FIXED after authorized push
34421a8; actual fresh export: artifacts/dependabot-after-second-check.json. Glib
Medium #1 remains OPEN and the dependency gate remains BLOCKED; no advisory was
dismissed. Six RustSec maintenance warnings remain. Full v1.0 is NOT COMPLETE.

Remote CI initially rejected a pre-existing unquoted --only-binary=:all: pip run
line as invalid YAML. A new Rust workflow-parser regression reproduced the exact
line-103 error, then passed with a folded run string. Both shipped YAML workflows
and string-valued run steps are checked. Before/fixed logs: security-workflow-
before-fix.log, security-workflow-final.log and security-workflow-linux-final.log
under artifacts/. This distinguishes syntax acceptance from successful CI jobs;
the glib gate is still expected to fail. Remote full CI is not claimed successful.

Latest regression: 143 Rust tests passed on Windows and 143 on Parrot/WSL Core+CLI,
one ignored each; frontend 26 and native 9 suites/77 checkpoints remain verified.
The followup only changes CI/test/scripts/docs, not production Rust/frontend logic;
the same final Windows binaries/installer remain valid. Current aggregate proof:
artifacts/security-current-verification.json. Exact latest logs:
artifacts/security-ci-followup-{rust,clippy,linux-stdout,linux-stderr}.log.

New optimized Linux CLI built offline and executed from artifacts/linux-cli-security-20261018-185625/reconx.
Actual separate CLI processes imported /usr/bin/true, independently matched SHA-256,
parsed Elf/X86_64 (29 sections, 581 strings), completed/persisted a real analysis
job, verified evidence, created an explicitly synthetic Needs Verification record,
and exported Markdown/JSON/HTML/PDF from a reviewed snapshot. Restart/process
persistence checks passed. Independent pypdf 6.19.0 inspection of the Linux-generated
3-page PDF confirmed Vietnamese text/redaction. Evidence: artifacts/linux-cli-security-20261018-185625/smoke/acceptance.json,
pdf-inspection.json, build-manifest.json, compiler.txt and SHA256SUMS.txt. Linux
Desktop, Secret Service, packages and other distributions remain UNVERIFIED.

New reusable command: python3 scripts/linux_release_smoke.py --binary PATH --output
NEW_DIRECTORY. It refuses existing output, retains its isolated test workspace,
uses a benign system ELF without executing it, and never calls a network/AI
service or credential manager. CI includes this acceptance and report artifacts.
No migration added; schema remains 012. Exact next task remains compatible glib
remediation/isolated Linux GUI prerequisites, then remaining safe offline/product
requirements. Source commit/push uses the user's existing authorization; no release
publication, force-push, normal user DB/key changes or privileged package installs.

---

## Current verified security/schema checkpoint — 2026-10-18

Based on 0e496ed; no reset or user-data migration. Product remains 0.1.0 and v1.0
is NOT READY. GitHub returned nine actual alerts: eight High in test-only pypdf
6.16.1 and one Medium in Linux GTK/glib 0.18.5. Pypdf is now pinned to 6.19.0 with
the official wheel hash; native PDF acceptance passes with an isolated interpreter.
Remote closure must be checked separately after pushing. Glib remains unresolved,
not suppressed; cargo audit also reports six unmaintained dependencies. The new
warnings-aware dependency gate correctly fails (exit 1). See DEPENDENCY_SECURITY.md.

Implemented bounded offline enum/3.1 const, numeric/string/count constraints,
allOf/anyOf/oneOf/not and additionalProperties using shared work limits and
three-way branch results. Unknown/redacted branches remain inconclusive; no
network/file resolver, regex execution or automatic finding verification is added.
Existing Desktop/CLI schema jobs/evidence show the new constraints with exact
source pointers. Also added read-only database version preflight before WAL,
with byte-preservation tests and actual CLI rejection of future/corrupt fixtures.
No migration was added; schema stays 012.

Current proof: artifacts/security-current-verification.json. Windows Rust 142
passed/0 failed/1 ignored; Linux Parrot/WSL Core+CLI 142/0/1; frontend 26 passed;
4 dependency-gate tests passed. Format, Clippy, TypeScript/Vite, optimized CLI/
Desktop and all 9 native suites/77 checkpoints pass. Independent pypdf 6.19.0 PDF
inspection and negative untrusted-TLS tests pass on Windows and Linux Core/CLI.
Windows NSIS generation succeeds; clean install/upgrade/uninstall, signing and
Linux GUI/packages/Secret Service remain unverified or blocked. Artifacts:
artifacts/windows-security-20261018-183806, including installer/, build-manifest.json and SHA256SUMS.txt.

Retained failures: security-schema-first.log (missing Map import),
security-schema-second.log (old unsupported-oneOf expectation) were fixed before
full verification. Native http-replay-1792323405520 failed because the runner
sent inside the backend's real 500 ms spacing limit; its alert proves policy
rejection. The runner now waits from the last observed fixture request rather
than relaxing policy. First-failure logs remain. All current suites were rerun
on the final NSIS-tagged binary pair. Screenshots, including new schema results,
were inspected. Synthetic fixture keys are separately cleaned; user keys are not.

Exact next safe task: resolve the compatible glib/GTK dependency issue with a
reviewed upstream fix/backport and actual isolated Linux SDK/runtime testing.
Keep the dependency gate failing until resolved. If blocked upstream, continue
bounded offline pattern/format/reference coverage and manual evidence-linked
permission annotations. General external replay, stateful authorization/scanner
automation and broader agents/research domains remain incomplete; do not promote
them based on this checkpoint. Do not repeat existing imports/providers/output
services or change typography/themes. RELEASE_READINESS.md and release-gaps.json
record remaining release gates.

Resume: cargo fmt --all -- --check; cargo clippy --locked --workspace --all-targets
-j2 -- -D warnings; cargo test --locked --workspace -j2; npm run build / npm test in
apps/desktop; python -m unittest discover -s scripts -p test_dependency_readiness.py
-v. Use CARGO_INCREMENTAL=0. Native suites use the explicit snapshot executables.
Audit: cargo audit --deny unsound --json; scripts/dependency_readiness.py combines
recorded exports without hiding warnings. Commit/push is authorized by the user's
later instruction; use only the clean HTTPS URL, no force-push or release publication.
Exact Git handoff is recorded locally in artifacts/source-push-result.json.

---

## Current HTTP request/identity slice - 2026-10-18

RUNTIME_VERIFIED bounded manual loopback slice: imported request drafts, secure project/target/origin identity manager, exact review/approval, real GET/POST/PUT/PATCH/DELETE/HEAD/OPTIONS transport with policy gates, persistent job state/history/captures, cancellation and comparison/evidence/report integration. Actual native acceptance exercises GET/POST and two OS-backed Bearer identities; other auth methods use real HTTP core fixtures with a deterministic secret-store adapter. Saving never sends traffic or marks authenticated.

Target deletion preserves HTTP history and disables retained identities. Negative self-signed TLS rejection is separately verified on Windows/Linux Core-CLI. Migration012 and pre-v12 backup/preservation tests pass; existing projects, HAR/OpenAPI/schema011 records, AI settings and credentials are preserved. Fonts, layout and exactly Light/Dark/ReconX remain. Current proof: artifacts/replay-current-verification.json; Windows/Linux Core+CLI Rust 133 each, frontend 26, nine native suites/77 checkpoints, optimized Desktop/CLI and format/Clippy/types. See HTTP_REPLAY.md for commands and actual limits.

Overall Web/API remains PARTIAL. No automatic BOLA/BFLA, stateful execution, arbitrary remote replay, proxy/redirect-following, multipart, scanner or autonomous HTTP agent is claimed. Captured role comparisons remain observations. Full JSON Schema, Linux Desktop/Secret Service, trusted-CA HTTPS success fixture, live cloud tests, installers/signing and full v1.0 remain incomplete/unverified.

---

Older checkpoints below are retained for provenance; the current checkpoint above takes precedence.

## Current HTTP request/identity slice - 2026-10-18

RUNTIME_VERIFIED bounded manual loopback slice: imported request drafts, secure project/target/origin identity manager, exact review/approval, real GET/POST/PUT/PATCH/DELETE/HEAD/OPTIONS transport with policy gates, persistent job state/history/captures, cancellation and comparison/evidence/report integration. Actual native acceptance exercises GET/POST and two OS-backed Bearer identities; other auth methods use real HTTP core fixtures with a deterministic secret-store adapter. Saving never sends traffic or marks authenticated.

Migration012 and pre-v12 backup/preservation tests pass; existing projects, HAR/OpenAPI/schema011 records, AI settings and credentials are preserved. Fonts, layout and exactly Light/Dark/ReconX remain. Current proof: artifacts/replay-current-verification.json; Windows/Linux Core+CLI Rust 133 each, frontend 26, nine native suites/76 checkpoints, optimized Desktop/CLI and format/Clippy/types. See HTTP_REPLAY.md for commands and actual limits.

Overall Web/API remains PARTIAL. No automatic BOLA/BFLA, stateful execution, arbitrary remote replay, proxy/redirect-following, multipart, scanner or autonomous HTTP agent is claimed. Captured role comparisons remain observations. Full JSON Schema, Linux Desktop/Secret Service, dedicated HTTPS fixture, live cloud tests, installers/signing and full v1.0 remain incomplete/unverified.

---

Older checkpoints below are retained for provenance; the current checkpoint above takes precedence.

## Current offline schema review - 2026-10-18

RUNTIME_VERIFIED bounded slice: Desktop and CLI can select an imported capture/operation and run persistent, cancellable offline schema observation jobs through the existing worker. Original hashes and normalized input provenance, exact origin/target isolation, JSON type/required/properties/items, status/media/header/parameter/auth observations and explicit unsupported/inconclusive cases are implemented. Source locations link both evidence artifacts. A mismatch never automatically creates or verifies a vulnerability.

Migration 011 adds a job binding table and takes pre-v11 consistent backups for old schemas. User data/settings/themes/fonts are preserved. Current evidence: artifacts/schema-current-verification.json; Windows and Linux Core/CLI Rust 124 passed each (one ignored), frontend 23, eight Windows native suites/66 checkpoints, optimized Tauri/CLI and format/Clippy/types. See HTTP_SCHEMA_ANALYSIS.md for actual coverage and limits.

Overall Web/API remains PARTIAL: no full JSON Schema, active replay, credential-backed stateful authorization matrix or scanner automation. Root-relative origins and ambiguous older duplicate captures cannot be used for this new review. Linux Desktop, installers/signing and live cloud verification remain UNVERIFIED. Existing captured matrices remain observational.

---

Older checkpoint sections below are retained for provenance. The checkpoint above takes precedence.

## Current Web/API verification - 2026-10-18

The offline HAR/OpenAPI workspace is RUNTIME_VERIFIED as a bounded slice. The overall Web/API product remains PARTIAL. Native file picker/drop review, normalized traffic, target-aware endpoint inventory, captured role/identity matrix, real offline jobs, explicitly approved AI result review, integrity-linked comparison evidence, Needs Verification findings and four report formats share the existing Rust Core/SQLite services. Schema 010 is additive and snapshots older databases; no project/key/settings reset.

Current verification: 112 Windows Rust tests and 112 Linux Core/CLI tests passed (one ignored per platform); 22 frontend tests; format/Clippy/types; optimized Tauri/CLI; 8 native suites/64 checkpoints. Exact binaries: `artifacts/windows-web-api-20261018-105742`; `artifacts/http-current-verification.json` records source/binary hashes and actual result paths. Source-level async load races and wrong origin/target correlation were fixed with regression evidence.

The existing typography and exactly Light/Dark/ReconX are retained. No imported request is replayed; labels are captured contexts, not credential-backed test accounts. Request replay, authenticated comparisons, full schema validation, scanner automation, many research domains, Windows installer/signing, Linux Desktop/packages and live cloud verification remain incomplete. See WEB_API_WORKSPACE.md, TEST_RESULTS.md and the active master checkpoint. Older Linux-toolchain blocker notes below are historical: the isolated toolchain was restored and current Core/CLI tests passed.

---

Older checkpoint sections below are retained for provenance. The current checkpoint above takes precedence.

## Web/API continuation - 2026-10-18

Schema 010 and the shared offline HTTP service are implemented in the existing
working tree: HAR/base64 import, OpenAPI JSON/YAML/local references, endpoint
correlation, persistent traffic/notes, captured identity matrix, offline header
jobs, separately approved AI result interpretation, comparison evidence, review-only
findings and four-format reports. Desktop native picker/drop review and CLI reuse
the same backend. Original traffic is Restricted hashed evidence; normalized
records/default exports redact credentials. No request is automatically replayed.

Current Windows checks: 111 Rust passed, zero failed, one ignored; 21 frontend
passed; formatting, warnings-denied Clippy and TypeScript/Vite passed. Optimized
build/native regression are in progress. Linux recheck is currently BLOCKED:
the prior task-local Cargo toolchain no longer exists in Parrot WSL. Do not reuse
historical Linux counts as verification of this change.

The complete Web/API module remains PARTIAL: no active request editor/replay,
credential-backed identity execution, full JSON Schema validator or scanner adapter.
See WEB_API_WORKSPACE.md for limits/commands and MASTER_EXECUTION_PROGRESS.md for
the exact active checkpoint. Existing fonts, three themes and SECURITY.md edits
are preserved. Historical records below do not supersede this checkpoint.

## AI Provider Manager - current verified source (2026-10-17)

Implemented on HEAD `d6af47e`, as explicitly requested after the external working-tree
replacement. No reset, checkout, remote inspection, commit or push. This section
supersedes historical counts and next-task descriptions below; product version
remains 0.1.0 and full v1.0 release acceptance is incomplete.

- TESTED bounded AI settings slice: typed catalog of 24 providers with documented
  defaults/presets, searchable setup dialog, model dropdown, secure key entry,
  atomic save, enable/default/remove profiles, actual model inference tests,
  bounded model discovery/cache, immediate AI Workspace selection and agent routes.
- Custom protocols, authentication headers, private headers, explicit proxies,
  discovery paths and streaming preferences use the shared Rust service. Stored
  keys/header values remain in the OS secret store, never ordinary SQLite fields.
- Saving does not initiate inference or mark Connected. Health records the last
  explicit test, latency and error code. Key/config changes invalidate health;
  concurrent changes cannot publish an obsolete successful test.
- Additive migration `009_ai_provider_manager.sql` reserves interrupted 007/008
  numbers, snapshots older databases and preserves existing/unknown tables. Existing
  credential references, projects, settings, typography and three themes are retained.
- Windows: formatting, Clippy with warnings denied, 99 Rust tests passed (one
  explicit OS-store fixture ignored), frontend type/build and 17 tests passed.
  Actual Tauri optimized build passed, plus 7 native suites / 54 checkpoints.
- Linux Parrot/WSL: 99 shared Core/CLI Rust tests passed, one ignored. Linux Desktop,
  Secret Service runtime, optimized packaging and installers remain UNVERIFIED.
- Cloud account/model access is UNVERIFIED. Native inference used actual loopback
  mock servers, synthetic OS-stored keys and the running Tauri/WebView2 app.
  No personal evidence or provider credentials were uploaded. User DB was untouched.

Verified portable Desktop/CLI: `artifacts/windows-ai-providers-20261017-185028/`.
Machine-readable results/hashes: `artifacts/ai-manager-verification.json`.
Settings screenshots: `artifacts/native-ai-providers-1792237828887/` (three themes).
See `AI_PROVIDER_MANAGER.md` and `TEST_RESULTS.md` for setup, commands and evidence.

Remaining AI work: account-specific capability validation, richer pricing/usage,
rate/cost budgets, per-account discovery metadata, multi-hop data-sharing-aware
fallback and live opt-in provider verification. Current agents use text-based
plans; saved routes do not imply every future security agent is implemented.
Selected-artifact review rejects configured proxies until its consent explicitly
covers the intermediary. Existing destination-bound artifact consent stays enforced.

Next exact product task: implement bounded OFFLINE HAR import/endpoint inventory on
this current schema, followed by offline OpenAPI. Audit current assets/evidence/jobs
first. Do not restore interrupted source or repeat completed settings work.
No imported request replay, credentials upload or scanner automation is implied.

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
