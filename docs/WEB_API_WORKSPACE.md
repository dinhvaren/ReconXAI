# Current manual extension - 2026-10-18

Offline HAR/OpenAPI/schema workflows below are preserved. A separate, individually reviewed owned-loopback request editor and secure project identity manager now reuse those captures, jobs, evidence and reports. Import/save never sends traffic. Commands, security boundaries and limits: [HTTP_REPLAY.md](HTTP_REPLAY.md). Current actual runtime proof: artifacts/replay-current-verification.json; overall Web/API remains PARTIAL. The existing captured role comparison is not an automated active authorization matrix.

---

Older checkpoints below are retained for provenance; the current checkpoint above takes precedence.

# Current manual extension - 2026-10-18

Offline HAR/OpenAPI/schema workflows below are preserved. A separate, individually reviewed owned-loopback request editor and secure project identity manager now reuse those captures, jobs, evidence and reports. Import/save never sends traffic. Commands, security boundaries and limits: [HTTP_REPLAY.md](HTTP_REPLAY.md). Current actual runtime proof: artifacts/replay-current-verification.json; overall Web/API remains PARTIAL. The existing captured role comparison is not an automated active authorization matrix.

---

Older checkpoints below are retained for provenance; the current checkpoint above takes precedence.

Current extension (2026-10-18): bounded offline schema observations are now available through the existing worker, Desktop and CLI. See [HTTP_SCHEMA_ANALYSIS.md](HTTP_SCHEMA_ANALYSIS.md) for exact coverage, source provenance, migration 011 and limitations. Active replay and credential-backed authorization testing remain unfinished.

# Offline Web/API workspace

The Web Security and API Security pages share a Rust service, schema 010 and the
existing evidence/job/finding/report pipeline. This is an offline traffic-analysis
slice; it is not the complete Web/API assessment or v1.0 release.

## Workflow

1. Select an existing project and, optionally, one of its targets.
2. Import HAR or OpenAPI through the native picker or drop files on the page.
3. Review the filenames, title and optional captured identity/role labels, then import.
4. Inspect traffic, decoded redacted JSON, headers, cookies, query/path parameter
   names, timings, documented endpoints and import provenance.
5. Queue a captured-header review. It executes real offline work through the existing
   Job Engine and saves immutable evidence; it never sends the imported request.
6. Compare two captured responses, optionally selecting a top-level JSON ownership
   field. Save a Needs Verification finding and review/export an existing report.
7. Optionally approve the exact selected header-result preview and destination for
   AI interpretation. Raw HAR/specification artifacts are not included in this request.

The identity matrix aggregates all captures for the selected project/target, not
just the 100-row traffic page. It groups origin, method, documented path (when
matched), user-supplied identity and role, and records counts, statuses, distinct
paths/body hashes and source capture IDs. These are captured observations, not
authenticated identities, an expected-permission policy or verified vulnerabilities.

## Formats and correlation

- HAR 1.2: request/response metadata, headers, cookies, body metadata, base64 response
  decoding, timings, duplicate endpoint counts and authentication mechanism labels.
- OpenAPI 3.0/3.1 JSON/YAML: servers, operations, inherited/overridden parameters,
  request/response schemas, security requirements and components. Same-document JSON
  Pointer references are checked; structural expansion detects cycles and stops at
  32 reference hops. Recursive schemas remain references, not infinitely expanded data.
- Absolute HTTP(S) server origins must match scheme, host and effective port.
  Root-relative servers match the captured origin and declared base path. Default
  server variables are expanded once, with bounded output and optional enum validation.
  Concrete paths take precedence over whole-segment path parameters. Partial-segment
  templates and document-relative server URLs are not fully supported.
- Target-bound definitions apply only to captures associated with that target.
  Project-level definitions can match project captures. Inventory/matrix groups
  retain target IDs so overlapping targets are not silently merged in the all-project view.
