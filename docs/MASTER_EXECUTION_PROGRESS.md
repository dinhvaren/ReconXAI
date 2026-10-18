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
