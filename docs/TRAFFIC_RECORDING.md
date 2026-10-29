# Traffic recording status and privacy boundary

Existing HAR imports normalize offline request/response captures into SQLite and
preserve original evidence. Existing explicitly approved loopback replay stores
actual request/response history, job associations, comparisons and evidence.
The HTTP Workspace inspects those real records. These are implemented features,
not a new universal traffic recorder or a claim that every scanner request is
captured in full.

HTTPX metadata observations are not complete HTTP request/response captures.
Their graph nodes link actual job/inventory/evidence records; they do not invent
request headers, bodies, cookies or TLS verification. A graph URL is not proof
that a full traffic transaction was recorded.

This slice does not add a local interception proxy, per-project recording on/off
policy, raw sensitive capture permission, retention engine, external Burp/ZAP
capture adapter or new Traffic page. No root CA is generated/installed and no
unrelated system traffic is intercepted. These capabilities remain unsupported
until implemented and independently verified.

The safe next recording slice is explicit opt-in capture of ReconX-owned HTTP
operations through the existing Core pipeline. It needs persistent policy,
project/target/job/session associations, body/content-type budgets, redacted
previews/exports, protected raw storage only with informed approval, retention
semantics and restart tests. Captured evidence must never be sent to a model
without a previewed destination/context approval.

Refer to HTTP_WORKSPACE.md / HTTP_REPLAY.md where present and the actual source
and master ledger for current capture limits; the status of broader recording
must not be inferred from HAR import or graph navigation alone.
