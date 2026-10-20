# AI authentication status — 2026-10-19

Existing API-key configuration and OS credential storage are preserved. Local
Ollama, LM Studio and compatible endpoint/model configuration remain available;
native protocol fixtures cover discovery, inference/streaming and failure paths.
A genuine local model server and live cloud accounts were not verified in this
UX/extension checkpoint. Saving a profile does not assert Connected.

Provider browser OAuth/delegated inference is **NOT IMPLEMENTED**. No approved
ReconX client registration is supplied or claimed. No fake provider login button,
session-cookie collection or reuse of another application's subscription tokens
is added. Official identity login and inference access must be audited separately
for each provider in milestone G.

Before enabling any supported flow, implement and test system browser PKCE,
state/issuer/scope/redirect validation, secure callback and token storage, expiry,
authorized refresh, disconnect/revocation and cancellation. Those items are
requirements, not functionality claimed to exist here.
