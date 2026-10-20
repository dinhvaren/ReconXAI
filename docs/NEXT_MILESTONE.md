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

## Current continuation checklist - 2026-10-18

- [x] Preserve schema011 offline workspace and prior source/data.
- [x] Add migration012, pre-v12 consistent backup and preservation tests.
- [x] Share reviewed request drafts, secure synthetic identities, exact fingerprint approval and manual loopback execution between Desktop/CLI.
- [x] Reuse existing worker, cancellation, evidence, traffic inspector/diff, findings and all four report formats.
- [x] Fix cached draft revisions on reopen and add frontend regression.
- [x] Verify Windows/Linux Core+CLI Rust 133 each, frontend 26, optimized Desktop/CLI and nine native suites/77 checkpoints; inspect final screenshots/PDF. Proof: artifacts/replay-current-verification.json.
- [ ] Next safe slice: additional bounded offline schema consistency coverage and manual permission/ownership annotations linked to captured evidence. Keep all sends reviewed and loopback-only.
- [ ] Trusted-CA HTTPS success fixture, abrupt HTTP process termination and Linux Secret Service runtime acceptance; document actual guarantees.
- [ ] Full JSON Schema, stateful authorization automation, broader scanner/agent/research domains, Linux Desktop, installers/signing and release acceptance remain incomplete.

Use artifacts/windows-replay-final-20261018-160538 for current manual acceptance. Do not reset user data, repeat completed milestones or claim full v1.0. No commit/push is authorized. Commands and restrictions: HTTP_REPLAY.md; precise continuation: MASTER_EXECUTION_PROGRESS.md.

---

Older checkpoints below are retained for provenance; the current checkpoint above takes precedence.

## Current continuation checklist - 2026-10-18

- [x] Preserve schema011 offline workspace and prior source/data.
- [x] Add migration012, pre-v12 consistent backup and preservation tests.
- [x] Share reviewed request drafts, secure synthetic identities, exact fingerprint approval and manual loopback execution between Desktop/CLI.
- [x] Reuse existing worker, cancellation, evidence, traffic inspector/diff, findings and all four report formats.
- [x] Fix cached draft revisions on reopen and add frontend regression.
- [x] Verify Windows/Linux Core+CLI Rust 133 each, frontend 26, optimized Desktop/CLI and nine native suites/76 checkpoints; inspect final screenshots/PDF. Proof: artifacts/replay-current-verification.json.
- [ ] Next safe slice: additional bounded offline schema consistency coverage and manual permission/ownership annotations linked to captured evidence. Keep all sends reviewed and loopback-only.
- [ ] HTTPS fixture, abrupt HTTP process termination and Linux Secret Service runtime acceptance; document actual guarantees.
- [ ] Full JSON Schema, stateful authorization automation, broader scanner/agent/research domains, Linux Desktop, installers/signing and release acceptance remain incomplete.

Use artifacts/windows-replay-final-20261018-155056 for current manual acceptance. Do not reset user data, repeat completed milestones or claim full v1.0. No commit/push is authorized. Commands and restrictions: HTTP_REPLAY.md; precise continuation: MASTER_EXECUTION_PROGRESS.md.

---

Older checkpoints below are retained for provenance; the current checkpoint above takes precedence.

## Current continuation checklist - 2026-10-18

- [x] Preserve offline HAR/OpenAPI, migration 010, AI settings and existing output pipeline.
- [x] Add bounded evidence-linked offline schema jobs through shared Rust/Desktop/CLI.
- [x] Verify normalized inputs, original hashes, exact target/origin, cycles and budgets.
- [x] Add migration 011 and preservation/backup tests, with no normal user DB migration during tests.
- [x] Pass Windows/Linux Core+CLI Rust 124 each, frontend 23 and optimized Desktop/CLI builds.
- [x] Pass eight native suites/66 checkpoints: schema selection, actual mismatches, restart, evidence/reports, no automatic vulnerability and preserved existing functionality.
- [ ] Next: reviewed request drafts plus a manual, bounded owned-loopback replay fixture using the existing jobs/scope policy. Keep unexpected destinations and redirect bypasses rejected.
- [ ] Then implement secure synthetic identity profiles before local multi-role comparison; status/body-size differences must not verify a vulnerability.
- [ ] Full JSON Schema, scanner adapters, broader research domains, independent Linux Desktop/installer/release acceptance remain unfinished.

