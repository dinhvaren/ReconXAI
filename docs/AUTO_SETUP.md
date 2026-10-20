## MCP extension checkpoint — 2026-10-19

The reviewed npm adapter and official Registry discovery now have real Windows
Desktop/CLI and Linux Core/CLI evidence. Current exact implementation, limitations,
in-progress runtime-selection fix and acceptance status: MASTER_EXECUTION_PROGRESS.md
and artifacts/mcp-current-verification.json. See MCP_REGISTRY.md,
DEPENDENCY_RESOLVER.md, EXTENSION_SECURITY.md and FIRST_RUN_SETUP.md.
The older one-adapter-only notes below are historical. First-run/general installers,
unreviewed package execution, OS sandbox and Linux Desktop are not completed.

---

# Automatic setup status — 2026-10-19

PARTIAL. Installing the reviewed first-party YARA-X extension automatically
resolves its pinned portable tool dependency, verifies archive/executable hashes,
executes version and benign signature health checks, and saves shared state.
Explicit plan approval is mandatory. Repair/reinstall reuse or replace only the
managed dependency; uninstall retains it and all project evidence.

`Core::tool_host` already detects OS/architecture, Linux distribution and available
package-manager executables. This is detection, not package-manager installation.
The first-run wizard, storage/runtime/credential-store inventory, broader approved
profiles, Winget/Scoop/apt/dnf/pacman execution, conflict resolution and WSL-backed
execution remain incomplete. No administrator elevation or system PATH change is
performed by the extension installer. Linux CLI portable installation has separate
runtime evidence; Linux Desktop first-run setup remains unverified.

Current exact evidence and next actions: MASTER_EXECUTION_PROGRESS.md and
RELEASE_READINESS.md. No app-startup bulk tool installation is implemented.
