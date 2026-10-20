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
