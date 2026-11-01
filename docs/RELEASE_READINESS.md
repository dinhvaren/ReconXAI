## 2026-11-01 - reconciliation fairness follow-up accepted

Version **0.1.0**, schema **020**, **v1.0 NOT READY**. Source base `a50fb7dcc8df1fb937d27ca277dea62654d18e2b`.
This follow-up supersedes the historical starvation limitation described below;
no new migration or network/model capability is introduced.
Proof: [agent-reconciliation-20261101.json](verification/agent-reconciliation-20261101.json).

Changed Core reconciliation so stable paused/terminal history does not consume
all 100 sweep slots. Actual current-attempt terminal changes, pending terminal
cleanup and exhausted budgets are prioritized. Session reads/resume reconcile
that exact ID independently of the global sweep. The existing queue, approval,
concurrency and policy gates remain authoritative.

Actual regressions create 105 prior paused/cancelled DAGs and 101 idle running
DAGs. New real jobs advance without UI/getter polling; old records neither execute
nor change revision. A completed scope task beyond the old window advances its
real dependencies. A test-only exhausted quota counter stops further work and
cleans the current queue without a targeted read. Cancellation during a real
pause request survives reopen without partial evidence or implicit retry.

Source tests/builds completed 2026-10-30; native and Linux CLI acceptance resumed
2026-11-01 on those exact source hashes and copied executables.

Final verification: **218 passed,0 failed,2 opt-in ignored on each Windows and
Linux Core/CLI**; **50 frontend tests**, TypeScript/Vite, fmt and warning-denied
Clippy pass. Native groups remain separate:
agents: 6, recon-tools: 13, http-replay: 11, outputs: 6, recon-local: 7.
Real PE/ELF CLI acceptance passes on each platform; 81 Rust/SQL source files match.
Retained failure: `artifacts/agents-reconcile-final-windows.log` records LNK1104
from recompilation while an older test executable remained open. Older execution
was allowed to exit; the serial full final-source run passed. No timeout or
security policy was weakened to pass tests.
The first fresh native HTTPX health check also failed after verified installation.
Actual Doctor and another complete native installation/recon fixture passed.
Cause remains unconfirmed; see the retained failure record in the proof.
This is not treated as certified production stability.

New unsigned candidates, prior accepted candidates preserved:
`artifacts/windows-agent-reconcile-final-20261030/{reconx-desktop.exe,reconx.exe}`;
`artifacts/linux-agent-reconcile-final-20261030/reconx`. Manifests/checksums included.
Final local Git delivery: `artifacts/agents-reconcile-git-delivery.json` when present.

Source/test changes: `crates/reconx-core/src/agent_runtime.rs`,
`crates/reconx-core/tests/agent_runtime.rs`; DAG guide and status/proof documents.
Reproduce full checks with the commands in the proof. Use the candidate executable
paths above for `scripts/agent_runtime_smoke.py` and existing native suites;
`node tests/e2e/agents.mjs` also checks the currently embedded frontend asset.
Use a fresh fixture output directory, never a real project database.

Overall multi-agent/autonomous workflow remains PARTIAL: no automatic replanning,
new recording-policy engine, worker cloud token/cost budget, general conditional
DAG or new Web/API worker. Linux GUI, installers/signing/live cloud and existing
dependency release gates remain open. A 100-session eligible sweep is still bounded;
the 105/101 tests do not certify large-workspace latency or throughput.
Next safe task: opt-in recording of ReconX-owned HTTP traffic through existing
capture services, with per-project/session policy, redaction, bounded bodies,
quota/retention and restart tests. No proxy interception or CA installation.
No background work continues after the session ends.

---

## 2026-10-30 — reviewed offline agent DAG accepted

Version **0.1.0**, schema **020**, **v1.0 NOT READY**. This checkpoint supersedes
older statements that no offline agent scheduling or job-boundary pause exists.
Source base `0b8e0f8acc77db9b8e73249d35c11be26075e6d2`; exact source hashes, commands, retained failures and candidate
checksums: [offline-agents-20261030.json](verification/offline-agents-20261030.json).
Full local evidence: `artifacts/agents-current-verification.json`.

Implemented a reviewed offline vertical slice over the existing Job Engine:
scope review → independent selected file analysis → SHA-256 evidence validation
→ actual Report Engine snapshot. Six typed role descriptors record real planning,
coordination and worker responsibilities; they are not six LLM personas. One
IMMEDIATE transaction binds exact reviewed dependencies/limits to existing jobs.
Schema020 adds coordination/attempt/message/state history without replacing data.
Atomic claims enforce dependencies, current attempts, concurrency and budgets.
Pause drains current jobs; resume revalidates scope and never repeats completed
work. Interrupted work needs explicit bounded retry; historical attempt IDs persist.
A full event journal cannot prevent cancellation. CLI `jobs run` now returns
when only paused DAG jobs remain. Desktop/CLI share the same runtime and graph.

A real ELF release test exposed PDF_GLYPH on extracted Unicode. PDF now displays
unsupported characters explicitly as `[U+XXXX]` with a note; evidence and approved
Unicode snapshots remain unchanged. No product timeout was relaxed.

