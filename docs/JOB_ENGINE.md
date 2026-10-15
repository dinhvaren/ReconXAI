# Durable local research jobs

The shared Rust Core owns the queue, authorization, execution, local artifacts and
results. React never parses binaries, invents progress or executes shell strings.
This milestone supports local inputs, not a scanner or autonomous offensive agent.

## Implemented handlers

| Identifier | Actual operation |
| --- | --- |
| local_static_binary | Verify managed sample; bounded static parsing; save immutable analysis artifact |
| binary_metadata | Run the same bounded parser and retain actual metadata/structure in a job-linked artifact |
| strings_extraction | Run the shared parser and retain bounded ASCII/UTF-16 LE strings alongside metadata |
| artifact_index | Verify sample hash and available analysis artifact, persist a local inventory summary |
| target_registration_review | Evaluate current scope, exclusions and authorization locally; no target traffic |

Metadata and strings jobs currently retain the shared full static record rather than
separate partially populated binary schemas. A scope-review result can truthfully
say awaiting_authorization: registration review does not grant execution rights.
Registration alone does not install or enable an analyzer.

## State and ownership

Migration 003 replaces the old job-state constraint transactionally. Existing job
IDs, logs and binary-analysis references survive. It adds queued/running/completed/
failed/cancelled states, start/end timestamps, timeout, stage/progress, cancellation
request, error text, result references and worker leases. A separate file_notes
table stores explicitly entered local notes. No credential/settings table is reset.

An IMMEDIATE SQLite transaction claims one queued job and counts active work.
There is a **global ceiling of two running jobs per database**, even with multiple
Desktop or CLI workers. CLI pools accept concurrency 1 or 2; Desktop starts two
workers. A cancellation request retains the running capacity slot until acknowledged.
Queued cancellation is immediate. All terminal transitions use worker/state fencing;
result publication and the completed transition share one transaction.

Workers heartbeat every 500 ms with a 10-second lease. Startup and queue polling
reconcile expired/legacy ownerless running work as JOB_INTERRUPTED (or cancelled
when already requested). Live leases are preserved when another client opens the
database. Queued work survives restart. Interrupted work is not automatically retried.
Graceful engine shutdown stops claims and cooperatively interrupts owned work.

Timeouts are configurable from 1 ms to one hour and start at claim, excluding queue
wait. Cancellation, shutdown, current scope and timeout are checked at bounded read/
hash/string stages. Progress records actual processed bytes and completed parsing
stages; it is not a time estimate. The parser still needs a bounded in-memory byte
slice, capped at 64 MiB per worker. Reads use 256 KiB chunks. Parser work is isolated
from the UI thread, with defensive cardinality limits and a Rust panic boundary.
It is **not OS-process sandboxing or hard preemptive termination** of parser calls.

The native research-job event carries persisted job snapshots. Job Center and the
global dashboard also read durable state periodically, including changes/recovery
from another client. These timers query actual state and never generate results.

## Desktop

Quick actions -> View Jobs opens the shared Job Center. Choose a real imported file
or target, handler and timeout, then Queue job. Search/filter by status/project;
inspect actual progress/logs, open exact saved results or cancel work. The file
workspace's Analyze File action uses the same queue. Target detail offers a real
local scope-review job. File notes survive restart; the research pane stacks below
analysis at smaller widths and scrolls independently at wide desktop sizes.

AI artifact review is optional and never automatic. The user chooses a metadata,
imports or sampled-strings preview, sees the gateway and explicitly approves sending
it. Backend validation binds approval to the exact current preview and captured
gateway. Paths and notes are excluded; strings may contain sensitive sample data,
which the UI states. This route deliberately does not use chat's model fallback.
General AI chat and its existing fallback behavior remain unchanged. AI output is
labelled unverified and is not a verified finding or decompilation.

## CLI

```powershell
reconx project create demo
reconx target add --project demo --type domain --value example.test --authorized
reconx scope show TARGET_ID --json
reconx scope validate TARGET_ID --json
reconx reverse import C:\Owned\sample.exe --project demo --authorized
reconx reverse analyze FILE_ID
reconx reverse metadata FILE_ID --json
reconx reverse strings FILE_ID --json
reconx reverse note FILE_ID --input local-notes.txt
reconx jobs enqueue --kind binary_metadata --file FILE_ID --timeout-ms 60000
reconx jobs enqueue --kind strings_extraction --file FILE_ID
reconx jobs enqueue --kind artifact_index --file FILE_ID
reconx jobs enqueue --kind target_registration_review --target TARGET_ID
reconx jobs run --concurrency 2
reconx jobs run --watch --concurrency 1
reconx jobs list --json
reconx jobs show JOB_ID --json
reconx jobs result JOB_ID --json
reconx jobs cancel JOB_ID --json
```

enqueue returns a durable queued record. Desktop executes queued work while open;
otherwise use jobs run to drain the queue or --watch for a foreground worker.
reverse analyze waits for capacity or an already claimed Desktop worker, then
returns the same core handler's committed result. This avoids a claim race when
both clients are open. Cancellation reports the actual state, including a pending
request while running. result reads the exact job result, not a later analysis.

## Verification and limits

tests/e2e/jobs.mjs uses actual Windows Tauri/WebView2, an owned real PE, independent
SHA-256, an actual native picker and a controlled 64 MiB raw file. It checks shared
target/analysis persistence, all handler outputs, notes/consent, native running/
cancelled/failed states, real process termination/restart, and three themes.
crates/reconx-core/tests/jobs.rs tests transactional migration, global concurrency,
live/expired leases, cancellation fencing, corruption, timeout and privacy boundaries.
All fixtures use isolated data and browser profiles. No public target is contacted.

Limits retained: no DAG/retries/pause-resume, recursive import, parser process
sandbox, cache GC, disassembler/decompiler, Ghidra/MCP, scanner, normalized findings
or report generation. Folder import has core tests; physical OS drop and folder
picker interactions remain unverified. Linux/ARM64 and installer execution remain
unverified. Stop older clients before upgrading their database to schema 3.

Tauri APIs were checked against installed source and official docs:
[Emitter](https://docs.rs/tauri/2.12.3/tauri/trait.Emitter.html),
[RunEvent](https://docs.rs/tauri/2.12.3/tauri/enum.RunEvent.html).
