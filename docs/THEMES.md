# Exactly three appearance themes

Active identifiers: `light`, `dark`, `reconx`. TypeScript uses `AppTheme = "light" | "dark" | "reconx"`. The user-facing labels are Light, Dark and ReconX. System is not an available theme, and OS appearance changes do not change any selection.

## Palettes

Light uses white/soft-gray main surfaces, gray navigation and rail, dark text, subtle borders and limited pink accents. Dark uses near-black main workspace, charcoal navigation/rail, graphite cards and subdued gray text/borders. ReconX preserves the existing burgundy/lavender/raspberry signature palette.

ReconX values were extracted directly from styles.css at repository revision ac18d15 before splitting the themes; no approximation or replacement was applied. The captured semantic tokens are in palettes/reconx-original.json. Examples: main #310019, sidebar #78667f, rail #65546d, right panel #750a44, cards #482135 and pink #eb3989. Native regression checks the literal original color tokens against this record and screenshots against the preceding burgundy build.

Original workspace/orbital/prism SVG files remain unchanged for ReconX. Neutral Light/Dark variants preserve their geometry with gray surfaces and restrained accent details, so Light has no large burgundy art backgrounds and Dark remains charcoal. Semantic image tokens select the assets. Components, controls, overlays, notifications, tables, terminals and scrollbars all use shared color tokens.

## State and persistence

src/theme.tsx owns one preference, persistence queue, error/rollback state and active menu. Topbar and sidebar use the same ThemeMenu; Settings Appearance uses exactly the same three choices. Changes apply immediately and persist without requiring Save preferences. Keyboard arrows/Home/End/Enter/Escape/Tab work. The Settings draft excludes theme, preserving other unsaved edits while appearance changes.

The existing shared SQLite settings record remains authoritative for desktop and CLI. Core::set_theme changes only appearance in a transaction. Startup reads saved appearance before the native window becomes visible; native chrome/background also match the selected theme. No OS media listener or automatic theme selection remains.

## Existing settings migration

| Existing value | Saved result |
|---|---|
| light | light |
| dark | dark |
| system | reconx |
| Missing field | reconx |
| New installation | reconx default |

Core::open/with_secrets performs an idempotent transaction on legacy settings. It validates the JSON first, updates only the theme field and records a migration audit event when required. It preserves routing, security mode, provider configurations, projects and credentials. No schema version change or database reset is performed. The serde legacy alias accepts old imported settings but always serializes the canonical reconx value; there is no fourth runtime enum variant or UI choice.

## Verification

scripts/verify.ps1 -Native -Visual -ToolInstall passed: Rust formatting, Clippy, workspace/CLI tests, TypeScript/Vite build, frontend tests and real native WebView2 integration. Debug record: artifacts/native-1791906428229/results.json, nine passing checkpoints.

The appearance matrix captures Dashboard/Settings and menus/dialogs in all three themes at 1920x1080 and 1366x768, validates exact choices, synchronized controls, primary backgrounds, original color preservation and typography. All three preferences survive full native process restarts. Emulated OS appearance changes do not change ReconX; unit tests confirm no OS appearance listener is installed. Legacy migration tests cover all four old cases, record preservation and repeat opening without additional migration events.

The separate viewport matrix checks 50 page/size combinations across five desktop sizes. Native regression retains project sharing, provider/model discovery, streaming/cancellation, restart persistence, secure credential controls and real verified ripgrep installation/health/uninstall. Tests use isolated databases and WebView2 profiles, never user credentials or public security targets.

Optimized release record: artifacts/native-1791906555280/results.json, nine passing checkpoints. Final portable binaries are in artifacts/windows-x64 with SHA256SUMS.txt and THIRD_PARTY_NOTICES.txt. The normal application was reopened at ReconX after legacy system migrated; security mode and default/fallback provider settings were compared before/after and retained. Screenshots are content viewport emulation inside real Tauri WebView2; Linux/ARM64 and other physical DPI configurations remain unverified. Future backend modules remain explicitly NOT_STARTED.
