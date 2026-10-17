# Evidence, findings and reports

This extends the existing Core, SQLite and Job Engine. No scanner or additional
scheduler is introduced. Desktop and CLI use identical typed services. Schema 005
adds evidence, findings, evidence links, immutable reports and export receipts;
the current schema-6 build snapshots existing schema 1-5 databases to a consistent pre-v6 backup before migration. Phase 1 alone used pre-v5 snapshots.

## Evidence

New completed jobs atomically publish an evidence reference with their completed
state. Stored content contains the actual result, job/source identifiers and, when
available, the complete static-analysis artifact. Failed/cancelled work creates no
result evidence. `from-job` collects older completed results idempotently.

Manual evidence supports HTTP request/response text, JSON, logs, screenshots,
binary/static/YARA artifacts, source references, MCP records, AI summaries, notes
and attachments. Artifacts live outside SQLite in UUID-named files beside the DB;
metadata retains project/target/job, SHA-256, size, source, timestamp, sensitivity
and review state. Views, reviews and exports check size and hash before use.
Symlink/reparse-point inputs and redirected artifact directories are rejected.
Original filenames/absolute source paths are not used as artifact paths.

Text entry is limited to 1 MiB, imports to 32 MiB. Text preview is limited to 1 MiB;
PNG/JPEG preview to 8 MiB. Screenshots are bounded to 8192 pixels per side and
16 megapixels using header-only dimension parsing before browser decoding.
Larger content stays available through explicit export.
Screenshots are identified by content headers; browser decoding is separate and
does not execute a sample. No binary/archive extraction is performed by this module.

`observed` means a real completed tool result, not a verified vulnerability.
`manually_reviewed` requires an explicit source-review note. AI summaries always
remain `unverified`. Integrity and review are independent. Review metadata may
change; the content and its hash are immutable. Evidence is retained when findings
are deleted and is never silently removed during duplicate merging.

```
reconx evidence list --project demo
reconx evidence show ID
reconx evidence import owned.txt --project demo --kind http_response
reconx evidence add --input metadata.json --text note.txt
reconx evidence from-job JOB_ID
reconx evidence review ID --note review.txt
reconx evidence verify ID
reconx evidence export ID --output redacted.txt
reconx evidence export ID --output original.bin --raw-approved
```

Binary/image export requires explicit raw approval because automatic image/binary
redaction is unavailable. Text exports redact by default; structured JSON exports
preserve JSON structure. No originals are overwritten. Raw evidence can contain
sensitive material; store the workspace on a private or encrypted volume. Unix
directories/files use 0700/0600. Windows currently inherits workspace ACLs, so
custom shared-directory ACLs require review; process isolation or encryption at
rest is not implied. Provider credentials remain in the existing OS secret store.

## Findings and CVSS

Findings retain target, asset, category, CWE/CVE, severity/confidence/status,
technical details, reproduction/impact/remediation, references, notes, provenance
and source evidence IDs. IDs are validated as identifiers, not looked up or
invented. Revisions reject stale Desktop/CLI edits. Cross-project/target evidence
links are rejected. Compare shows original records; merge retains the duplicate
record and content, marks Duplicate and combines evidence links transactionally.

Verified requires explicit manual confirmation, a verification note and at least
one integrity-checked, manually reviewed, non-AI source record. AI confidence or
a matching file signature cannot grant this state. There is no automatic AI
finding-creation/promotion service.

CVSS uses the reviewed RustSec `cvss` library. The application independently
requires all base metrics and rejects duplicate/undefined metrics; partial vectors
must not receive library default scores. CVSS 3.1 returns the base score; CVSS 4.0
uses the supplied metric groups. Severity remains a separate researcher choice.

```
reconx findings list --project demo
reconx findings show ID
reconx findings create --input finding.json
reconx findings update ID --input finding.json --revision 1
reconx findings update ID --input finding.json --revision 1 --confirm-verified
reconx findings compare A B
reconx findings merge PRIMARY DUPLICATE
reconx findings remove ID --revision 2
reconx findings cvss 'CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H'
```

