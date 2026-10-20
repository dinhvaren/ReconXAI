## MCP extension checkpoint — 2026-10-19

The reviewed npm adapter and official Registry discovery now have real Windows
Desktop/CLI and Linux Core/CLI evidence. Current exact implementation, limitations,
in-progress runtime-selection fix and acceptance status: MASTER_EXECUTION_PROGRESS.md
and artifacts/mcp-current-verification.json. See MCP_REGISTRY.md,
DEPENDENCY_RESOLVER.md, EXTENSION_SECURITY.md and FIRST_RUN_SETUP.md.
The older one-adapter-only notes below are historical. First-run/general installers,
unreviewed package execution, OS sandbox and Linux Desktop are not completed.

---

# Extension Marketplace — 2026-10-19

Status: first-party native adapter implementation; actual native installation
acceptance now passes. Broader marketplace milestone B remains PARTIAL.

Extensions has discovery/search/category filtering, installed and available-update
views, recently changed installation records, details, permission/dependency
inspection, installation-plan approval, enable/disable, health checks, safe removal,
authorized file invocation and persistent execution history. Empty update views
show that none is available. Publisher ratings, counts and verification badges are
not invented. Source and size fields remain honest about unknown metadata.

Current adapter: `reconx.yara-signatures`. It executes the existing bounded offline
YARA-X Job Engine handler on a selected project file; observations flow into the
existing evidence pipeline. It does not load arbitrary rules or produce automatic
verified vulnerability findings. Permissions require an explicitly imported file.

CLI equivalents:

```text
reconx extensions search yara
reconx extensions info reconx.yara-signatures
reconx extensions plan reconx.yara-signatures
reconx extensions install reconx.yara-signatures --fingerprint <reviewed-fingerprint> --approve
reconx extensions doctor reconx.yara-signatures
reconx extensions invoke reconx.yara-signatures --file <imported-file-id>
reconx jobs run
reconx extensions history reconx.yara-signatures
reconx extensions disable reconx.yara-signatures
reconx extensions enable reconx.yara-signatures
reconx extensions remove reconx.yara-signatures
```

All commands support the CLI's global `--json` and `--database`. No credentials
are passed as arguments. A matching build-version update can only be reviewed if
the compiled catalog changes; no update/rollback success has been tested against
a different published extension release.

Next: official MCP registry discovery with bounded pagination/cache, preserving
the actual server.json metadata and distinguishing registry presence from trust;
then approved pinned package/runtime resolution and actual MCP protocol health.
Ghidra/Burp/Frida/debugger integration names are targets, not installed extensions.
Community manifest execution, broad categories with real installable adapters,
remote extension update feeds and rollback remain unavailable.

Executed proof: `artifacts/ux-extensions-current-verification.json`; Windows native5
checkpoints, actual CLI lifecycle28 commands per host, real schema13→14 preservation,
120 current native UX captures and first-party install/invoke/disable/remove/reinstall.
Broader marketplace and Linux Desktop acceptance remain incomplete.
