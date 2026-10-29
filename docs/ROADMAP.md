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

---

Earlier roadmap/checkpoint history:

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

## Current roadmap checkpoint - manual HTTP slice, 2026-10-18

Delivered the manually approved owned-loopback request/identity vertical slice on migration012 through the same Desktop/CLI services, worker, traffic, evidence and report pipeline. Current proof: artifacts/replay-current-verification.json; Windows/Linux Core+CLI Rust 133 each, frontend 26, nine Windows native suites/77 checkpoints. This is not the full active Web/API milestone.

Next: deepen bounded offline schema observations and evidence-linked manual authorization expectations/ownership review, then address remaining trusted-CA TLS/termination/credential-store acceptance gaps. Keep raw evidence separate from interpretation and all local sends individually reviewed. Automatic authorization testing, stateful request sequences, arbitrary-target scanner orchestration and future domains remain incomplete. Linux Desktop, packaging/installers/signing and live cloud interoperability require independent proof. Preserve the existing UI/three themes/fonts, user data and 0.1.0 version; v1_release.ready remains false.

---

Older checkpoints below are retained for provenance; the current checkpoint above takes precedence.

## Current roadmap checkpoint - manual HTTP slice, 2026-10-18

Delivered the manually approved owned-loopback request/identity vertical slice on migration012 through the same Desktop/CLI services, worker, traffic, evidence and report pipeline. Current proof: artifacts/replay-current-verification.json; Windows/Linux Core+CLI Rust 133 each, frontend 26, nine Windows native suites/76 checkpoints. This is not the full active Web/API milestone.

Next: deepen bounded offline schema observations and evidence-linked manual authorization expectations/ownership review, then address dedicated TLS/termination/credential-store acceptance gaps. Keep raw evidence separate from interpretation and all local sends individually reviewed. Automatic authorization testing, stateful request sequences, arbitrary-target scanner orchestration and future domains remain incomplete. Linux Desktop, packaging/installers/signing and live cloud interoperability require independent proof. Preserve the existing UI/three themes/fonts, user data and 0.1.0 version; v1_release.ready remains false.

---

Older checkpoints below are retained for provenance; the current checkpoint above takes precedence.

## Current roadmap checkpoint - offline schema slice, 2026-10-18

Implemented and verified bounded offline schema observations, migration 011, shared existing jobs/evidence, Desktop operation selection and CLI parity. Windows/Linux Core+CLI Rust 124 each; frontend 23; eight native suites/66 checkpoints; optimized Desktop/CLI. artifacts/schema-current-verification.json contains exact proof. This is partial schema coverage, with explicit unsupported/inconclusive outcomes rather than automatic vulnerability findings.

Next deliver reviewed request drafts and manually approved owned-loopback fixture replay, then secure synthetic auth profiles and evidence-backed local authorization comparisons. General scanner automation, full JSON Schema, broader agents/research domains, Linux Desktop and release installers/signing remain incomplete. Preserve existing fonts/themes/layout/data; keep product 0.1.0 and v1_release.ready=false.

---

Older checkpoint sections below are retained for provenance. The checkpoint above takes precedence.

## Current roadmap checkpoint - 2026-10-18

Delivered and tested the offline Web/API vertical slice on schema 010 through Desktop and CLI. HAR/OpenAPI -> actual captured inspection/identity comparison -> persistent evidence -> Needs Verification finding -> reviewed Markdown/JSON/HTML/PDF works. Current proof: 112 Rust tests per Windows/Linux Core-CLI, 22 frontend tests and 64 native checkpoints; see `artifacts/http-current-verification.json`.

Next deepen bounded offline response/documentation validation, followed by hardening and the remaining approved local research capabilities. Full request replay/authenticated identity execution, broader recon/scanner integrations, multi-domain agents, external debugger/mobile/cloud/network/lab modules, installers/signing and Linux Desktop remain incomplete. A catalog entry, parser field or route is not completion of a whole research domain. Keep product 0.1.0 and release.ready=false until independent acceptance.

CI now includes native HTTP and provider suites and disables incremental cache; it has not been pushed or run remotely. Existing historical checkpoints below remain evidence of their stated versions, not proof of current whole-product readiness.

---

Older checkpoint sections below are retained for provenance. The current checkpoint above takes precedence.

## Current Web/API checkpoint - 2026-10-18

Offline HAR/OpenAPI import and traffic inventory now exist on schema 010. Preserve
them and complete current optimized/native verification before extending modules.
The captured identity matrix, integrity-linked comparisons and review-only output
pipeline are implemented; request replay, authenticated identity tests and general
scanner automation are not. WEB_API_WORKSPACE.md states exact support and limits.
The product remains 0.1.0, with v1.0 release criteria open. Linux re-verification is
blocked by a missing task-local toolchain; Linux Desktop and installers are unverified.

## Historical AI setup acceptance update - 2026-10-17

