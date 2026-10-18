# Manual loopback HTTP workspace

This adapter sends one individually reviewed request to an owned loopback service. It is not an autonomous authorization tester or a scanner for arbitrary destinations. Imported HAR entries never run automatically. Existing offline imports, schema observations, comparisons, evidence and report services are reused.

## Desktop

1. Register an authorized URL, REST API or GraphQL loopback target. Enable an active mode on both target and application. POST/PUT/PATCH/DELETE require Lab Research on both.
2. In Web/API Security, select the project and target. Import a capture, select it and choose **Edit request / replay to loopback**. Captured credentials and query values are discarded; withheld bodies are not copied.
3. Use **Authentication profiles** to save an optional project/target/origin-bound identity. Anonymous, Bearer, Cookie, Basic, API Key header and Custom Header are supported. Saved is not authenticated. Existing credential values cannot be displayed or retrieved through the UI.
4. Edit method, URL, public headers, content type, UTF-8 body, timeout and identity. **Save request** persists a draft without sending anything.
5. **Review send** displays the exact destination, pinned socket, identity and budget. Check the approval box and choose **Send approved request**. Any edit requires a new review; revision/policy changes invalidate queued approvals.
6. Execution history shows actual Job Engine states, results and logs. **Cancel request** aborts client work; it cannot undo an action the service already received. **Open captured response** uses the existing inspector/diff/schema views; **Save result evidence** uses the existing evidence service.
7. Saved requests, captures, jobs and identity metadata survive restart and are shared with the CLI. A successful HTTP response never proves authentication or authorization.

## CLI

Use `--help` for full arguments. `--json` is a global output option; successful mutation commands without a returned record may have empty output.

```text
reconx http requests --project PROJECT [--target TARGET]
reconx http seed CAPTURE [--target TARGET]
reconx http save request.json [--id DRAFT --revision REVISION]
reconx http inspect DRAFT
reconx http preview DRAFT
reconx http replay DRAFT --fingerprint REVIEW_HASH --approve
reconx jobs run
reconx http history DRAFT
reconx http compare CAPTURE_A CAPTURE_B
reconx auth profiles --project PROJECT [--target TARGET]
reconx auth add --project PROJECT --target TARGET --name Alice --role user --kind bearer
reconx auth update ID metadata.json --revision REVISION [--replace-key]
reconx auth expire ID --revision REVISION
reconx auth remove ID --revision REVISION
reconx auth cleanup
```

`auth add` / replacement prompt securely. `--key-stdin` supports controlled input without command-line secrets. Never put credentials in draft files or shell arguments. Basic credentials use `username:password`; Cookie profiles use explicit `name=value` pairs. Secret values must satisfy the validation limits reported by the service.

Draft input contains only: `project_id`, `target_id`, `source_exchange_id`, `title`, `method`, `url`, `headers` (name/value array), `content_type`, `body`, `auth_profile_id`, `timeout_ms`, `follow_redirects:false`, `proxy:null`. `id`, `revision`, `updated_at` are returned metadata, not input fields. Pass the previous revision separately when updating. `http preview` returns the hash required for that exact approval. The Desktop worker processes the same queue; `jobs run` drains it in CLI-only use and takes no job ID.

## Backend limits and security

- Destination: loopback IPv4/IPv6 only; exact `localhost` is pinned to `127.0.0.1` without DNS. Other DNS names, mapped IPv6, embedded credentials and URL fragments are rejected. Proxies, redirect following, automatic retries and cookie jars are disabled. Redirect responses are captured, never followed. System proxy settings are ignored.
- Project and target association, authorization, exact URL scope/exclusions, active mode and identity origin are checked in Rust. Approved fingerprints bind draft, identity revision, target settings and application mode. Existing jobs retain cancellation, lease recovery and bounded concurrency.
- One request/approval, at most 20 approvals/project/minute, minimum 500 ms between project dispatches. Too-fast dispatch fails visibly; it does not silently retry. Timeout 500–10000 ms, connect timeout 2000 ms.
- Request body 64 KiB, at most 32 public headers, 2048 bytes/header; response body 256 KiB including chunked responses; at most 64 response headers/64 KiB. TLS certificate verification remains enabled. JSON, form-encoded and raw UTF-8 editing are supported; multipart and binary uploads are unavailable.
- Public header allowlist: Accept, Cache-Control, If-None-Match, If-Modified-Since, X-Request-ID. Authentication headers/cookies come from the secure profile, not draft metadata. Known credential patterns, redacted placeholders and the selected identity's exact credential components are rejected in public draft fields.
- Windows credentials use the existing OS store; tests use synthetic data only. SQLite contains namespace-scoped opaque references. Credential replacement uses immutable secret references and optimistic revisions. Retired keys are journaled for cleanup; a failed cleanup is reported and `auth cleanup` retries it. This is not an atomic transaction spanning SQLite and the OS credential service.
- Credential namespace is bound to the canonical database path. Moving/copying a database to another path fails closed for these identities; reconfigure their keys there. The reference is not an authentication credential.
- Removing a target detaches its request/identity rows and preserves captures/jobs. Its retained identities become `target_removed`/expired, with Edit/Test disabled; they can still be removed to delete their OS credential. Historical drafts cannot be replayed without their original registered target. No target is silently recreated.
- Returned known credential values and common encodings are removed before the application capture is persisted, including JSON keys and echoed values. Existing export redaction also applies. This is pattern/exact-value redaction, not a universal data-loss prevention guarantee.
- Stored captures are sanitized application representations, not exact wire transcripts. Original received body SHA-256, byte size and total elapsed time are retained. Non-UTF-8 bodies are omitted; the existing HAR parser may withhold non-JSON body previews. No detailed transport phase timing is asserted.
- A rollback guard removes only a newly created capture artifact if its database transaction fails. A hard process termination can leave an unreferenced sanitized artifact. Interrupted jobs recover to a terminal state and are never automatically replayed.

## Persistence and verification

Migration `012_manual_http_replay.sql` adds identity metadata, retired secret journal, request drafts, replay job bindings and dispatch pacing. It reuses existing projects, targets, captures and jobs. Existing versioned migration handling makes a consistent pre-v12 backup before upgrading an older workspace. Stop older clients before an upgrade. Acceptance uses isolated temporary databases, not normal user data.

Core tests execute real local HTTP fixtures for send/identity switching, credential echo redaction, approval invalidation, project/origin isolation, redirect rejection, oversized/chunked responses, timeout, cancellation and target-deletion preservation. The native suite is `node tests/e2e/http-replay.mjs` (or `scripts/verify.ps1 -HttpReplay`), using explicit `RECONX_DESKTOP_EXE` / `RECONX_CLI_EXE` paths when testing release snapshots. `node tests/e2e/https-replay.mjs` is a separate negative TLS test: it requires an existing OpenSSL executable (`RECONX_TEST_OPENSSL` overrides its path), generates a one-day local certificate, verifies rejection without changing OS trust, and deletes the ephemeral test key/certificate. See `TEST_RESULTS.md` and the master ledger for current results; compilation alone is not runtime verification.

## Incomplete scope

Active BOLA/BFLA automation, stateful request sequences, automatic role/ownership decisions, general remote replay, proxy/redirect configuration, multipart, OAuth/refresh and autonomous AI HTTP execution are not implemented. Existing captured-role comparisons remain observations requiring review. Live cloud tests, Linux Desktop/Secret Service runtime and installer acceptance are separate unverified work.
