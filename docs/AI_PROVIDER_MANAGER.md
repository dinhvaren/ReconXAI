# AI Provider & Model Manager

Settings → **AI Providers** opens the configuration section. Choose **Add AI
Provider**, a provider, a model preset, paste a key and save. Keys are optional
for keyless local servers. Save is a local operation; it does not test inference.
**Test connection** explicitly sends a small inference request to the selected
model. A saved configuration remains **Saved · Not tested** until that succeeds.
Connection badges describe the last test, not continuous provider availability.

The model selector in AI Workspace lists enabled profiles. Each profile has its
own model, destination, secret reference and advanced options. Switching it changes
subsequent requests immediately. Configure multiple models from one vendor by
adding profiles; profile names can be edited in Advanced Settings.

## Catalog and discovery

`crates/reconx-ai/src/provider-catalog.json` is the source of truth, including
official documentation URLs and review dates. Native Responses, Messages and
Gemini adapters are separate; compatible vendors use their documented interfaces.
The registry includes all requested cloud/inference providers and keyless local
Ollama, LM Studio, vLLM, compatible local and custom gateway profiles.

Built-in presets are available before key entry. **Refresh Models** requests live
IDs only after an explicit action. Anthropic/Gemini pagination is bounded to 16
pages and 4096 models; invalid IDs, repeated cursors and oversized responses are
rejected. Refreshing an unchanged saved profile caches IDs without replacing the
selected model. A live model listing does not establish model inference access.
Unknown capabilities/context/deprecation are null, never invented. An advanced
custom ID remains available for account-specific or newly released models.

Alibaba's current Singapore Model Studio API requires a workspace ID; the dialog
builds its documented URL automatically. Other regional/plan endpoints can use
Advanced Settings or a custom gateway. Coding-plan Z.ai credentials can require
a different destination from the general international Z.ai endpoint.

## Credential and destination boundaries

API keys and custom header values use the existing OS SecretStore: Windows
Credential Manager, Linux Secret Service when available. No plaintext fallback.
SQLite stores public configuration, header names, health and model IDs only.
Stored secrets are never returned to the UI. Show/hide affects entered key text
only. Saving a replacement key or changing configuration invalidates the test
state. Concurrent key changes cannot publish an obsolete successful test.

Destination/protocol changes with stored credentials are rejected. Remove the key
and custom headers first, then explicitly configure the new destination. Custom
header JSON is saved in secure storage; `{}` removes headers. Host/framing/cookie
and proxy-authorization overrides are prohibited. HTTPS or explicit loopback HTTP
is required; credentials/query strings/fragments in gateway/proxy URLs are rejected.
Redirects and environment proxies are disabled. The optional proxy is explicit.
Discovery overrides are relative paths, not alternate credential destinations.
Selected-artifact AI review currently rejects a configured proxy with
`ARTIFACT_PROXY_APPROVAL_REQUIRED`: the existing consent binds the provider
destination, not an additional intermediary. Ordinary user-entered chat can use
an explicitly configured proxy. Proxy-aware artifact consent remains future work.

Gateway error bodies are omitted. Auth/model/access/quota/rate-limit/timeouts have
safe error codes. Output filtering includes entered keys and custom header values,
including partial streaming frames. A subprocess/MCP server does not receive the
AI credential interface. The app still makes no OS sandbox claim for MCP programs.

## CLI

```text
reconx ai catalog
reconx ai catalog anthropic
reconx ai providers list
reconx ai providers add --vendor anthropic --name "Reasoning" --default
reconx ai profiles create --vendor lmstudio --model <installed-model> --default
reconx ai profiles list
reconx ai profiles set-default <profile-id>
reconx ai providers test <profile-id>
reconx ai providers remove <profile-id>
reconx ai models list
reconx ai models refresh <profile-id>
reconx ai models select <profile-id>
reconx ai routing show
reconx ai routing configure <secret-free-routes.json>
reconx ai routing resolve orchestrator
reconx ai gateway <profile-id> <secret-free-options.json>
reconx ai headers <profile-id> --stdin
```

Cloud creation prompts for a hidden key, or accepts `--key-stdin` for a controlled
pipeline. Never put keys in command arguments. Existing `ai providers`, `configure`,
`key`, `key-status`, `models <profile-id>`, `test` and `chat` syntax remains available.
All implemented commands accept global `--json` and share Desktop storage.

## Routing and remaining limits

Agent overrides resolve to saved profiles; no overrides means use default for all.
The existing orchestrator and reverse review UI initialize from those routes.
Evidence consent still binds the exact provider/destination/preview and never
silently follows ordinary chat fallback. Existing chat fallback remains limited
to explicitly configured pre-transmission connection failure; HTTP errors/timeouts
are not automatically replayed. No model-generated command execution is added.

Live cloud account access is unverified until separately opted in. Local protocol
fixtures do not establish that model weights or inference engines are installed.
Discovery metadata is conservative; per-account capability enforcement, pricing
updates, sophisticated multi-hop fallback and full multi-domain agents remain
separate work. See TEST_RESULTS.md for actual platform/runtime acceptance evidence.

## Verification scope

The native acceptance script is `tests/e2e/ai-providers.mjs` (also
`npm run test:ai-providers`). Run it against matching Desktop and CLI binaries
using `RECONX_DESKTOP_EXE` and `RECONX_CLI_EXE`; it creates its own workspace,
uses synthetic secrets in Windows Credential Manager, calls loopback mock
inference servers, restarts the actual WebView2 application and removes its
fixture profiles/credentials. It never uses a personal provider account.
`scripts/verify.ps1 -AiProviders` includes the same suite after build/checks.

The tests establish configuration, persistence, routing and wire-protocol
behavior. They do not establish billing access, current account quotas or
availability of every catalog model. Local model discovery requires a running
compatible server and does not download model weights.

## Source map

| Responsibility | Source |
| --- | --- |
| Typed catalog and official presets | `crates/reconx-ai/src/catalog.rs`, `provider-catalog.json` |
| Protocols, discovery, secret filtering and gateway options | `crates/reconx-ai/src/lib.rs`, `streaming.rs`, `options.rs` |
| Shared profiles, health, secure persistence and routes | `crates/reconx-core/src/ai_manager.rs`, `lib.rs` |
| Additive schema and migration recovery | `crates/reconx-storage/migrations/009_ai_provider_manager.sql`, `src/lib.rs` |
| Native command bridge | `apps/desktop/src-tauri/src/lib.rs` |
| Setup, models, cards and routing UI | `apps/desktop/src/components/AiProviderManager.tsx`, `ai.ts`, `ai.css` |
| Settings and active chat model | `apps/desktop/src/pages/WorkspacePages.tsx`, `src/main.tsx` |
| Existing agent entry points | `apps/desktop/src/components/FileResearchPanel.tsx`, `pages/OrchestrationPage.tsx` |
| CLI provider/profile/model/routing commands | `apps/cli/src/main.rs` |
| Regression coverage | `crates/reconx-core/tests/ai_manager.rs`, `crates/reconx-ai/tests/gateway.rs`, `apps/cli/tests/cli.rs`, `apps/desktop/src/AiProviderManager.test.tsx`, `tests/e2e/ai-providers.mjs` |

Migration 009 deliberately does not reuse 007/008 from interrupted development
snapshots. Upgrade tests retain unknown tables and preserve pre-upgrade copies.
This milestone does not restore the removed Web/API source; the next import
milestone must build on the current schema and existing evidence services.