| Verification on final source/candidate | Actual result | Evidence |
|---|---|---|
| Windows Rust workspace | 215 passed, 0 failed, 2 opt-in ignored | `artifacts/agents-windows-post-pdf.log` |
| Linux Core/CLI, Parrot/WSL | 215 passed, 0 failed, 2 ignored; release CLI succeeds | `artifacts/agents-linux-post-pdf.log` |
| fmt / Windows+Linux warning-denied Clippy | Pass | `agents-fmt-final`, `agents-post-pdf-clippy`, Linux log |
| Frontend / TypeScript / Vite | 50 passed; production build passes | `artifacts/agents-frontend-current.log`, build log |
| Real CLI workflows | 4 acceptance groups/platform, owned PE/ELF and one MOCK planning call | Windows/Linux release-acceptance results |
| Windows native offline DAG | 6 groups; actual tasks, approval, graph, restart and resume | `artifacts\offline-agents-native-1793329952904\results.json` |
| Native existing recon/HTTP/output/HEAD | 13, 11, 6, 7 groups, respectively; separately executed | `artifacts/agents-final-native.json` |
| Source parity | 81 Rust/SQL files match byte hashes | `artifacts/agents-linux-source-comparison.json` |

Unsigned development candidates (previous candidates preserved):
`artifacts/windows-offline-agents-20261030/{reconx-desktop.exe,reconx.exe}` and
`artifacts/linux-offline-agents-20261030/reconx`, with manifests/checksums.
No installer, signing, Linux GUI or live cloud inference is verified here.

Retained failures: unbounded parallel Windows tests plus builds caused one old
schema expansion job timeout; the isolated test passed in 12.17 seconds and final
complete suites pass with two test threads. Product timeout remains unchanged.
The first real ELF report failed on PDF glyph coverage; fixed and retested on
final source. Harness mistakes (workflow tasks returns a list; initial Linux
script BOM/CRLF) are retained locally, not reported as product successes.

Overall multi-agent/autonomous requirements remain **PARTIAL**. No new traffic
recorder, Web/API worker, general conditional DAG, worker model routing/token/cost
budget, autonomous replanning or offensive continuation is implemented. Runtime
network/model counts are zero by dispatch policy; cost is `null`, not a guessed
zero price. Pause is at job boundaries, not subprocess suspension. Reconciliation
is bounded to 100 noncompleted sessions/sweep; graph source history is capped.
Large-workspace scheduling is not certified. Existing dependency/signing/installer
release blockers remain; advisory data was not refreshed.

Continuation: explicit opt-in recording of ReconX-owned HTTP traffic using existing
capture services, with project/session/job/evidence links, redaction, body/quota/
retention budgets and restart tests. Keep interception/CA trust changes disabled.
Runtime follow-up includes fair reconciliation beyond 100 sessions and targeted
running-cancel/large-history stress acceptance. General autonomous offensive
chains are outside this implemented slice.

Reproduce (use fresh fixture output paths):

```powershell
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -j2 -- -D warnings
cargo test --locked --workspace -j2 -- --test-threads=2
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo build --locked --release -p reconx -p reconx-desktop --features reconx-desktop/custom-protocol -j2
python scripts/agent_runtime_smoke.py --binary artifacts/windows-offline-agents-20261030/reconx.exe --sample artifacts/agent-benign.exe --output artifacts/agents-repro-UNUSED
$env:RECONX_DESKTOP_EXE='D:\ReconXAI\artifacts\windows-offline-agents-20261030\reconx-desktop.exe'
$env:RECONX_CLI_EXE='D:\ReconXAI\artifacts\windows-offline-agents-20261030\reconx.exe'
$env:RECONX_VERIFY_ASSET='1'
node tests/e2e/agents.mjs
```

Files/architecture: `crates/reconx-core/src/agent_runtime.rs`, `jobs.rs`,
`research_sessions.rs`, `research_graph.rs`, `reporting.rs`, storage migration020,
Tauri/CLI handlers, ResearchPage/AgentRuntimePanel/ResearchGraph, tests and CI.
See MULTI_AGENT_ARCHITECTURE.md, DAG_SCHEDULER.md, AUTONOMOUS_RESEARCH.md,
EXPLORATION_GRAPH.md, TRAFFIC_RECORDING.md and UNIFIED_TOOL_EXECUTION.md.
Final local Git delivery is recorded in `artifacts/agents-git-delivery.json`.
No background execution continues after the session ends.

---

## Current follow-up checkpoint — 2026-10-29: resilient provenance history

Version **0.1.0**, schema **019**, **v1.0 NOT READY**. This follow-up supersedes
only the historical graph limitation in the checkpoint below. Source base
`15f25820bd28e3f32a613957bd78ec91504209d0`; exact source hashes, commands and runtime records:
[research-history-20261029.json](verification/research-history-20261029.json).
Local proof: `artifacts/research-current-verification.json`; final Git delivery:
`artifacts/research-git-delivery.json` when present. No new migration.

Completed: graph review/export survives removed target and imported-file records;
original IDs, actual job/evidence links and observed assets persist. Missing
sources are explicitly marked `source_state: removed`; a never-projected missing
file has no fabricated name/hash/specifications. Historical metadata cannot grant
execution approval. The actual Desktop history selector now opens removed-target
sessions without an active target; new finding/report/AI/approval actions remain
disabled, while existing evidence, graphs and reports remain readable. Delayed
history reads cannot cross a project change. Cross-project file changes fail closed and preserve the last
snapshot. Six concurrent readers converge on one revision. No real user record
was deleted: removal tests use disposable databases or a backup of an owned fixture.

| Verification on this follow-up | Actual result | Evidence |
|---|---|---|
| Windows Rust workspace / Clippy / fmt | 204 passed, 0 failed, 2 opt-in ignored; both checks pass | `artifacts/research-history-windows-commands.json` |
| Linux Core/CLI | 204 passed, 0 failed, 2 ignored; Clippy and release CLI pass; 78 Rust/SQL files match | `artifacts/research-history-linux.log`, source-comparison JSON |
| Frontend / TypeScript / Vite | 46 passed; production build passes | `artifacts/research-history-ui-final-frontend-tests.log`, frontend-build log |
| Windows native research/recon/graph | 13 acceptance groups, including actual historical-record IPC; 35 captures | `artifacts/recon-tools-native-1793280820356/results.json` |
| Native HTTP / outputs / legacy HEAD | 11 / 6 / 7 groups, separately executed on this candidate | `artifacts/research-history-native-regressions.json` |
| Windows/Linux release graph CLI | 4 acceptance groups per platform; copied real observation/evidence survives target removal and restart | release fixture records in proof |

