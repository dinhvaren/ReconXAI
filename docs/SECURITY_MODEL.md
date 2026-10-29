# Security model

Current dependency/release assessment is in DEPENDENCY_SECURITY.md and
RELEASE_READINESS.md. Linux glib unsoundness is unresolved; successful functional
tests do not waive it. No process-sandbox or complete v1.0 claim is made.

This build exposes local project/target management, static binary analysis, managed
utilities/YARA-X, reviewed benign MCP calls, evidence/findings/reports, offline
HAR/OpenAPI observations, individually reviewed bounded loopback HTTP replay,
and separately approved fixed HTTPX/passive Subfinder jobs.
AI chat is distinct from the allowlisted local-file job planner. Security modes
cannot unlock absent active workflows; model output never becomes a shell command.

Credentials use native OS storage. Configurations reject secret fields and gateway URLs containing authentication, query strings or fragments. HTTPS is required except for loopback HTTP. Secret references bind provider ID, protocol and gateway SHA-256 fingerprint. Gateway changes with a key require key removal first. Legacy UUID-only entries from the early foundation build are not reused or deleted automatically; those keys require reentry.

AI clients reject redirects, disable implicit proxies and preserve TLS verification. Prompts, responses, timeout and output tokens are bounded. Streams cap text at 256 KiB and progress updates at 256, with 25 ms throttling. Completion events are required; partial/cancelled output stays unverified. Credential suffixes are withheld across event boundaries and exact echoes are redacted.

Explicit fallback handles only definite pre-transmission connection errors. HTTP errors, ambiguous timeouts and partial responses are not replayed. Audit metadata excludes prompts, replies and credentials.

Selected local artifacts have a separate consent boundary: the backend compares
the approved preview and gateway against current data, then uses the captured
provider configuration with no fallback. No binary, source path or note is uploaded
automatically. Optional sampled strings may contain sample secrets; their preview
and explicit transmission approval are required. AI interpretation remains unverified.

Local jobs use a closed typed handler list, not shell strings or LLM-provided code.
SQLite claims cap cross-client concurrency at two. Cancellation retains capacity
until acknowledged; timeouts are cooperative and result commit is fenced by worker
ownership/current state. Expired leases are reconciled without blindly retrying
interrupted work. This is not hard process isolation or a general tool executor.

Only embedded manifests can drive tool installation. Official downloads use pinned URLs, restricted HTTPS GitHub redirects, and matching archive/executable SHA-256. Only the pinned regular entry is copied; archives are not unpacked wholesale. Managed binaries are rehashed before a fixed --version invocation with timeout/kill-on-drop. Directory containment and symbolic links are checked. Uninstall removes only named managed files and preserves unexpected files.

No arbitrary process/shell execution, untrusted plugin installation, unrestricted
scanning or privileged package installation is exposed. HTTPX sends approved
bounded HTTP metadata probes; Subfinder uses selected passive sources. A tool is
not a process sandbox. Version and execution commands have private temporary
cwd/home configuration, with ambient credentials/proxy variables removed.

The webview has no broad shell/filesystem/network permissions. Native dialogs are
opened through a custom Rust command; file import/export uses bounded typed services.
CSP restricts content; AI text is not rendered as HTML. SQLite work runs outside the
UI thread. Network operations are asynchronous.

Target registration is not authorization to execute a tool. Scope exclusions win;
host/path component and URL-origin boundaries are checked. Local analysis requires
recorded permission, matching immutable import source identity and an active project.
Workers recheck policy/cancellation at stage boundaries. No mode executes samples
or unrestricted shell strings/scanners. Reviewed AI plans can queue only typed
local-file jobs in the older file planner. Target-bound research sessions can
additionally queue separately reviewed HTTPX/passive Subfinder capabilities.
Their exact target/scope/tool/configuration fingerprint is checked before work
and publication; the model cannot override it. HTTPX DNS destinations are pinned
to numeric transport IPs, redirects and protocol fallback are disabled, and
private/special DNS results are rejected. Explicit numeric lab targets have a
separate allowed path. Passive-source disclosure requires approval; empty/error
results do not establish source success or absence of subdomains. HTTPX TLS fields
are not a verified certificate trust verdict. See RECON_TOOL_ADAPTERS.md for the
per-job limits, source constraints and remaining runtime gaps.

Streaming ingestion uses a separate configurable disk budget;
the deep parser retains a 64 MiB input limit. Bounded prefixes, output cardinality
limits, ZIP directory checks and panic containment defend parser entry points.
These controls are not an OS process sandbox. Import cancellation removes incomplete
staging during normal unwinding; forced process termination can leave private
staging files. Automatic crash-staging cleanup and workspace disk quotas remain
unimplemented. Complete published blobs are conservatively retained on later errors.

Managed sample/result integrity is checked with SHA-256. Raw strings and hex are
kept in private local artifact files, not ordinary database configuration. These
files are not encrypted; custom data-directory users must provide suitable ACLs.
Imports never execute samples or extract archive entries. Originals are preserved.
Deletion preserves cached blobs until a future explicit GC facility; cancellation
does not forcibly interrupt a parser already inside one bounded stage.

SQLite uses ordinary account/filesystem protection, without encryption. Notes/metadata exports are user-authored and not automatically redacted. Rust secret buffers are zeroized where practical; browser inputs, libraries and OS services may copy them. Same-account malicious processes, filesystem races and comprehensive process sandboxing are outside the claimed protection.

Existing MCP/provider credentials are not imported. MCP children have no OS sandbox;
manually approved loopback HTTP inventory/replay are bounded by their documented
policy. General active network workflows remain unimplemented. Tests isolate
temporary data and local fixtures; WebView2 debugging is set only in the test child
environment. Generated test data is excluded from Git and release packages.

