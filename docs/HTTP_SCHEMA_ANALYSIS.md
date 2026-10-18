# Offline HTTP schema observations

This feature compares an imported HAR capture with one selected OpenAPI operation.
It sends zero requests. The result is a **partial consistency review**, not a
vulnerability finding or a certificate that an API conforms to its specification.

## Desktop and CLI

Open Web Security or API Security, select the project/target and a traffic capture.
Under Local capture analysis, select **Compare with documented operation**, then
**Check documented schema**. The existing worker runs the job. Its status, cancel
action, logs, observations and source locations appear below the controls. Open
result evidence to use the existing evidence/finding/report workflow. AI result
review retains its separate destination-bound preview and consent requirement.

```text
reconx --json web traffic --project <project>
reconx --json api endpoints --project <project>
reconx --json api schema-check <capture-id> <operation-id>
reconx --json jobs run
reconx --json jobs result <job-id>
reconx --json evidence from-job <job-id>
```

`schema-check` queues work; `jobs run` drains the existing durable queue. Desktop
normally has that worker running already. A duplicate active capture/operation job
is rejected. Results/logs/evidence survive restart and use the same SQLite database
in Desktop and CLI. No second scheduler or separate configuration is introduced.

## Supported observations

- JSON request/response `type`, `required`, `properties` and homogeneous `items`.
  OpenAPI 3.0 explicit `nullable` and 3.1 null type unions are distinguished.
- Exact response code, then uppercase status range, then `default` precedence.
- Documented media types, case-insensitive Content-Type and media wildcards;
  conflicting HAR MIME metadata is an observation, with the header taking priority.
- Required response-header and request query/header/cookie presence. Path presence
  follows the matched complete path template. Redacted/serialized parameter values
  are explicitly inconclusive; this feature does not guess their types.
- Declared HTTP Basic/Bearer and API-key mechanism presence. Security alternatives
  use OR between requirement objects and AND within one object. Presence does not
  authenticate an account or establish authorization. OAuth and other contextual
  mechanisms remain unsupported/inconclusive.
- Same-document JSON Pointer schema references, including finite instances of
  recursive schemas. Non-progressing cycles are detected. Each constraint location
  points to the original OpenAPI document, including resolved component references.
- Direction-aware required properties: OpenAPI 3.0 readOnly/writeOnly exceptions;
  3.1 application-dependent directional cases are kept inconclusive.

Every observation has one of `confirmed_documented_mismatch`, `possible_mismatch`,
`insufficient_evidence`, `unsupported_validation_case`. “Confirmed” describes a
constraint mismatch in supplied captured data, **not a verified vulnerability**.
Missing HAR bytes, withheld previews and redacted fields never produce a fabricated
missing-property/type verdict. No finding is automatically created or promoted.

## Evidence and limits

Migration **011_offline_schema_review.sql** adds only the job-to-capture/operation
binding table/index. Existing `analysis_jobs`, evidence, projects and settings are
reused. Before upgrading schemas 1–10, storage takes a consistent `VACUUM INTO`
`pre-v11-<timestamp>.bak` snapshot, including committed WAL data. Upgrade is one
transaction; backup failure prevents migration. Stop older clients before upgrading.
For recovery, stop all clients and retain the original and backup before restoring
the verified pre-upgrade snapshot with its compatible app. Do not edit user_version
to force a downgrade. Unix backups receive mode 0600; this change does not claim
new Windows ACL isolation guarantees.

The job binds both original SHA-256 hashes and normalized capture/operation content.
It verifies original artifacts and checks normalized SQLite records against those
artifacts. It requires the same project and **exact target association**, plus an
absolute documented server matching scheme, host, effective port, method and path.
Root-relative server definitions cannot establish that origin and are rejected for
this review. Endpoint browsing continues to support them independently.

Each observation stores source evidence IDs, an original HAR JSON Pointer, a
separate pointer within the decoded JSON body, and an OpenAPI JSON Pointer. Payload
values/credentials are not copied into observation messages. Private original
evidence stays Restricted; the job uses redacted normalized body structure. Default
exports retain existing redaction and reviewed-report gates.

Imports retain existing 16 MiB / 2,000 entry / 256 KiB decoded-body limits. Schema
review adds depth 32, 10,000 validation steps, at most 200 observations and a 1 MiB
observation-output budget. Cancellation/timeout checkpoints use the existing worker.
Budget exhaustion yields explicit incomplete coverage. No external file/network
reference resolution, regex execution, imported code or request replay occurs.

Older captures without a source locator are uniquely matched to their original HAR.
Ambiguous identical duplicates fail with SOURCE_AMBIGUOUS and may be reimported;
the original records are preserved. New imports store the entry pointer directly.

## Remaining work

This is not a full JSON Schema validator. Composition, additionalProperties,
enum/const, numeric/string constraints, formats, tuples, custom dialects and URI
resource scope are not implemented; encountered unsupported constraints are labeled.
Only observed JSON structure is evaluated, never hidden values. Absence of a mismatch
does not mean the entire schema passed. Active replay, credential-backed identities,
stateful authorization testing, scanner adapters and autonomous Web/API execution
remain separate unfinished milestones. Linux Desktop, installers and live cloud
interoperability are not verified by the Core/CLI tests.

Protocol references: [OpenAPI 3.0.4](https://spec.openapis.org/oas/v3.0.4.html),
[OpenAPI 3.1.1](https://spec.openapis.org/oas/v3.1.1.html),
[JSON Schema recursion guidance](https://json-schema.org/draft/2020-12/json-schema-core#section-9.4.1).
Current execution evidence is recorded in TEST_RESULTS.md and the master ledger.
