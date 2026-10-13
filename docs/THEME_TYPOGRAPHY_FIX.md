# Typography and appearance repair

The earlier Light/Dark/System design below is historical. The active product now has exactly Light/Dark/ReconX and no OS-following option; see THEMES.md for the current state and migration. The typography, centralized persistence, draft protection and control repairs remain in use.

## Audit and causes

Inspected the actual native screenshot at artifacts/windows-x64/desktop-preview.png and App.tsx, WorkspacePages.tsx, styles.css, the typed IPC interface, Tauri commands, shared Settings and SQLite persistence before modifying them.

Appearance lived in a Settings draft and required Save; the topbar bypassed that draft and waited for an asynchronous full-settings write/refresh. The sidebar's settings icon opened Settings rather than selecting appearance. A refreshed settings object reset draft edits. Startup always began in the dark palette. Many CSS roles used 7-11px text; numerous component colors were fixed. The Local workspace option also fell back to the first project, and page changes retained the preceding page's scroll position.

## Implementation

src/theme.tsx owns the chosen preference, resolved appearance, OS media listener, persistence queue, rollback/error state and active menu. Topbar and sidebar share ThemeMenu; Settings Appearance reads and writes the same preference. The Settings draft excludes appearance entirely. Appearance applies immediately and persists automatically; Save preferences applies the other draft fields without overwriting the chosen mode. Serialized writes prevent a slow earlier choice from winning. A failed appearance write restores the last committed mode and displays an error. Revision guards prevent stale snapshots from reverting an in-flight choice.

New additive get_theme/set_theme IPC operations use the existing core and database. Core::set_theme reads/modifies/writes only appearance inside an immediate SQLite transaction, preserving the latest provider routing and security preferences. No schema migration, database reset or credential change is needed. CLI settings use the same record. Refresh reads external CLI changes; automatic cross-process live broadcast is not implemented.

The native window starts hidden. main.tsx loads authoritative saved appearance before rendering; the shared applyTheme helper sets CSS variables/color-scheme. apply_window_theme matches the Windows window/background and reveals the initialized UI. Startup errors show a retry screen rather than a guessed application configuration. OS changes use prefers-color-scheme while System is selected; explicit modes remove that listener.

Theme menus show the selected radio item and support pointer selection, outside dismissal, arrows, Home/End, Enter/Space, Escape with restored focus, and Tab into the next toolbar control. Only one menu opens at a time.

## Typography and palette

DM Sans is bundled for navigation, body and forms; IBM Plex Mono for code, technical data and terminals. Native fallbacks are Segoe UI Variable/Segoe UI for ordinary UI and appropriate monospace fallbacks for technical data. No runtime Google Fonts requests. Official pinned sources, commits and computed SHA-256 values are recorded in src/assets/fonts/sources.json. SIL OFL license files accompany each family; scripts/build.ps1 copies THIRD_PARTY_NOTICES.txt beside portable executables.

Shared roles: page title 26px/700, section 18px/650, card 15px/600, navigation/body/controls 14px, form labels and descriptions 13px, metadata 12px, code 13px. Variable DM Sans provides the intermediate weights. Removed sub-12px text across existing selectors, expanded navigation/control hit areas, reduced uppercase labels and removed the decorative waveform. Forms and activity rows have more consistent spacing and hierarchy.

All component CSS colors resolve semantic tokens. Light includes off-white main workspace, white cards/inputs, lavender navigation/rail and a subdued rose activity surface with dark text. Dark retains the reference's burgundy layout with less saturated plum cards and distinct inputs. Success/error/warning, focus, border, hover, native controls and scrollbars are themed. Original artwork has explicit on-art text/overlay tokens; it intentionally retains its authored palette in both modes.

## Visible control audit

| Control | Result |
|---|---|
| Topbar/sidebar appearance icons | Real shared Light/Dark/System menus |
| Settings Appearance | Immediate, automatic persistence; no Save prerequisite |
| Save preferences | Other draft settings plus current central appearance |
| Local workspace/project selector | Explicit local context now clears selected project; real project choice remains available |
| Page navigation | Opens real page and resets workspace scroll |
| Search, groups, panel toggles, notifications | Existing working handlers retained |
| Project/provider forms, credentials, chat/cancellation | Existing IPC preserved; native regression exercises shared records, gateway and credential controls |
| Tool health/uninstall | Existing operations retained; disabled prerequisites have explanatory titles |
| Future modules | Clearly NOT_STARTED; navigation/view tabs work without pretending to execute unavailable services |

## Verification

Windows debug record: artifacts/native-1791904765297/results.json. Eight checkpoints passed without the optional installer scenario. viewport-matrix.json checks 50 page/viewport combinations. appearance-matrix.json checks Dashboard/Settings in both themes at 1366x768 and 1920x1080, font roles/loading, 26px titles, 14px navigation, minimum 12px text and unclipped navigation. Captured and inspected dark/light Dashboard, Settings, menu and notification dialog screenshots.

System tests emulate the OS color-scheme media signal inside real Tauri WebView2; they do not alter the host OS preference. Both Light and System survive full process restart. Screenshots are native WebView2 content captures, with viewport emulation; Linux/ARM64 and all monitor/DPI combinations remain unverified.

Runtime setup initially failed because a test and the visible application shared WebView2's profile with different browser arguments. The harness now gives its child an isolated WEBVIEW2_USER_DATA_FOLDER as well as an isolated project database. An overly broad font assertion demanded an inactive mono face on Dashboard; the test now verifies DM Sans where UI is rendered and IBM Plex Mono where Settings renders actual technical code. Technical font loading is verified rather than inferred from a CSS declaration.

Release verification and final artifacts are recorded in TEST_RESULTS.md. Future MCP, research, findings and reporting backend services remain outside this repair.
