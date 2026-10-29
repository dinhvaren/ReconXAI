# Managed reconnaissance adapters

ReconX AI 0.1.0 has two compiled ProjectDiscovery adapters. This is a bounded
reconnaissance slice, not an automated penetration test or a v1.0 release claim.
Shared Core services drive Desktop, CLI, Job Engine, asset inventory and evidence.

## Reviewed packages

| Tool | Pinned release | Platforms | Health requirement |
|---|---|---|---|
| HTTPX | 1.12.0 | Windows/Linux x86_64 | Exact executable hash, `-version`, actual owned-loopback GET with expected JSON/title/status |
| Subfinder | 2.17.0 | Windows/Linux x86_64 | Exact executable hash, `-version`, offline source inventory containing the reviewed source |

The manifests in `manifests/tools/` pin official release archives and their
published SHA-256 checksums. Executable hashes were computed from the verified
archives. No downloaded executable is run before its hash is checked. PATH
discovery accepts only an exact pinned binary, avoiding accidental use of the
unrelated Python `httpx` package. Installing an extension can adopt that binary
without downloading it again; it still copies into managed storage and runs health.
Ready requires a successful capability health check, not just an archive download.
Repair, disable, health checks and removal reuse the shared lifecycle. Removing
or repairing never discards historical evidence. Verified PATH adoption can
atomically replace a receipt-owned corrupted managed binary, then repeats health;
an unexpected receipt-less file or symlink is rejected and preserved. Removing
an extension retains shared tools and evidence. Updating is limited to a reviewed
pinned catalog change; automatic adoption of arbitrary upstream versions and
general multi-version rollback are not implemented.

Upstream provenance and argument documentation:

