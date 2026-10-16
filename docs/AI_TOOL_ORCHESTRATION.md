# AI-to-tool orchestration: reviewed local analysis

The Rust capability registry is the source of truth. It combines implemented local
analysis, verified YARA-X availability and cached MCP tool schemas/permissions.
React never maintains independent execution capabilities. Discovery does not grant
permission, and registering a target does not authorize a tool.

## Real workflow

AI Workspace → Plan local analysis → select project/imported authorized file and
provider → preview exact metadata/capabilities → consent to this gateway → generate
strict JSON plan → inspect each capability → approve → existing Job Engine runs
actual local jobs → review results/logs → optionally consent to evidence interpretation.

The planner receives only displayed metadata (format/size/SHA-256/file ID), currently
available local capabilities and schema instructions. No sample bytes, local paths,
notes or credentials are sent. It does not silently use chat fallback providers.
Only 1–5 unique allowlisted local capability IDs and bounded explanations can pass
validation. Unknown fields, shell arguments, network tools and MCP capabilities are
rejected. File authorization/current scope, tool integrity and project state are
checked independently of model output. Approval is transactional, consumed once
and queues typed jobs in the existing global two-worker queue. No second scheduler.

Local capabilities: static parsing, metadata, strings, artifact index and fixed
YARA-X signatures. MCP tools are discoverable but require a separate, exact manual
invocation in MCP Hub. There is no automatic offensive target workflow.

Completed local job summaries can be previewed and sent to the chosen provider only
after explicit destination-bound consent. Explanations are saved separately as
unverified AI output, never promoted to verified findings. Arbitrary MCP output is
not accepted for this automatic interpretation path.

```
reconx workflow capabilities --project <project-id>
reconx workflow preview <file-id>
reconx workflow plan --input consent.json
reconx workflow list --project <project-id>
reconx workflow approve <plan-id> --approve
reconx jobs run
reconx workflow evidence <job-id>
reconx workflow interpret --input evidence-consent.json
reconx workflow interpretations <job-id>
```

Planning consent JSON: `file_id`, `provider_id`, `gateway`, exact `preview` string,
`consent`. Interpretation uses `job_id` instead of file_id. No API keys in either.

Persistence: migration 004 adds MCP configuration/discovery, invocation approvals,
tool installation snapshots, local AI plans/job links and separate interpretations.
Existing project/target/file/job tables are retained. Existing schema 1–3 databases
get a SQLite `VACUUM INTO` snapshot beside the private DB before this migration;
failure to create that backup aborts the upgrade. Backups contain private research
data and should be retained with the same protection as the workspace. Recovery is
manual: stop all clients before restoring a snapshot; do not run an old binary on
the upgraded DB. Migrations are additive and transactional; no destructive down
migration is automatic.

The integration tests use a clearly labelled deterministic local HTTP model adapter
to choose capabilities, then execute actual local analysis and interpret persisted
results. This does not verify a live cloud account or guarantee model quality.
Per-agent model routing, general agent/DAG scheduling, Authorized Auto, network recon,
findings/evidence normalization and report export remain future work. Scope review
continues as the existing deterministic local job, not a fictional Recon Agent.
