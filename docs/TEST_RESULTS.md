# Verification record

Windows x64, Rust 1.95.0 MSVC, Node 24.15.0, npm 11.7.0. Updated 2026-10-13.

## Latest full verification

Command: scripts/verify.ps1 -Native -ToolInstall

- Rust format and Clippy with warnings denied: passed.
- Rust workspace tests: 28 passed, 0 failed, 1 ignored by default.
- Explicit Windows OS credential round trip: 1 passed; ephemeral test entry removed.
- Frontend TypeScript/Vite build: passed.
- Frontend tests: 3 passed, 0 failed.
- Embedded custom-protocol debug desktop/CLI: built and ran.
- Native WebView2 scenario: passed with six checkpoints.

Debug record: artifacts/native-1791894959180/results.json; screenshots are beside it.

Checkpoints: project sharing in both directions; model discovery/non-streaming/streaming chat against a local mock gateway; native cancellation and cancelled audit record; persistence after full native restart; real pinned ripgrep install, archive/executable integrity, fixed version health checks, shared CLI inventory and uninstall.

Rust coverage includes migrations, CLI parsing/errors, concurrent database writers, project persistence/archive/import/export, gateway policy, credential isolation/ID reuse, provider payloads/model lists/redaction, stream framing/completion/cancellation, safe fallback/prevention of HTTP replay, manifests/integrity checks and preservation of unexpected uninstall files.

## Release verification

scripts/build.ps1 generated optimized Windows desktop/CLI binaries and SHA256SUMS.txt in artifacts/windows-x64. All six native checkpoints also passed using those release binaries: artifacts/native-1791895084172/results.json. Installer installation is not inferred from a binary build.

## Resolved failures

Missing icon, premature tempdir deletion, native exact-select matching and Clippy conditions were fixed. SHA-256 digest bytes are explicitly hex-encoded for the installed API. The fallback fixture now allows 10 seconds for Windows connection refusal; ambiguous production timeouts still cannot trigger replay.

Local mock output is deterministic protocol test data, not live cloud responses or vulnerability evidence. Cloud accounts, Linux/ARM64, installer generation/installation, CI jobs and future modules remain unverified. No public infrastructure was actively tested.
