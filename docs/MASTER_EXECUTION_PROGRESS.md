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
Local full proof: `artifacts/research-current-verification.json`. Final commit/push metadata, when present, is in
`artifacts/research-git-delivery.json`; the source revision above is the tested
base, not a claim that later commits were already tested. No normal user
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

### Exact safe continuation

1. Read this checkpoint/proof; preserve schema019 and accepted candidates. Do not
   repeat the completed recon/graph implementation. No background task continues
   after the session ends. Any CI run is separate and not claimed as remotely verified here.
2. Harden historical graph reads for removed targets/files, then test large-session
   budget failures/concurrent refreshes and improve complete-state provenance.
3. Implement explicit opt-in recording of ReconX-owned HTTP operations through the
   existing capture service: persistent per-project policy, bounded/redacted data,
   job/session/evidence/graph references and retention. No silent interception or
   root-CA installation. Preserve raw traffic only under an explicit protected policy.
4. Continue only non-offensive analysis coordination over existing typed local
   handlers, with real job/evidence provenance and truthful feature availability.
   No unrestricted offensive autonomous loop or generated shell execution.
5. Keep live cloud, external-domain source tests, Linux GUI, signing/clean installer,
   dependency-security and remaining-domain gates explicit; they are not waived.

Reproduction (isolated fixtures, no user datasets):

```powershell
$env:CARGO_INCREMENTAL='0'
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -j2 -- -D warnings
cargo test --locked --workspace -j2
npm --prefix apps/desktop run build
npm --prefix apps/desktop test -- --run
# Point RECONX_DESKTOP_EXE / RECONX_CLI_EXE at accepted independent copies.
# Put only the hash-verified pinned tools on the child test PATH.
$env:RECONX_VERIFY_ASSET='1'
node tests/e2e/recon-tools.mjs
```

CLI graph reproduction uses `scripts/research_graph_smoke.py --binary <CLI>
--source <owned-recon-fixture-workspace> --output <new-private-directory>`; it
creates a consistent copy and never mutates the source fixture. Real tool/recon
acceptance uses `scripts/recon_tools_smoke.py` with an approved pinned tool path or
explicit approved official download. A graph command never schedules new work.

Recent failed attempts are retained in the proof: a fixture locked the build's
CLI executable (fixed by using independent copies), MCP's 500 ms test-start budget
was flaky under load (now proves invocation + JOB_TIMEOUT), and Marketplace search
selected the wrong detail panel (fixed and regression tested). No failures were
relabelled as passes. The current suites above were rerun successfully.

### Files and decisions

Core: `recon_tools.rs`, `research_graph.rs`, existing jobs/research_sessions,
extensions/evidence/capabilities/tooling and shared reexports. Tools: compiled
manifests, private bounded process adapter and safe adoption/repair. Storage:
additive migrations018/019 and schema/backup preservation tests. Desktop/CLI:
typed command wiring, actual recon inventory/review controls, graph inspector and
native install progress. Tests: parser/policy/dependency/provenance/security,
frontend stale-result handling and actual native acceptance; no fake production
results. Detailed hashes/commands/platform timestamps are in the proof. Local
artifacts/data are excluded from source control; no release was published.

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

| Current verification | Actual result | Evidence |
|---|---|---|
| Windows workspace Rust | 191 passed, 0 failed, 2 explicitly ignored | `artifacts/large-import-rust-verified.log` |
| Parrot/WSL Core + CLI Rust | 191 passed, 0 failed, 2 explicitly ignored; 71 Rust/SQL files match staged source | `artifacts/large-import-linux-final.log` |
| Clippy and rustfmt | Warning-denied Windows/Linux Clippy; Windows rustfmt pass | `large-import-clippy-verified.log`, Linux log, `recovery-combined-fmt.log` in artifacts |
| Frontend / TypeScript / Vite | 39 passed; current production frontend build passes | `artifacts/recovery-final-frontend-tests.log`, `recovery-final-frontend-build.log` |
| Windows optimized Desktop + CLI | Actual portable executables, both NotSigned | `artifacts/windows-product-recovery-20261020-ux-final`, `artifacts/product-recovery-final-assets.log` |
| Linux optimized CLI | Actual ELF; Core/CLI acceptance only | `artifacts/linux-cli-large-import-final-20261020/reconx` |
| Native integrated research | 9 acceptance checks | `artifacts/research-session-1792506323444/results.json` |
| Native large-file import | 6 acceptance checks | `artifacts/large-import-native-1792506310716/results.json` |
| Native HTTP regression | 11 acceptance checks | `artifacts/http-replay-1792506338490/results.json` |
| Native evidence/findings/reports | 6 acceptance checks; independent PDF text/redaction inspection | `artifacts/outputs-1792506349760/results.json` |
| Native shell UX | 216 page/theme/size observations; 3 shell/theme checks | `artifacts/ux-after-1792506362426/results.json` |
| Real research CLI | 36 commands on each host; mock planning + real HEAD/static jobs + four reports | `artifacts/research-cli-{windows,linux}-combined-20261020/results.json` |
| Large CLI fixtures | 80/256/512/1024 MiB; independent source/blob hashes; current largest metadata job; Linux running SIGINT | `artifacts/large-import-cli-windows-20261020/results.json`, `large-import-cli-linux-final-proof-20261020/results.json` |
| Actual schema-16 → 17 CLI upgrade | 14 commands each host; old queued job/evidence preserved; valid exact backup; old client rejects newer DB without byte changes | `artifacts/research-upgrade-{windows,linux}-20261020/results.json` |

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

### Diagnostics and visual evidence

The first expanded native research assertion read the top-bar target before the
independent CLI snapshot refreshed. Its failure screenshot/log remain retained;
the corrected test waits for both displayed and SQLite-persisted IDs and passes.
A worker-start timeout run executed zero frontend tests under concurrent heavy
linking; bounded two-worker reruns pass all39. A source-edit/compile race diagnostic
is superseded by a fresh full191-test run. None of these failed attempts is counted
as a pass. Exact retained paths are in the verification JSON.

Current research and large-file views each have 12 actual captures (three themes,
1280×720/1366×768/1440×900/1920×1080). The shell UX run adds 216 observations over
18 pages at those same themes/sizes, checks sampled control heights30/34/38,
disabled explanations and overflow, plus theme keyboard/restart and emulated
raster scaling. Five representative final images (large Light1280, research
Dark1366 and ReconX1920, AI Workspace Dark1366 and Targets Light1280) were opened
and inspected; not every capture was manually
inspected. No user screenshot attachment was available for pixel comparison.
The existing typography and exactly three themes are preserved.

### Exact continuation

1. Preserve this accepted pair/proof and schema017. Do not repeat streaming or
   session implementation; no normal user dataset/credential migration is needed.
2. Finish Milestone 3 UX/context audit for modal editing, target switching and
   keyboard navigation in populated workspaces; the current 216 observations
   verify measured controls/overflow, not every workflow or physical display DPI.
3. Milestone 4: implement a second useful reviewed integration in another category,
   prioritizing native HTTPX: official release/source/checksum review, installer
   manifest, fixed typed approved loopback probe, timeout/cancel, normalized assets
   and evidence, Desktop/CLI acceptance. Registry metadata or `--version` alone
   must not mark an execution adapter Ready. External replay remains denied until
   its separate scope/DNS/redirect policy is actually implemented and tested.
4. Then continue the requested DAG/agent, accounting/local-engine, installer/updater
   and remaining-domain phases. Keep all release/security blockers visible.

Re-run only checks justified by subsequent changes. Reproduction:

```powershell
$env:CARGO_INCREMENTAL='0'
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -j2 -- -D warnings
cargo test --locked --workspace -j2
Push-Location apps/desktop
npm run build
npm test
Pop-Location
# Rebuild embedded assets after frontend edits; use a fresh candidate directory.
(Get-Item apps/desktop/src-tauri/src/lib.rs).LastWriteTime=Get-Date
cargo build --locked --release -p reconx -p reconx-desktop --features reconx-desktop/custom-protocol -j2
$env:RECONX_CLI_EXE='D:\ReconXAI\artifacts\windows-product-recovery-20261020-ux-final\reconx.exe'
$env:RECONX_DESKTOP_EXE='D:\ReconXAI\artifacts\windows-product-recovery-20261020-ux-final\reconx-desktop.exe'
$env:RECONX_VERIFY_ASSET='1'
node tests/e2e/research-session.mjs
node tests/e2e/large-import.mjs
node tests/e2e/http-replay.mjs
node tests/e2e/outputs.mjs
node tests/e2e/ux.mjs after
```

Every native harness uses its own workspace and spawned PID; do not close the
user's normal application. Use `wsl.exe` explicitly: a shell profile's `wsl`
function can mask execution. Linux staged source and exact commands/logs are in
the machine-readable proof; no Linux GUI pass is claimed. Large fixture directories
are isolated and retained. Avoid recreating multi-GiB fixture copies needlessly;
`large_import_smoke.py --resume --reanalyze` reuses its explicitly validated fixture.

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

### Changes and continuation

New Core `research_sessions.rs`, migration017, shared-context `workspace.tsx`,
ResearchPage and StartResearchButton. Updated App/navigation/target dialogs and
project-aware existing workspaces; existing Agent Workflows, chat, imports, MCP,
Job Engine, evidence and reporting remain available. Existing queue/cancel/HEAD
helpers now accept an enclosing transaction; no parallel scheduler was introduced.
New CLI `workspace` and `research` groups and matching Tauri operations. Architecture
and JSON review formats: `docs/AI_RESEARCH_SESSIONS.md`.

Regression failures fixed: schema upgrade fixtures initially retained new017 tables
while simulating old versions; fixtures now remove only their isolated newer layout,
without making production migrations permissive. Native restart initially depended
on browser-local session hints; restore now reads SQLite history. Async selection/list
responses are fenced; deleted/archived selections invalidate IDs. Quick Create Finding
remains usable. History status now follows actual jobs and planning context closes
after plan creation. Native first attempt's failed restart proof is retained under
`artifacts/research-session-1792487208696/results.json`. A subsequent fixture compile
failure used a helper module as a program; test now compiles an explicit benign main.
An early full-test invocation overlapped source edits and failed rustdoc; only the
final complete log counts above are accepted. All 69 staged Rust/SQL source hashes
match the Linux test snapshot.

Exact reproducible commands (use isolated data and explicit accepted binaries):

```powershell
$env:CARGO_INCREMENTAL='0'
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -j2 -- -D warnings
cargo test --locked --workspace -j2
cd apps/desktop
npm run build
npm test
cd ../..
cargo build --locked --release -p reconx -p reconx-desktop --features reconx-desktop/custom-protocol -j2
$env:RECONX_CLI_EXE='D:\ReconXAI\artifacts\windows-research-current-20261020\reconx.exe'
$env:RECONX_DESKTOP_EXE='D:\ReconXAI\artifacts\windows-research-current-20261020\reconx-desktop.exe'
$env:RECONX_VERIFY_ASSET='1'
node tests/e2e/research-session.mjs
node tests/e2e/http-replay.mjs
node tests/e2e/outputs.mjs
```

Desktop source asset changes require a fresh embedded-asset build; the native test
can assert the actual script path, preventing a stale executable from being accepted.
Build logs: recovery-release-build-final.log and recovery-desktop-current-assets.log.
Linux stage/run: `artifacts/stage-recovery-linux.py`, `artifacts/recovery-linux-run.sh`;
the latter uses the retained1.95 toolchain and native Linux temp workspace, excludes
Desktop and builds Core/CLI offline. CLI acceptance command:
`python scripts/research_session_smoke.py --binary <explicit binary> --sample <benign PE/ELF> --output <new isolated directory>`.
Each CLI call is a separate process; logs carry actual exit codes/timestamps. Never
execute the imported sample. `scripts/verify.ps1 -Research` and `npm run test:research`
provide reproducible native entry points.