Current unsigned development candidates:
`artifacts/windows-research-history-ui-20261029/{reconx-desktop.exe,reconx.exe}`;
`artifacts/linux-research-history-20261029/reconx`. Checksums/manifests are included.
The earlier accepted candidates remain intact. Counts are per suite/platform,
not a sum of repeated tests. Native HTTP/output/HEAD suites are recorded separately with their own fixtures;
no checkpoint counts are added across repeated runs.

Changed code/tests: `crates/reconx-core/src/research_graph.rs`,
`crates/reconx-core/tests/research_graph.rs`, `scripts/research_graph_smoke.py`,
`tests/e2e/recon-tools.mjs`, `apps/desktop/src/pages/ResearchPage.tsx`,
`apps/desktop/src/Research.test.tsx`. Documentation/status/proof updates accompany them.
The broader five-capability request remains **PARTIAL**; this does not implement
multi-agent scheduling, a new recorder or an autonomous offensive loop. Linux GUI,
live cloud/passive source access, installers/signing/updater and existing dependency
security release gates remain open. Advisory data was not refreshed.

Next safe action: implement an explicit opt-in policy for recording ReconX-owned
HTTP traffic using the existing shared capture service, with project isolation,
redaction, body/retention budgets and real job/session/evidence references. Keep
proxy interception and CA trust changes disabled. Further graph work includes
large-session budget failures/complete transition history; removed-source handling
is now covered and should not be reimplemented. No background work continues
after the session ends.

Reproduce this follow-up:

```powershell
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -j2 -- -D warnings
cargo test --locked --workspace -j2
npm --prefix apps/desktop test
npm --prefix apps/desktop run build
cargo build --locked --release -p reconx -p reconx-desktop --features reconx-desktop/custom-protocol -j2
$env:RECONX_DESKTOP_EXE = 'D:\ReconXAI\artifacts\windows-research-history-ui-20261029\reconx-desktop.exe'
$env:RECONX_CLI_EXE = 'D:\ReconXAI\artifacts\windows-research-history-ui-20261029\reconx.exe'
$env:RECONX_VERIFY_ASSET = '1'
node tests/e2e/recon-tools.mjs
python scripts/research_graph_smoke.py --binary $env:RECONX_CLI_EXE --source artifacts/research-graph-recon-release-acceptance/workspace --output artifacts/history-repro-UNUSED
```

Use a fresh output path. Linux exact build/test command and staged source hashes
are in the proof; Linux Desktop is not inferred from Core/CLI success.

## Current verified checkpoint — 2026-10-29

Version **0.1.0**, SQLite schema **019**, **v1.0 NOT READY**. This supersedes
older checkpoints below. Source base `c920130bab108a55f228b69789d47caeb8e8850f`;
verification ran on the preserved working tree, with source hashes recorded in
[research-provenance-20261029.json](verification/research-provenance-20261029.json).
Local full proof: `artifacts/research-current-verification.json`. No normal user
workspace/credential was reset, sample executed or public target scanned.

Implemented in this slice:
- Pinned real HTTPX/Subfinder adapters, official archive/executable hashes, exact
  pinned PATH reuse, private version/execution config, real local GET/source-list
  health, safe receipt-owned repair and preservation of unexpected destinations.
- Reviewed typed jobs, exact target/scope/tool approval, IP-pinned HTTPX transport,
  bounded output/timeout/cancellation, inventory/evidence, shared Desktop/CLI and
  target research sessions. Migration018 preserves prior session/selection data.
- Read-only persistent asset/exploration provenance over actual sessions, plans,
  approvals, jobs, inventory, evidence, manual findings and reports. Migration019
  adds graph tables, not a queue. Graph review cannot execute tools or AI.
- Research Session graph filters/search/pagination, keyboard/pointer pan, zoom,
  neighborhood focus, source inspection, JSON export and stable restart IDs.
  Local import paths/URL credentials are omitted from graph metadata; source
  evidence integrity remains a separate check. Marketplace search selection and
  duplicate raw/normalized evidence storage were fixed, not hidden by test changes.

| Current verification | Actual result | Evidence |
|---|---|---|
| Windows Rust workspace | 202 passed, 0 failed, 2 explicit opt-in ignores | `artifacts/research-graph-final-rust.log` |
| Parrot/WSL Core + CLI | 202 passed, 0 failed, 2 ignored; 78 Rust/SQL files match | `artifacts/research-graph-linux.log`, source comparison JSON |
| Frontend / TS / Vite | 44 passed; current production build passes | `research-graph-final-frontend-tests.log`, `research-graph-readable-frontend-build.log` |
| Formatting / Clippy | rustfmt check + warning-denied Windows/Linux Clippy pass | proof command ledger; `artifacts/research-final-fmt.log` |
| Windows native research/recon/graph | 12 acceptance groups, 33 actual captures | `artifacts/recon-tools-native-1793259682347/results.json` |
| Native HTTP / outputs regression | 11 / 6 acceptance groups, separately recorded | `artifacts/http-replay-1793259879215/results.json`, `artifacts/outputs-1793259890313/results.json` |
| Legacy manual HEAD regression | 7 groups; current collapsed controls and existing data/exports work | `artifacts/recon-local-1793260674071/results.json` |
| Real tools CLI | 7 groups including repair; Windows/Linux actual owned HTTPX | release CLI fixture records in proof |
| Graph CLI / persistence | Actual schema018 fixture → 019, 16 nodes/17 edges, jobs unchanged, filters/export/backup | Windows/Linux graph acceptance JSON in proof |