The current-source AI Provider Manager slice is TESTED: 24-provider registry,
secure setup/model profiles, real protocol fixtures, discovery, custom gateway
options, saved health and agent routes. Windows release/native regression passed
54 checkpoints; shared Rust tests passed 99 on Windows and 99 on Linux/WSL;
frontend passed 17. See MASTER_EXECUTION_PROGRESS.md for current paths and scope.
Live cloud accounts, capability/pricing refresh, advanced budget/fallback control,
Linux Desktop and installers remain unverified or incomplete. Next implement
bounded offline HAR/OpenAPI ingestion on schema 009 using existing evidence/jobs.
Historical roadmap sections below remain for provenance, not current test totals.

## Current continuation - 2026-10-17

Phase 1 has Windows native/debug/optimized acceptance for evidence, manual findings
and four report formats. Preserve this slice. Phase 2 implements manually approved
loopback HEAD inventory using the same targets, jobs, assets and output services;
Windows optimized native regression passed (44 checkpoints). Linux Core/CLI also
passed 86 tests plus actual ELF/report acceptance; Linux Desktop remains unverified. Next prioritize offline HAR/OpenAPI ingestion and
header/cookie observations with explicit evidence, followed by selected-artifact AI
correlation. Do not call remaining recon/scanner, mobile, cloud or release phases
complete. Linux/runtime/installers and process isolation remain unresolved.

# Development roadmap

Current v1.0 execution ledger: MASTER_EXECUTION_PROGRESS.md. Phase 1 now provides a
real local job -> evidence -> manually reviewed finding -> redacted report pipeline
with Desktop/CLI parity. Windows release/native verification passed, and bounded
owned-local HTTP inventory now persists observations and evidence. Next implement
offline traffic/configuration imports using these same records. Do not infer
scanner/agent completion from navigation entries or skip unresolved output defects.

2026-10-16: Milestone 2 adds reviewed portable YARA-X, an actual stdio/Streamable
HTTP MCP gateway, a unified capability registry and explicitly approved local AI
plans using the existing Job Engine. Windows native verification covers real
tool installation/signatures, benign MCP calls, shared CLI records and restart.
The AI acceptance model is a deterministic local fixture; live cloud accounts and
Linux runtime remain unverified. Preserve this slice and extend normalized local
evidence, manually reviewed findings and redacted reports next. General tool
lifecycles, third-party MCP profiles and agent/DAG scheduling remain incomplete.

Historical priority checkpoints follow; the current implementation and remaining
scope are recorded in IMPLEMENTATION_STATUS.md and module-status.json.

2026-10-15: the target/file slice now uses durable local jobs, bounded cross-client
claims, timeouts/cancellation, exact result links and lease recovery. Job Center,
local notes and explicit selected-artifact AI review are implemented. Preserve the
typography and three themes. Windows debug/release verification is recorded in
NEXT_MILESTONE.md. Next prioritize normalized local evidence, manual findings and
redacted report export. General DAG orchestration, scanners and MCP remain separate
future milestones; their presence in navigation is not evidence of implementation.

2026-10-14 priority update: target/scope management and native local static-analysis
inputs now work through the shared core. Continue from this verified slice, rather
than rebuilding the UI or duplicating infrastructure. Next: local fixture inventory
and evidence normalization, a permissioned benign MCP test gateway, then bounded
local analysis plans/agent routing and findings/report export. Full public-target
scanning and autonomous offensive actions are outside the implemented capability set.
Add native folder/drop verification, parser process isolation, crash recovery,
cache GC and richer string/hex pagination before calling reverse analysis complete.
Crash recovery for local jobs is now implemented; parser process isolation remains pending.

1. Foundation: Windows native desktop/CLI/shared SQLite project/settings milestone verified.
   Desktop redesign: reference-inspired four-region shell, illustrated dashboard, consistent secondary pages, keyboard search, activity/notifications, resizable AI context and exactly Light/Dark/ReconX verified on Windows. Future module actions still require backend implementation.
2. AI integration: provider CRUD, secure credentials, model discovery, text/streaming/cancellation, defaults and safe connection fallback implemented. Remaining: persisted health/usage/cost, rate/budget accounting, context, advanced parameters, per-agent routing and extensions. Real cloud compatibility unverified.
3. Tool Manager: ripgrep and YARA-X verified Windows portable lifecycle/profile slice; general dependency graphs, unmanaged adoption, package manager/WSL adapters and multi-version upgrades/rollback remain pending.
4. MCP: actual stdio/Streamable HTTP discovery, permissions and reviewed benign calls through existing jobs. Extend OAuth, pagination, sandboxing and validated integration profiles only after independent verification.
5. Jobs and AI plans: reviewed local capability plans, actual tool jobs and consented unverified interpretation are implemented. General agents/routing/DAG scheduling remain pending.
6. Targets: inventory and deterministic scope/authorization. Autonomous public-target offensive execution is outside the implemented scope.
7. Analysis: offline HAR and owned binary inspection with structured observations/evidence.
8. Reporting: findings management and redacted Markdown/HTML/JSON, then PDF.
9. Packaging: portable Windows builds implemented; verify installers, Windows/Linux CI, signing and release audit.
10. Extensions: local mobile/IaC/container/CI configuration review.

CI and NSIS/Debian configuration exist but are unverified. Linux and ARM64 remain unverified. A stub or roadmap page never counts as an implemented capability.