Current work after this checkpoint: large-file import design/audit; no implementation
yet. First next edit must separate ingestion from `reconx-analysis::MAX_FILE_BYTES`,
not change the parser cap. Add cancellation and partial-file cleanup tests before
advertising large imports. Preserve existing databases and accepted binaries. Source
protection: `git status --short` before resuming; keep all uncommitted changes.

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

## Current checkpoint: verified UX and first-party extension — 2026-10-19

Active priority: user product milestones A-H. A's bounded Windows UX and B's
first-party YARA-X installation/invocation slice are accepted. Full A cross-platform
visual acceptance and broader B/C marketplace/MCP setup remain PARTIAL. Preserve
all earlier schema/security work. Version remains **0.1.0**, schema **014**;
**v1.0 is NOT READY**. No commit, push, installer publication or release.

Authoritative current proof: artifacts/ux-extensions-current-verification.json.
Windows portable pair: D:\ReconXAI\artifacts\windows-extensions-verified-20261019-172615.
Linux optimized CLI: artifacts/linux-cli-extensions/reconx. Actual hashes/signature
status are beside the binaries. Windows Desktop and CLI are actually NotSigned.
The OneDrive Desktop ReconX AI.lnk now points to the new accepted Windows pair;
artifacts/desktop-shortcut-extensions.json verifies its target and binary hash.
The normal user workspace/credentials were not opened or migrated by this work.

Implemented UX: shared 30/34/38 px controls, stable Button DOM and disabled reasons;
actual Lucide icons; preserved Segoe UI Variable/Source Sans 3/IBM Plex Mono;
exactly one rail Appearance menu (Light/Dark/ReconX), informational Settings;
48px rail, collapsible navigation and optional activity drawer; real compact
project/jobs/audit dashboard, useful recon empty state, direct provider navigation,
collapsed provider catalog and build-derived version. Native visual inspection
fixed ReconX token specificity, inconsistent default HTTP project binding,
provider/reverse density and extension detail/install-dialog padding/alignment.
No original user-referenced screenshot files were attached, so no comparison with
those missing files is claimed. Actual baseline: artifacts/ux-before-1792391426265.
Current native matrix: artifacts/ux-after-1792405586148: **120 captures**, ten pages ×
three themes × four sizes (1280×720/1366×768/1440×900/1920×1080). Screenshots were
actually viewed. Extra native checks cover rail width, disabled explanations,
one theme menu, panel/navigation collapse, keyboard/persistence/restart and HTTP
default project binding. 125/150/200% are CDP raster scaling, not physical OS DPI.
Reviewable before/after viewer: artifacts/ux-comparison.html.

Implemented bounded Marketplace: one compiled YARA-X native adapter; strict
versioned manifest, real dependency/source/hash inspection, exact approved plan,
portable dependency installation/reuse, actual version and benign signature health
check before Ready, safe failure state, enable/disable/remove, shared-tool retention,
input-file selection, real Job Engine execution/evidence and persistent history.
CLI and Desktop share the same Rust methods/state. Migration014 adds registrations,
tool requirements and atomic invocation permission bindings. Workers recheck the
enabled revision at checkpoints and final commit. Disable/removal cancels queued
or running bound work. Uninstall preserves tools, artifacts and history; tools
needed by any installed registration cannot be removed. Native runtime directly
tests queued cancellation; concurrent running-extension revocation is not separately
claimed as runtime verified. No arbitrary native library/plugin or shell execution.

Verified: Windows Rust **171 passed/0 failed/1 ignored**, Linux Core/CLI **171/0/1**;
extension tests3/storage12; frontend **33/0**; TypeScript/Vite, rustfmt and warning-
denied Clippy on Windows and Linux Core/CLI. Actual source hashes match for 64 Rust,
SQL and manifest files in the Linux snapshot: artifacts/extensions-source-hashes.json.
Optimized Windows pair and Linux CLI built. Final native extension acceptance:
artifacts/extensions-1792405575406 (5 checkpoints); fresh final HTTP/schema14 and outputs:
20 checkpoints. Current binary's UX matrix120 passes. Earlier same-checkpoint
existing suites pass77; schema14 tools/MCP/AI-plan6 and HTTP replay11 also pass.
Provider inference fixtures are not live account verification. Actual CLI extension
lifecycle: **28 commands per host**, real pinned YARA-X, evidence hashes, integrity,
disable/enable, rejected stale/unapproved plans, retained tools/history and reinstall.
Both hosts use the actual schema13 binary to create queued work, then verify the
intact pre-v14 backup and successful migrated execution. Database integrity and
foreign-key checks pass. npm audit: zero vulnerabilities. Cargo.lock is unchanged;
the previously verified glib unsoundness plus maintenance backlog still blocks v1.

Commands/logs: npm test, npm run build, npm audit; cargo fmt --all -- --check;
cargo clippy --locked --workspace --all-targets -j2 -- -D warnings; cargo test
--locked --workspace -j2; cargo build --locked --release -p reconx -p reconx-desktop
--features reconx-desktop/custom-protocol -j2; Linux offline equivalents excluding
Desktop; scripts/extension_smoke.py --legacy-binary <schema13> --approve-install;
tests/e2e/extensions.mjs, ux.mjs, http-workspace.mjs, outputs.mjs and existing native
suites. Final logs: artifacts/extensions-verified-tests.log,
extensions-linux-final-tests.log, extensions-final-clippy.log,
extensions-dialog-refined-tests.log, extensions-dialog-refined-frontend-build.log,
extensions-final-native-build.log, extensions-final-native-*.log and
extensions-final-regression-*.log. Final YAML workflow parsing and py_compile pass.

Retained failures: early Button conditional wrapper lost pointer/ref continuity
(19 frontend pass/7 fail), fixed by a stable native button and regression test;
extension UI DOM shim/API/Send compile errors fixed before acceptance; native
reinstall test held a locator from the prior WebView, fixed and rerun. Native
screenshots caught/fixed actual padding defects. Linux staging first omitted
fixture/workflow files, then shared Cargo cache retained an old compile-time
CARGO_MANIFEST_DIR; staged complete source and touched workspace source to force
recompile. Combined Windows WSL stdout/stderr handles overwrote log segments;
final test logging is redirected inside Linux, with complete171/0/1 results.
No guard was relaxed to make any test pass.

Next exact task: official MCP Registry discovery in this same Extensions page,
with a bounded typed Rust client, pagination, cache and clear untrusted metadata.
Official API documentation was read; public list probe timed out in this environment
(artifacts/mcp-official-registry-probe.log). No registry client/package installer is
shipped yet and no connection is claimed. Contract: fixed official HTTPS base,
GET /v0.1/servers, bounded search/limit, version=latest and opaque metadata.nextCursor.
Preserve server.json and its _meta, do not execute descriptions/package instructions.
Then approved pinned package/runtime dependency resolution and real MCP protocol
health must precede Ready. Current native extension manifest rejects all arbitrary
registry IDs. Read docs/MCP_INTEGRATION.md and EXTENSION_ARCHITECTURE.md first.

Remaining blockers: general community/runtime extensions, official registry setup,
full dependency profiles/first-run wizard, native platform sandbox/resource controls,
extension download cancellation and verified different-version update/rollback;
signed app updater, provider billing audit/accounting and authorized OAuth/local
server acceptance; Linux Desktop/Secret Service/packages and physical DPI; installer
lifecycle/signing, glib and other full-v1 research/security gates. No fake installer,
cloud balance, login, third-party MCP readiness or verified finding is generated.

Exact files for this assignment: apps/desktop package.json/package-lock.json;
src/App.tsx/App.test.tsx/main.tsx; src/components/controls.tsx/icons.tsx/ui.tsx,
navigation.ts/ThemeMenu.tsx/QuickActions.tsx/ActivitySidebar.tsx/AiProviderManager.tsx,
FileResearchPanel.tsx/HttpAuthManager.tsx/HttpRequestWorkspace.tsx/JobHistory.tsx/
TargetDialog.tsx; src/pages/Dashboard.tsx/ReconPage.tsx/HttpPage.tsx/ReversePage.tsx/
JobsPage.tsx/McpPage.tsx/OrchestrationPage.tsx/RecordsPage.tsx/TargetsPage.tsx/
WorkspacePages.tsx/ExtensionsPage.tsx; src/controls.css/workbench.css/extensions.css/
extensions.ts/Controls.test.tsx/ExtensionsPage.test.tsx; core src/extensions.rs,
jobs.rs/lib.rs and tests/extensions.rs; storage src/lib.rs and migration014;
CLI main.rs and Tauri lib.rs; scripts/extension_smoke.py/http_schema_smoke.py;
e2e UX/extensions and appearance/native regression updates; .gitignore/CI;
UI_UX_OVERHAUL, EXTENSION_ARCHITECTURE, EXTENSION_MARKETPLACE, AUTO_SETUP,
TOOL_DEPENDENCY_MANAGER, MCP_INTEGRATION, AUTO_UPDATE, AI_USAGE_BILLING,
AI_PROVIDER_CAPABILITY_MATRIX, AI_OAUTH and current implementation/readiness/test
ledgers/status JSON. Earlier dirty schema013/conditional files remain preserved.
Runtime data, screenshots, binaries and private install logs stay under ignored
artifacts. The retained DOCX/user databases were not overwritten.

Resumable checkpoint only: no work continues after this turn stops.

---

## Previous checkpoint: bounded conditional schemas - 2026-10-19

Completed opt-in format assertions and then the next bounded offline schema
slice: OpenAPI 3.1 if/then/else. A definite predicate selects only its adjacent
branch; unknown/redacted/cyclic/limited predicates select neither and remain
inconclusive. Predicate diagnostics stay speculative. Orphan branches are ignored
and cannot interact across allOf/reference boundaries. Selected constraints retain
their original pointers. Saved format policy applies inside predicates; shared
depth/work/observation/cancellation limits remain in force. 3.0 conditional keywords
are explicitly unsupported. Full schema annotation collection is not claimed.

Current proof: artifacts/schema-conditional-current-verification.json. Full Windows
Rust and Parrot/WSL Core+CLI: 167 passed/0 failed/1 ignored each; schema 41/storage
11; frontend 26; TypeScript/Vite, format and warning-denied Clippy pass. Optimized
Windows Desktop/CLI and Linux CLI built. Two fresh native suites pass 20 checkpoints
(HTTP/schema 14, outputs 6), including actual selected-branch evidence, format
checkbox, reports and restart. The conditional screenshot was actually inspected.
Other native suites retain historical proof, not freshly rerun status.

Separate actual CLI processes on both platforms repeat the real schema-12 queued
job upgrade/backup/integrity check and validate conditional+format observations,
evidence hashes, Needs Verification-only finding, persistence and all four report
exports. Independent pypdf 6.19.0 parses both PDFs and checks source pointers and
redaction. The normal user database/credentials were not opened or changed.

Windows portable candidate: artifacts/windows-schema-conditional-20261019-125941.
Linux CLI: artifacts/linux-cli-schema-conditional/reconx. Actual hashes and build
manifests accompany both. The previous format candidate/evidence are preserved.
The user's OneDrive Desktop ReconX AI.lnk now points to the accepted conditional
candidate. Actual shortcut/hash record: artifacts/desktop-shortcut-schema-conditional.json.
The application was not launched against the normal user workspace by this task.
No additional migration: schema remains 013, product version 0.1.0. No new installer,
signing, commit, push or publication. v1.0 is NOT READY.

Files changed for this second slice: crates/reconx-core/src/http/schema.rs and
tests/http_schema.rs; scripts/http_schema_smoke.py; tests/e2e/http-workspace.mjs;
docs/HTTP_SCHEMA_ANALYSIS.md and PROJECT_DOCUMENTATION.vi.md; the master/readiness,
implementation/test/roadmap/next ledgers and JSON status registries. The first slice
and all earlier dirty work remain preserved. The delivered DOCX is not overwritten.

