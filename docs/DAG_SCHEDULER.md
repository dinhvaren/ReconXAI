# Persistent reviewed offline DAG

Source: `crates/reconx-core/src/agent_runtime.rs`, `src/jobs.rs` and migration
`020_local_agent_runtime.sql`. Schema020 is additive, uses existing migration
backups and preserves old jobs, imports, projects, credentials and graph records.

## Scheduling

The reviewed proposal compiles to scope review → independent selected file jobs
→ integrity validation → report. Validation requires every earlier task to
succeed; reporting depends on validation. Cycles, self/dangling/duplicate edges,
duplicate/oversized IDs, unknown fields and plans outside 1–12 tasks are rejected.
Users do not submit arbitrary worker programs or conditional expressions.

`analysis_jobs` is the sole execution queue. New tables hold coordination,
assignments, tasks/dependencies, attempt/job mappings, messages and transitions.
Claims use existing fenced workers and SQLite IMMEDIATE transactions. Direct
job execution cannot bypass pause, dependencies, concurrency or budgets. Old
attempts cannot become current by requeueing them.

| State | Actual meaning |
|---|---|
| PENDING | Waiting for successful dependencies |
| READY | Eligible for atomic Job Engine claim |
| RUNNING | A real current attempt owns a running job |
| PAUSE_REQUESTED | Current job may finish; new claims stop |
| PAUSED | Queued work waits at a job boundary |
| RETRY_WAIT | Explicit replacement attempt waits 500/1,000 ms backoff |
| COMPLETED | Actual job result/evidence committed |
| FAILED | Actual attempt failed |
| CANCELLED | Cancellation/stopped runtime prevents execution |
| BLOCKED | Failed/interrupted prerequisite prevents progress |

There is no WAITING_APPROVAL task: the research session requires approval before
runtime jobs are created. SQL triggers reject illegal transitions and record each
initial/change state with its actual job/attempt. No automatic retries occur.

## Pause, recovery and limits

Pause drains running jobs and prevents new claims; it does not freeze arbitrary
threads/subprocess memory. Resume revalidates authorization and retains completed
results. Expired worker leases become JOB_INTERRUPTED under the existing engine,
pause the runtime and block remaining tasks. Explicit retry is required for
eligible incomplete offline work. Old attempt IDs/results persist and completed
tasks never rerun. Network/state-changing work is not dispatched or retried here.

Cancellation persists queued cancellation and cooperative running-job stop
requests even if the event journal is full. Deadline/invocation exhaustion stops
new actions and checkpoints recheck time/policy. Paused time counts toward the
original deadline. Report persistence occurs inside its actual job commit; a
rolled-back artifact write can leave an unreferenced private file, consistent
with the existing mechanism. Automatic orphan cleanup is not implemented.

Binary strings can contain Unicode absent from the bundled PDF font. PDF export
now displays those scalars/control codes as explicit `[U+XXXX]` tokens and adds a
display note. Original evidence and the approved Unicode JSON/HTML snapshot are
unchanged; existing redaction still happens before formatting. This prevents a
real ELF-string report from failing solely because one glyph is unavailable.

## Shared UI and CLI

The Research Session Offline agent DAG panel reviews actual dependencies and
limits. Changing limits invalidates approval. Start/pause/resume/retry/cancel use
Core; stale session/revision responses are discarded. Events use 100-item pages.
Historical removed targets cannot grant new execution authority.

```text
reconx --json agents list
reconx --json research dag-preview <session-id>
reconx --json research dag-start review.json
reconx --json agents status <session-id> --after 0
reconx --json research pause <session-id>
reconx --json research resume <session-id>
reconx --json research retry <session-id>
reconx --json research cancel <session-id>
reconx --json workflow tasks <session-id>
reconx --json workflow graph <session-id>
reconx --json jobs run --concurrency 2
```

Review JSON contains `session_id`, exact preview `fingerprint`, `limits` and
`approved: true`, no credentials. Preview accepts strict `--limits` JSON.
`jobs run` returns when only paused/stopped DAG work remains, preserving queued
rows for resume. Ordinary jobs are unchanged. No generic `research start` alias
or new traffic CLI is implemented in this slice.

Reconciliation processes at most 100 noncompleted sessions/sweep; this is not a
certified large-workspace scheduler. Graph projections have source budgets.
Cloud token/cost limits, general DAG editing, autonomous continuation, complete
approval history, cross-session graph clustering, recording policies, Linux GUI
and signed installer/update acceptance remain separate open requirements.