Counts are per platform/suite, not summed across repeated runs. Mock planning and
interpretation remain **MOCK_VERIFIED**; no live cloud account claim. Subfinder's
real installation/version/source inventory and parser are tested, but live passive
domain enumeration is **NOT_TESTED** without an owned authorized domain/source.
Remote HTTPX/TLS runtime is unverified; certificate metadata is not a trust verdict.

Accepted development artifacts (unsigned; no installer/release publication):
`artifacts/windows-research-graph-final-20261029/{reconx-desktop.exe,reconx.exe}` and
`artifacts/linux-research-graph-20261029/reconx`. Each folder contains checksums and
a build manifest. Linux GUI remains **UNVERIFIED**; WSL Core/CLI is not GUI proof.

The five-capability request remains **PARTIAL**. No automatic offensive continuation
loop, multi-agent runtime/message bus, general DAG/pause/resume/budgets, universal
invocation envelope, new capture policy/proxy/CA or global asset graph is claimed.
Existing HAR/manual-loopback traffic capture works; a dedicated opt-in recorder is
not implemented. Graphs are bounded session projections refreshed on review, not
an inference system or complete transition-history engine. Refresh after target/
file removal remains a known gap. No hypothesis is automatically Verified.
The existing glib/RustSec, installer/signing/updater, broader domains and Linux GUI
release gates remain open; advisory data was not refreshed in this feature slice.

Guides: [RECON_TOOL_ADAPTERS.md](RECON_TOOL_ADAPTERS.md),
[EXPLORATION_GRAPH.md](EXPLORATION_GRAPH.md),
[UNIFIED_TOOL_EXECUTION.md](UNIFIED_TOOL_EXECUTION.md),
[AUTONOMOUS_RESEARCH.md](AUTONOMOUS_RESEARCH.md),
[MULTI_AGENT_ARCHITECTURE.md](MULTI_AGENT_ARCHITECTURE.md),
[TRAFFIC_RECORDING.md](TRAFFIC_RECORDING.md).

---

Earlier checkpoint history (not current test totals):

## Current product-recovery checkpoint — 2026-10-20

Version **0.1.0**, schema **017**, **v1.0 NOT READY**. This checkpoint supersedes
older results below without combining their test counts. Source base
`b041b667d6c918ce0e22c91f1c65222a2fe91aa2`; changes were uncommitted when verified.
Current evidence: `artifacts/product-recovery-current-verification.json`. No normal user database, credentials or imported
dataset was reset; no sample was executed and no release was published.
Repository verification metadata: [product-recovery-20261020.json](verification/product-recovery-20261020.json).
The local `artifacts/product-recovery-git-delivery.json`, when present, records the
final commit/push and working-tree state; the revision above is the test baseline.

The bounded local research and streaming-ingestion acceptance slices pass;
**the product release gate remains false**. These portable executables are internal
development candidates, not installer-certified or signed v1.0 artifacts. Full
hashes, command/platform/exit-code/timestamp evidence and source checks are in the
machine-readable proof. A passing functional suite does not clear dependency,
Linux GUI, installer or signing gates.

The shared SQLite project/target selection now uses persistent IDs and revision
checks. Start AI Research entry points use one Core session service and one view.
The session reviews metadata-only AI context, validates strict capability IDs,
consumes one-shot approval and atomically queues existing handlers. Supported
steps are offline scope review, local metadata/strings, available fixed YARA-X
and a separately approved owned-loopback HEAD. The exact HEAD destination and
one-request/5-second/no-redirect limits are visible before approval. Actual jobs,
logs, evidence, separately consented unverified AI interpretation, manual Needs
Verification findings and reviewed four-format reports stay linked. SQLite history
restores sessions even when WebView localStorage did not flush. Jobs/Workflows file
selection updates the same target; project changes fence stale async results.

Streaming ingestion is independent of deep parsing: paths cross IPC, copy/hash runs
in 256 KiB chunks with disk checks, source-mutation checks, progress, cancellation
and atomic publication. The import budget defaults to 8 GiB (configurable policy
ceiling 64 GiB); actual runtime sizes are verified only through 1 GiB. Files above
64 MiB retain complete bytes/hash, stream full entropy and use a 1 MiB strings/hex
prefix. Unknown section counts are null and unsupported deep tabs are disabled.
Deep parsing, fixed YARA-X and full byte comparison retain separate 64 MiB budgets.
Native picker, drop and folder paths share Core; native acceptance exercises the
picker, running cancellation, project switching and restart. No new migration after
017; checked i64/u64 decoding fixes large-file size preservation.

Guides: [AI_RESEARCH_SESSIONS.md](AI_RESEARCH_SESSIONS.md) and
[STREAMING_FILE_IMPORT.md](STREAMING_FILE_IMPORT.md).

These passes do not complete the broad workbench. Live cloud AI and actual local
inference engines are unverified (research AI tests use deterministic owned mocks).
General remote replay, active authorization matrix, stateful APIs, full schema,
DNS/RDAP/CT/scanners, advanced analyzers/mobile/cloud/network/lab, general DAG and
standing auto-policy, pause/resume, budgets/billing/OAuth, two useful Marketplace
analyzer categories, first-run dependency setup, signed updater, clean installer
lifecycle and Linux GUI remain PARTIAL/BLOCKED/UNVERIFIED. The known glib unsound
and maintenance security gate remains unresolved; advisory data was not refreshed
in this functional slice. Windows binaries are unsigned.

