# Targets and local binary analysis

Both clients use shared Core services, the existing database and migration 002.
No provider configuration or project reset is needed.

## Targets

Create/select a project, then **+ Add Target** on Dashboard, Projects, Targets &
Scope, or **Quick actions → Add Target**. The dialog registers 17 input types,
description, grouping tags, authorization, allow/exclude rules and preferred mode.
Inventory supports search, project/type filters, edit, confirmed removal, native
JSON import/export, details, scope validation and linked imported assets/jobs.

```powershell
reconx target add --project demo --type domain --value example.test --name "Owned domain" --authorized
reconx target list --project demo --json
reconx target show TARGET_ID
reconx scope validate TARGET_ID --value api.example.test
reconx target export --project demo --output targets.json
reconx target import targets.json
reconx target update TARGET_ID --input target-input.json
reconx target remove TARGET_ID
```

`update` accepts a complete TargetInput JSON (one entry from the export's targets
array), not the ID/timestamp-bearing show response. Imports are atomic, version 1,
1–500 records, at most 1 MiB JSON; duplicate type/value pairs are rejected. Exports
never overwrite files. Reassign project IDs deliberately for a different workspace.
Existing project metadata exports exclude targets/sample blobs; use target export
separately. Tags provide grouping; hierarchical groups are not implemented.

Scope rules: exact host, `*.example.test`, HTTP(S) origin/path prefix, IP, CIDR,
absolute local path. Wildcards cover descendants, not the apex. Scheme, port and
path component boundaries matter. Exclusions take precedence. Empty allowed scope
uses the exact target value. URL credentials/query/fragment are rejected; IDNs use
punycode. Missing local paths can be registered; existing paths must match the
file/directory category. Registration and validation produce no network traffic
and grant no executable capabilities. Unsupported analyzers remain unavailable.

## Local binaries

Select a project → Reverse Engineering → Import File/Folder or native drop.
Confirm permission in the checkbox, then Analyze File. An imported file gets a
local target; existing authorization/scope is preserved. Edit an existing target
to confirm permission if necessary. Metadata tabs expose real structure and history.

```powershell
reconx reverse import C:\owned\sample.exe --project demo --authorized
reconx reverse import C:\owned\samples --project demo --authorized --folder
reconx reverse list --project demo --json
reconx reverse analyze FILE_ID
reconx reverse metadata FILE_ID --json
reconx reverse strings FILE_ID --json
reconx reverse compare FILE_A FILE_B --json
reconx jobs list --project demo --json
reconx jobs show JOB_ID
reconx jobs cancel JOB_ID
```

Without --project, CLI import requires exactly one active project. Without
--authorized, import works but analysis is blocked. Metadata/strings read the
latest saved result, never silently start a job. Content parsing supports PE,
ELF and thin Mach-O. ZIP/APK directory metadata is read without extraction;
AndroidManifest.xml identifies APKs. Unknown formats expose raw bytes, SHA-256,
entropy, ASCII and anchored UTF-16 LE strings. Architecture, entry point,
sections, segments and imports/exports are shown when available. Comparison is
byte/section/import comparison, not semantic patch analysis. No decompiler exists.

Limits: 64 MiB/sample; 4096 sections/imports/exports/package entries; 4000 sampled
strings of at most 256 characters; 100 direct folder entries; 512-byte hex preview.
ASCII sampling reserves capacity for UTF-16 and is not exhaustive. UI shows the
first 200 matches; search narrows them, CLI returns all retained rows. ZIP64,
recursive import, disassembly, YARA, Ghidra and AI artifact transmission are pending.
Section entropy scans share a 64 MiB byte budget; overlapping ranges cannot amplify
work without limit. Skipped section entropy is reported as unavailable.

Samples and raw result JSON stay in workspace imports; SQLite stores references,
checksums, metadata and logs. SHA-256 detects modified managed samples/artifacts.
Originals are never modified or executed. Use private filesystem permissions:
normal per-user Windows storage inherits account ACLs; Unix import directories
use 0700. Artifacts are not an encrypted vault and are not shareable reports.
No raw strings/hex are automatically sent to AI.

Cancellation is cooperative between bounded stages. Status/logs persist in both
clients. A crash may leave a running job requiring cancellation; automatic recovery,
retries, pause/resume and general DAG scheduling are pending. Removing imported
records preserves originals, cached blobs and job history. Cache GC is pending.

## Verification

`scripts/verify.ps1 -Inputs` runs checks and the real native input scenario.
The harness uses isolated Tauri/WebView2 and CLI children with one temporary
database. Windows helper automation selects owned PE/JSON through real native
dialogs. Hidden test windows retain the real renderer/IPC/dialogs and prevent
physical input racing automation. Records/screenshots live in ignored artifacts.
Linux runtime and installers remain unverified.