The visual redesign preserves IPC contracts and backend policy. Search uses project/provider metadata and module routes, never credentials. API key entry remains masked; status actions return configured/unconfigured only. Screenshots are captured before filling the ephemeral fixture credential, which is removed by the test. Original artwork and licensed font assets are bundled; no reference-site artwork or runtime font/image tracking is included. Future module pages cannot invoke absent analysis services.

Appearance changes are typed and serialized; failures roll back locally and remain visible. Legacy system/missing preference migrates to reconx in a validated SQLite transaction changing only the theme field and adding an audit event. Three-theme migration does not read/write API keys, change gateway destinations, reset projects or introduce an execution channel. Native test databases and WebView2 profiles are isolated from normal application data.


## Milestone 2 trust boundaries

MCP server programs are explicitly trusted user-account processes, not OS-sandboxed
plugins. A cleared environment and disjoint OS credential references prevent
implicit provider-key forwarding; they do not deny account-level filesystem or
credential API access. Only reviewed read-only closed-world tools can enter the
manual invocation path, with schema validation, exact approval, bounded output,
timeout and no automatic retry. Remote redirects are disabled. Referenced schemas
and persisted credential input fields are rejected. MCP discovery/results are
recursively redacted for known server secrets, and SDK wire diagnostics are filtered
out even when RUST_LOG enables detailed targets. MCP output never becomes authority.

AI plans can select only available fixed offline file capability IDs for the
user-selected authorized file. They cannot choose shell arguments, change the input
path, grant scope or autonomously invoke MCP/network scanners. Metadata planning
and evidence interpretation have separate exact-preview/gateway consent, no silent
fallback, and separately labelled unverified AI output. See MCP_HUB.md and
AI_TOOL_ORCHESTRATION.md for implemented boundaries and unsupported capabilities.

## Evidence, manual verification and report redaction (2026-10-17)

Evidence hashes prove retained bytes match a record; they do not prove a security
conclusion. AI summaries cannot become reviewed source evidence. Verified findings
need explicit independent review, matching-project evidence with a checked hash
and a review note. Reports preserve statuses and warnings instead of asserting
vulnerabilities from HTTP status, banners, signatures or model confidence.

Artifacts use generated UUID filenames, reject final symlink/reparse inputs/roots
and have bounded reads. Unix artifacts use 0600/0700; Windows inherits workspace
ACLs. This is not encryption, an untrusted same-account process boundary, or a
claim of race-free descriptor-relative filesystem isolation. Custom shared
workspaces must have their permissions reviewed. Export never overwrites files.

Raw evidence stays private; raw binary/image exports require explicit approval.
Text/JSON reports redact known credentials, header name/value pairs, private keys
and local paths. Unknown patterns may remain; preview is mandatory and redactions
are visible. A native fixture found an embedded-JSON header leak in report text;
structured redaction now precedes embedding. Detectable legacy unsafe snapshots
are preserved but export is blocked until a new snapshot is prepared/reviewed.

## Single-request loopback inventory

Only exact localhost (pinned to 127.0.0.1) or loopback IPs, matching target scope,
active project, declared authorization and permitted application/target modes are
accepted. Exact manual approval is bound to policy/input; revalidation occurs
while awaiting headers and before result commit. One HEAD, no proxy/redirect,
retry/auth/cookies/body analysis; fixed deadlines/header budgets and a persisted
20 approvals/project/minute ceiling. Cancellation cannot undo an already-sent
request. A response may contain secrets, so raw headers are private evidence;
normalized assets/job summaries omit cookie/auth fields. These are availability
observations only. AI cannot choose this adapter or enable public scanning.

## Reviewed HTTP replay and offline schema observations

The separate replay adapter supports one exact approved request to an owned
loopback service, with origin/project/target-bound OS-store identities, TLS
verification, bounded UTF-8 bodies, cancellation and sanitized captures. It rejects
general DNS destinations, proxies, followed redirects, automatic retries and cookie
jars. Fingerprints bind current input/identity/policy revisions; dispatch budgets
are enforced in Rust. A native test that sent too quickly was correctly rejected;
the runner now waits for the budget instead of changing enforcement. See
HTTP_REPLAY.md for exact limits. HTTP success proves neither authentication nor
authorization, and cancellation cannot undo an action already received by a server.

Offline schema constraints/composition share a global work/recursion/output budget.
Alternative branches use match/mismatch/unknown states; redacted values and
unsupported checks never become successful validation by omission. A closed scalar
keyword subset uses the existing JSON Schema library, without reference/URI/regex
resolvers. Original hashes and source pointers remain attached; observations are
not automatically verified findings. Pattern/format/multi-file coverage is incomplete.

Database preflight reads existing schema metadata through a read-only connection
before WAL, and rechecks under the write transaction. Future/corrupt fixture files
remain byte-identical when rejected. This does not provide cross-version concurrent
upgrade coordination or a full backup of artifact files and OS credentials. Stop
older clients before upgrading and protect backups as private workspace data.


Research provenance reads are materialized from actual stored records only.
Graph review never invokes a tool/model. IDs bind session and persistent source
identity, edges enforce same-session foreign keys, pages/exports use consistent
read transactions and strict size/count limits. URL credentials/query/fragment and
local import locations are omitted from graph metadata; linked previews are
redacted/bounded. A graph edge is not artifact integrity or vulnerability proof.
See EXPLORATION_GRAPH.md; removed-source/large-workspace limitations remain open.