Commands: cargo fmt --all -- --check; cargo clippy --locked --workspace --all-targets
-j2 -- -D warnings; cargo test --locked --workspace -j2; isolated Linux equivalent
--offline --exclude reconx-desktop; npm test; npm run build; cargo build --locked
--release -p reconx -p reconx-desktop --features reconx-desktop/custom-protocol -j2;
isolated Linux release CLI build; scripts/http_schema_smoke.py --legacy-binary on
both hosts; node tests/e2e/http-workspace.mjs and outputs.mjs; independent pypdf
inspection. Logs use artifacts/schema-conditional-*; Linux snapshot/source hashes
match the current backend/lockfile. Exact source staging/run scripts are retained.

Retained failure: schema-conditional-focused.log (38 pass/3 fail) caught new test
expectations that counted unrelated HTTP metadata observations and an external
reference already rejected at import. Corrected expectations preserve that import
boundary. Former unsupported-conditional fixtures now use still-unsupported
unevaluatedProperties. Focused final schema suite: 41/0; full suites above pass.
Native fixture predicates use actual field presence while retaining the independent
captured id type mismatch. No production execution guard was loosened.

Dependency audit is the fresh format-stage result from this same session with an
identical Cargo.lock: npm zero; glib unsound warning plus six unmaintained warnings;
deny-unsound/readiness return 1/BLOCKED. cargo tree --workspace --target
x86_64-unknown-linux-gnu -i glib confirms the Tauri/GTK path. The first query omitted
--workspace and did not select Desktop; its failed log is retained. Historical
GitHub export is not claimed a fresh remote audit. Linux GUI/Secret Service/packages,
installer lifecycle/signing, broader modules and full release acceptance stay open.

Next exact action: implement explicit bounded offline multi-file schema import
with selected file manifests, per-file evidence hashes and resource provenance.
Inspect http.rs resolve/validate_refs/openapi_document/parse_openapi/Core::http_import,
schema.rs source_ref/schema_sources and evidence::read_regular before any resolver
change. Current jobs bind one original OpenAPI hash; per-file provenance must be
designed before resolving a bundle. Add traversal/symlink/size/cycle tests, no implicit network/file
fetch, and Desktop/CLI parity. Additional unevaluated/schema keywords remain open.
No work continues after this saved session checkpoint without another active turn.

---

## Historical conditional continuation - 2026-10-19

The format milestone below is accepted. Next work implements OpenAPI 3.1
if/then/else through the same bounded offline schema worker. No new migration or
network resolver is planned. Preserve three-way predicate outcomes, source
pointers, shared cancellation/work budgets and 3.0 dialect exclusions. Current
accepted binaries remain the format candidate below; the Desktop shortcut now
points to artifacts/windows-schema-format-20261019-124827/reconx-desktop.exe
(artifacts/desktop-shortcut-schema-format.json). Conditional source is not yet
runtime-verified. v1.0 stays NOT READY.

---

## Completed checkpoint: opt-in offline format assertions - 2026-10-19

Completed the pending format slice through shared Core/Job Engine/Desktop/CLI.
Fourteen syntactic formats are available with an explicit default-off job option.
Annotation-only formats do not invalidate alternative branches. Unsupported,
redacted and over-budget inputs stay inconclusive; no network/entity-existence
check or automatic finding verification occurs. New jobs bind options into their
fingerprint and verify the binding again before preparing the result.

Migration 013_schema_review_options.sql is additive. Consistent pre-v13 VACUUM INTO
backups precede the transactional upgrade. Legacy queued jobs keep their original
fingerprint/default-off policy; existing results remain intact. Actual optimized
schema-12-to-13 CLI upgrades passed on Windows and Parrot/WSL, including the queued
job, version-12 backup/integrity and upgraded foreign keys. Normal user databases
and credentials were not opened or changed by this acceptance.

Proof: artifacts/schema-format-current-verification.json. Full Windows Rust and
Parrot/WSL Core+CLI: 163 passed/0 failed/1 ignored each; schema 37/storage 11;
frontend 26; TypeScript/Vite, format, warning-denied Clippy and optimized Windows
Desktop/CLI plus Linux CLI builds pass. Two fresh native suites pass 19 checkpoints
(HTTP/schema 13, outputs 6), exercising the default-off checkbox, real invalid-date
assertion, saved policy, original source pointers, evidence, reports and restart.
Independent pypdf 6.19.0 parsed both five-page CLI PDFs and checked redaction.
Other native suites retain historical proof and are not claimed freshly rerun.

Portable Windows candidate: artifacts/windows-schema-format-20261019-124827.
Linux CLI candidate: artifacts/linux-cli-schema-format/reconx. Both have hashes
and build manifests. No new installer, signing, push, commit or publication.
Version remains 0.1.0; schema 013; v1.0 is NOT READY. Fresh npm audit: zero;
cargo-audit: one glib unsound warning and six unmaintained warnings. Deny-unsound
and dependency readiness return 1/BLOCKED. GitHub export is historical input.
Linux Desktop/Secret Service/packages and installer lifecycle remain unverified.

Files changed for this slice: storage src/lib.rs and migration 013; core http.rs,
http/schema.rs and tests/http_schema.rs/research_inputs.rs; CLI main.rs; Tauri
lib.rs; Desktop http.ts, HttpPage.tsx, SchemaResults.tsx and HttpPage.test.tsx;
scripts/http_schema_smoke.py and tests/e2e/http-workspace.mjs; schema documentation,
implementation/test/roadmap/readiness/master ledgers and JSON status registries.
Prior regex/tuple/replay/CI edits remain preserved and uncommitted.

Commands: cargo test --locked --workspace -j2; isolated Linux equivalent excluding
Desktop with --offline; cargo fmt --all -- --check; cargo clippy --locked
--workspace --all-targets -j2 -- -D warnings; npm test; npm run build; cargo build
--locked --release -p reconx -p reconx-desktop --features
reconx-desktop/custom-protocol -j2; isolated Linux release CLI build; Python
scripts/http_schema_smoke.py with --legacy-binary on both hosts; node
tests/e2e/http-workspace.mjs and outputs.mjs; independent PDF parsing; cargo-audit
--deny unsound --json; npm audit --json; scripts/dependency_readiness.py.
Linux commands used explicit wsl.exe via Python argv: the user's PowerShell
profile defines a wsl function that ignores arguments. Empty first-attempt logs
were not counted as test proof. Actual Linux source hashes match the workspace.

Retained regressions: schema-format-first.log records outdated pre-format
unsupported-keyword expectations, corrected for annotation semantics. The initial
storage test compile mismatch (i64/u32) was fixed; schema-format-storage.log is
retained. Full final suites pass; no failure is relabeled as success.

Next exact action: inspect the remaining schema applicators/resource resolver and
continue the next bounded offline slice with original-source provenance, shared
budgets, unknown-branch handling and real Desktop/CLI acceptance. Full multi-file
and URI resource resolution, conditional/unevaluated keywords, broader Phase 1 and
the remaining module/platform/security/release gates are still incomplete.

---

## Historical resumed verification: format assertions - 2026-10-19

The user explicitly resumed development after the stop checkpoint below. Preserve
the schema-12 accepted candidates and all prior changes. Full Windows regression
and isolated Parrot/WSL Core/CLI regression are now running against schema-13
source. Native, optimized-binary and real queued-job migration acceptance remain
pending. The release version stays 0.1.0; v1.0 is NOT READY.

---

## Historical pause at user request: format assertions - 2026-10-19

The user explicitly requested stop ("dừng"). Stop implementation/build/test work
here. No new commit, push, release or normal-workspace migration was performed.

Latest progress before stop: 37 focused schema tests pass
(artifacts/schema-format-second.log), including real checks for fourteen formats,
default annotation semantics, opted-in assertions, restart, queued-policy tampering
and legacy fingerprints. Storage migration suite: 11 pass
(artifacts/schema-format-storage-fixed.log). Frontend: 26 pass and TypeScript/Vite
build passes (schema-format-frontend-{tests,build}.log). An initial storage-test
integer-type mismatch was fixed; the failing log is schema-format-storage.log.
Full-workspace warning-denied Clippy completed successfully before stop, exit 0
(artifacts/schema-format-clippy-first.log). No further build or test was started.

Implemented but NOT end-to-end accepted: additive schema 013 options table and
pre-v13 consistent backups; typed default-off per-job format policy bound to
fingerprints with legacy queued-job compatibility; bounded syntactic validators;
Desktop checkbox/result policy display; CLI --assert-formats; expanded native and
CLI smoke scripts including optional --legacy-binary for real schema-12 upgrade.
Latest smoke-script/native changes have syntax checks only and have not run against
a schema-13 release build. No schema-13 portable candidate is runtime-verified.

Additional current files: crates/reconx-storage/migrations/013_schema_review_options.sql,
crates/reconx-storage/src/lib.rs, crates/reconx-core/src/http.rs and http/schema.rs,
crates/reconx-core/tests/http_schema.rs and research_inputs.rs, apps/cli/src/main.rs,
apps/desktop/src-tauri/src/lib.rs, apps/desktop/src/http.ts,
apps/desktop/src/pages/HttpPage.tsx, components/SchemaResults.tsx, HttpPage.test.tsx,
scripts/http_schema_smoke.py and tests/e2e/http-workspace.mjs. All changes remain
local/uncommitted; preserve prior regex/tuple/dependency work.

Next exact action after an explicit resume: run full Windows and isolated Linux
suites; verify the
new native checkbox and actual old-schema queued-job upgrade with real binaries;
inspect backup integrity, source/evidence/policy persistence and all report exports;
then finalize schema-13 ledgers/candidates. Keep v1.0 NOT READY. The Desktop
shortcut continues to launch the accepted schema-12 regex candidate, not this
unfinished source. No normal user DB was opened with schema-13 code.

---

## Previous active continuation: format assertions - 2026-10-19 (IN PROGRESS)

The regex milestone below is verified and preserved. Work now adds migration
013_schema_review_options.sql, typed default-off per-job options, bounded format
validation, legacy fingerprint compatibility, Desktop controls and CLI parity.
Current source is not yet accepted for this new slice. Do not promote an untested
schema-13 binary or migrate normal user data. Latest accepted binaries remain
schema 012 in artifacts/schema-pattern-current-verification.json; the Desktop
shortcut now points to artifacts/windows-schema-pattern-20261019-105539.

Changed so far: reconx-storage migrations/013 and src/lib.rs; core http.rs and
http/schema.rs; CLI main.rs; Tauri lib.rs; Desktop http.ts, HttpPage.tsx,
SchemaResults.tsx and HttpPage.test.tsx; schema/research-input tests. Next: finish
format/migration/policy-integrity tests, run focused suites, fix failures, then
full Windows/Linux regression, optimized builds, native/CLI acceptance and ledger
finalization. The first schema-format run uses earlier unsupported-format test
expectations; those are being replaced with explicit unsupported conditional
cases because default-off formats now have annotation semantics. No normal user
database or credential migration has been performed. v1.0 remains NOT READY.

---

## Current checkpoint: bounded offline regex validation - 2026-10-19

Implemented `pattern` in OpenAPI 3.0/3.1 and `patternProperties` in 3.1 through
the existing schema Job Engine/Core/Desktop/CLI paths. All matching patterns and
explicit properties apply; additionalProperties only applies after determining
that no property/pattern matched. Unknown regex results defer that decision.
Original HAR/OpenAPI pointers and evidence hashes persist; observations never
automatically create or verify security findings. Prior completed work is retained.

Patterns use the existing library's ECMA-aware translation and non-backtracking
engine: 1024 bytes/pattern, 64 cached expressions, 128 KiB approximate compiled
state and DFA cache each, and a shared 8 MiB pattern/input-byte product budget.
Specialized groups, engine-only anchors, invalid/unsupported syntax, redaction,
engine errors and resource limits remain explicit/inconclusive. This is a bounded
subset, not a full ECMA grammar or a hard OS resource sandbox. Details and limits:
HTTP_SCHEMA_ANALYSIS.md. No dependency update or database migration was added;
schema stays 012, version 0.1.0, v1.0 is NOT READY.

