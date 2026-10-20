# Streaming file ingestion and bounded analysis

Complete file preservation is independent of the deep-parser memory budget.
`Core::import_file_with_progress` is shared by Desktop and CLI. It accepts a path,
project, authorization flag, import byte budget and cancellable progress callback.
The default budget is 8 GiB, configurable up to a 64 GiB ceiling; these are policy
limits, not claims that every size or format has been runtime verified. See current
test artifacts for the sizes actually executed.

The importer rejects non-regular/symbolic-link inputs, checks actual free disk space
for the complete copy plus a reserve, opens the source and checks size/mtime changes.
It hashes and copies in 256 KiB chunks to a private temporary artifact. A large
input retains only a 1 MiB prefix in memory. Small inputs may use the independently
bounded 64 MiB parser for existing format validation. Raw file bytes never cross
React/Tauri IPC. Originals are never modified or executed.

Cancellation and parse/I/O failures drop the incomplete staging file. After the copy
is synced, the target and imported-file metadata are written in one transaction.
The content-addressed blob is published without overwriting an existing blob;
existing bytes are fully verified before reuse. Existing target authorization is
preserved. SQLite INTEGER sizes now use checked i64/u64 conversion without a u32
truncation. No additional migration after 017 is needed.

Desktop sends only paths through `import_file_stream`. Two native copy operations
maximum run per process, on blocking workers with IPC updates limited to 20 Hz.
Cancel and leaving/changing the project cancel the active operation; completed
files remain preserved. Folder import is nonrecursive and limited to 100 entries,
with atomicity per file, not for the complete batch. The CLI supports:

```powershell
reconx reverse import owned.exe --project PROJECT_ID --authorized --max-mib 8192 --progress
reconx reverse import owned-folder --project PROJECT_ID --folder --max-mib 2048
reconx reverse analyze FILE_ID
reconx reverse metadata FILE_ID
```

Progress goes to stderr; JSON results remain on stdout. Ctrl+C cooperatively cancels
the CLI worker. Native Windows cancellation and Linux CLI SIGINT have separate
acceptance tests. A saved file/target is not itself an analysis result.

For files over 64 MiB, existing Job Engine metadata/strings/index handlers stream
complete integrity and entropy, then inspect only the first 1 MiB for strings/hex.
Their results explicitly mark limited analysis. Unknown section counts are null,
not zero. Deep sections/segments/imports/exports/package UI tabs are disabled with
an explanation. Full-file byte comparison and the fixed YARA-X byte adapter retain
their separate 64 MiB budgets. Signature labels identify observed magic bytes,
not validation of a whole executable/archive or a supported decompiler.

Limits remain: no general Ghidra/Rizin/decompiler integration, no continuation of a
partially copied file after process termination, no OS sandbox, and no general
per-workspace disk quota. Abrupt process termination can leave a private `.import-`
staging file; automatic crash-staging garbage collection is not yet implemented.
An already published complete content blob may also remain if a later database
commit fails; it is conservatively retained, never reported as a successful import.
Windows custom workspace ACL hardening remains separate from inherited permissions.
Blocking filesystem calls depend on OS/device behavior; cancellation does not make
arbitrary slow or unavailable filesystem drivers preemptible.

Tests: `crates/reconx-core/tests/large_import.rs`, deterministic CLI
`scripts/large_import_smoke.py` and native `tests/e2e/large-import.mjs`.
Fixtures are generated benign padded PE/ELF files, never executed. Independent
Python hashlib/Node streaming hashes compare both original and managed bytes.
Whole-product release readiness remains false.
