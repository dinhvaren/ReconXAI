## MCP extension checkpoint — 2026-10-19

The reviewed npm adapter and official Registry discovery now have real Windows
Desktop/CLI and Linux Core/CLI evidence. Current exact implementation, limitations,
in-progress runtime-selection fix and acceptance status: MASTER_EXECUTION_PROGRESS.md
and artifacts/mcp-current-verification.json. See MCP_REGISTRY.md,
DEPENDENCY_RESOLVER.md, EXTENSION_SECURITY.md and FIRST_RUN_SETUP.md.
The older one-adapter-only notes below are historical. First-run/general installers,
unreviewed package execution, OS sandbox and Linux Desktop are not completed.

---

# Shared tool dependency manager — 2026-10-19

PARTIAL. Existing `reconx-tools::ToolManager` remains the only portable tool
installer and inventory authority. The extension catalog references its actual
manifests, verified executable receipts and health state. CLI, Desktop and existing
AI capability discovery reuse this backend; no separate extension tool-status
database replaces inventory checks.

Migration014 records extension→tool requirements. Tool removal is blocked while
any installed registration, including a disabled extension, requires it. Removing
an extension cancels its bound jobs and removes its dependency reference; it
preserves tools and evidence. User-managed external applications are not removed.

Current executable adapters are ripgrep and YARA-X. The YARA-X extension performs
real signature scans through the shared Job Engine. Other tool names in the
catalog are explicitly pending integration. No package-manager installation,
general dependency graph resolver, runtime environment manager, version rollback
or arbitrary native plugin loader is claimed. See TOOL_MANAGER.md and
EXTENSION_ARCHITECTURE.md for integrity/path/subprocess boundaries and limitations.