Actual final proof: `artifacts/schema-pattern-current-verification.json`.
Windows Rust 157 passed/0 failed/1 ignored; Parrot/WSL Core+CLI 157/0/1; schema
suite 32 (seven new regressions), frontend 26, TypeScript/Vite, format and
warning-denied Clippy pass. Optimized Windows Desktop/CLI and Linux CLI built.
Two fresh native suites passed 18 checkpoints (HTTP/schema and outputs). Other
native suites retain their older proof and are not claimed freshly rerun.
Separate actual optimized CLI processes on each platform verified the new patterns,
source evidence, restart persistence and all four reviewed report formats.
Independent pypdf 6.19.0 inspection verified both generated PDFs and redaction.
Linux Desktop/Secret Service/packages, installer lifecycle and signing remain open.

Fresh dependency recheck: npm zero findings; cargo-audit still reports glib
unsoundness plus six maintenance warnings. `--deny unsound` returns 1, readiness
stays BLOCKED. GitHub state is historical input; no remote CI/push/release was
performed or claimed successful. See schema-pattern-dependency-readiness.json.

### Regressions found and fixed

- Initial assertions that patterns were unsupported became obsolete when support
  was added; updated them to real supported/unsupported cases before full tests.
- A new test reproduced acceptance of engine-specific inline syntax; specialized
  groups/anchors are now explicitly outside this supported subset. Before-fix
  evidence: schema-pattern-hardening-before.log. Final tests pass on both hosts.
- A loaded Windows run hit HTTP_TIMEOUT during a chunked-response size test.
  The test now allows the existing policy's 10-second maximum for size rejection,
  still requires RESPONSE_LIMIT and leaves the independent 500 ms timeout and
  genuine cancellation assertions intact. An initial 15-second fixture choice
  was correctly rejected by policy, then corrected. Production replay limits and
  behavior were not loosened. All failing logs are retained; final full suites pass.

### Files, commands and artifacts

This continuation changed crates/reconx-core/src/http/schema.rs,
crates/reconx-core/tests/http_schema.rs, crates/reconx-core/tests/http_replay.rs,
scripts/http_schema_smoke.py, tests/e2e/http-workspace.mjs,
docs/HTTP_SCHEMA_ANALYSIS.md, docs/PROJECT_DOCUMENTATION.vi.md and the implementation,
test, roadmap, release, dependency and module ledgers. All pre-existing local
changes, prior binaries, DOCX, normal databases and user credentials were preserved.

Commands: cargo fmt --all -- --check; cargo clippy --locked --workspace
--all-targets -j 2 -- -D warnings; cargo test --locked --workspace -j 2; focused
http_schema and HTTP size/timeout/cancellation regressions; npm test; npm run build;
cargo build --locked --release -p reconx -p reconx-desktop --features
reconx-desktop/custom-protocol -j 2; node tests/e2e/http-workspace.mjs and outputs.mjs;
python scripts/http_schema_smoke.py with each actual optimized CLI and separate
NEW output directories. Python/Node syntax checks and dependency-readiness tests
also ran. Raw logs use artifacts/schema-pattern-*; the summary retains exact paths.

Linux final source: `/var/tmp/reconx-schema-pattern-final-1792342997/source`.
Task-local Rust 1.95.0 and offline dependencies; cargo test --locked --offline
--workspace --exclude reconx-desktop -j2 and cargo build --locked --offline --release
-p reconx -j2. Explicit missing DBUS exercises fail-closed credential behavior.
Production/schema/fixture source hashes match Windows. No privileged install,
global toolchain edit, network target scan or live cloud transmission occurred.

Windows portable candidate: `artifacts/windows-schema-pattern-20261019-105539`;
Linux CLI: artifacts/linux-cli-schema-pattern-final/reconx. Manifests and
SHA256SUMS.txt retain exact accepted binary/source hashes. No new installer was
built. Existing unsigned installer is historical and not upgraded to a v1.0 claim.
Source base `ce32ee8cd5a6e4453a3c6feadac648118b89c267`; changes remain local/uncommitted, no push or reset.

### Current unfinished task and next exact action

Proceed with opt-in format assertions. Inspection confirmed existing schema jobs
have no options column, and their fingerprints must remain valid for queued jobs.
Keep formats annotation-only by default; add typed options to the existing schema
enqueue path, immutable per-job options/provenance, a Desktop checkbox and CLI
--assert-formats. Preserve legacy queued fingerprints when adding durable options.
First add an additive migration with a consistent pre-upgrade backup and explicit
schema-12/queued-job preservation tests. No schema 013 or format assertion feature
has been implemented in this checkpoint. Whitelist bounded syntactic formats;
unknown formats and assertion-vocabulary/dialect cases must remain explicit.
References: JSON Schema validation section 7.2.1 and existing library options.

Broader Phase 1 and the remaining research/platform/security/release gates remain
PARTIAL/NOT_STARTED/BLOCKED as recorded. This is a resumable checkpoint, not a
claim of work continuing after the session ends or of a completed v1.0 release.

---

## Current checkpoint: bounded offline tuples, uniqueness and dependencies - 2026-10-18

User-requested launch shortcut created and read back successfully:
`C:\Users\hqhuy\OneDrive\Desktop\ReconX AI.lnk` points to the exact tested
`artifacts/windows-schema-arrays-20261018-234036/reconx-desktop.exe`, with the
existing ReconX icon and candidate working directory. Binary SHA-256 matched its
build manifest. The application was not launched or a normal workspace modified
for shortcut creation. Record: `artifacts/desktop-shortcut-schema-arrays.json`.

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

### Exact work and evidence

- Backend: `crates/reconx-core/src/http/schema.rs`.
- Seven new integration regressions: `crates/reconx-core/tests/http_schema.rs`;
  complete schema suite 25 passed. Existing job/evidence/CLI routes are reused.
- Native acceptance: `tests/e2e/http-workspace.mjs`; actual 3.1 dependency and
  tuple/uniqueness assertions, exact pointers and persisted results.
- New reusable offline acceptance: `scripts/http_schema_smoke.py`; refuses existing
  output, uses synthetic sources/new DBs, starts separate real CLI processes,
  verifies source evidence and produces reviewed Markdown/JSON/HTML/PDF.
- CI: `.github/workflows/ci.yml` invokes the new smoke test after the release build
  and retains its JSON/report artifacts. Remote CI was not run or claimed passed.
- Documentation: `docs/HTTP_SCHEMA_ANALYSIS.md`, `docs/PROJECT_DOCUMENTATION.vi.md`,
  `docs/RELEASE_READINESS.md`, `docs/release-gaps.json`, `docs/module-status.json`,
  `docs/MASTER_EXECUTION_PROGRESS.md`, `docs/IMPLEMENTATION_STATUS.md`,
  `docs/NEXT_MILESTONE.md`, `docs/TEST_RESULTS.md`, `docs/ROADMAP.md` and
  `docs/DEPENDENCY_SECURITY.md`. Earlier delivered DOCX was not overwritten;
  it remains its historical snapshot.
- Database migrations: NONE. Normal user databases/credentials were not migrated
  or inspected. Tests use isolated synthetic DBs; native auth fixtures used and
  removed only their own synthetic OS-backed secrets.

Commands executed (logs retained under artifacts/schema-arrays-*):

```text
cargo fmt --all
cargo fmt --all -- --check
cargo test --locked -p reconx-core --test http_schema -j 2
cargo clippy --locked --workspace --all-targets -j 2 -- -D warnings
cargo test --locked --workspace -j 2
npm test
npm run build
cargo build --locked --release -p reconx -p reconx-desktop --features reconx-desktop/custom-protocol -j 2
cargo test --locked -p reconx-core --test release_workflows -j 2
python -m py_compile scripts/http_schema_smoke.py
python -m unittest discover -s scripts -p test_dependency_readiness.py
node --check tests/e2e/http-workspace.mjs
node tests/e2e/http-workspace.mjs
node tests/e2e/outputs.mjs
node tests/e2e/http-replay.mjs
python scripts/http_schema_smoke.py --binary target/release/reconx.exe --output artifacts/schema-arrays-cli-windows
artifacts/audit-tools/bin/cargo-audit.exe audit --json
artifacts/audit-tools/bin/cargo-audit.exe audit --deny unsound --json
cargo tree --locked --workspace --target x86_64-unknown-linux-gnu -i glib
npm audit --json --registry https://registry.npmjs.org
python scripts/dependency_readiness.py --github artifacts/dependabot-after-second-check.json --npm artifacts/schema-arrays-npm-audit.json --cargo artifacts/schema-arrays-rustsec.json --output artifacts/schema-arrays-dependency-readiness.json
```

Linux used an isolated source snapshot `/var/tmp/reconx-schema-arrays-1792341275/source`,
task-local Rust 1.95.0, offline Cargo dependencies and the existing isolated target
directory. Explicit absent DBUS tests preserve fail-closed keyring behavior.
Commands: cargo test --locked --offline --workspace --exclude reconx-desktop -j2;
cargo build --locked --offline --release -p reconx -j2; python3
scripts/http_schema_smoke.py with the actual optimized ELF. Production source/test
hashes match Windows. No privileged package installation or global toolchain edit.

Native records:
- `artifacts/http-workspace-1792341720146/results.json` (12 checkpoints).
- `artifacts/outputs-1792341679403/results.json` (6).
- `artifacts/http-replay-1792341692388/results.json` (11).
- `artifacts/schema-arrays-cli-{windows,linux}/acceptance.json`, schema-job.json
  and pdf-inspection.json; each PDF has five independently parsed pages.
- Windows portable candidate: `artifacts/windows-schema-arrays-20261018-234036`;
  Linux CLI: `artifacts/linux-cli-schema-arrays/reconx`. SHA256SUMS.txt and
  build-manifest.json retain exact tested binary/source hashes. Authenticode
  reports NotSigned for both Windows binaries. Neither is a signed v1.0 release.

Retained failures: first schema run had 23 passed/2 failed because new tests counted
existing HTTP metadata diagnostics as body-validation failures; body-specific
assertions fixed this before full verification. First large-region screenshot was
not useful inside the nested scroller; final runner captures individual actual
observation cards, which were viewed. Initial cargo audit lookup was unavailable
on PATH; the existing isolated cargo-audit 0.22.2 executable ran the real audit.
An initial Linux retention shell invocation failed argument quoting; a retained
script file then copied/hashed the exact accepted binary successfully.

Source base `ce32ee8cd5a6e4453a3c6feadac648118b89c267`; all new changes remain local and uncommitted.
No reset, commit, push, release publication, user document overwrite or destructive
operation was performed. This is a saved checkpoint, not a claim of ongoing work
after this session ends. All unfinished v1.0 gates remain visible.

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

## Source handoff: commit and push authorized - 2026-10-18

The user explicitly authorized committing and pushing the current verified source. This supersedes older no-commit/no-push continuation instructions below. No force-push, history rewrite, user-data upload or release publication is authorized by this handoff. Use the credential-free repository URL and normal credential-manager authentication; never print or reuse an embedded Git remote token.

Before staging, re-ran `cargo test --locked --workspace -j2` (133 passed, 0 failed, 1 ignored), `npm run build`, `npm test` (26 passed), `cargo fmt --all -- --check` and warnings-denied Clippy. Logs: artifacts/completion-baseline-rust.log, completion-baseline-frontend-build.log, completion-baseline-frontend-tests.log and completion-precommit-clippy.log. All 212 source fingerprints and both executable hashes in artifacts/replay-current-verification.json still match the previously verified native build (9 suites, 77 checkpoints); native suites were not repeated for this documentation-only handoff. No new product behavior was added during this handoff.

Scanned all 248 Git-eligible files for high-confidence credential patterns and reviewed the four synthetic credential-URL test cases. No matching live key/private key was identified; this pattern scan is not a comprehensive secret audit. Generated artifacts, databases, imported user files, logs, dependency directories and local credentials remain excluded. Push outcome must be checked separately against the actual remote branch; this entry does not assert that a push has already succeeded. Product remains 0.1.0 and v1.0 remains incomplete. The exact next implementation task and existing limitations are recorded in the verified checkpoint below.

