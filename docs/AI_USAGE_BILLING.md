# AI Usage and Billing status — 2026-10-19

NOT IMPLEMENTED as an account dashboard. Existing native protocol adapters decode
per-reply input/output token metadata, including supported streaming events. That
is not provider account usage, actual billed cost or remaining balance.

No official billing/balance integrations, durable per-request accounting ledger,
pricing version registry, custom pricing or estimated-cost dashboard are added
by the UX/extension checkpoint. Do not display missing balances as zero or mark
any provider billing API verified from the local inference fixtures.

Milestone F must first audit official documentation for the actual provider
registry, including management-credential permissions, then implement typed Rust
capability profiles and separately labelled official/local records. Preserve
existing credential storage and avoid recording raw prompts for accounting.
