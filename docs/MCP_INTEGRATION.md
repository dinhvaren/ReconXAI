## MCP extension checkpoint — 2026-10-19

The reviewed npm adapter and official Registry discovery now have real Windows
Desktop/CLI and Linux Core/CLI evidence. Current exact implementation, limitations,
in-progress runtime-selection fix and acceptance status: MASTER_EXECUTION_PROGRESS.md
and artifacts/mcp-current-verification.json. See MCP_REGISTRY.md,
DEPENDENCY_RESOLVER.md, EXTENSION_SECURITY.md and FIRST_RUN_SETUP.md.
The older one-adapter-only notes below are historical. First-run/general installers,
unreviewed package execution, OS sandbox and Linux Desktop are not completed.

---

# MCP integration status — 2026-10-19

The existing gateway supports reviewed stdio and HTTP configurations, real
discovery, schema inspection and approved benign invocation through the Job
Engine. MCP_HUB.md describes those implemented paths. Native tests exercise an
actual local compatible fixture server, credential/environment isolation,
permission denial, persistence and restart. A fixture server is not verification
of Burp, Ghidra, Frida or debugger integrations.

Marketplace MCP discovery/installation is **NOT IMPLEMENTED** in this checkpoint.
The current extension is a native YARA-X adapter, not an MCP package. Official
registry pagination/cache, preservation of server.json metadata, trusted package
pinning and runtime dependency resolution are the next implementation task.
Registry publication alone will not be treated as trust or capability verification.
MCP OAuth, third-party environments and OS sandbox security properties remain
incomplete/unverified. No arbitrary README installation commands are executed.

Next-task research on 2026-10-19 used the [official registry API documentation](https://github.com/modelcontextprotocol/registry/blob/main/docs/reference/api/official-registry-api.md).
It documents the production base, `/v0.1/servers`, search/latest filters and cursor
pagination. The bounded live list probe timed out; its failure is retained at
`artifacts/mcp-official-registry-probe.log`. Documentation access is not a passed
runtime connection, and no Registry package was installed by that probe.