---

## Verified checkpoint: reviewed loopback HTTP and secure identities - 2026-10-18

Preserved all preceding uncommitted schema 011 work and user data; no reset, Git remote inspection, commit or push. Product remains 0.1.0; full Web/API and v1.0 are incomplete. This checkpoint supersedes every earlier next-task/count entry.

Delivered a bounded manual vertical slice: imported capture -> credential-free editable draft -> explicitly reviewed exact fingerprint -> one real loopback HTTP request through the existing persistent Job Engine -> sanitized captured response/history -> evidence -> comparison requiring review -> Needs Verification finding -> reviewed Markdown/JSON/HTML/PDF. No network execution occurs on import, editing, saving or profile registration. Manual cancellation, size/time/rate limits and fail-closed scope/identity policy reuse the shared Rust core, not React analysis code or another scheduler.

Authentication manager: project/target/exact-origin identity metadata; Anonymous, Cookie, Bearer, API Key header, Basic and Custom Header; OS-store credentials with no retrieval/display API; replace, expire, delete and reviewed test entry point. Credentials use immutable workspace-scoped references; optimistic revisions prevent stale updates. Retired secret cleanup has a persistent journal and visible failure/retry path (`auth cleanup`). Saving and HTTP success never imply an identity is authenticated or authorized. No cloud transmission was performed.

Migration 012_manual_http_replay.sql adds http_auth_profiles, http_auth_retired_secrets, http_request_drafts, http_replay_jobs and http_replay_dispatch plus indexes. Older workspaces receive a consistent pre-v12 backup. Schema 011 imports/captures/schema job history, project settings and keys are retained; the new preservation test checks those records and foreign keys. Normal user databases were not opened/migrated during acceptance. Stop older clients before a real upgrade.

Final preservation fix: the newly introduced, unreleased migration012 uses nullable target foreign keys with ON DELETE SET NULL. Removing a target preserves historical drafts/captures/jobs; its retained identities become target_removed/expired and Edit/Test are disabled with an explanation. Removing those identities still deletes their OS credentials. Core and the final native 11-checkpoint HTTP suite verify this. Earlier experimental test snapshots are retained as historical artifacts, not normal user databases.

Current verification: Windows Rust 133 pass/0 fail/1 ignored; Parrot/WSL Core+CLI 133/0/1; frontend TypeScript/Vite and 26 tests pass. Format, warnings-denied Clippy, optimized CLI/Desktop with custom-protocol, and the Tauri build command pass. All nine native suites pass with 77 checkpoints (http-replay 11, http-workspace 12, native 8, inputs 9, jobs 8, integrations 6, outputs 6, recon-local 7, ai-providers 10). Exact immutable binaries: artifacts/windows-replay-final-20261018-160538. Source/binary hashes, per-suite records and logs: artifacts/replay-current-verification.json. Embedded frontend: index-CIdHwiFQ.js / index-CwAV9qPE.css.

New native proof: artifacts/http-replay-1792314338414/results.json. Actual native HAR picker/import, Alice/Bob OS-backed identities, reviewed POST/GET, real identity-dependent fixture output, redacted credential echo, CLI/GUI shared jobs, no followed redirect, genuine cancellation, full restart and three theme/viewport dialogs pass. Screenshots include reviewed-request-editor.png, actual-response-redacted.png and auth-light-1920/auth-dark-1440/auth-reconx-1366.png in that record's directory. The screenshots were visually inspected; final PDF was independently parsed by pypdf, with no tested credential markers in PDF/JSON/HTML/Markdown or default traffic export. See its pdf-inspection.json. No release installer or Linux Desktop is established by these tests.

Retained failures/fixes: replay-full-rust-first.log exposed a Windows fixture stream inheriting nonblocking mode; accepted sockets now explicitly use blocking reads with timeouts. A JSON validator initially compared formatting rather than redaction count and was corrected. The first native review failure (artifacts/http-replay-1792312481422/results.json) had no captured backend alert; source inspection found the editor could offer stale cached revisions while fetching CLI updates. It now clears cached choices and disables selection during loading, with a dedicated frontend regression test. Do not claim the original uncaptured failure's exact cause was proven. The second native run (artifacts/http-replay-1792312573558/results.json) reached cancellation/themes but its runner parsed empty successful delete output as JSON; the runner now handles it. Earlier Linux staging missed MCP fixture files and reused cached tests embedding an older manifest path; the isolated stage now includes all fixtures and binds that test to its actual stage. Failed logs remain, and only synthetic keys created by those failed fixtures were cleaned up (artifacts/replay-fixture-key-cleanup.json).

Changed source: crates/reconx-core/src/http/{auth,replay}.rs, http.rs and jobs.rs; storage lib.rs/migration012 and preservation tests; core tests/http_replay.rs/research_inputs.rs; CLI main.rs; Tauri lib.rs; HttpPage and HttpRequestWorkspace/HttpAuthManager, HTTP styles/frontend tests; tests/e2e/http-replay.mjs; package.json, verify.ps1, CI and .gitignore. Prior schema files/migration011 remain intact. No unrelated shell/theme/font redesign.

Limits: loopback IPs or exact localhost pinned without DNS; no remote adapter, proxy, redirect following, automatic retries/cookie jars, multipart or binary uploads. State changes require Lab Research on both app/target and individual approval. Request body 64 KiB; response 256 KiB; timeout 0.5-10s; one request/approval, max20/project/minute, minimum500ms dispatch spacing. Public headers are allowlisted; secrets go through profiles. Captures are sanitized application representations, not wire transcripts; original response-body hash and total time are retained, non-UTF8 bodies omitted and existing non-JSON preview withholding retained. Known-pattern/exact-secret redaction is not universal DLP. Database/OS secret operations are not a cross-resource transaction; moving a DB requires profile key reconfiguration. A hard crash may leave an unreferenced sanitized artifact; shared lease recovery never automatically replays requests. Negative self-signed HTTPS rejection passed separately on Windows and Linux Core/CLI (replay-tls-windows.log/replay-tls-linux.log), with no OS trust changes, zero HTTP requests and ephemeral test keys removed. Trusted-CA HTTPS success, abrupt HTTP termination and Linux Secret Service runtime acceptance remain unverified.

Exact next safe task: deepen offline response/schema consistency coverage and add evidence-linked manual authorization expectations/object ownership annotations. Keep HTTP execution individually reviewed and loopback-only; do not introduce autonomous BOLA/BFLA/scanner workflows. Active authorization matrix automation, stateful sequences, broader scanners/agents/research domains, independent Linux Desktop, installers/signing and live cloud interoperability remain NOT_STARTED/PARTIAL/UNVERIFIED as recorded. Do not repeat completed imports, providers, output pipeline or this manual replay slice.

Resume commands: cargo fmt --all -- --check; cargo clippy --locked --workspace --all-targets -j2 -- -D warnings; cargo test --locked --workspace -j2; cd apps/desktop then npm run build and npm test. Use CARGO_INCREMENTAL=0 for heavy builds. Native: set RECONX_DESKTOP_EXE/RECONX_CLI_EXE to the immutable pair above, then node tests/e2e/http-replay.mjs (or scripts/verify.ps1 -HttpReplay). Linux: reuse artifacts/replay-linux-run.sh with its isolated toolchain/stage and separate stdout/stderr. CI was edited locally, not pushed or run remotely. Prepared summary only: feat(http): add reviewed loopback replay and secure project test identities.

---

Older checkpoints below are retained for provenance; the current checkpoint above takes precedence.

## Final preservation fix in progress - 2026-10-18

A final audit found that the new draft/identity target foreign keys restricted target deletion. Migration012 is still unreleased (only isolated test databases were opened); its target links now use ON DELETE SET NULL, preserving history. Removed-target identities are derived as expired/target_removed, and the new UI disables Edit/Test with an explanation. Added core and native deletion/retained-history acceptance. This source is newer than the preceding verified pair, so do not treat the old manifest as proof for these final edits yet. Windows/Linux tests, final release rebuild and the nine native suites must finish before updating the status. Logs currently use replay-target-preservation-* / replay-deletion-*; save the exact final binaries and update docs again. Do not reset prior schema011 work or user data.

---

## Verified checkpoint: reviewed loopback HTTP and secure identities - 2026-10-18

Preserved all preceding uncommitted schema 011 work and user data; no reset, Git remote inspection, commit or push. Product remains 0.1.0; full Web/API and v1.0 are incomplete. This checkpoint supersedes every earlier next-task/count entry.

Delivered a bounded manual vertical slice: imported capture -> credential-free editable draft -> explicitly reviewed exact fingerprint -> one real loopback HTTP request through the existing persistent Job Engine -> sanitized captured response/history -> evidence -> comparison requiring review -> Needs Verification finding -> reviewed Markdown/JSON/HTML/PDF. No network execution occurs on import, editing, saving or profile registration. Manual cancellation, size/time/rate limits and fail-closed scope/identity policy reuse the shared Rust core, not React analysis code or another scheduler.

Authentication manager: project/target/exact-origin identity metadata; Anonymous, Cookie, Bearer, API Key header, Basic and Custom Header; OS-store credentials with no retrieval/display API; replace, expire, delete and reviewed test entry point. Credentials use immutable workspace-scoped references; optimistic revisions prevent stale updates. Retired secret cleanup has a persistent journal and visible failure/retry path (`auth cleanup`). Saving and HTTP success never imply an identity is authenticated or authorized. No cloud transmission was performed.

Migration 012_manual_http_replay.sql adds http_auth_profiles, http_auth_retired_secrets, http_request_drafts, http_replay_jobs and http_replay_dispatch plus indexes. Older workspaces receive a consistent pre-v12 backup. Schema 011 imports/captures/schema job history, project settings and keys are retained; the new preservation test checks those records and foreign keys. Normal user databases were not opened/migrated during acceptance. Stop older clients before a real upgrade.

Current verification: Windows Rust 133 pass/0 fail/1 ignored; Parrot/WSL Core+CLI 133/0/1; frontend TypeScript/Vite and 26 tests pass. Format, warnings-denied Clippy, optimized CLI/Desktop with custom-protocol, and the Tauri build command pass. All nine native suites pass with 76 checkpoints (http-replay 10, http-workspace 12, native 8, inputs 9, jobs 8, integrations 6, outputs 6, recon-local 7, ai-providers 10). Exact immutable binaries: artifacts/windows-replay-final-20261018-155056. Source/binary hashes, per-suite records and logs: artifacts/replay-current-verification.json. Embedded frontend: index-C7xvQdZ7.js / index-CwAV9qPE.css.

New native proof: artifacts/http-replay-1792313456492/results.json. Actual native HAR picker/import, Alice/Bob OS-backed identities, reviewed POST/GET, real identity-dependent fixture output, redacted credential echo, CLI/GUI shared jobs, no followed redirect, genuine cancellation, full restart and three theme/viewport dialogs pass. Screenshots include reviewed-request-editor.png, actual-response-redacted.png and auth-light-1920/auth-dark-1440/auth-reconx-1366.png in that record's directory. The screenshots were visually inspected; final PDF was independently parsed by pypdf, with no tested credential markers in PDF/JSON/HTML/Markdown or default traffic export. See its pdf-inspection.json. No release installer or Linux Desktop is established by these tests.

Retained failures/fixes: replay-full-rust-first.log exposed a Windows fixture stream inheriting nonblocking mode; accepted sockets now explicitly use blocking reads with timeouts. A JSON validator initially compared formatting rather than redaction count and was corrected. The first native review failure (artifacts/http-replay-1792312481422/results.json) had no captured backend alert; source inspection found the editor could offer stale cached revisions while fetching CLI updates. It now clears cached choices and disables selection during loading, with a dedicated frontend regression test. Do not claim the original uncaptured failure's exact cause was proven. The second native run (artifacts/http-replay-1792312573558/results.json) reached cancellation/themes but its runner parsed empty successful delete output as JSON; the runner now handles it. Earlier Linux staging missed MCP fixture files and reused cached tests embedding an older manifest path; the isolated stage now includes all fixtures and binds that test to its actual stage. Failed logs remain, and only synthetic keys created by those failed fixtures were cleaned up (artifacts/replay-fixture-key-cleanup.json).

