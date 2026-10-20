# Target-bound AI Research sessions

Version 0.1.0; this is a bounded local research workflow, not a complete automated
penetration test. The same Core service backs Desktop and CLI.

`workspace_selection` (migration 017) persists active project/target IDs and a CAS
revision. Removing a selected target or archiving its project invalidates selection.
The Desktop provider serializes changes, ignores stale asynchronous reads, polls
CLI changes and keeps IDs across navigation. A project change clears its target.
Record editors and file/target lists use that context; explicit report selections
may still include several targets within the project.

Start AI Research on Dashboard, Projects, Targets, Recon, Reverse or AI Workspace
opens one Research view. Select target/model/mode, review context, explicitly consent
to AI planning, then review the typed plan and approve execution. Available steps
are offline scope review, imported-file metadata/strings, installed fixed YARA-X
signatures and an individually approved owned-loopback HEAD where policy permits.
General domain enumeration, remote HTTP, decompilation, mobile/cloud/source analyzers
and arbitrary MCP execution are not supplied by this session adapter.

Manual mode builds an offline plan without AI. AI Assisted uses the selected
configured adapter to propose strict JSON with capability IDs and reasons only.
Authorized Auto currently supports only offline steps and still requires review of
the session's bounded plan. It is not a reusable standing authorization policy.
No model can supply shell commands, execution arguments or additional capabilities.
Required credentials are loaded by the existing AI service, never copied into the
planning context. Planning sends metadata only (IDs, kind, hash/size, allowed steps),
not original file paths, target URLs, user notes or binary contents.

The approval fingerprint binds target/scope, app mode, file identity, model/options
and available capabilities. A configuration change requires a new plan. Approval
is consumed once; every job is enqueued in one SQLite transaction through existing
Job Engine checks. A network step needs additional explicit approval; a failed
approval rolls back every queued step. Workers still validate current policy.

The session ledger links real jobs and their existing evidence, findings and report
snapshots. Status derives from persisted job states. Cancellation uses existing
fenced worker cancellation. Restart restores sessions from SQLite even when WebView
localStorage has not flushed. UI history currently loads twenty recent project
sessions; older IDs can be inspected through CLI. Pause/resume and DAG dependencies
are not implemented. Missing usage is shown as unavailable; no estimated zero cost.

AI correlation needs a separate exact evidence preview and destination/model consent.
Its result is a separate unverified AI-summary evidence record. Manually recorded
findings attach the session evidence and remain Needs Verification. Report preparation
and export reuse the existing review-hash/redaction/Markdown/JSON/HTML/PDF services.
Raw evidence and AI interpretation remain distinct. A linked interpretation does
not independently prove that the model's claims or citations are correct.

CLI examples (IDs are obtained from actual records):

```powershell
reconx workspace show
reconx workspace select --project PROJECT_ID --target TARGET_ID
reconx research preview TARGET_ID --provider PROVIDER_ID
reconx research plan reviewed-context.json
reconx research approve SESSION_ID --fingerprint EXACT_HASH --approve
# Only for a displayed permitted owned-loopback HEAD step:
reconx research approve SESSION_ID --fingerprint EXACT_HASH --approve --approve-network
reconx jobs run
reconx research show SESSION_ID
reconx research evidence SESSION_ID
reconx research interpret reviewed-evidence.json
reconx research finding SESSION_ID --title "Review note" --details-file note.txt
reconx research report SESSION_ID
```

`reviewed-context.json` contains `request`, `fingerprint`, `context`, `consent` from
the preview. `reviewed-evidence.json` contains `id`, `provider_id`, `gateway`, `model`,
the exact `preview`, and `consent`. Neither format accepts raw credentials.
Commands use the same `--database` / `RECONX_DATA_DIR` selection as other services.
Saving profiles and testing an actual provider connection remain separate states.

Verification: `crates/reconx-core/tests/research_sessions.rs`, frontend
`Research.test.tsx`, native `tests/e2e/research-session.mjs` and cross-platform
`scripts/research_session_smoke.py`. Local mock AI is explicitly MOCK_VERIFIED;
it proves protocol/workflow handling, not a live cloud account or local-model engine.
Use `artifacts/product-recovery-current-verification.json` for the latest actual commands,
platforms and artifacts. Whole-product release readiness remains false.
