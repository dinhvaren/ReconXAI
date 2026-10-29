# Shared execution boundary

Core, Desktop and CLI already share typed execution services and the SQLite Job
Engine. This document maps the actual boundary; it does not claim a newly complete
universal tool runner.

| Provider | Existing entry point | Approval/output boundary |
|---|---|---|
| Native local analysis | Typed local JobInput/JobKind handlers | Immutable import identity, project/target scope, bounded artifacts |
| Managed YARA-X | Compiled signatures capability | Verified executable, fixed rules/arguments, real matches/evidence |
| Managed HTTPX/Subfinder | ReconPlan/Core recon enqueue | Exact fingerprint, fixed arguments, distinct passive/active consent |
| User-configured MCP | Discovered schema and McpApproval | Trusted server, allowed tool, bounded parameters/result, one-shot review |
| Owned loopback HTTP | Existing HTTP replay/local HEAD services | Exact destination/request review, isolated auth, timeout/cancellation |
| Offline HTTP/schema review | Existing import/review handlers | Original evidence hash and bounded local parsing; no network replay |

Capability metadata/availability is resolved from actual installed dependencies
and configuration. Missing tools are installed only after the Marketplace plan
is approved. Registry presence is not readiness. Model text never becomes a shell
command, arbitrary executable argument list or a replacement for Rust policy.

Jobs retain configuration/approval references, timestamps, stage/logs, terminal
state and actual result/evidence references. The durable queue bounds concurrency,
honors cancellation and does not blindly replay work after restart. Managed tools
are rehashed before execution; an external process is still not an OS sandbox.

The schema019 provenance layer only reads these records. It is not another queue
or an alternate execution path. Graph queries/exports cannot schedule a capability.
The CLI and Tauri graph commands call the same Core service.

Remaining work includes a fully uniform invocation/output schema across provider
families and a general audited lifecycle/permission interface. No unrestricted
agent-mediated offensive workflow is introduced. Per-tool limits remain explicit
in their own services and guides; a single registry interface must not bypass them.