Large-file limits remain explicit: forced termination may leave private import
staging; no automatic crash-staging GC/resume, general workspace quota, complete
large-file decompilation or OS process sandbox. Custom Windows ACL hardening is
still open. The configurable 64 GiB ceiling is not runtime verification at 64 GiB.
Authorized Auto currently means reviewed offline steps, not reusable standing
authorization or uncontrolled scanning. Missing usage/cost is unavailable, never
fabricated as zero. MCP same-user subprocesses are not sandboxed.

---

Historical checkpoints below retain their original scope and dates.

## Product recovery: target-bound research — 2026-10-20

Version **0.1.0**, schema **017**, **v1.0 NOT READY**. This checkpoint supersedes
older counts below; historical sections remain historical. Source base b041b66;
changes are uncommitted. No reset, normal user-data cleanup, commit, push or release.

Implemented and runtime verified: shared persisted project/target IDs with revision
checks; Start AI Research entry points; one integrated session view; metadata-only
AI planning consent; strict capability-ID plans; exact one-shot approval and atomic
job batch; separately approved owned-loopback HEAD; real local metadata/strings;
existing YARA-X availability detection; actual progress/logs; evidence; separately
reviewed unverified AI interpretation; manual Needs Verification finding; report
preview and four existing export formats. Desktop and CLI use the same Core and DB.
No arbitrary shell/MCP/scanner execution or automatic Verified finding was added.

Proof: `artifacts/research-current-verification.json`. Accepted portable Windows
pair: `artifacts/windows-research-current-20261020`; Linux CLI:
`artifacts/linux-cli-research-20261020/reconx`. Both Windows executables are
actually NotSigned. Native tests assert the embedded frontend asset matches current
Vite output; earlier candidates are preserved. Native screenshots are genuine
WebView2 captures; no user attachment was available for pixel comparison.

Current results: Windows Rust **186 passed / 0 failed / 2 ignored**; Linux Core/CLI
**186/0/2**; frontend **39 passed**. Warning-denied Clippy both hosts; rustfmt and
TypeScript/Vite pass. Native current build: research **8**, HTTP regression **11**,
output regression **6** acceptance checks. Research captures cover exactly three
themes × four sizes (1280/1366/1440/1920), no document overflow; Light1280, Dark1366
and ReconX1920 captures were opened and inspected. Real CLI research acceptance is
**36 commands per host**, including approved fixture HEAD, mock AI plan/interpretation,
hash verification, real PE/ELF static jobs, review finding and four redacted reports.
These are separate suites, not a claim of broad domain coverage.

The previously pending cached-MCP-runtime fix also passes actual Windows negative
acceptance: **37 commands**, running install cancellation, previous Ready retention,
cached package binding to the newly approved managed Node path/hash, actual MCP
health and failed corrupted-package update rollback. Evidence:
`artifacts/mcp-cli-research-negative-20261020/negative-results.json`.

Limits: only listed bounded local capabilities; domain registration alone is not
recon/scanning. Remote replay, active authz matrix, stateful APIs, multi-domain DAG,
pause/resume, reusable standing auto-policy, accounting/cost budgets, live cloud
accounts, actual Ollama/LM Studio/vLLM engines, advanced decompiler/mobile/cloud/network
adapters, two useful marketplace analyzer categories, setup/installer/signed updater
and Linux GUI acceptance remain incomplete or unverified. AI tests use owned local
deterministic protocol fixtures only. The known glib/dependency security release
gate remains unresolved; this checkpoint did not rerun the advisory audit.

**Next P0:** replace the 64 MiB ingestion architecture with bounded streaming copy
and SHA-256, cancellable progress, disk-space checks, atomic publication and u64
storage. Preserve a separate bounded deep-parser budget. Current `reverse.rs`
`import_file` still reads/inspects the entire <=64 MiB input, `list_files` still
decodes size through u32, and `prepare_file_analysis` calls the bounded sample
reader. No 80/256/512 MiB or 1 GiB import pass is claimed. Do not merely enlarge
MAX_FILE_BYTES or pass raw file bytes through React/IPC.

---

## MCP Registry / reviewed npm runtime checkpoint — 2026-10-19

Version **0.1.0**, schema **016**; **v1.0 NOT READY**. A bounded Registry + npm
installation slice is runtime verified. A subsequent cached-runtime selection fix
is currently rebuilding/retesting; the accepted binaries below remain preserved.
No commit/push/publication. Normal user data/credentials were not opened for tests.

Authoritative bounded proof: artifacts/mcp-current-verification.json. Windows pair:
artifacts/windows-mcp-candidate-1792415560; Linux CLI:
artifacts/linux-cli-mcp-1792415142/reconx. Windows binaries are actually NotSigned.
The OneDrive Desktop ReconX AI.lnk points to this tested Windows pair; record:
artifacts/desktop-shortcut-mcp.json. Earlier schema014 candidates are retained.

Implemented: actual official v0.1 Registry search/details/version listing, encoded
IDs/cursors, pagination, runtime filters, bounded HTTP/schema/retry policies and
SQLite offline/stale cache. Actual public Core CLI search(2/page), encoded detail,
version lookup, next cursor, offline and clear all pass (6 calls):
artifacts/registry-core-live-1792415328. Native Registry live/offline also passes:
artifacts/registry-native-1792415690918. Public metadata is untrusted; no package
execution is authorized by registry presence. Unknown OS/schema/install methods
stay explicit. Version-history UI currently displays one page.