Changed source: crates/reconx-core/src/http/{auth,replay}.rs, http.rs and jobs.rs; storage lib.rs/migration012 and preservation tests; core tests/http_replay.rs/research_inputs.rs; CLI main.rs; Tauri lib.rs; HttpPage and HttpRequestWorkspace/HttpAuthManager, HTTP styles/frontend tests; tests/e2e/http-replay.mjs; package.json, verify.ps1, CI and .gitignore. Prior schema files/migration011 remain intact. No unrelated shell/theme/font redesign.

Limits: loopback IPs or exact localhost pinned without DNS; no remote adapter, proxy, redirect following, automatic retries/cookie jars, multipart or binary uploads. State changes require Lab Research on both app/target and individual approval. Request body 64 KiB; response 256 KiB; timeout 0.5-10s; one request/approval, max20/project/minute, minimum500ms dispatch spacing. Public headers are allowlisted; secrets go through profiles. Captures are sanitized application representations, not wire transcripts; original response-body hash and total time are retained, non-UTF8 bodies omitted and existing non-JSON preview withholding retained. Known-pattern/exact-secret redaction is not universal DLP. Database/OS secret operations are not a cross-resource transaction; moving a DB requires profile key reconfiguration. A hard crash may leave an unreferenced sanitized artifact; shared lease recovery never automatically replays requests. Dedicated HTTPS fixture, abrupt HTTP termination and Linux Secret Service runtime acceptance remain unverified.

Exact next safe task: deepen offline response/schema consistency coverage and add evidence-linked manual authorization expectations/object ownership annotations. Keep HTTP execution individually reviewed and loopback-only; do not introduce autonomous BOLA/BFLA/scanner workflows. Active authorization matrix automation, stateful sequences, broader scanners/agents/research domains, independent Linux Desktop, installers/signing and live cloud interoperability remain NOT_STARTED/PARTIAL/UNVERIFIED as recorded. Do not repeat completed imports, providers, output pipeline or this manual replay slice.

Resume commands: cargo fmt --all -- --check; cargo clippy --locked --workspace --all-targets -j2 -- -D warnings; cargo test --locked --workspace -j2; cd apps/desktop then npm run build and npm test. Use CARGO_INCREMENTAL=0 for heavy builds. Native: set RECONX_DESKTOP_EXE/RECONX_CLI_EXE to the immutable pair above, then node tests/e2e/http-replay.mjs (or scripts/verify.ps1 -HttpReplay). Linux: reuse artifacts/replay-linux-run.sh with its isolated toolchain/stage and separate stdout/stderr. CI was edited locally, not pushed or run remotely. Prepared summary only: feat(http): add reviewed loopback replay and secure project test identities.

---

Older checkpoints below are retained for provenance; the current checkpoint above takes precedence.

## Work in progress: manual loopback HTTP replay - 2026-10-18

Preserved all prior uncommitted schema 011 work and user data. Added migration 012, secure project/target/origin-bound HTTP identity metadata, reviewed request drafts, exact fingerprint approvals, one-request loopback HTTP execution through the existing Job Engine, sanitized actual traffic/evidence, response history, typed Desktop and CLI services. No automatic replay, general remote HTTP, proxy/redirect following, stateful authorization automation or scanner orchestration has been added. Product remains 0.1.0 and overall Web/API remains PARTIAL. See HTTP_REPLAY.md for services, limits and commands.

Current verified source: Windows Rust 133 passed/0 failed/1 ignored; Parrot/WSL Core+CLI 133/0/1; frontend TypeScript/Vite and 25 tests pass; warnings-denied Clippy passes. Logs: artifacts/replay-final-{rust,clippy,frontend-build,frontend-tests}.log and replay-linux-current-{stdout,stderr}.log. Windows release CLI/Tauri rebuild and final native suites are still IN PROGRESS. Do not mark this new native milestone complete until the final matched binaries and all suites pass.

Actual preliminary native runs have already exercised OS-backed Alice/Bob credentials, real POST/GET, redacted credential echo, response comparison, Needs Verification finding, four report formats, restart and redirect blocking. artifacts/http-replay-1792312481422/results.json failed at a later review click (cause not established); artifacts/http-replay-1792312573558/results.json reached cancellation and three theme screenshots then hit a runner JSON parse of a successful empty-output deletion. Runner now accepts empty mutation output and records safe error alerts; neither failed run is claimed as a full pass. Added modest scoped spacing/width for the two new dialogs after inspecting actual screenshots; no shell, theme or typography redesign.

Corrected an intermittent Windows test fixture: explicitly set accepted streams to blocking mode, since the nonblocking listener could cause a request read to return before HTTP bytes arrived. Retained artifacts/replay-full-rust-first.log; full corrected suite passes. Earlier Linux source staging missed the MCP fixture, then reused cached tests embedding the old CARGO_MANIFEST_DIR. Current stage contains all e2e fixtures and recompiles that test against its actual stage; full current suite passes. Failed logs remain in artifacts/replay-linux-{stdout,stderr}.log and replay-linux-final-{stdout,stderr}.log.

Exact continuation: await current optimized build; run npm run tauri -- build --no-bundle against the final frontend (index-_LMva_I0.js); then python artifacts/verify-replay-native.py. This snapshots the matching release executables and runs nine suites sequentially. Inspect new screenshots and independently parse the new PDF. Fix real errors, rerun affected checks, produce artifacts/replay-current-verification.json, update all six status docs and module-status.json to schema 12 with actual results. Do not push/commit. Normal user data was never opened by acceptance tests.

Current changed files: core http/auth.rs, http/replay.rs, http.rs/jobs.rs; storage migration012/lib tests; CLI main; Tauri bridge; HttpPage, HttpRequestWorkspace, HttpAuthManager, HttpReplay tests/http.css; tests/e2e/http-replay.mjs; package.json, verify.ps1, .gitignore and HTTP_REPLAY.md. Prior schema files remain uncommitted and must be preserved. auth add uses secure prompt/stdin, never secret args; no raw secrets in ordinary SQLite. New native tests use only isolated synthetic identities and fixture records. No cloud model data transmission.

---

Older checkpoints below are retained; this in-progress checkpoint takes precedence.

## Current checkpoint: offline schema observations - 2026-10-18

Continued the clean local tree at HEAD fe76e79; no reset, remote inspection, commit or push. Product remains 0.1.0; v1.0 and the overall Web/API module remain incomplete. This checkpoint supersedes historical next-task/count descriptions below.

Implemented a bounded, shared offline schema review in crates/reconx-core/src/http/schema.rs. Request/response JSON type/required/property/array-item checks, response status/range/default precedence, media-type checks, required header/parameter presence and authentication-mechanism observations have explicit classifications. Unsupported constraints/dialects, redacted values, omitted bodies, cycles and resource limits remain inconclusive. Each observation links original evidence IDs, HAR field/decoded JSON locations and the actual OpenAPI constraint pointer. No finding is automatically created or verified; network_requests=0.

Migration 011_offline_schema_review.sql adds only a binding table/index to the existing Job Engine. Older schemas receive a consistent pre-v11 backup; normalized inputs and both original hashes are checked. Existing HAR/OpenAPI records and AI settings remain intact. New captures retain their original entry pointer; older uniquely identifiable captures work without rewriting them, while ambiguous identical duplicates require reimport for this new review. Normal user data was not opened/migrated by acceptance tests.

Desktop: select a capture, choose a documented operation, Check documented schema, inspect progress/results/provenance or open evidence. CLI: api schema-check <capture> <operation>, jobs run, jobs result <job>, evidence from-job <job>. Target/project associations must match exactly and the documented absolute server must match scheme/host/effective port/method/path. Relative server definitions remain browseable but cannot establish an origin for schema review. Source fingerprints prevent changed queued inputs from producing stale results.

Verified: Windows Rust 124 passed/0 failed/1 ignored; Linux Parrot/WSL Core+CLI 124/0/1; frontend 23 passed. Formatting, warnings-denied Clippy, TypeScript/Vite, optimized Tauri and CLI pass. Eight native suites pass with 66 checkpoints using immutable binaries in artifacts/windows-schema-20261018-113252. artifacts/schema-current-verification.json records binary/source hashes and exact per-suite results. Native HAR/OpenAPI/schema selection, intentional JSON mismatch, no automatic finding, result evidence, all four report formats, restart and CLI parity passed. Independent pypdf inspection found schema evidence in the six-page report and none of six synthetic credential markers. Three themes and desktop sizes 1920/1440/1366 were exercised. Linux Desktop, installers/signing, live cloud accounts, full JSON Schema and active authenticated testing remain unverified/incomplete.

Retained failures: artifacts/http-workspace-1792297973145/results.json selected the deliberately minimal YAML definition instead of the JSON definition that actually requires id/owner. The backend correctly produced no such mismatch; the native test now selects the explicit JSON operation. Intermediate schema test fixtures contained duplicate entries and were made independent; the new CLI test initially supplied a job ID to jobs run, and one migration assertion still expected schema 10. These were corrected, with raw logs retained. WSL commands are now passed as a script file to avoid Windows argument expansion. No passing rerun is presented as proof that those failed invocations passed.

Changed files: core http.rs/http/schema.rs/jobs.rs; storage lib.rs/migration 011; core http_schema.rs/research_inputs.rs tests and schema HAR fixture; CLI main.rs/tests; Tauri lib.rs; HttpPage/http.ts/SchemaResults/frontend test; tests/e2e/http-workspace.mjs; HTTP_SCHEMA_ANALYSIS.md and status documents. Existing typography, three themes, backend/provider settings and SECURITY.md are preserved.

Exact next safe task: implement reviewed request drafts and a manually approved, strictly bounded owned-loopback HTTP fixture replay slice through this same Job Engine, with out-of-scope/redirect/DNS destination rejection, timeout/cancel/size/redaction tests. Then protect synthetic authentication profiles before any local multi-identity comparison. Do not add general scanner automation or claim arbitrary-target authorization testing is implemented. Preserve offline browsing/import and existing scope/approval boundaries.

Resume commands: cargo fmt --all -- --check; cargo clippy --locked --workspace --all-targets -j2 -- -D warnings; cargo test --locked --workspace -j2; cd apps/desktop then npm run build and npm test. Set CARGO_INCREMENTAL=0 for heavy builds. Native acceptance: select the immutable binaries with RECONX_DESKTOP_EXE/RECONX_CLI_EXE and run node tests/e2e/http-workspace.mjs from the repository root. Reuse the isolated Linux toolchain/stage paths in artifacts/schema-linux-{run.sh,stage.txt}; capture streams separately. Stop old clients before upgrading a real user database. Prepared summary only: feat(web): add bounded evidence-linked offline schema review jobs.

---

Older checkpoint sections below are retained for provenance. The checkpoint above takes precedence.

## Verified Web/API checkpoint - 2026-10-18

Product version remains 0.1.0. Full v1.0 is NOT release-ready. The bounded offline Web/API slice is RUNTIME_VERIFIED on Windows; the complete Web/API module is PARTIAL.

Completed: bounded HAR 1.2/base64 and OpenAPI 3.0/3.1 JSON/YAML ingestion; native picker/drop review; persistent redacted traffic and notes; origin/port/target-aware endpoint correlation; all-page captured identity/role matrix; original-source integrity/provenance; offline header jobs using the existing worker/evidence; separately consented mock AI interpretation; Needs Verification findings; reviewed Markdown/JSON/HTML/PDF output. Original traffic stays Restricted evidence and no imported request is replayed.

The latest hardening fixes concrete-path precedence, bounded one-pass server expansion, total correlation budgets, foreign-target documentation matches, overlapping-target grouping and incomplete first-page exports. A real stale Recon UI failure was reproduced by a deterministic test and fixed by discarding superseded async loads. Build/verify scripts disable incremental compilation for their duration and restore the caller setting; no cache/data deletion was performed.

