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

## UX/extension checkpoint — 2026-10-19

Windows UX and one first-party YARA-X extension lifecycle are runtime verified;
the full marketplace/setup/updater/billing/OAuth assignment remains PARTIAL.
Windows and Linux Core/CLI Rust171/0/1 each; frontend33; final native25 checkpoints;
120 page/theme/size captures; actual CLI dependency lifecycle28 commands per host.
Schema014 preserves actual schema13 queued work/backups. Version0.1.0, v1 NOT READY.
Evidence: artifacts/ux-extensions-current-verification.json. Exact commands/files,
failures and continuation: MASTER_EXECUTION_PROGRESS.md. Next: bounded official MCP
Registry client/cache/pagination in Extensions, then approved verified package setup.
The live registry probe timed out; no registry connection/readiness is claimed.
Signed updater, account billing/OAuth, Linux GUI, installer and security gates stay open.

---

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

---

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

---

## Current offline regex checkpoint - 2026-10-19

Bounded pattern and 3.1 patternProperties now run through shared jobs, evidence,
Desktop and CLI. Specialized/unsupported regex syntax and limits stay explicit.
Final Windows and Parrot/WSL Core-CLI Rust: 157/0/1 each; frontend 26; native
HTTP/schema+outputs: 18 fresh checkpoints; optimized CLI/evidence/four-format
reports pass on both hosts. Exact scope/failures/commands/next action:
MASTER_EXECUTION_PROGRESS.md and artifacts/schema-pattern-current-verification.json.
No migration; schema 012/version 0.1.0. v1.0 remains NOT READY. Next: explicit,
default-off durable format assertion policy; Linux GUI/installers/signing and
glib/maintenance review remain unresolved. No push or release publication.

---

## Current checkpoint: bounded offline tuples, uniqueness and dependencies - 2026-10-18

Phase 1.5 advanced safely through the existing shared Core/Job Engine/Desktop/CLI.
uniqueItems now uses numeric/structural equality; OpenAPI 3.1 prefixItems/items
and dependentRequired/dependentSchemas retain exact source locations. Redacted,
invalid, cyclic and budget-limited cases remain explicit/inconclusive. No network
resolver, external replay, automatic finding verification or migration was added.
The schema remains 012, version 0.1.0, and v1.0 is NOT READY.

Current proof: `artifacts/schema-arrays-current-verification.json`. Windows Rust
150 passed/0 failed/1 ignored; Parrot/WSL Core+CLI 150/0/1; frontend 26 passed;
format, warning-denied Clippy, TypeScript/Vite and optimized Windows Desktop/CLI
and Linux CLI builds pass. Fresh native HTTP/schema, output and replay acceptance
passes 29 checkpoints across three suites. Other suites retain their earlier
historical proof and were not reclassified as freshly rerun. Native screenshots
of actual uniqueness/tuple observations were inspected. Separate optimized CLI
processes on Windows and Linux verified jobs, source evidence, persistence and
all four exports; pypdf 6.19.0 independently parsed both five-page PDFs/redaction.
Linux Desktop/Secret Service/packages and Windows installer lifecycle/signing
remain unverified; this checkpoint produced portable binaries, no new installer.

Fresh cargo-audit still reports one glib unsound warning and six maintenance
warnings; npm audit reports zero advisories. The explicit deny-unsound check exits
1 and dependency readiness stays BLOCKED. Earlier GitHub alert export is retained
as historical input, not a new remote verification. No advisory was suppressed.

Current unfinished task: complete remaining safe offline schema coverage
(patterns/patternProperties, formats with explicit assertion policy, resource/
multi-file resolution) with bounded CPU/memory and original evidence provenance.
Next exact action: inspect the existing JSON Schema library's regex limits and
dialect semantics, then implement the next bounded offline validator slice with
positive/negative/redaction/budget cases. Broader Web/API, recon, AI, tool/MCP,
reverse/mobile/cloud/network and release requirements remain PARTIAL/NOT_STARTED
as recorded in module-status.json and RELEASE_READINESS.md.

Exact changed files, commands, retained failures and next action: MASTER_EXECUTION_PROGRESS.md.

---

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
