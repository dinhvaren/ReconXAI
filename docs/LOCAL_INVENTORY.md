# Manually approved loopback HTTP inventory

This is a bounded local availability adapter, not a reconnaissance scanner or
vulnerability detector. It reuses the target/scope service, existing job worker,
evidence store and reports. No external tools or additional scheduler are added.

## Workflow

Create/select a project and add an authorized URL, REST API or GraphQL target.
Both Settings and the target must select Authorized Auto or Lab Research. From
target details choose **Plan local HTTP inventory**, or open Reconnaissance and
select the target. Review the exact URL, HEAD method, single-request budget and
five-second deadline, then explicitly approve. Preview does not contact anything.
Desktop runs queued work through its existing background worker; CLI users run
the shared worker command below. Logs/progress reflect actual stages, not timers.

```powershell
reconx --json recon preview <target-id>
reconx --json recon run <target-id> --fingerprint <preview-fingerprint> --approve
reconx --json jobs run
reconx --json recon assets --project <project-id>
reconx --json jobs result <job-id>
reconx --json evidence list --project <project-id>
```

Run queues work; it does not imply completion. CLI help explains this distinction.
Changing scope, authorization, target input or application security mode invalidates
the exact approval. Application appearance changes do not alter authorization.
The generic job enqueue and AI planner cannot select this adapter.

## Boundaries and actual output

Only loopback IP literals and exact localhost are accepted. localhost is pinned to
127.0.0.1; use [::1] for an IPv6-only service. HTTP and HTTPS are supported with
normal TLS certificate validation. No URL credentials/query/fragment, authentication,
cookie jar, proxy, redirect follow-up, automatic retry, crawl or response-body
analysis. One HEAD per approved job; at most 20 approvals/project/minute and the
existing global maximum of two running jobs across clients. A cancellation or
scope revocation may occur after the single request has already been sent; it
stops waiting and prevents publication rather than undoing network traffic.

The HTTP parser limits header count to 64. A further aggregate 64 KiB header check
runs after parsing; this is not a preallocation memory cap. Client connection
and total deadlines are two and five seconds. Cancellation/policy checkpoints run
while waiting. Failed/cancelled jobs do not publish response evidence or assets.

Completion atomically stores a stable endpoint asset (upsert), last observation,
safe job result, private normalized HTTP response evidence and job-output evidence.
Headers with non-UTF8 values are represented losslessly as base64. Private raw
headers can contain cookies; default evidence/report exports redact credential
headers including their base64 representation. Evidence remains an immutable
snapshot for each successful job; asset metadata represents its latest success.
Stored server names are self-reported hints, not verified fingerprints. No HTTP
status, banner or missing header is automatically called a vulnerability.

Requests/responses are normalized metadata, not packet captures. The adapter does
not inspect bodies or perform authentication/authorization vulnerability tests.
Opening Evidence/Reports from this page preserves the selected project context.
Assets list shows at most the latest 1000 rows. Permissioned subprocesses and raw
HTTP evidence are not encrypted or OS-sandboxed by this feature.

## Persistence, recovery and limits

Migration 006 adds assets and local_http_approvals with indexes and foreign keys.
Existing schema versions 1-5 receive a consistent pre-v6 SQLite snapshot before
any migrations. Migration 005 output tables are retained. Recovery follows the
existing job leases; interrupted active work is not silently replayed. Approval
budget is consumed even if a job is cancelled or fails. No automatic retry.

DNS, RDAP/WHOIS, certificate transparency, historical URLs, crawling, Nmap/Nuclei
and public target adapters remain unimplemented. Authorized Auto is a saved mode,
not autonomous network execution. IPv6/TLS production combinations and Linux
runtime require separate verification. See TEST_RESULTS.md for actually executed
Windows fixtures; do not infer platform verification from shared Rust code.

## Verification commands

```powershell
cargo test --locked -p reconx-core --test local_inventory
cargo test --locked -p reconx --test cli
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify.ps1 -LocalInventory
```

The native harness owns its loopback fixture, application process, isolated data
and WebView2 profile. It records real request methods, preview/approval, success,
cancellation/failure, evidence/report redaction and restart parity. No public
security tests or cloud uploads are performed. Windows CI includes this scenario;
CI configuration is not itself proof of an executed CI result.
