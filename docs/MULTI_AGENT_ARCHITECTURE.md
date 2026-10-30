# Reviewed offline agent runtime — schema020

ReconX AI remains 0.1.0; the overall multi-agent requirement remains PARTIAL.
This implementation coordinates typed offline Core handlers, not independent
LLM conversations or an autonomous pentest loop. Network, MCP, generated commands
and model-driven task expansion are rejected by this runtime. Existing separately
reviewed network/AI workflows are preserved.

| Stable role | Actual responsibility | Model requirement |
|---|---|---|
| orchestrator | Transactional lifecycle, dependencies, quotas and reconciliation | None |
| planner | Existing persisted research proposal, optionally AI-generated after exact context consent | Existing selected provider |
| recon | Offline target registration/scope review, no scanner dispatch | None |
| reverse | Existing metadata, bounded strings and fixed offline YARA-X jobs when selected/available | None |
| validation | Read actual dependency evidence bytes and verify SHA-256 | None |
| reporting | Existing Report Engine snapshot committed with its job | None |

Descriptors declare IDs, capabilities, permissions, schemas, timeout and error
policy. These roles must not be advertised as six models collaborating or as a
completed Web/API worker. Planner/coordinator responsibilities do not invent
additional tool jobs.

`crates/reconx-core/src/agent_runtime.rs` compiles the reviewed proposal into an
offline DAG. Original project, target, sample and context/model fingerprint are
revalidated. Approval binds exact steps and limits. Assignments, dependencies,
attempts and existing `analysis_jobs` are created in one SQLite IMMEDIATE
transaction. There is no second execution queue or new database.

Atomic claims enforce current attempt, dependency completion, global/per-session
concurrency, deadline and invocation quota. Execution checkpoints revalidate role,
capability, target/project identity, archived state and scope. Model/tool text
cannot override these checks. Only successfully committed real evidence is used
for validation/reporting. Integrity does not verify a vulnerability; no automatic
Verified finding is created.

Typed events contain session/task/agent/correlation IDs, event type, timestamp,
structured payload and evidence references. Actual assignment/start, selected
progress, completion/failure, evidence links, plan acceptance, pause/resume/cancel
and session completion are journaled. Enum variants reserved for future work do
not mean those operations execute. This is typed dispatch and a durable journal,
not a general distributed bidirectional agent mailbox.

The journal is bounded to 2,048 messages/session, 8 KiB payload and 32 evidence
references/event, with duplicate correlation/type rejection. A full journal stops
new work but cannot roll back cancellation/pause; authoritative SQL task-state
history persists separately. Readers use 100-event pages.

Limits are concurrency 1–2, invocations 1–24, wall-time 1–3,600 seconds and explicit
retries 0–2. Paused time consumes the original wall-time budget. Runtime model and
network counts are zero because dispatch excludes those operations; runtime cost
is unknown (`null`), not a guessed zero price. Existing planning usage is separate.

Worker inference/token/cost budgets, general conditional DAGs, Web/API workers,
automatic replanning, new traffic policy and autonomous offensive continuation
remain unimplemented. These local workers receive no credential values. Native
processes retain actual OS permissions; subprocess execution is not sandboxing.

Tests: `crates/reconx-core/tests/agent_runtime.rs`, schema020 migration coverage,
`scripts/agent_runtime_smoke.py`, frontend tests and `tests/e2e/agents.mjs`.
Planning fixtures are MOCK_VERIFIED; jobs, evidence and reports use real code.
See the current test ledger for executed outcomes and `DAG_SCHEDULER.md` for
state/recovery semantics. This slice does not complete v1.0.
