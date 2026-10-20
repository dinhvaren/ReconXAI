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

**Assert supported formats** is unchecked by default. Enable it before queuing a
job to check the bounded syntactic formats listed below. Each result displays its
saved policy. Changing the checkbox does not change an existing job or result.
Old completed results without a recorded policy are identified as legacy results.

```text
reconx --json web traffic --project <project>
reconx --json api endpoints --project <project>
reconx --json api schema-check <capture-id> <operation-id>
reconx --json api schema-check <capture-id> <operation-id> --assert-formats
reconx --json jobs run
reconx --json jobs result <job-id>
reconx --json evidence from-job <job-id>
```

`schema-check` queues work; `jobs run` drains the existing durable queue. Desktop
normally has that worker running already. A duplicate active capture/operation job
is rejected. Results/logs/evidence survive restart and use the same SQLite database
in Desktop and CLI. No second scheduler or separate configuration is introduced.

Reusable offline CLI acceptance (Windows or Linux):

```text
python scripts/http_schema_smoke.py --binary PATH_TO_CLI --output NEW_DIRECTORY
python scripts/http_schema_smoke.py --binary PATH_TO_CLI --legacy-binary SCHEMA_12_CLI --output ANOTHER_NEW_DIRECTORY
```

This starts separate real CLI processes against a new synthetic workspace, checks
tuple/uniqueness/dependency/pattern/opted-in format observations and source hashes, retains job/evidence
and exports all four report formats. It refuses an existing destination and
invokes no network, AI or credential service. CI runs it after the release build;
local success does not establish remote CI or Linux Desktop acceptance.
The optional legacy binary creates a real schema-12 queued job, then the new binary
upgrades the isolated workspace and checks the consistent backup, foreign keys and
execution with the legacy fingerprint. It never opens the normal user database.

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
- Enum and OpenAPI 3.1 const equality use the existing JSON Schema library, including
  mathematical numeric equality and integers beyond the f64 exact range. Redacted
  descendants make equality inconclusive. Const is explicitly unsupported in 3.0.
- Minimum/maximum/exclusive bounds, multipleOf, Unicode code-point string lengths,
  array lengths and object-property counts respect the 3.0/3.1 constraint dialect.
  Invalid declarations are reported without copying library messages or values.
- allOf, anyOf, oneOf and not use three-way branch outcomes. Failed alternatives
  do not leak speculative errors into the final report; unsupported/redacted cases
  cannot be counted as successful matches. All branches share the work budget.
