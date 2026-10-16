# MCP Hub

The Rust core uses the official `rmcp` SDK, pinned through Cargo.lock (3.5.1).
It negotiates current protocol 2026-07-28 with legacy 2025-11-25 fallback.
Supported transports are stdio and Streamable HTTP (JSON and request SSE).
The HTTP adapter caps bodies before parsing, forbids redirects/proxies and disables
session-expiry replay. Connections are process-local; configs/discovery/permissions
and invocation history persist in the common SQLite database.

## Configure and connect

Desktop: MCP Hub → Add MCP Server → review executable/arguments/directory or
HTTPS endpoint (loopback HTTP is also accepted) → explicitly trust → Save → Connect.
Executables must be existing absolute paths. Connection really starts the process.
No integrations are preconfigured and no Burp/Ghidra/Frida/debugger schemas are invented.

For a benign local fixture, use your installed Node executable and the absolute
path to `tests/e2e/mcp-fixture.mjs` as its only argument. Allow tool `echo` after
reviewing its schema. The fixture exposes actual MCP tools and performs no external
actions. It also supports `--http` for isolated transport tests.

CLI configuration files contain these `McpServerConfig` fields: id (empty generates
a UUID), name, transport (`stdio`/`streamable_http`), executable, args,
working_directory, endpoint, environment (secret **names**, never values),
bearer_auth, enabled, trusted, timeout_ms (100–120000) and allowed_tools.
Unused optional transport fields must be null, unused arrays empty.

```
reconx mcp add --input server.json
reconx mcp list
reconx mcp test <server-id>
reconx mcp connect <server-id>
reconx mcp tools <server-id>
reconx mcp inspect <server-id> echo
reconx mcp invoke <server-id> echo --input input.json --project <project-id>
reconx mcp invoke <server-id> echo --input input.json --project <project-id> --fingerprint <reviewed-fingerprint> --approve
reconx jobs run
reconx mcp history <server-id>
reconx mcp disconnect <server-id>
reconx mcp remove <server-id>
```

CLI connect/test/tools performs a real handshake/discovery and closes the session
on command completion; it cannot close a Desktop-owned connection. Use Desktop's
Disconnect/Reconnect for that process. Config edits also disconnect the local
session and invalidate old approvals. Reconnect after changing a credential.

## Credentials and trust boundary

Store bearer or named environment secrets through Desktop's masked credential form
or `reconx mcp key <id> --name bearer` (hidden interactive entry). Values use the
existing OS secret abstraction and are bound to server config revision; no plaintext
fallback exists. Configuration changes require credentials to be entered again.
AI credential references are never passed to MCP. Child processes start with a
cleared environment, a short OS-runtime allowlist and explicitly declared server
secrets only. Known MCP secrets echoed in results are redacted.
The same recursive redaction applies to discovery metadata, including escaped values
and object keys. Named credential fields are rejected in persisted invocation inputs;
configure credentials through the secure server store instead.

**stdio is not an OS sandbox.** A trusted child runs under the user's account and
can access files/credential APIs permitted to that account. No claim of enforced
filesystem/network isolation is made. Configure an external sandbox before trusting
untrusted code. Permission declarations and tool annotations are not a sandbox.

This milestone permits only explicitly allowlisted, manually approved tools with
`readOnlyHint=true`, `openWorldHint=false`, no destructive hint, valid input schemas
and exact-input approval fingerprints. Referenced JSON schemas are rejected; no
schema resolver can fetch network/filesystem resources. Sensitive/debugger/process/
shell/network tool classes and arbitrary autonomous MCP execution remain unavailable.

Discovery accepts one page, at most 64 tools/256 KiB. stdio sessions have a 4 MiB
wire-read ceiling; normalized results and HTTP bodies are capped at 256 KiB. Eight
sessions maximum. Jobs have cancellation/timeouts and no automatic call retries.
They use an isolated short-lived MCP session so interruption cleanup cannot break
the Hub's interactive connection. Approvals are revalidated before connection,
after discovery and before committing results. MCP content remains untrusted text.

OAuth/SSO flows, pagination beyond the bound, external schema references, interactive
MRTR/task follow-ups, hard process sandboxing and preconfigured security integrations
remain unimplemented. Connection errors suppress raw server bodies and stderr to
avoid leaking credentials; inspect the server separately when needed.

References: [official SDK](https://github.com/modelcontextprotocol/rust-sdk),
[official transports](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports).