Exact source: crates/reconx-core/src/http/schema.rs; commands/limits in HTTP_SCHEMA_ANALYSIS.md. Proof: artifacts/schema-current-verification.json. Do not repeat imports/provider/output milestones or replace the current UI. Stop older clients before applying schema 011 to a real user workspace. No commit/push is authorized.

---

Older checkpoint sections below are retained for provenance. The checkpoint above takes precedence.

## Current continuation checklist - 2026-10-18

- [x] HAR/OpenAPI migration 010, shared services and native import/inspection.
- [x] Source integrity, origin/port and target partitioning, bounded correlation, complete bounded project export.
- [x] Captured identity matrix, offline jobs, explicit selected-result AI review and review-only finding/report flow.
- [x] Reproduce/fix stale Recon job display; run Windows/Linux Rust 112 each, frontend 22 and optimized builds.
- [x] Pass 8 current native suites/64 checkpoints, inspect screenshots/PDF and record matching hashes at `artifacts/http-current-verification.json`.
- [x] Add offline HTTP and AI setup to the existing CI configuration; remote CI was not executed or published.
- [ ] Next: bounded offline documented-response/schema observations with actual evidence and reference provenance.
- [ ] Complete the remaining request editor/identity/replay acceptance separately; current captures are never automatically replayed.
- [ ] Continue incomplete research domains, process isolation/ACL hardening and independent Desktop/installer release acceptance.

Do not redo AI provider setup or the output engine. Preserve the existing files, schema 010, three themes and user data. Keep v1.0 blocked until its actual criteria pass. The prior Node runner abort is retained with an unknown cause; current native checks passed. Current Linux Core/CLI passed after toolchain restoration; Linux Desktop is still unverified.

---

Older checkpoint sections below are retained for provenance. The current checkpoint above takes precedence.

## Current continuation - 2026-10-18

- [x] Preserve schema 010 offline HAR/OpenAPI and the existing outputs/AI services.
- [x] Recheck 111 Windows Rust tests, 21 frontend tests, types/build and lint.
- [x] Correct origin/port correlation, bound server-template work and preserve source hashes.
- [x] Implement all-page captured identity matrix and complete bounded redacted exports.
- [ ] Finish optimized Tauri/CLI build and all eight native regression suites.
- [ ] Inspect clean Web/API screenshots in all three themes at 1920/1366 widths.
- [ ] Record matching artifact hashes and update every status registry.
- [ ] Restore or provision an isolated Linux toolchain before claiming current Linux tests.

Do not repeat the AI setup or prior output milestones. Continue the safe offline
Web/API acceptance slice; full replay/authenticated testing and remaining product
domains are still PARTIAL/NOT_STARTED. Signing, Linux Desktop and installers are
not verified. Heavy verification scripts disable Rust incremental cache and restore
the invoking environment afterward. No cache cleanup or user-data deletion occurred.

## Historical AI Provider Manager - current verified source (2026-10-17)

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

# v1.0 continuation - 2026-10-17

Read MASTER_EXECUTION_PROGRESS.md first. This section supersedes the historical
Milestone 2 checklist below; no previous code or user data was reset.

- [x] Migration 005, immutable evidence, manual findings/CVSS, four report formats.
- [x] Phase 1 Desktop/CLI integration, Windows native/release acceptance and screenshots.
- [x] Phase 2 migration 006, exact local HEAD approval, normalized assets, private HTTP
  evidence, existing jobs/cancellation and CLI/Desktop bindings.
- [x] Actual TCP fixtures and v5 migration preservation tests.
- [x] Complete local-adapter native/release tests, visual inspection and default-DB upgrade.
- [x] Update actual test records and portable artifact hashes after final build.
- [x] Linux Core/CLI: 86 tests and actual ELF/evidence/four-format report smoke.
- [ ] Linux Desktop SDK/runtime and optimized CLI/package verification.
- [ ] Next vertical slice: bounded offline HAR/OpenAPI import, endpoint inventory
  and deterministic header/cookie observations; preserve raw traffic privately.