Verified current source:

- Windows format and warnings-denied Clippy pass; 112 Rust tests pass, zero fail, one explicit OS-store fixture ignored.
- Frontend TypeScript/Vite and 22 tests pass. Actual optimized Tauri and CLI builds pass.
- All 8 native suites pass, totaling 64 checkpoints, including the extended 10-checkpoint HTTP suite. Actual file dialogs, job states/cancellation/failure/recovery, secure synthetic keys, YARA-X, benign MCP, local mock AI, target partitioning, reports and restart are exercised.
- Linux Parrot/WSL shared Core/CLI: 112 pass, zero fail, one ignored. Rust 1.95.0 was restored in a task-local /var/tmp directory from the official checksum-verified rustup installer, without sudo/profile changes. Linux Desktop, Secret Service runtime and Linux packaging are not established by these tests.

Exact matching portable artifacts: `artifacts/windows-web-api-20261018-105742/` (two executables, SHA256SUMS.txt and font notices). Manifest/source hashes: `artifacts/http-current-verification.json`. Current HTTP native record: `artifacts/http-workspace-1792296085396/results.json`. Main logs: artifacts/http-verified-{format,clippy,rust,tauri,cli-release}.log; artifacts/http-verified-frontend-tests.log; artifacts/http-current-linux.log and its separate stderr log. The embedded frontend is index-BNYH02oQ.js.

Actual screenshots were viewed: `artifacts/http-workspace-1792296085396/http-reconx-1920.png`, light-1920, dark-1366 and http-target-partition-matrix.png. HTTP workspace captures cover all three themes at 1920/1366. An independent pypdf parse verified the actual 4-page HTTP report and absence of synthetic credentials: `artifacts/http-workspace-1792296085396/pdf-inspection.json`.

Retained failures: stale Recon UI at artifacts/recon-local-1792294752360/results.json (fixed and retested); a prior Node native-runner abort 3221226505 at artifacts/http-release-verification.json (cause unestablished, fresh current suite passes); initial Linux staging/toolchain failures in http-final-linux*.log. Mixed WSL stdout/stderr capture was corrected; final logs use separate streams. A passing rerun does not establish the abort's root cause.

Source changes: crates/reconx-core/src/http.rs, jobs.rs, lib.rs; storage migration 010/storage lib; CLI main/tests; Tauri typed request bridge; HttpPage/http models/styles/tests; ReconPage async-load guard/regression; App/navigation; fixtures and native HTTP suite; Cargo lock/dependency; build/verify/CI and documentation. Existing SECURITY.md edits are preserved. No user DB, project, credential, sample or evidence was deleted; no Git remote inspection, commit or push occurred.

Migration: 010_http_workspace.sql only; no 011 is needed for derived matrix/correlation fields. Schemas before 10 receive a consistent pre-v10 backup. Preservation/recovery tests are part of the current full suite. Stop older clients before an actual user-workspace upgrade.

Current task is complete for the offline acceptance slice. Exact next safe task: extend offline documented-response/schema consistency observations, using actual captures and local references with bounded validation; expose evidence-linked observations, never automatic Verified findings. Start in crates/reconx-core/src/http.rs and tests/http_workspace.rs. Do not repeat provider setup, evidence/report foundations or this import milestone.

Remaining Phase 1 acceptance: full JSON Schema/reference compatibility, request editing/replay and protected authenticated test-identity execution. The current captured matrix does not execute accounts or prove BOLA/BFLA. Project export currently covers the full project rather than the active target/search filter. Later scanner/agent, debugger/mobile/cloud/network/lab modules remain PARTIAL/NOT_STARTED. Windows installer/signing, Linux Desktop/packages, live cloud access and hard OS process isolation remain open. Keep release.ready=false.

Resume: read this checkpoint, inspect git status --short (no remotes), then use the current portable binaries and tests/e2e/http-workspace.mjs for targeted acceptance. scripts/verify.ps1 -Native -Visual -Inputs -Jobs -Integrations -Outputs -LocalInventory -AiProviders -HttpWorkspace runs the full Windows verification. Linux task paths are in artifacts/http-final-linux-{toolchain,stage}.txt; reuse isolated source/build caches, capture stdout/stderr separately, and never claim Desktop support from CLI tests.

Prepared commit summary only (no commit created): feat(web): persist offline HAR/OpenAPI workspaces and evidence-linked capture reviews.

---

Older checkpoint sections below are retained for provenance. The current checkpoint above takes precedence.

## Final verification in progress - 2026-10-18

The source additionally partitions HTTP inventory/matrix/documentation matches by
target association (including the all-project view). New HTTP service tests total
11; latest Windows full suite is running toward 112 passed, one ignored. Frontend
now has 22 passing tests. A new deterministic ReconPage test first reproduced an
older queued load overwriting a completed job; a request-version guard and project
cleanup fixed it. Actual native loopback acceptance then passed all 7 checkpoints.

Keep both earlier records: artifacts/recon-local-1792294752360/results.json is the
reproduced stale UI failure; artifacts/http-release-verification.json records a
later Node native-runner abort (3221226505, no diagnostic) after recon/http/AI passed.
The abort cause is not established. Do not hide either failure or claim the current
new target-partition source passed native acceptance until it is rebuilt/retested.

Linux Rust 1.95.0 was restored in an isolated /var/tmp task directory using the
official rustup-init binary and its verified SHA-256; no sudo or profile edits.
The full Core/CLI suite then passed 111 tests, one ignored. Initial staging missed
manifests and native-sample.rs; those omissions were corrected and failed logs kept.
Do not report the former missing-toolchain blocker as unresolved after restoration.
Capture Linux stdout/stderr inside WSL separately to prevent Windows handle overlap.

Next: finish artifacts/http-verified-{format,clippy,rust,tauri,cli-release}.log;
refresh the current source in the isolated Linux stage and rerun its full suite;
copy exact optimized binaries to a new artifact folder and run all 8 native suites;
inspect current screenshots/PDF, then consolidate all status documents/registry.
The full product and active replay/authenticated execution remain incomplete.

## Earlier continuation - 2026-10-18

The existing schema-010 offline Web/API slice is preserved. On resumption the
working tree still contained all HTTP changes and the earlier SECURITY.md edit.
Baseline HTTP tests passed 6/6 and frontend 20/20. D: now has about 78 GiB free;
no cache deletion was performed by this session. Build/verify scripts now disable
incremental compilation for their duration and restore the caller's preference.

Implemented next: origin-aware OpenAPI correlation (scheme/port, concrete-path
precedence), one-pass bounded server variable expansion and a total pattern-match
budget; a real captured identity/role matrix across all pages; complete bounded
redacted capture export; source-hash provenance and integrity checks on comparisons;
target-filtered imports/history. No migration beyond additive 010 is needed.
HAR import, comparison and analysis remain offline. Replay, identity credential
execution, general scanner automation and the full product remain PARTIAL.

Current verification: 10 HTTP service tests passed, TypeScript/Vite build passed,
21 frontend tests passed, formatting and Clippy passed. Full Rust tests, optimized
Tauri/CLI, native regression and Linux shared Core/CLI are running or next.
Logs use artifacts/http-final-*.log. Exact next action: finish these checks,
inspect clean three-theme screenshots, update every status document and record
the matching binary hashes. Never substitute historical native results for the
current source. No Git remote access, user DB opening, push or cloud test.

## Historical runtime and disk-space checkpoint

Current Web/API source passed 107 Rust tests on Windows and 107 on Linux shared
Core/CLI (one ignored each), frontend 20 tests, TypeScript/Vite, formatting and
Clippy. Debug native acceptance passed 8 actual checkpoints:
`artifacts/http-workspace-1792241586062/results.json`. HAR/native picker, OpenAPI
JSON/YAML, actual offline jobs/evidence, separately consented mock AI, capture
comparison, unverified finding, four CLI report exports and restart are verified.
No production release or full Phase 1/v1.0 completion is claimed.

User next requested a disk-usage diagnosis. Read-only measurement found ~69.7 GiB
logical project size: target ~64.0 GiB (debug incremental ~25.6 GiB, debug deps
~29.8 GiB) and artifacts ~5.5 GiB. D: has ~24.9 GiB free of 238.5 GiB. NTFS hard
links can cause logical sums to exceed unique allocation. No cleanup was performed.
See artifacts/disk-usage-latest.json. Avoid more large builds until cache growth is
addressed. Preserve source, SECURITY.md changes, raw evidence and source snapshots.

Next: agree/perform appropriate generated-cache cleanup, reduce future debug/test
cache growth, finish optimized Windows build/native regression, inspect clean
workspace screenshots (existing ones include the deliberately tested error banner),
harden large server-template correlation budgets, then update all status docs.
Active replay/authenticated identity execution and remaining product phases are
still incomplete. Linux Desktop/signing/installers/live cloud remain unverified.

## Web/API offline workspace - current work (2026-10-17)

Base HEAD: `70bc08d`. The pre-existing SECURITY.md edit is preserved. No Git remote
configuration, credentials, user DB, imported samples or evidence were inspected
or reset. Baseline current-source Rust tests (99 passed, one ignored) and frontend
17 tests passed again before implementation.

Implemented in source (native acceptance IN PROGRESS): HAR 1.2 bounded import,
base64 content decoding, redacted request/response inventory, native picker/drop
review, OpenAPI 3.0/3.1 JSON/YAML with same-document pointers and parser budgets,
endpoint correlation, captured-role diff/ownership indicators, notes, restricted
original evidence, review-only findings and exports. A fixed offline captured-header
handler uses the existing Job Engine, evidence and explicit selected-result AI
consent. CLI shares every backend service. Schema 010 is additive and snapshots
older databases to pre-v10 backups. Existing AI routes and schema 009 remain intact.

This is an OFFLINE analysis slice, not full Phase 1 or v1.0 acceptance. Active
replay/request editing, authenticated test identities, automatic BOLA/BFLA testing,
full JSON Schema validation and external/multi-file references remain unavailable.
No imported request is sent to any endpoint. AI review is a separate explicit
preview/destination approval and is tested only against an owned loopback mock.

Current tasks: finish Rust/Clippy/frontend checks; build actual Tauri/CLI; run
`tests/e2e/http-workspace.mjs` through the native file picker and restart; inspect
screenshots; run native regression; test Linux shared Core/CLI if WSL is available.
Source before changes: artifacts/web-api-source-1792240236/before.tar.gz.
Do not mark the new slice RUNTIME_VERIFIED until the native result exists.

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

Resume checks: inspect this ledger and `git status --short`, then
`powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify.ps1 -AiProviders`
for the AI slice. Full native regression also uses `-Native -Visual -Inputs -Jobs
-Integrations -Outputs -LocalInventory`. Do not print credential-bearing remotes.
Suggested commit summary only (no commit created):
`feat(ai): add secure provider/model setup, discovery, health and shared routing`.

# ReconX AI v1.0 execution ledger

Updated: 2026-10-17. Baseline a29c5db was clean before this work. Local changes
are preserved; no Git remote configuration, credential rotation, commit or push.
Existing layout/fonts and exactly Light/Dark/ReconX remain unchanged.

## Current checkpoint

- TESTED bounded output slice: actual jobs -> immutable evidence -> manual findings
  with verification gates/CVSS -> reviewed redacted Markdown/JSON/HTML/PDF reports.
- TESTED bounded inventory slice: one manually approved HEAD to a scoped loopback
  URL through the existing worker, with assets/private evidence, cancel/failure,
  restart and CLI parity. Full reconnaissance is PARTIAL, not complete.
- Windows optimized Desktop/CLI are built and all six native suites passed (44
  checkpoints). Normal Desktop is open from artifacts/windows-x64; recorded owned
  PID 10840, title ReconX AI, zero stderr at verification. Do not kill it merely
  to rebuild; check identity and active jobs before any normal shutdown.
