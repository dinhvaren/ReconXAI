# MCP Registry adapter

ReconX uses the documented `v0.1` API at
[the official MCP Registry](https://registry.modelcontextprotocol.io/docs).
The adapter is in `crates/reconx-core/src/registry.rs`; Desktop and CLI call the
same service. Upstream metadata is untrusted and does not grant installation or
execution permission.

Implemented: search, opaque cursors, bounded page sizes, encoded identifiers and
versions, latest/details/version listing, runtime filtering, incremental
`updated_since` requests, preserved server.json, official metadata schema
validation, bounded retries/backoff/Retry-After, HTTPS timeouts and a 2 MiB response
limit. An unknown schema remains explicit and cannot authorize installation.
The supported server schema does not establish universal OS compatibility, so
platform filtering fails explicitly rather than inventing platform support.

Migration015 adds durable versioned cache tables. Cache pages expire after
15 minutes; offline and network-fallback results disclose cached/stale status.
Clearing the cache does not remove installed extensions or research data.
Runtime filtering applies after upstream pagination, so a filtered page can be
empty while its cursor still points to further results. Version listing exposes
the upstream next cursor; the current version-history UI displays one page.

```text
reconx --json mcp catalog search everything --limit 2 --refresh
reconx --json mcp catalog search everything --limit 2 --cursor <opaque-cursor>
reconx --json mcp catalog info <namespace/server> --version <version>
reconx --json mcp catalog versions <namespace/server>
reconx --json mcp catalog search everything --limit 2 --offline
reconx --json mcp catalog clear
```

Protocol references: [published OpenAPI specification](https://github.com/modelcontextprotocol/registry/blob/main/docs/reference/api/openapi.yaml)
and [server.json specification](https://github.com/modelcontextprotocol/registry/blob/main/docs/reference/server-json/generic-server-json.md).
Bundled schema origin/hash and reviewed npm metadata are in `manifests/mcp/SOURCES.json`.

Local HTTP tests exercise pagination, caching/restart, stale fallback, malformed
JSON, response limits, rate limits and unavailable entries. Public HTTPS requests
have also experienced genuine timeouts with Rust, curl and Python. Consult the
current release ledger for the latest live verification; a successful root page
or Python metadata probe is not Core Registry acceptance.
