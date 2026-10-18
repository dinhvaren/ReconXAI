# Release readiness — 2026-10-18

**ReconX AI 0.1.0 development build. v1.0 is NOT READY.** Functional acceptance
does not override dependency, platform, installation or missing-feature gates.
The machine-readable feature/gap snapshot is [release-gaps.json](release-gaps.json).
The detailed current local evidence is `artifacts/security-current-verification.json`.

| Gate | Status | Actual scope / remaining work |
|---|---|---|
| Existing shared Core/Desktop/CLI | TESTED | Windows native regressions; Linux Core/CLI only |
| Dependency security | BLOCKED | Eight pypdf High advisories patched and confirmed fixed by GitHub. glib unsoundness remains; six maintenance warnings require review |
| Offline HAR/OpenAPI | TESTED, bounded | Imports, capture comparison, schema constraints/composition, evidence and report flow; patterns/formats/multi-file/resource resolution incomplete |
| HTTP replay/identities | PARTIAL | Individually approved loopback request only; general remote replay/stateful authorization matrix/workflow features incomplete |
| Recon | PARTIAL | Reviewed local HEAD inventory only; broad passive/active adapters absent |
| Reverse analysis | PARTIAL | Local static parsing and fixed YARA-X signatures; live debugger/disassembler/decompiler integrations absent |
| Tool Manager | PARTIAL | Reviewed ripgrep/YARA-X on Windows; broader adapters, package managers, dependency graphs and rollback incomplete |
| MCP | PARTIAL | Actual local stdio/HTTP fixtures and approved benign calls; third-party tool environments, OAuth and OS sandboxing unverified/incomplete |
| AI | PARTIAL / MOCK_VERIFIED | Provider setup, local protocol fixtures and reviewed local analysis plans; live cloud accounts, full agents/DAG/budgets/advanced fallback not verified |
| Evidence/Findings/Reports | TESTED, bounded | Traceable records, CVSS and four export formats; richer layouts/taxonomy/bulk/performance work remains |
| Other research domains | NOT_STARTED | Dedicated Network, Mobile, Cloud/Container, CI/CD and Red Team Lab workspaces absent |
| Storage preservation | TESTED, bounded | Transactional additive migrations through 012; read-only future/corrupt DB rejection; complete workspace backup/restore not implemented |
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
