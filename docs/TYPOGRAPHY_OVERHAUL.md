# Typography refinement — 2026-10-14

This is a presentation-only refinement of the existing shell and pages. Routing,
Rust services, SQLite, CLI, target/file flows, provider credentials and the exact
Light/Dark/ReconX selection are preserved. No palettes or illustrations were replaced.

## Measured root causes

The before matrix uses the actual embedded Tauri/WebView2 application and a separate
fixture workspace. At 1440×900, page titles were 26px/700 with -0.4px tracking;
navigation was 14px/500, active navigation 600, form labels 13px/600 and activity
labels reached 650. Multiple late CSS patches competed with earlier rules. A native
matrix contained 279 sampled text elements above weight 600. The banner repeated
the page's large bold heading treatment and promotional copy.

The DM Sans file is a real variable font; its weight range is not a synthetic-bold
problem. Its cmap includes the Vietnamese sample. The renderer's trial nevertheless
used a mixture of DM Sans and Arial for that sample; this does not prove missing
glyphs in the file. More significantly, the unsuffixed CSS family `Segoe UI Variable`
resolved to Arial on this host despite SegUIVar.ttf being installed. The registered
`Segoe UI Variable Text` and `Segoe UI Variable Display` names resolved correctly.
These were measured using CDP CSS.getPlatformFontsForNode, not inferred from CSS.

## Selected fonts and roles

- Windows: **Segoe UI Variable Text**, then Segoe UI Variable.
- Bundled cross-platform fallback: **Source Sans 3**, then Noto Sans / Segoe UI.
- Technical data: existing **IBM Plex Mono**, then Cascadia Code / Consolas.
- DM Sans remains solely for the existing wordmarks; SVG/icon artwork is untouched.

Source Sans 3 3.052 was obtained from Adobe's official repository at immutable commit
87b37a2daaed80fcb8e8ccb0085c4d72ddade12e. Source URL, actual SHA-256 and OFL license are
stored beside the font. Segoe is used from the OS and is never redistributed. No
runtime CDN, forced antialiasing or browser smoothing flags were introduced.

`src/typography.css` defines one shared scale: 12/13/14/15/17/24px and weights
400/500/600. Page titles use 24/600; section titles 17/600; cards 15/600; nav 14/400,
active nav 500; form labels 13/500; inputs 14/400; buttons 14/500; secondary text
13/400; metadata 12/400; code 13/400. Body/control line heights are 1.5/1.4.
Ordinary tracking is normal and labels use sentence casing. Weight 700 is retained
only for existing wordmarks. The banner is a smaller 20px project heading and factual
project copy; its illustration/layout remain in place.

370 conflicting typography declarations were removed from styles.css, which now
owns layout and palettes rather than a second type system. Shared roles cover
headings, sidebars, activity, forms, cards, tables, menus, dialogs, AI prose and local
analysis panes. Code selectors are narrow so ordinary chat/UI text stays sans-serif.
Native window startup loads the same UI/code/wordmark token families before reveal,
rather than retaining a separate DM Sans-only loading path.
An existing min-height:40px input rule was restricted to text inputs, fixing checkbox
alignment without changing their handlers or state.

## Native before/after evidence

Directory: `artifacts/typography-refinement/{before,after}`.

Each phase includes all six requested pages (Dashboard, Projects, Targets & Scope,
Reverse Engineering, AI Workspace, Settings), all three themes and 1920×1080,
1440×900, 1366×768: **54 page/theme/viewport observations per phase**. Three additional
Settings captures per phase exercise device scale 1.25, 1.5 and 2 through CDP.
This is actual WebView2 rasterization under emulation, not a claim that Windows
monitor scaling was physically changed. Linux rendering remains unverified.
The final after captures use direct CDP PNG output with verified pixel dimensions:
1800x1125, 2160x1350 and 2880x1800 for a 1440x900 logical viewport. Original before
DPI images use a viewport crop; the 54 ordinary before/after captures use their
full requested dimensions.

Computed styles, platform fonts, widths, clipping checks and results.json are saved
with screenshots. English/Vietnamese probes resolved to a single Segoe UI Variable
font; bundled Source Sans fallback resolved from the application origin. FontTools
also found no missing sample glyphs in Segoe, Source Sans or IBM Plex Mono.

After: no sampled UI text above weight 600; ordinary 400-weight elements increased
from 1179 to 2232 across the main matrix. No visible text below 12px, clipped nav
labels or workspace horizontal overflow was detected. Actual screenshots were
inspected for hierarchy, alignment, readability and all three appearances.

Before/after fixture records are real and retained; AI prose comes from an explicitly
local deterministic test gateway and makes no vulnerability claims. Chat is transient
and regenerated, so audit timestamps/activity may differ between captures.

Run `RECONX_TYPOGRAPHY_ARTIFACT` with an isolated directory, then
`node tests/e2e/typography.mjs before` before changing source and `... after` after
an embedded build. The default after run also validates source font fallback and
computed roles. Never direct test fixtures at a user database.

Design/API references: [Microsoft Windows typography](https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/typography),
[Adobe Source Sans](https://github.com/adobe-fonts/source-sans).