- Linux shared Core/CLI: 86 Rust tests passed, one explicit OS-store test ignored.
  Actual CLI imported /usr/bin/true ELF, independently matched SHA-256, retained
  29 sections, saved evidence/unverified observation and exported all four report
  formats. Independent pypdf extracted Vietnamese from the Linux-generated PDF.
- Linux Desktop is UNVERIFIED: Parrot lacks webkit2gtk-4.1 development SDK. No
  privileged package or profile changes. Optimized Linux CLI/installers unverified.
- Full v1.0 is NOT ready. Do not bump the product version or claim all domains work.

## Current/next task

Next product vertical slice: bounded OFFLINE HAR ingestion and endpoint inventory,
then OpenAPI import and deterministic header/cookie observations. Reuse existing
assets/evidence/jobs and project/target boundaries. Keep raw authentication and
cookies only in private artifacts; typed normalized records omit secret values.
Use local synthetic traffic, hard input/depth/entry limits and real import/native
picker/CLI/restart tests. Import must never replay requests or execute code. Design
one migration only after auditing schema 006; do not duplicate target/job tables.

Before resuming: inspect Git status and this ledger. Do not rerun a completed
phase merely because another document's historical section lists it as planned.
Current Windows verification:
  powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify.ps1 -Outputs -LocalInventory
Full native regression flags also include -Native -Visual -Inputs -Jobs -Integrations.
Build: scripts/build.ps1; do not overwrite a running normal executable. Tauri
build --no-bundle was also executed successfully on the current product source.

## Feature inventory

| Area | State | Scope / remaining acceptance |
| --- | --- | --- |
| Projects / targets / scope | TESTED | Existing Windows native/CLI and Linux Core/CLI fixtures |
| Fonts / three themes | TESTED | Preserved; Linux Desktop rendering unverified |
| Shared jobs | TESTED | Claims/cancel/timeout/recovery; no general DAG/pause/retry |
| Local binary workspace | PARTIAL | Tested PE/ELF/thin Mach-O/ZIP/APK metadata; no decompiler |
| Evidence | TESTED | Immutable artifacts/hash/review/import/export and job provenance |
| Manual findings/CVSS | TESTED | Lifecycle/revisions/links/duplicates, complete supplied 3.1/4 vectors |
| Reports | PARTIAL | Four tested formats; seven labels share text-first layouts |
| Loopback HTTP inventory | TESTED | Exact manual approval only; no scanners or AI selection |
| Full passive/active recon | PARTIAL | DNS/RDAP/CT/history/crawling/scanner adapters unimplemented |
| Web/API analysis | NOT_STARTED | Offline HAR/OpenAPI next; no identity matrix/replay service |
| Network assessment | NOT_STARTED | No network import/service inventory module |
| Tool manager | PARTIAL | Windows ripgrep/YARA-X lifecycle; remaining catalog integrations pending |
| MCP | PARTIAL | Real stdio/HTTP benign fixtures; no OS sandbox/OAuth/security adapters |
| AI provider protocols | PARTIAL | Local fixtures; live accounts/rate/cost/budget/routing unverified |
| Reviewed local AI plans | TESTED | Mock planner, real local jobs, consented unverified interpretation |
| General multi-agent orchestration | NOT_STARTED | No multi-domain DAG/agents/Authorized Auto execution |
| Ghidra/debugger integrations | NOT_STARTED | Generic discovery is not validated interoperability |
| Android/iOS | PARTIAL | APK identification/thin Mach-O only; advanced/device work unsupported |
| Cloud/container/CI/CD | NOT_STARTED | No real analyzers yet |
| Red team lab | NOT_STARTED | No lab inventory/trust model |
| Windows portable release | TESTED | Current exact optimized artifacts passed native acceptance |
| Windows installer | PARTIAL | Configuration only; generation/install/signing unverified |
| Linux Core/CLI | TESTED | Parrot WSL2 debug runtime; not a Desktop/package claim |
| Linux Desktop | BLOCKED | Missing WebKit SDK; no privileged automatic installation |
| Linux/ARM release packaging | PARTIAL | Configuration only; runtime/package acceptance still required |
| v1.0 release | BLOCKED | Remaining domains, hardening and installer acceptance open |

## Migrations and decisions

005_research_outputs.sql adds evidence/findings/links/reports/exports. Artifacts
are private UUID-named files outside SQLite. Completed job transactions publish
result references plus observed evidence. Original bytes are immutable; integrity
is distinct from vulnerability verification. Verified findings require explicit
manual confirmation, a review note and reviewed non-AI source evidence.

006_local_inventory.sql adds exact HTTP approvals and normalized assets. Approval
fingerprints bind target input and security policy; validation runs while waiting
and before commit. No second scheduler, unrestricted shell or autonomous network
path was added. One HEAD, no redirect/proxy/auth/cookie jar/body download/retry,
five-second deadline, 20 approvals/project/minute, existing global two-job limit.

Current migration code snapshots schemas 1-5 to pre-v6 backups before a single
additive transaction. Default schema 4 -> 6 upgrade preserved hashes/counts of
all 13 original tables, snapshot matches before state, no foreign-key errors.
Record: artifacts/release-upgrade-1792216876468/after.json. No test data was seeded
into the normal user database. Recovery is forward repair/restore a private backup
with clients stopped, not an automatic destructive downgrade.

Reports are immutable snapshots requiring exact-preview approval. Raw images and
binaries stay separate. Structural redaction covers credential fields/header
pairs/base64 values and nested JSON; deeper serialized JSON is visibly withheld.
Detectable unsafe older snapshots require regeneration, never silent rewriting.
PDF uses bundled licensed Source Sans 3, explicit weight 400 and glyph/line limits.

## Verified commands and records

Windows: cargo fmt --all -- --check; cargo clippy --locked --workspace --all-targets
-- -D warnings; cargo test --locked --workspace: 86 passed, 0 failed, 1 ignored.
Frontend npm run build / npm test: 13 passed. Modified native-fixture tests were
also rerun individually after the Linux fixture correction (CLI 5 + inputs 8).
Tauri optimized no-bundle build and optimized CLI build passed. Exact artifact
hashes: artifacts/windows-x64/SHA256SUMS.txt. Known Git-token/private-key pattern
scan found zero matches in these two binaries; this is not a complete secret audit.

Optimized native acceptance (all passed):
- artifacts/native-1792217533338/results.json (8)
- artifacts/inputs-1792217578167/results.json (9)
- artifacts/jobs-1792217588488/results.json (8)
- artifacts/integrations-1792217614199/results.json (6)
- artifacts/outputs-1792217631931/results.json (6)
- artifacts/recon-local-1792217644790/results.json (7)

Latest debug output/local runs:
- artifacts/outputs-1792217139477/results.json
- artifacts/recon-local-1792217154231/results.json
Other debug suites passed earlier in this phase; TEST_RESULTS.md keeps records.
Actual screenshots for all three themes and a rasterized release PDF were viewed.
Native report PDFs were independently parsed with installed pypdf; no fake result
or vulnerability was generated. Owned fixtures and mock models are labelled.

Linux: artifacts/linux-verify-1792217270990/acceptance.json, cargo-test-final.log,
test-result-final.json and installer.json. Official rustup-init SHA-256 checked
before execution. Isolated Rust 1.95.0 and source live under
/tmp/reconx-linux-1792217270990 inside Parrot; CARGO_HOME=cargo and RUSTUP_HOME=rustup
below that directory. No profile/global package changes. Source snapshot excludes
.git, credentials, user datasets, node_modules and output artifacts.
Command: cargo test --locked --workspace --exclude reconx-desktop with task-local
Rust environment, Node 20.19.2 and two compiler jobs. CLI process/restart consistency,
actual ELF parsing, loopback/MCP/mock-model and all output fixtures ran on Linux.
Evidence Unix mode 600 and directory mode 700 were independently checked.

Linux initially rejected oversized DWARF test executables. Shared test-only helper
now strips debug sections from a private executable copy using binutils, retaining
the original; production limits unchanged. Initial CLI smoke used an invalid
severity alias info; fixture corrected to informational, product enum unchanged.

## Fixed defects and remaining risks

Fixed during execution: incomplete CVSS vectors silently defaulted by the library;
HAR name/value secret redaction; embedded JSON/base64 report leak; Windows path
pattern incorrectly redacting HTTP URLs; close buttons silently ignoring busy
clicks; native accessibility labels; test locator/restart/cross-process picker
issues; Winsock accepted sockets inheriting nonblocking mode. Failed artifacts
are retained and the actual fixes/acceptance are recorded in TEST_RESULTS.md.

Remaining: latest-1000 list limits, richer taxonomy/report layouts and attachment
presentation; custom Windows workspace ACL review, no hard process sandbox, no
same-account race-free filesystem guarantee, unencrypted private artifacts, orphan
artifact recovery/retention, unknown-secret manual review, live provider accounts,
agent budgets/routing, remaining research modules, Linux Desktop/packages, Windows
installer/signing and complete dependency/license/release review. No arbitrary
exploitation or autonomous offensive execution is implemented.

## Modified files

- .github/workflows/ci.yml
- .gitignore
- Cargo.lock
- README.md
- apps/cli/src/main.rs
- apps/cli/tests/cli.rs
- apps/desktop/package-lock.json
- apps/desktop/package.json
- apps/desktop/src-tauri/src/lib.rs
- apps/desktop/src/App.test.tsx
- apps/desktop/src/App.tsx
- apps/desktop/src/components/QuickActions.tsx
- apps/desktop/src/components/TargetDialog.tsx
- apps/desktop/src/components/navigation.ts
- apps/desktop/src/components/ui.tsx
- apps/desktop/src/main.tsx
- apps/desktop/src/pages/TargetsPage.tsx
- apps/desktop/src/pages/WorkspacePages.tsx
- apps/desktop/src/research.ts
- crates/reconx-core/Cargo.toml
- crates/reconx-core/src/capabilities.rs
- crates/reconx-core/src/jobs.rs
- crates/reconx-core/src/lib.rs
- crates/reconx-core/src/mcp.rs
- crates/reconx-core/src/reverse.rs
- crates/reconx-core/tests/research_inputs.rs
- crates/reconx-storage/src/lib.rs
- docs/ARCHITECTURE.md
- docs/DECISIONS.md
- docs/FUNCTIONALITY_GAP_AUDIT.md
- docs/IMPLEMENTATION_STATUS.md
- docs/NEXT_MILESTONE.md
- docs/ROADMAP.md
- docs/SECURITY_MODEL.md
- docs/TEST_RESULTS.md
- docs/module-status.json
- scripts/verify.ps1
- tests/e2e/select-file.ps1
- apps/desktop/src/pages/ReconPage.tsx
- apps/desktop/src/pages/RecordsPage.tsx
- apps/desktop/src/records.css
- crates/reconx-core/src/evidence.rs
- crates/reconx-core/src/findings.rs
- crates/reconx-core/src/recon.rs
- crates/reconx-core/src/reporting.rs
- crates/reconx-core/tests/local_inventory.rs
- crates/reconx-core/tests/outputs.rs
- crates/reconx-storage/migrations/005_research_outputs.sql
- crates/reconx-storage/migrations/006_local_inventory.sql
- docs/LOCAL_INVENTORY.md
- docs/MASTER_EXECUTION_PROGRESS.md
- docs/RESEARCH_OUTPUTS.md
- tests/e2e/native-sample.rs
- tests/e2e/outputs.mjs
- tests/e2e/recon-local.mjs
- tests/e2e/requirements.txt

## Prepared commit summary

feat(core): persist reviewed research outputs and approved loopback inventory

- Add evidence/finding/CVSS services and immutable redacted four-format reports.
- Reuse jobs for exact manually approved local HTTP inventory and asset evidence.
- Add Desktop/CLI workflows, safe migration backups and actual restart acceptance.
- Preserve typography/themes/providers; verify Windows native and Linux Core/CLI.

No commit or push was performed. Do not publish releases without explicit approval.