- External/multi-file references, YAML aliases/anchors/merge keys, arbitrary tags,
  automatic network resolution and full JSON Schema validation are unavailable.
  An unmatched endpoint is an observation, not a security finding by itself.

## Privacy and integrity

Normalized SQLite records and previews mask all query values, cookie values,
non-allowlisted headers and recognized sensitive JSON fields. Non-JSON/binary body
previews are withheld. Names and nonsensitive HTTP metadata remain useful for review.
Redaction uses conservative header rules plus heuristic JSON/text detection; it
cannot prove that arbitrary free text contains no private information. Review
exports and exact AI previews before sharing.

Original files are immutable Restricted evidence attachments with SHA-256 hashes.
They can contain credentials and are intentionally retained outside normal SQLite
fields. Normal previews omit them; raw local export requires separate explicit
approval. Comparison creation verifies original evidence integrity and records both
source evidence IDs/hashes. Integrity verification is not vulnerability verification.

Import/normalization happens before persistent insertion. Invalid inputs do not
create partial import records. Schema 010 is additive; existing schemas are
snapshotted before migration. No projects, AI credential references or settings are reset.

## Limits

| Resource | Limit |
| --- | --- |
| Input file | 16 MiB, regular file, no final symlink/reparse point |
| HAR entries / OpenAPI operations | 2,000 per import |
| Decoded body | 256 KiB each; 512 KiB encoded source text |
| Headers/cookies/parameters | 256 per supported collection |
| Parsed tree | Depth 48; 200,000 nodes/events |
| References | 4,096; structural expansion 32 hops |
| Server templates | 32 per operation; 32 variables; 256-byte defaults |
| Server/path expansion | 8 KiB; 128 path segments |
| Correlation | 1,000,000 capture × compiled-server-pattern comparisons |
| Workspace inventory | 5,000 captures/operations; 32 MiB per collection |
| Traffic page | 100 captures; matrix/inventory still cover all loaded captures |
| Redacted export | All bounded captures, maximum 64 MiB; never a silent first-page export |

Over-budget work returns a visible error. Filter to one target or split the
workspace; no silent truncation of an accepted traffic export is performed.

## CLI

Use the same data directory/database as Desktop. All commands support global JSON
output. Imports do not replay traffic or contact a model.

```text
reconx --json web import-har capture.har --project demo --identity alice --role reader
reconx --json api import-openapi openapi.yaml --project demo
reconx --json web traffic --project demo --offset 0
reconx --json web inspect <capture-id>
reconx --json api endpoints --project demo
reconx --json api inspect <operation-id>
reconx --json api authz-matrix --project demo
reconx --json api diff <left-capture> <right-capture> --owner-field owner
reconx --json api finding <comparison-id>
reconx --json web analyze <capture-id>
reconx --json jobs run
reconx --json web export --project demo new-redacted-output.json
```

## Verification and remaining work

See TEST_RESULTS.md and MASTER_EXECUTION_PROGRESS.md for exact current results and
matching native binaries. Deterministic fixtures include synthetic credentials,
malformed/oversized HAR, JSON/YAML OpenAPI, cycles, invalid server templates, origin
mismatches, pagination and evidence tampering. Native acceptance uses the real
Tauri/WebView2 process, native file picker, isolated SQLite, restart and an explicitly
approved loopback mock model. Mock AI results are not live cloud verification.

Request editing/replay, secure identity credential execution, active role tests,
automatic BOLA/BFLA validation, full schema comparison, external references and
validated Burp/ZAP integrations remain incomplete. Replay stays visibly disabled.
Linux Desktop, installers, signing and live cloud accounts are separately unverified.

Protocol references: [HAR 1.2](https://webperfwg.org/specs/HAR/Overview.html),
[OpenAPI 3.0.4](https://spec.openapis.org/oas/v3.0.4.html),
[OpenAPI 3.1.1](https://spec.openapis.org/oas/v3.1.1.html).
