# Security model

This build exposes local project/configuration/target management, authorized read-only binary analysis, explicitly initiated AI text requests, and a pinned local utility installer/version checker. Security modes cannot unlock absent active workflows. Model output never becomes a command.

Credentials use native OS storage. Configurations reject secret fields and gateway URLs containing authentication, query strings or fragments. HTTPS is required except for loopback HTTP. Secret references bind provider ID, protocol and gateway SHA-256 fingerprint. Gateway changes with a key require key removal first. Legacy UUID-only entries from the early foundation build are not reused or deleted automatically; those keys require reentry.

AI clients reject redirects, disable implicit proxies and preserve TLS verification. Prompts, responses, timeout and output tokens are bounded. Streams cap text at 256 KiB and progress updates at 256, with 25 ms throttling. Completion events are required; partial/cancelled output stays unverified. Credential suffixes are withheld across event boundaries and exact echoes are redacted.

Explicit fallback handles only definite pre-transmission connection errors. HTTP errors, ambiguous timeouts and partial responses are not replayed. Audit metadata excludes prompts, replies and credentials.

Only embedded manifests can drive tool installation. Official downloads use pinned URLs, restricted HTTPS GitHub redirects, and matching archive/executable SHA-256. Only the pinned regular entry is copied; archives are not unpacked wholesale. Managed binaries are rehashed before a fixed --version invocation with timeout/kill-on-drop. Directory containment and symbolic links are checked. Uninstall removes only named managed files and preserves unexpected files.

No arbitrary process/shell execution, untrusted plugin installation, target scanning or privileged package installation is exposed. A tool is not a process sandbox.

The webview has no broad shell/filesystem/network permissions. Native dialogs are
opened through a custom Rust command; file import/export uses bounded typed services.
CSP restricts content; AI text is not rendered as HTML. SQLite work runs outside the
UI thread. Network operations are asynchronous.

Target registration is not authorization to execute a tool. Scope exclusions win;
host/path component and URL-origin boundaries are checked. Local analysis requires
recorded permission, matching immutable import source identity and an active project.
Workers recheck policy/cancellation at stage boundaries. No mode executes samples,
shell strings, scanners or AI tool plans. A 64 MiB input limit, output cardinality
limits, ZIP directory checks and panic containment defend parser entry points.
These controls are not an OS process sandbox.

Managed sample/result integrity is checked with SHA-256. Raw strings and hex are
kept in private local artifact files, not ordinary database configuration. These
files are not encrypted; custom data-directory users must provide suitable ACLs.
Imports never execute samples or extract archive entries. Originals are preserved.
Deletion preserves cached blobs until a future explicit GC facility; cancellation
does not forcibly interrupt a parser already inside one bounded stage.

SQLite uses ordinary account/filesystem protection, without encryption. Notes/metadata exports are user-authored and not automatically redacted. Rust secret buffers are zeroized where practical; browser inputs, libraries and OS services may copy them. Same-account malicious processes, filesystem races and comprehensive process sandboxing are outside the claimed protection.

Existing MCP/provider credentials are not imported. MCP isolation and scope workflows remain unimplemented. Tests isolate temporary data and loopback mock services; WebView2 debugging is set only in the test child environment. Test data is excluded from Git and release packages.

The visual redesign preserves IPC contracts and backend policy. Search uses project/provider metadata and module routes, never credentials. API key entry remains masked; status actions return configured/unconfigured only. Screenshots are captured before filling the ephemeral fixture credential, which is removed by the test. Original artwork and licensed font assets are bundled; no reference-site artwork or runtime font/image tracking is included. Future module pages cannot invoke absent analysis services.

Appearance changes are typed and serialized; failures roll back locally and remain visible. Legacy system/missing preference migrates to reconx in a validated SQLite transaction changing only the theme field and adding an audit event. Three-theme migration does not read/write API keys, change gateway destinations, reset projects or introduce an execution channel. Native test databases and WebView2 profiles are isolated from normal application data.