Reviewed npm adapter reconx.mcp-everything installs official MCP Everything1/2
from compiled locks (108/94 tarballs), validates SHA-512, safely extracts without
npm scripts, reuses compatible Node>=24 or automatically installs pinned portable
Node24.22.0 with archive/executable SHA-256, checks real free space, and uses the
existing persistent Job Engine. Actual initialize/list_tools/schema health precedes
atomic activation. Generated MCP config has no secrets and starts with no allowed
tools. Desktop permission approval and exact-input invocation approval are separate.
Actual Echo output persists; no automatic tool calls or fabricated findings.
Everything is a protocol test server, not a scanner or Ghidra/Burp integration.

Schema015 adds bounded metadata cache;016 adds managed package descriptors and
installation operations/history. Previous configurations survive failed/cancelled
activation; user-edited configs fail reconciliation rather than being overwritten.
Enable/disable/remove, integrity checks, reviewed updates/rollback and shared-runtime
retention reuse Core state. Legacy YARA APIs explicitly reject managed MCP IDs.
CLI mcp install/doctor, catalog, tools dependencies and extension permissions share
the same backend. JSON null operations now emit machine-readable null.

Verified before the last cache-runtime fix: Windows Rust180/0/2 and Linux Core/CLI
180/0/2; warning-denied Clippy both hosts; rustfmt, TypeScript/Vite; frontend36/0.
76 Rust/SQL/manifest/lock source files match the Linux snapshot. Native current
Windows acceptance30 checks: managed MCP4 + Registry1 + preserved YARA5 + HTTP14 +
outputs6. Actual CLI lifecycle27 commands per host covers real package installation,
Echo, restart, disable/enable, health,2→1→2, file-tamper rejection, managed removal,
retained history and actual schema14→16 backup. Windows native separately forces
managed Node installation; Linux detects Node20 as incompatible and auto-installs24.
Native screenshots of all three themes and the installation dialog were viewed.
Frontend and both reviewed npm lock audits report zero vulnerabilities; historical
glib/maintenance backlog still blocks release (no new cargo audit claim).

Failures/fixes: Registry upstream timeouts reproduced with Rust/curl/Python; bounded
30s attempts and retries eventually passed actual API acceptance. Throttled excessive
per-chunk logs and corrected total-byte reporting. Windows shared-load15s MCP timeout
was real; removed duplicate tree hashing and set managed MCP60s bounded timeout,
then real CLI/native Echo passed. Native test corrected Connect/Discover capitalization.
Actual screenshot exposed missing dialog padding; fixed shared-body spacing/width.
Linux native filesystem testing avoids Windows-mounted-filesystem metadata overhead.
The last discovered issue was cached package receipts retaining a previously selected
Node; source now rebinds receipt to the exact approved verified runtime atomically.

Current unfinished task: finish acceptance of that cached-runtime fix. Running logs:
artifacts/mcp-runtime-fix-{check,test,release}.log and the log named by
artifacts/mcp-linux-current-log.txt. Once built, run
scripts/managed_mcp_failure_smoke.py --binary <new-cli> --data <isolated prior-smoke>
--approve-install on Windows and Linux; it tests actual nonzero-byte download
cancellation, exact managed runtime rebinding and failed-update preservation.
Then repeat native managed MCP acceptance using the new pair, update proof and
shortcut only after acceptance, and replace this in-progress paragraph.