- [HTTPX official releases](https://github.com/projectdiscovery/httpx/releases/tag/v1.12.0)
- [HTTPX usage](https://docs.projectdiscovery.io/tools/httpx/usage)
- [Subfinder official releases](https://github.com/projectdiscovery/subfinder/releases/tag/v2.17.0)
- [Subfinder usage](https://docs.projectdiscovery.io/tools/subfinder/usage)

## Execution policy

`recon_plan` performs an offline review. Running requires explicit approval bound
to the exact target, scope, application/target mode, configuration, pinned binary
and extension revision. Worker checkpoints and result publication recheck policy.
Changes invalidate approval; the model cannot supply shell strings or arbitrary
flags. Each process has a private temporary configuration, cleared environment,
bounded output, wall timeout and cancellation/kill/wait handling. These controls
are **not an OS sandbox**: trusted external binaries retain the user's OS identity.

HTTPX probes only registered HTTP(S) URL/API targets, or an explicitly bounded
selection of in-scope passive subdomains linked to the same target. The default
budget is four URLs, maximum eight, one child at a time, one URL/second, 1–15 second
probe timeout, no redirects, no protocol fallback, no retry, no proxy, no supplied
credentials, 64 KiB response read and 1 MiB process output. A domain's discovered
hosts use HTTPS only; the adapter does not guess ports or fall back to HTTP.

DNS is resolved at execution with a timeout. DNS names returning any private or
special address are rejected. An explicitly registered numeric-IP lab URL may
use private/loopback addresses, excluding link-local/metadata, unspecified,
multicast and mapped-IPv6 destinations. `localhost` is pinned to 127.0.0.1.
The process receives a numeric transport URL plus the original Host/SNI, so it
cannot re-resolve the original name or follow a redirect to escape scope.
TLS fields are HTTPX observations: **certificate trust is not a verified verdict**.
TLS metadata can involve extra handshakes; a URL budget is not an exact connection
count. Existing strict-TLS manual replay and loopback HEAD retain their own policy.

Subfinder is passive only, with reviewed keyless `crtsh`, `hackertarget` and
`certspotter` sources. Default source: crtsh; maximum three selected sources,
2 requests/second, two-minute enumeration budget, 1 MiB process output and
1,000 normalized hosts. Parent names must be valid DNS domains. Results are
deduplicated, source-attributed where reported, checked against parent suffix
and filtered against target scope before storage. Discovered names are never
automatically resolved or probed without the separate HTTPX approval.
Source errors and empty output are inconclusive; they do not prove the absence
of assets. Individual source success is not independently confirmed. This adapter
does not yet configure API-key-dependent source credentials.

## One research session

Install/verify the tools in Extensions, select an authorized target and choose
**Start AI Research**. The existing selected model receives the reviewed planning
context only after consent. Available capabilities include `subfinder.passive`
and `httpx.probe`; unavailable tools remain unavailable. A fresh domain plan must
put discovery before probing, or use existing target-linked discovered hosts.
Separate execution consent lists passive-source disclosure and active budgets.
The Job Engine persists a discovery → probe dependency. Failed/cancelled/interrupted
discovery prevents its dependent probe; it cannot reuse stale discovery from an
older run. This is a bounded dependency, not general DAG or pause/resume support.
Authorized Auto does not silently acquire standing network permission.

Raw stdout/stderr and the plan are preserved as confidential ToolOutput evidence;
normalized results use separate StructuredJson evidence, each with SHA-256.
Assets retain project/target/job/source/evidence references. Empty passive output
is labelled `indeterminate_empty`; HTTP availability is not a vulnerability.
AI interpretation requires separate evidence/model consent. Findings remain
reviewable; report exports use the existing redaction/review lifecycle.

Failures retain job state/logs. A failed process currently does not retain a raw
failure-output artifact, and a later failure prevents publication of partial
probe batches. These limitations must not be presented as successful empty scans.

## CLI

Desktop and CLI use the same Core/database. Example commands (use an authorized
target ID; a run queues a job, while Desktop workers or `jobs run` execute it):

```text
reconx extensions plan httpx
reconx extensions install httpx --fingerprint <reviewed-fingerprint> --approve
reconx extensions plan subfinder
reconx extensions install subfinder --fingerprint <reviewed-fingerprint> --approve
reconx tools verify httpx
reconx tools verify subfinder
reconx recon plan <target-id> --tool httpx --max-assets 4 --timeout-secs 10
reconx recon run --plan reviewed-httpx-plan.json --approve
reconx jobs run
reconx recon assets --project <project-id>
reconx recon history --target <target-id>
reconx recon export --project <project-id> --output inventory.json
```

Use `--json` for structured command output. To preserve edited configuration,
save the JSON plan and pass `reconx recon run --plan <file> --approve`. The legacy
`recon run <target-id> --fingerprint ...` form still queues the separately
reviewed loopback HEAD workflow, not an HTTPX job.
CLI `--tool subfinder` plans passive discovery. The JSON plan exposes selected
sources; editing any reviewed field requires a fresh matching Core fingerprint.

## Verification boundaries

Reproducible owned-loopback acceptance:

```text
python scripts/recon_tools_smoke.py --binary <reconx-path> --output <new-private-directory> --tools <verified-tool-directory>
```

Alternatively `--approve-downloads` downloads the reviewed official packages.
Tests use actual installed HTTPX, a local HTTP fixture, redirect sentinel,
delayed response, cancellation, MOCK AI planning and real evidence/report paths.
Subfinder parser tests use explicitly labelled synthetic/recorded JSONL; source
inventory and package health do not count as live passive discovery. An owned
external domain/source authorization is required before that live test. No public
active scan or live cloud AI is implied by these tests. See the current execution
ledger and machine-readable proof for actual commands, platform results and gaps.

## Persistence and source

Migration 018 adds reviewed plans and job dependency references without replacing
HAR/OpenAPI or session data. Migration tests preserve schema017 selection/session
rows, validate a pre-migration backup and check foreign keys.

- `crates/reconx-tools/src/recon.rs`: process/configuration, health, adoption.
- `crates/reconx-core/src/recon_tools.rs`: policy, plans, parsers, assets/evidence.
- `crates/reconx-core/src/jobs.rs`: handlers, cancellation and dependency admission.
- `crates/reconx-core/src/research_sessions.rs`: model planning and exact approval.
- `apps/desktop/src/components/ReconToolControls.tsx`: reviewed manual execution.
- `apps/desktop/src/components/ReconAssetInventory.tsx`: shared actual observations.
- `tests/e2e/recon-tools.mjs`: native Marketplace/research acceptance.

DNS record collection, RDAP, CT/history adapters independent of Subfinder,
Katana/Nuclei and full recon DAG remain later work. Linux GUI, production signing,
clean installers and the existing dependency-security release gate remain separate.
