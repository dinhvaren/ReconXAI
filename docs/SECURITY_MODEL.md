# Security model

This build exposes local project/configuration management, explicitly initiated AI text requests, and a pinned local utility installer/version checker. Security modes cannot unlock absent active workflows. Model output never becomes a command.

Credentials use native OS storage. Configurations reject secret fields and gateway URLs containing authentication, query strings or fragments. HTTPS is required except for loopback HTTP. Secret references bind provider ID, protocol and gateway SHA-256 fingerprint. Gateway changes with a key require key removal first. Legacy UUID-only entries from the early foundation build are not reused or deleted automatically; those keys require reentry.

AI clients reject redirects, disable implicit proxies and preserve TLS verification. Prompts, responses, timeout and output tokens are bounded. Streams cap text at 256 KiB and progress updates at 256, with 25 ms throttling. Completion events are required; partial/cancelled output stays unverified. Credential suffixes are withheld across event boundaries and exact echoes are redacted.

Explicit fallback handles only definite pre-transmission connection errors. HTTP errors, ambiguous timeouts and partial responses are not replayed. Audit metadata excludes prompts, replies and credentials.

Only embedded manifests can drive tool installation. Official downloads use pinned URLs, restricted HTTPS GitHub redirects, and matching archive/executable SHA-256. Only the pinned regular entry is copied; archives are not unpacked wholesale. Managed binaries are rehashed before a fixed --version invocation with timeout/kill-on-drop. Directory containment and symbolic links are checked. Uninstall removes only named managed files and preserves unexpected files.

No arbitrary process/shell execution, untrusted plugin installation, target scanning or privileged package installation is exposed. A tool is not a process sandbox.

Tauri has no shell/filesystem/network plugins for the webview. CSP restricts content; AI text is not rendered as HTML. SQLite work runs outside the UI thread. Network operations are asynchronous.

SQLite uses ordinary account/filesystem protection, without encryption. Notes/metadata exports are user-authored and not automatically redacted. Rust secret buffers are zeroized where practical; browser inputs, libraries and OS services may copy them. Same-account malicious processes, filesystem races and comprehensive process sandboxing are outside the claimed protection.

Existing MCP/provider credentials are not imported. MCP isolation and scope workflows remain unimplemented. Tests isolate temporary data and loopback mock services; WebView2 debugging is set only in the test child environment. Test data is excluded from Git and release packages.