A report redaction defect for HTTP JSON header pairs was reproduced by the native
local fixture and fixed. Evidence raw export and report snapshots are separate
trust boundaries. Older detectable unsafe snapshots require a new reviewed preview.

Prepared commit summary (no commit/push performed):
`feat(core): persist reviewed research outputs and approved loopback inventory`

# Milestone 2 - Tools, MCP and local AI orchestration

Local HEAD 7587e63 was clean before implementation. Baseline verify.ps1 passed
57 Rust tests (one OS-store fixture ignored) and 11 frontend tests. Existing inputs,
queue, typography and three themes are retained. No remote configuration is printed.

- [x] Verified portable YARA-X installation, health, registry/profile and local invocation.
- [x] MCP SDK gateway, configuration, discovery, approved benign calls and history.
- [x] Desktop/CLI management and unified capability registry.
- [x] Validated AI plans using existing local jobs with explicit artifact consent.
- [x] Migration backup, regression, native runtime/restart acceptance, release build.
- [x] Document platform verification and incomplete lifecycle/integration capabilities.

This milestone does not add offensive scanners or autonomous network operations.

## Verified checkpoint - 2026-10-16

Formatting/Clippy passed; 66 Rust passed, one explicit OS-store fixture ignored;
12 frontend tests and the TypeScript/Vite build passed. Actual Windows debug and
optimized release each passed 31 native checkpoints: regression 8, inputs 9,
existing jobs 8 and Tool/MCP/AI integration 6. Release records:
`artifacts/native-1792144968533`, `inputs-1792145335173`,
`jobs-1792145365667`, `integrations-1792145390620`.
One earlier release input attempt recorded an extra character and failed; the
unchanged rerun passed. Its cause is unestablished, not silently marked successful.
The normal release startup and backed-up default database migration preserved all
existing record digests. See TEST_RESULTS.md for hashes, limitations and evidence.

Actual acceptance includes official YARA-X download plus both integrity checks,
version/repair/removal/profile operations, real stdio MCP discovery/manual call,
HTTP authentication tests, local model JSON planning, approval, actual local tool
jobs and saved unverified AI interpretation. The model is an explicitly labelled
deterministic local fixture. Linux and live cloud accounts remain UNVERIFIED.

## Milestone 2 source map

- Tool lifecycle: `crates/reconx-tools/src/lib.rs`, `src/local.rs`,
  `manifests/tools/yara-x.json`, `SOURCES.md`; shared registry/profile/history in
  `crates/reconx-core/src/tooling.rs`.
- MCP and reviewed plans: `crates/reconx-core/src/mcp.rs`, `mcp_http.rs`,
  `capabilities.rs`, existing `jobs.rs`/`lib.rs` and `Cargo.toml`; `Cargo.lock`.
- Persistence: `crates/reconx-storage/migrations/004_capabilities.sql` and
  `crates/reconx-storage/src/lib.rs`. The additive migration and pre-v4 backup
  retain existing tables; there is no destructive automatic rollback.
- Clients: `apps/cli/src/main.rs`, `apps/desktop/src-tauri/src/lib.rs`,
  `apps/desktop/src/pages/McpPage.tsx`, `OrchestrationPage.tsx`,
  `WorkspacePages.tsx`, `src/App.tsx`, `src/main.tsx`, `src/research.ts`,
  `src/integrations.css`, `src/components/{ActivitySidebar,QuickActions}.tsx`
  and `src/components/navigation.ts`. Existing theme/typography assets are retained.
- Verification: `crates/reconx-core/tests/integrations.rs`, `research_inputs.rs`,
  storage/tool unit tests, `apps/desktop/src/App.test.tsx`,
  `tests/e2e/{mcp-fixture,integrations}.mjs`, `apps/desktop/package.json`,
  `scripts/verify.ps1`, `.github/workflows/ci.yml`, `.gitignore`.
- Persistent records: `README.md`, `docs/{ARCHITECTURE,SECURITY_MODEL,ROADMAP,
  IMPLEMENTATION_STATUS,TEST_RESULTS,NEXT_MILESTONE}.md`, `docs/module-status.json`,
  `docs/{TOOL_MANAGER,MCP_HUB,AI_TOOL_ORCHESTRATION}.md`.

