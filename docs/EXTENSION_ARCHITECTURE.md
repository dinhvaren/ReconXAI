## MCP extension checkpoint — 2026-10-19

The reviewed npm adapter and official Registry discovery now have real Windows
Desktop/CLI and Linux Core/CLI evidence. Current exact implementation, limitations,
in-progress runtime-selection fix and acceptance status: MASTER_EXECUTION_PROGRESS.md
and artifacts/mcp-current-verification.json. See MCP_REGISTRY.md,
DEPENDENCY_RESOLVER.md, EXTENSION_SECURITY.md and FIRST_RUN_SETUP.md.
The older one-adapter-only notes below are historical. First-run/general installers,
unreviewed package execution, OS sandbox and Linux Desktop are not completed.

---

# Extension architecture — 2026-10-19

Status: bounded native-extension foundation, runtime verified on Windows Desktop/CLI and Linux CLI.
The first catalog contains YARA-X File Signatures only. No general community
package runtime, dynamic library loader or third-party MCP installation is claimed.

Core `extensions.rs` is shared by CLI and Tauri's typed service requests. A strict,
versioned `ExtensionManifest` declares identity, publisher, build version, exact
ReconX compatibility, platforms/architectures, runtime/dependencies, source,
permissions, configuration schema, capabilities, entrypoint, health and uninstall.
Validation accepts only the complete compiled manifest; imported fields cannot
authorize new code, arguments, permissions or download locations. Unknown fields
are rejected. This deliberately bounded format is not an alternative MCP
`server.json` format; future registry imports must preserve that official document
and place ReconX metadata in its namespaced `_meta` extension.

The extension references Tool Manager's existing real YARA-X manifests, including
archive and executable SHA-256 values and exact release URLs. Unknown installation
size is displayed as unavailable. Installed dependency state is read from the same
Tool Manager inventory; no second tool-status cache is created. Only a successful
hash/version check and actual benign signature scan grant extension Ready status.
Names, catalog presence and configuration alone never grant Ready status.

Additive migration **014_extensions.sql** stores extension registrations, shared
tool requirements and invocation bindings. Existing migration/backups remain.
SQLite makes the job and its extension permission binding atomic. A worker checks
the current enabled revision at checkpoints and under the final result transaction.
Disable/uninstall cancels queued or running bound work. Existing unbound native
tool jobs retain their original policy and functionality. History and evidence
survive removal; shared tools are retained. Tool Manager blocks removing a tool
while any registered extension, including disabled ones, requires it.

An install plan fingerprints the exact compiled manifest, observed dependency
state and extension revision. Explicit approval is mandatory. State changes
invalidate approval. Installation performs only pinned existing Tool Manager
operations, with its archive boundaries, allowlisted redirects, cleared subprocess
environment, resource limits and integrity verification. No descriptions or README
commands are executed. No AI credential-store interface is passed to the extension.
This is capability enforcement, **not an OS sandbox**. Platform sandboxing,
download cancellation and stronger process resource isolation remain incomplete.

An in-progress install prevents enable/remove/reinstall. After ten minutes a
doctor operation can recover an interrupted installation without deleting tools
or data. Failed health checks persist a safe error code and cannot enable an
extension. Tool versions remain pinned. No available update or rollback is
fabricated when the compiled catalog contains no different compatible version.

Tests: `crates/reconx-core/tests/extensions.rs`, storage migration tests,
`apps/desktop/src/ExtensionsPage.test.tsx`, native
`tests/e2e/extensions.mjs`, and actual-process `scripts/extension_smoke.py`.
Consult the master/readiness ledgers for their current executed results.

Executed proof: `artifacts/ux-extensions-current-verification.json`; Windows native5
checkpoints, actual CLI lifecycle28 commands per host, real schema13→14 preservation,
120 current native UX captures and first-party install/invoke/disable/remove/reinstall.
Broader marketplace and Linux Desktop acceptance remain incomplete.