Scoring implementation: [RustSec cvss](https://docs.rs/cvss/3.0.0/cvss/).
Standards: [FIRST CVSS 3.1](https://www.first.org/cvss/v3.1/specification-document),
[FIRST CVSS 4.0](https://www.first.org/cvss/v4.0/specification-document).

## Report snapshots

Prepare a project-scoped snapshot of explicitly selected findings/evidence. Linked
evidence is included automatically and checked for integrity. Preview displays
redactions, statuses and quality warnings. Export requires that exact saved preview
hash. Exports recheck original evidence integrity and never overwrite a file.
JSON preview displays all structured export fields, and changing the export format
clears the approval checkbox. Native save dialogs suggest the selected format's
extension rather than reusing a target-export filename.
Later edits/deletion of a finding do not rewrite a historical report snapshot.

Seven template labels share a research-report structure; bounty templates label
proof-of-concept sections explicitly. These are editable input fields and observed
facts, not generated vulnerability claims. This is a text-first report engine,
not a visual report designer or an automatic executive-summary generator.

The snapshot contains scope, methodology, tools, finding details/reproduction/
impact/remediation/references, evidence and interpretation limits. Draft/false
positive/duplicate states stay visible. No findings selected explicitly does not
mean an assessment found no vulnerabilities. Text evidence above 64 KiB is omitted
with a visible warning and hash. Raw binary/images are omitted with warnings;
metadata/hash remain. There is a maximum of 100 findings/100 evidence per report
and an 8 MiB snapshot bound. List views currently show the latest 1000 records;
individual evidence/finding/report IDs remain directly accessible.

Default redaction covers credential field names, auth/cookie headers, common token
prefixes, private-key blocks, URL credentials and local machine paths. `[REDACTED]`
and counts remain visible. Pattern detection cannot guarantee removal of unknown
secrets; manual review is mandatory before sharing. No raw attachment is silently
embedded. HTML escapes content and denies external loads/scripts with a CSP;
Markdown escapes active markup. JSON remains structured and UTF-8.

PDF uses `krilla` with the existing bundled OFL Source Sans 3 at explicit weight
400, native font subsetting
and pagination; no browser, remote fonts, JavaScript or external network renderer.
English/Vietnamese extraction is tested independently with pypdf. Unsupported glyphs
produce an explicit error; advanced bidirectional/mixed-script layout and embedded
screenshots are not claimed. Extremely large PDFs are bounded to about 400 pages.

```
reconx report prepare --input report.json
reconx report list --project demo
reconx report show ID
reconx report export ID --format pdf --output report.pdf --reviewed-sha256 PREVIEW_SHA
```

Formats: `markdown`, `json`, `html`, `pdf`. Prepare input fields: `project_id`,
`title`, `template`, `summary`, `methodology`, `finding_ids`, `evidence_ids`.
Use global `--json` and `--database` as with other CLI commands. File inputs avoid
putting sensitive evidence or notes in command-line arguments. All processing here
is local and performs no AI/cloud transmission.

Native acceptance: `scripts/verify.ps1 -Outputs` (requires the installed independent
pypdf parser; CI uses the hash-pinned fixture requirement). Final observed results
are recorded in TEST_RESULTS.md and MASTER_EXECUTION_PROGRESS.md.

## Structured-artifact redaction correction (2026-10-17)

Native loopback acceptance reproduced a report-only leak of name/value HTTP
headers encoded inside JSON evidence text. Report preparation now structurally
redacts complete JSON artifacts before embedding text, including sensitive
value_base64 header pairs. Evidence originals remain untouched. Regression tests
cover preview, complete JSON snapshot and exports. Older snapshots with detectable
unredacted structured headers are preserved but refused export with
REPORT_REDACTION_REQUIRED; prepare and review a new snapshot. This does not make
unknown credential patterns detectable or remove the need for manual preview.

Evidence and report exports now share structured-content redaction, including
JSON pasted as ordinary HTTP text and nested serialized JSON. Embedded JSON
strings beyond four layers are visibly omitted with a separate-review marker;
raw originals are retained. This bounds traversal and avoids silently exporting
credentials hidden at deeper layers. Busy output dialogs visibly disable close
until the operation finishes instead of accepting a click and ignoring it.