## Next milestone

1. Normalize completed local job observations into evidence records with source,
   hashes and exact result references; preserve original results separately.
2. Add manually editable findings with explicit confidence/verification status.
   AI interpretations remain unverified, separate from deterministic evidence.
3. Export redacted Markdown/JSON/HTML reports, then independently verify PDF.
4. Add offline HAR/OpenAPI ingestion using local fixtures and the same services.

The broader tool manager/MCP/workflow phases remain IN_PROGRESS: package managers,
WSL, unmanaged executable adoption, general dependency graphs, multi-version
rollback, OAuth, sandboxing, validated Ghidra/debugger adapters, general agents and
DAG scheduling have not been implemented. Linux CI is configured but not executed.

## Prepared Milestone 2 commit summary

feat(core): integrate reviewed local tools, MCP and AI capability plans

- Add pinned YARA-X portable lifecycle and fixed offline signatures to shared jobs.
- Add bounded MCP stdio/HTTP discovery, secret isolation, approvals and history.
- Add explicit local AI plan/evidence consent and one-time typed job approvals.
- Preserve existing state with additive migration 004 and automatic backups.
- Add Desktop/CLI integrations and real Windows native/fixture acceptance tests.

No commit, push or upload has been performed.

---

# Functional research milestone — 2026-10-15

At initial inspection, local HEAD 4ef7e4d matched the public repository HEAD. Working tree was clean at
inspection; the committed typography, palettes and business services are retained.
Baseline scripts/verify.ps1 passed: 45 Rust tests, 1 intentionally ignored OS-store
fixture, 11 frontend tests, formatting, Clippy and TypeScript/Vite build.

## Implementation checklist

- [x] Inspect actual target/file services, migration 002, native commands and CLI.
- [x] Preserve Segoe UI Variable Text, local Source Sans 3/IBM Plex Mono and exactly
  Light/Dark/ReconX. Do not duplicate existing target/import implementations.
- [x] Add scope show and verify target CRUD/duplicate policy/restart in both clients.
- [x] Migration 003: persistent queued jobs, start/deadline/progress/result metadata,
  worker leases and local research notes; preserve all version-2 records/artifacts.
- [x] Shared allowlisted local job engine with atomic claims, bounded concurrency,
  cancellation, configurable timeout, error/log persistence and expired-lease recovery.
- [x] Real job types: scope/registration review, static binary, metadata, strings,
  local artifact indexing. No shell strings, sample execution or network scanners.
- [x] Desktop Job Center with filters, state/progress/logs/results/cancellation;
  native job events and CLI worker/enqueue/result commands.
- [x] Connect target detail and reverse workspace to the engine; local notes,
  source path and explicitly consented selected-artifact AI handoff.
- [x] Deterministic tests for migrations, concurrency, failure, timeout,
  cancellation, interruption and CLI/Desktop consistency.
- [x] Native acceptance: Research Demo + example.test; owned PE import/hash/
  sections/strings/restart; real running/completed/failed/cancelled jobs/screenshots.
- [x] Final release build/runtime regression, data-preservation check and status/report.

## Existing limits to retain explicitly

Static analysis is read-only and capped at 64 MiB; parsers never execute samples.
No decompiler, scanner, MCP execution or findings/report engine is claimed here.
Linux/ARM64 and physical OS drag/drop remain unverified until actually tested.
No user data, credentials or Git remote configuration is reset; no push is authorized.

## Verified debug checkpoint

Final scripts/verify.ps1 -Native -Inputs -Jobs -Visual passed: 57 Rust tests,
11 frontend tests and 25 native checkpoints. Records:
artifacts/native-1792077037276, inputs-1792077081584 and jobs-1792077094197.
This includes the CLI/already claimed Desktop worker race regression. Native
captures were inspected and compact form/pane bounds asserted. Optimized release also passed the same 25 native checkpoints; records:
artifacts/native-1792077312439, inputs-1792077355999 and jobs-1792077366043.
Normal Desktop was reopened after a backed-up, verified schema-3 upgrade; original
record counts and settings/providers/projects/files/results digests are preserved.

