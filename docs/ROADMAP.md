# Development roadmap

1. Foundation: Windows native desktop/CLI/shared SQLite project/settings milestone verified.
   Desktop redesign: reference-inspired four-region shell, illustrated dashboard, consistent secondary pages, keyboard search, activity/notifications, resizable AI context and exactly Light/Dark/ReconX verified on Windows. Future module actions still require backend implementation.
2. AI integration: provider CRUD, secure credentials, model discovery, text/streaming/cancellation, defaults and safe connection fallback implemented. Remaining: persisted health/usage/cost, rate/budget accounting, context, advanced parameters, per-agent routing and extensions. Real cloud compatibility unverified.
3. Tool Manager: Windows ripgrep installation/version/uninstall verified. Remaining: dependency graph, unmanaged inventory, profiles, other local-analysis adapters, updates/rollback/cancellation and signatures.
4. MCP: centralized registry/discovery against an owned mock server; isolation and explicit tool policies before invocation.
5. Local-analysis jobs: typed plans, persistent states, cancellation/checkpoints and budgets.
6. Targets: inventory and deterministic scope/authorization. Autonomous public-target offensive execution is outside the implemented scope.
7. Analysis: offline HAR and owned binary inspection with structured observations/evidence.
8. Reporting: findings management and redacted Markdown/HTML/JSON, then PDF.
9. Packaging: portable Windows builds implemented; verify installers, Windows/Linux CI, signing and release audit.
10. Extensions: local mobile/IaC/container/CI configuration review.

CI and NSIS/Debian configuration exist but are unverified. Linux and ARM64 remain unverified. A stub or roadmap page never counts as an implemented capability.