Exact source files for this milestone: crates/reconx-core/src/{registry,managed_mcp,
extensions,jobs,mcp,lib}.rs; tests/managed_mcp.rs; crates/reconx-tools/src/{packages,
lib}.rs and Cargo.toml; Cargo.lock; crates/reconx-storage/src/lib.rs and migrations
015_mcp_registry.sql/016_managed_mcp.sql; apps/cli/src/main.rs;
apps/desktop/src-tauri/src/lib.rs; apps/desktop/src/{extensions.ts,extensions.css,
ManagedMcp.test.tsx,App.tsx,pages/ExtensionsPage.tsx,components/RegistryBrowser.tsx,
components/ManagedMcpDetails.tsx}; manifests/mcp/*; scripts/managed_mcp*.py;
schema16 fixture updates in scripts/{extension,http_schema}_smoke.py and
tests/e2e/extensions.mjs; new tests/e2e/{managed-mcp,registry}.mjs. Prior changes remain.

Commands: cargo fmt --all -- --check; cargo clippy --locked --workspace --all-targets
-j2 -- -D warnings; cargo test --locked --workspace -j2; cargo build --locked
--release -p reconx -p reconx-desktop --features reconx-desktop/custom-protocol -j2;
Linux --offline equivalents excluding Desktop; npm test/build/audit; npm audit
--package-lock-only for each reviewed package; scripts/managed_mcp_smoke.py;
tests/e2e/{managed-mcp,registry,extensions,http-workspace,outputs}.mjs. Exact artifacts
and hashes are in the proof. No fake pass is assigned to earlier failed iterations.

Remaining: generalized npm/PyPI/Cargo/OCI/MCPB installers and runtime constraints,
more reviewed integration adapters, large/paginated MCP catalogs/schema refs,
remote authentication/OAuth, OS sandbox, disk-full/permissions/crash recovery
acceptance, first-run profile wizard, installer lifecycle, Linux GUI/Secret Service,
signed updater/signing, billing/OAuth/local model runtimes and original research
backlog. No universal one-click installation or Linux Desktop claim.

# Release readiness — 2026-10-19

**ReconX AI 0.1.0 development build; schema014. v1.0 is NOT READY.**

Current verified slice: Windows UX/component/navigation refinement and one working
first-party YARA-X Marketplace adapter. Windows Rust171/0/1, Linux Core/CLI171/0/1,
frontend33/0, format/Clippy/TypeScript/Vite pass; npm audit zero. Final native
extension5 + HTTP/output20 checkpoints; 120 real page/theme/size captures were
checked and inspected. CLI actual extension lifecycle28 commands per host includes
real dependency execution, evidence and schema13→14 backup/queued-work preservation.
Full source/runtime proof: artifacts/ux-extensions-current-verification.json.

Windows portable: D:\ReconXAI\artifacts\windows-extensions-verified-20261019-172615.
Linux CLI: artifacts/linux-cli-extensions/reconx. Both are development candidates;
Windows binaries are actually NotSigned. Desktop shortcut points to the accepted
new Windows executable; no normal user DB was opened for testing.

| Product deliverable | Current status | Remaining acceptance |
|---|---|---|
| UI/components/theme/layout | RUNTIME_VERIFIED Windows slice | Linux GUI, physical DPI, remaining page/control accessibility |
| Extension Marketplace | PARTIAL; YARA-X lifecycle runtime verified | Community/MCP catalog, broad integrations, compatible version updates/rollback |
| Automatic tool setup | PARTIAL; real portable YARA-X on Windows/Linux CLI | General dependency graph/package managers, first-run profiles/wizard |
| Shared tool/permission state | IMPLEMENTED and tested bounded slice | OS sandbox, install cancellation, concurrent running-extension revocation acceptance |
| Application updater | NOT_STARTED; signed publication BLOCKED | Tauri updater, trusted feed/channel, signed fixture/download/install/restart |
| AI Usage/Billing | NOT_STARTED account layer | Official24-provider audit, permissions, local durable accounting and distinct costs |
| OAuth/local AI | Existing API keys/local protocol fixtures preserved | Authorized provider clients/PKCE/refresh and actual local-model runtime |
| Cross-platform release | BLOCKED / INCOMPLETE | Linux Desktop/Secret Service/packages, installer lifecycle/signing, glib and wider v1 gates |

Official registry read-only probe timed out; no client, installation or third-party
capability readiness is claimed. Full master ledger records exact files, commands,
failures/fixes, evidence and the next typed-registry-client task. Cargo.lock is
unchanged; historical glib/maintenance blockers remain. No commit/push/publication.

## Current conditional-schema checkpoint - 2026-10-19

Bounded OpenAPI 3.1 if/then/else now uses real captured predicates, selected-branch
provenance and the existing shared budgets/format policy. Windows and Parrot/WSL
Core+CLI Rust: 167/0/1 each; schema 41/storage 11; frontend 26; native HTTP/schema
and outputs: 20 fresh checkpoints. Optimized binaries, real schema-12 upgrades,
evidence/four-format reports, PDF inspection and restart pass on both CLI hosts.
Proof: artifacts/schema-conditional-current-verification.json; exact commands,
files, candidates and next multi-file-import action: MASTER_EXECUTION_PROGRESS.md.
No new migration; schema 013/version 0.1.0. v1.0 NOT READY: glib, Linux GUI,
installer/signing, full schema coverage and unfinished research modules remain.
No commit, push or release publication.


## Current opt-in format checkpoint - 2026-10-19

Default-off per-job syntactic formats, durable fingerprint policy and additive
schema-013 migration are runtime-verified. Windows and Parrot/WSL Core+CLI Rust:
163/0/1 each; schema 37/storage 11; frontend 26; native HTTP/schema+outputs: 19
fresh checkpoints. Real old-binary queued-job upgrades, consistent backups,
evidence, four report formats, independent PDF parsing and restart pass on both
hosts. Proof: artifacts/schema-format-current-verification.json; exact commands,
files, candidates, limitations and next action: MASTER_EXECUTION_PROGRESS.md.
Version 0.1.0; v1.0 NOT READY. glib/maintenance, Linux GUI, installer/signing and
unfinished modules remain unresolved. No commit, push or release publication.

**ReconX AI 0.1.0 development build. v1.0 is NOT READY.** Functional acceptance
does not override dependency, platform, installation or missing-feature gates.
The machine-readable feature/gap snapshot is [release-gaps.json](release-gaps.json).
Previous format/upgrade evidence: `artifacts/schema-format-current-verification.json`. Earlier whole-application evidence remains `artifacts/security-current-verification.json`.

| Gate | Status | Actual scope / remaining work |
|---|---|---|
| Existing shared Core/Desktop/CLI | TESTED | Windows native regressions; Linux Core/CLI only |
| Dependency security | BLOCKED | Eight pypdf High advisories patched and confirmed fixed by GitHub. glib unsoundness remains; six maintenance warnings require review |
| Offline HAR/OpenAPI | TESTED, bounded | Imports, capture comparison, scalar/size/uniqueness constraints, 3.1 tuple/dependent schemas, composition, 3.1 if/then/else, bounded pattern/3.1 patternProperties, evidence and report flow; opt-in bounded syntactic formats; advanced regex/custom formats/multi-file/resource resolution incomplete |
| HTTP replay/identities | PARTIAL | Individually approved loopback request only; general remote replay/stateful authorization matrix/workflow features incomplete |
| Recon | PARTIAL | Reviewed local HEAD inventory only; broad passive/active adapters absent |
| Reverse analysis | PARTIAL | Local static parsing and fixed YARA-X signatures; live debugger/disassembler/decompiler integrations absent |
| Tool Manager | PARTIAL | Reviewed ripgrep/YARA-X on Windows; broader adapters, package managers, dependency graphs and rollback incomplete |
| MCP | PARTIAL | Actual local stdio/HTTP fixtures and approved benign calls; third-party tool environments, OAuth and OS sandboxing unverified/incomplete |
| AI | PARTIAL / MOCK_VERIFIED | Provider setup, local protocol fixtures and reviewed local analysis plans; live cloud accounts, full agents/DAG/budgets/advanced fallback not verified |
| Evidence/Findings/Reports | TESTED, bounded | Traceable records, CVSS and four export formats; richer layouts/taxonomy/bulk/performance work remains |
| Other research domains | NOT_STARTED | Dedicated Network, Mobile, Cloud/Container, CI/CD and Red Team Lab workspaces absent |
| Storage preservation | TESTED, bounded | Transactional additive migrations through 014, real schema-12/13 queued-job upgrade/backups on both CLI hosts; read-only future/corrupt DB rejection; complete workspace backup/restore not implemented |
| Windows portable application | TESTED | Matching optimized Desktop/CLI pair; actual native workflow regression |
| Windows installer | IMPLEMENTED / UNVERIFIED installation | NSIS produced and hashed. No clean-environment install/upgrade/uninstall acceptance; unsigned |
| Linux optimized CLI | TESTED on Parrot/WSL2 | Real ELF analysis, persisted jobs/evidence, four report formats and separate-process persistence; other distributions unverified |
| Linux Desktop/packages | BLOCKED / UNVERIFIED | WSLg exists but GLib/GTK/WebKit SDK missing; GUI, Secret Service and packages not tested; glib advisory unresolved |
| Signing | BLOCKED | No validated signing infrastructure supplied |
| Complete product E2E/release | BLOCKED | Missing modules/platform gates above; no final v1.0 release or publication |

## Windows build candidate

`artifacts/windows-security-20261018-183806/` contains:

- `reconx-desktop.exe` and `reconx.exe`.
- `installer/ReconX AI_0.1.0_x64-setup.exe` (per-user NSIS build, NotSigned).
- `THIRD_PARTY_NOTICES.txt`, `SHA256SUMS.txt`, `build-manifest.json`.

Build manifest identifies actual hashes/tool versions and explicitly sets
`release_ready:false`, `clean_install_verified:false`. This directory is an
internal development candidate, not an installer-certified v1.0 distribution.
The normal Windows installation, registry, user DB and credentials were not
modified for installer testing. Existing WebView2 supports native acceptance;
first-run WebView2 provisioning on a clean machine is still unverified.
No release was published and no binary/user dataset is committed to Git.

Linux CLI candidate: `artifacts/linux-cli-security-20261018-185625/reconx`.
Its actual release-mode ELF acceptance is in `smoke/acceptance.json`; a separate
pypdf 6.19.0 inspection checked the Linux-generated three-page PDF's Vietnamese
text and redaction. Hashes, native compiler and dynamic libraries are recorded
beside the binary. This establishes the tested Parrot/WSL CLI slice only.

The initial remote workflow run rejected an unquoted pip flag containing a colon.
It was reproduced by a new Rust YAML parser regression and corrected with a
folded run string. The workflow's remote execution and known glib security-gate
failure are tracked independently from local functional acceptance.

## Dependency and credential assessment

See [DEPENDENCY_SECURITY.md](DEPENDENCY_SECURITY.md) for each of the nine actual
GitHub advisories, affected dependency path, reachability assessment and patch or
remaining blocker. Default cargo-audit exit 0 does not mean no unsound warning.
The explicit `cargo audit --deny unsound` gate returns 1 until glib is resolved;
CI intentionally reports this instead of ignoring or dismissing the advisory.
Source fixes and GitHub alert closure are independent fields in the recorded report.

A bounded artifact-text scan found no token/private-key/credential-URL matches in
its documented scope. It does not prove old copies or Git history are clean, and
it does not revoke any previously exposed credential. User-managed revocation,
same-account MCP/tool trust, custom Windows workspace ACLs and process isolation
remain distinct security concerns.

## Continuation

Resolve compatible glib/GTK safety remediation and verify it with an isolated
Linux SDK/runtime before promoting Linux Desktop. Continue bounded offline schema
coverage and evidence-linked manual policy annotations while upstream/environment
work is blocked. Keep all unfinished module and installation gates visible. Do not
label arbitrary target scans, autonomous authorization testing, third-party MCP
integrations or live cloud accounts verified based on local protocol fixtures.
Use MASTER_EXECUTION_PROGRESS.md for exact commands and the current handoff.

## Format slice verification

| Feature | Implementation / platforms | Unit and integration | Actual end-to-end proof | Limits / blockers |
|---|---|---|---|---|
| Opt-in formats | RUNTIME_VERIFIED; Windows Desktop/CLI, Parrot/WSL CLI | Schema suite 37, frontend 26; full Rust 163/0/1 each host | Native checkbox/invalid date/source pointer/restart; both optimized CLI smoke exports | Fourteen bounded syntactic formats only; custom and full assertion vocabulary unsupported |
| Schema-13 preservation | RUNTIME_VERIFIED; Windows and Parrot/WSL Core/CLI | Storage 11, legacy fingerprint and queued-policy tampering tests | Real schema-12 binaries queue jobs; new binaries back up/migrate/execute; backup integrity/FKs checked | No normal user DB opened; complete workspace restore and installer upgrade still pending |

Current portable candidates and exact commands: MASTER_EXECUTION_PROGRESS.md.

## Conditional slice verification

| Feature | Status / platforms | Tests | Runtime proof | Remaining limits |
|---|---|---|---|---|
| Bounded 3.1 if/then/else | RUNTIME_VERIFIED; Windows Desktop/CLI and Parrot/WSL CLI | Schema 41; four new regressions; full Rust 167/0/1 each | Native selected-branch pointer/screenshot and restart; both optimized CLI evidence/report exports | Three-way partial evaluation; no complete annotation tree or 3.0 conditional support |

Current proof: `artifacts/schema-conditional-current-verification.json`. Full JSON Schema, broader Phase 1 and whole-product v1.0 gates are still incomplete.