## Prepared commit summary

feat(core): add durable local research jobs and integrated file workspace

- Preserve version-2 records with migration 003; add fenced queue/leases and notes.
- Share typed local analysis, scope review, progress, cancellation and recovery
  across Desktop and CLI; retain existing typography/themes/providers.
- Add Job Center, exact result links and explicit destination-bound artifact review.
- Include previously ignored native input fixtures and add real job acceptance tests.

No commit or push has been performed.

## Files created or modified

Existing source/user data are preserved; previously ignored deterministic input
fixtures are included for reproducible clean-checkout verification after commit.

- [.github/workflows/ci.yml](../.github/workflows/ci.yml)
- [.gitignore](../.gitignore)
- [README.md](../README.md)
- [apps/cli/src/main.rs](../apps/cli/src/main.rs)
- [apps/cli/tests/cli.rs](../apps/cli/tests/cli.rs)
- [apps/desktop/package.json](../apps/desktop/package.json)
- [apps/desktop/src-tauri/src/lib.rs](../apps/desktop/src-tauri/src/lib.rs)
- [apps/desktop/src/App.tsx](../apps/desktop/src/App.tsx)
- [apps/desktop/src/components/FileResearchPanel.tsx](../apps/desktop/src/components/FileResearchPanel.tsx)
- [apps/desktop/src/components/JobHistory.tsx](../apps/desktop/src/components/JobHistory.tsx)
- [apps/desktop/src/components/ui.tsx](../apps/desktop/src/components/ui.tsx)
- [apps/desktop/src/pages/JobsPage.tsx](../apps/desktop/src/pages/JobsPage.tsx)
- [apps/desktop/src/pages/ReversePage.tsx](../apps/desktop/src/pages/ReversePage.tsx)
- [apps/desktop/src/pages/TargetsPage.tsx](../apps/desktop/src/pages/TargetsPage.tsx)
- [apps/desktop/src/research.ts](../apps/desktop/src/research.ts)
- [apps/desktop/src/styles.css](../apps/desktop/src/styles.css)
- [crates/reconx-analysis/src/lib.rs](../crates/reconx-analysis/src/lib.rs)
- [crates/reconx-core/src/jobs.rs](../crates/reconx-core/src/jobs.rs)
- [crates/reconx-core/src/lib.rs](../crates/reconx-core/src/lib.rs)
- [crates/reconx-core/src/reverse.rs](../crates/reconx-core/src/reverse.rs)
- [crates/reconx-core/tests/jobs.rs](../crates/reconx-core/tests/jobs.rs)
- [crates/reconx-core/tests/research_inputs.rs](../crates/reconx-core/tests/research_inputs.rs)
- [crates/reconx-storage/migrations/003_job_engine.sql](../crates/reconx-storage/migrations/003_job_engine.sql)
- [crates/reconx-storage/src/lib.rs](../crates/reconx-storage/src/lib.rs)
- [docs/ARCHITECTURE.md](../docs/ARCHITECTURE.md)
- [docs/DECISIONS.md](../docs/DECISIONS.md)
- [docs/IMPLEMENTATION_STATUS.md](../docs/IMPLEMENTATION_STATUS.md)
- [docs/JOB_ENGINE.md](../docs/JOB_ENGINE.md)
- [docs/NEXT_MILESTONE.md](../docs/NEXT_MILESTONE.md)
- [docs/RESEARCH_INPUTS.md](../docs/RESEARCH_INPUTS.md)
- [docs/ROADMAP.md](../docs/ROADMAP.md)
- [docs/SECURITY_MODEL.md](../docs/SECURITY_MODEL.md)
- [docs/TEST_RESULTS.md](../docs/TEST_RESULTS.md)
- [docs/module-status.json](../docs/module-status.json)
- [scripts/verify.ps1](../scripts/verify.ps1)
- [tests/e2e/inputs.mjs](../tests/e2e/inputs.mjs)
- [tests/e2e/jobs.mjs](../tests/e2e/jobs.mjs)
- [tests/e2e/select-file.ps1](../tests/e2e/select-file.ps1)
