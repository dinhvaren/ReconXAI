# Decisions

1. Implement functional shared services before expanding module count.
2. Use SQLite WAL with immediate transactional migrations. Reject newer schema versions.
3. Archive projects instead of removing research data.
4. Store only secret references in ordinary configuration. Use Windows Credential Manager or Linux Secret Service; never fall back to plaintext.
5. Use provider-aware APIs: OpenAI Responses; Anthropic Messages; Gemini generateContent; Ollama chat; compatible chat completions for declared compatible gateways.
6. Disable HTTP redirects and implicit proxies. Bound prompts, responses, timeout and output tokens.
7. Never expose shell/tool execution to AI text. Persisted security mode cannot unlock absent capabilities.
8. Test actual Tauri WebView2 through local development-only CDP. The test sets the debugging environment only for its own child process and cleans it up.
9. Preserve unsupported modules as explicit roadmap pages, without fake findings or working-looking execution controls.
10. Native E2E failures were selector issues, not evidence of successful connectivity. The first passing E2E result is recorded separately.
11. Bind secrets to gateway/protocol fingerprints to prevent reused provider IDs or configuration races from redirecting credentials.
12. Fallback only for definite pre-transmission connection errors; never replay ambiguous requests.
13. Require stream completion markers, preserve interrupted output as unverified, and bound text/IPC updates.
14. Start Tool Manager with a benign zero-dependency ripgrep adapter. Pin archive and executable hashes verified from actual official assets; never execute arbitrary third-party manifests.
15. Build portable releases before claiming installers. CI configuration alone does not establish platform compatibility.
16. Preserve existing IPC/business logic during the UI redesign. Use a contiguous rail/navigation/main/activity layout and shared tokens rather than a repeated statistics dashboard.
17. Bundle licensed Manrope and original abstract SVG artwork. The supplied community reference remains an ignored design artifact and is not redistributed as product artwork.
18. Distinguish configured provider settings from verified connectivity; use only real audit events for activity. Project card progress denotes profile completeness, not research execution.
19. Retain Dark/Light/System preferences and accessible controls. Check responsive pages in the actual native WebView2; record viewport emulation separately from physical desktop/other-platform verification.
20. Centralize appearance independently of Settings drafts; serialize writes, roll back failures and reject stale snapshot hydration. Update only theme inside a SQLite transaction so appearance never rewrites stale routing/mode preferences.
21. Read authoritative appearance before revealing the native window. Keep OS-following behavior in one media listener; match native chrome/background through an additive typed Tauri command.
22. Replace micro-label typography with locally bundled DM Sans/IBM Plex Mono and explicit 12-26px roles. Preserve licensed sources and notices; eliminate fixed component colors in favor of semantic palette roles.
23. Isolate native test browser profiles with WEBVIEW2_USER_DATA_FOLDER. Database isolation alone does not avoid WebView2 browser-option conflicts with an already running application.