- OpenAPI 3.1 if/then/else uses the same three-way predicate outcome. A definite
  match selects the adjacent then schema; a definite mismatch selects else.
  Unsupported, redacted, cyclic or budget-limited predicates leave both branches
  unevaluated with an explicit conditional_branch observation. Predicate failures
  are speculative and never appear as failures of the enclosing schema. Only the
  selected branch retains constraint observations and its original pointers.
  Missing branches impose no constraint; orphan then/else are ignored and cannot
  interact across allOf/reference boundaries. The saved format option also applies
  inside predicates. Conditional keywords remain unsupported in OpenAPI 3.0.
  This review does not collect a complete JSON Schema annotation result tree.
  Semantics: [JSON Schema core section 10.2.2](https://json-schema.org/draft/2020-12/json-schema-core#section-10.2.2).
- additionalProperties supports booleans and a schema, with original property and
  schema pointers. OpenAPI 3.1 patternProperties evaluates every matching schema,
  including explicit properties and overlapping patterns. Matching uses search
  semantics unless the pattern contains anchors. Only properties matching neither
  properties nor any pattern receive additionalProperties checks. An unknown
  pattern result defers that decision instead of creating a false mismatch.
- String pattern constraints use the existing JSON Schema library's ECMA-aware
  translation and non-backtracking regex engine. Unanchored search, ASCII digit
  classes and ECMA whitespace have explicit regressions. Lookarounds,
  backreferences, invalid expressions and compilation-limit failures remain
  unsupported/inconclusive. Specialized group constructs (except noncapturing
  groups) and engine-only anchors are conservatively rejected, even when their
  literal/escaped spelling could otherwise be valid. This is a bounded supported
  subset, not a full ECMA expression grammar. patternProperties is unsupported
  in OpenAPI 3.0.
- Format assertions are an explicit per-job option in both 3.0 and 3.1. With the
  option off, formats produce annotation observations and do not affect branch
  matching. With it on, the existing JSON Schema library checks these syntactic
  formats: date, time, date-time, duration, email, hostname, ipv4, ipv6, uri,
  uri-reference, uri-template, uuid, json-pointer and relative-json-pointer.
  Unsupported/custom formats (including regex), invalid declarations, redacted
  input and exhausted limits are explicit/inconclusive. Only strings are checked.
  No DNS lookup, URL connection, email delivery or resource-existence test occurs.
  This bounded option does not implement the complete Format-Assertion vocabulary.
  Default annotation behavior and optional syntactic checks follow the distinction
  in [JSON Schema validation section 7.2](https://json-schema.org/draft/2020-12/json-schema-validation#section-7.2).
- uniqueItems uses JSON Schema numeric and structural equality in both supported
  OpenAPI dialects. Any redacted descendant or exhausted equality budget makes
  the comparison inconclusive, including within not/alternative branches.
- OpenAPI 3.1 prefixItems validates each present tuple position independently;
  items validates only the remaining positions. Short tuples are allowed unless
  minItems requires more elements. Boolean tail schemas and same-document
  references retain their exact locations. Invalid tuple boundaries defer tail
  checking. Legacy array-valued items remains explicitly unsupported.
- OpenAPI 3.1 dependentRequired and dependentSchemas activate on property presence,
  including null/false/redacted values. dependentSchemas evaluates the whole
  object. Declarations, unique dependency names, escaped JSON Pointers, cycles
  and resource limits are checked. Redacted property names are inconclusive.
  The 3.1 tuple/dependent keywords are explicitly unsupported in OpenAPI 3.0.

Every observation has one of `confirmed_documented_mismatch`, `possible_mismatch`,
`insufficient_evidence`, `unsupported_validation_case`, or `format_annotation`. “Confirmed” describes a
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

Migration **013_schema_review_options.sql** adds an options table referencing the
existing schema-job binding. Current upgrades from schemas 1–12 take a consistent
`pre-v13-<timestamp>.bak` snapshot before the transactional migration. Existing jobs
and results are preserved. Legacy queued jobs have no options row and keep their
original fingerprint/default-off policy. New jobs bind their explicit options to
the input fingerprint; changed/deleted options fail execution with SOURCE_CHANGED.
The binding is checked again before preparing the result. These hashes detect
accidental changes; they are not protection against a principal able to rewrite
the entire database. Backup/restore of a complete workspace remains unfinished.

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
Patterns add a shared 64-entry compiled cache, 1024 UTF-8 bytes per pattern,
128 KiB compiled-state and 128 KiB lazy-DFA limits per expression, and an 8 MiB
sum of pattern-byte/input-byte products for matching. All speculative branches
share these budgets. Library overhead is additional; this is not an OS process
memory or wall-clock sandbox. Engine failures are read through validate rather
than collapsed to nonmatches by is_valid. Redacted names/values remain unknown.
Formats allow 4096 UTF-8 bytes per input and a shared 1 MiB sum of input bytes
across all branches, with at most fourteen cached allowlisted validators. Imported
schemas never enter the format engine: only a closed, application-created format
schema is compiled. Parser/library overhead is additional to these input limits.
Budget exhaustion yields explicit incomplete coverage. No external file/network
reference resolution, imported code or request replay occurs.

Older captures without a source locator are uniquely matched to their original HAR.
Ambiguous identical duplicates fail with SOURCE_AMBIGUOUS and may be reimported;
the original records are preserved. New imports store the entry pointer directly.

## Remaining work

This is not a full JSON Schema validator. Advanced regex features, additional/custom
formats, legacy tuple/dependency syntax, unevaluated keywords, custom
dialects, multi-file references and URI resource scope remain unsupported.
Encountered unsupported constraints are labeled; no external resolver is enabled.
Only observed JSON structure is evaluated, never hidden values. Absence of a mismatch
does not mean the entire schema passed. Reviewed loopback replay and protected
identities are separately documented in HTTP_REPLAY.md; stateful authorization
testing, scanner adapters and autonomous Web/API execution remain incomplete.
Linux Desktop, installers and live cloud
interoperability are not verified by the Core/CLI tests.

Protocol references: [OpenAPI 3.0.4](https://spec.openapis.org/oas/v3.0.4.html),
[OpenAPI 3.1.1](https://spec.openapis.org/oas/v3.1.1.html),
[JSON Schema recursion guidance](https://json-schema.org/draft/2020-12/json-schema-core#section-9.4.1).
Current execution evidence is recorded in TEST_RESULTS.md and the master ledger.
