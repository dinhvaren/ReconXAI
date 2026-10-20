# Reviewed dependency resolver

Implemented adapter: `reconx.mcp-everything`, official npm package
`@modelcontextprotocol/server-everything` at reviewed versions1.0.0 and2.0.0.
Compiled lockfiles resolve108 and94 package instances respectively. This is a
bounded supported adapter, not permission to run arbitrary Registry packages.
PyPI/uv, Cargo, OCI, MCPB, NuGet and arbitrary standalone packages currently
require manual setup; remote MCP configuration remains in the existing Hub.

The Rust resolver checks direct/transitive edges, closest Node resolution,
missing/circular edges, linked packages and scripts, source origin and integrity.
Node.js >=24 is the reviewed runtime policy for this adapter. A compatible
existing executable is reused with its approved path/version/hash; otherwise a
portable Node24.22.0 is offered in the exact approval plan. Official distribution
archive and executable SHA-256 values are compiled from actual downloaded
artifacts. Only Node itself and its license need extraction; npm installation
scripts and npm/npx executables are unnecessary for this installer.

Packages live under the selected data directory's `extensions` directory, outside
projects/source. The plan checks OS/x64 architecture, free disk space, runtime,
exact dependencies, current registration and configuration revision. Approval is
invalidated if those inputs change. No administrator access or global PATH edit
is requested. Host system-library incompatibility remains a truthful runtime
failure, not automatic privileged package installation.

```text
reconx --json tools dependencies reconx.mcp-everything
reconx --json extensions plan reconx.mcp-everything --version 2.0.0
reconx --json extensions install reconx.mcp-everything --version 2.0.0 --fingerprint <approved-plan> --approve --wait
reconx --json mcp install reconx.mcp-everything --fingerprint <approved-plan> --approve --wait
reconx --json extensions permissions reconx.mcp-everything --tool echo --approve
reconx --json mcp doctor
```

`--managed-runtime` on plan/install explicitly chooses an isolated portable runtime.
Without `--wait`, installation queues in the existing Job Engine and requires the
Desktop worker or `reconx jobs run`. Cancellation/timeout leaves the former active
configuration intact. Schema016 stores operation plans, history, receipts,
discovery and reconciliation state. Package activation occurs in the same SQLite
transaction as successful job completion after actual MCP health verification.

Reviewed version selection supports updates/reinstall and rollback by a new exact
plan. General remote update feeds are not implemented. Shared runtimes, verified
download cache and previous staged versions are retained; shared/user-managed
programs are not removed on extension uninstall. General cache/version garbage
collection remains future work.
