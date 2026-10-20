# Desktop UX checkpoint — 2026-10-19

Status: Windows native implementation verified for the ten principal
pages including Extensions. First-party extension installation is runtime verified. Linux GUI and physical
Windows DPI switching remain unverified. Product version remains 0.1.0.

Shared controls live in `apps/desktop/src/components/controls.tsx`; size tokens
and the compatibility bridge for existing actions live in `controls.css`.
Compact/standard/large actions use 30/34/38 px heights. The stable Button element
preserves focus and pointer gestures when asynchronous prerequisites change.
Disabled actions expose their actual prerequisite to keyboard and pointer users.
Tests exercise those behaviors, password masking, dropdown dismissal/navigation
and tab selection. Native dialog behavior is verified by Desktop tests; the
frontend dialog shim is only a DOM testing fixture.

Lucide React 1.59.0 supplies actual typed icons through one mapping. Primary
Segoe UI Variable Text, Source Sans 3 fallback and IBM Plex Mono are preserved.
Typography was not replaced. Layout tokens use a 48 px icon rail, collapsible
navigation, and an optional activity drawer. Activity visibility and navigation
collapse persist. Dashboard uses actual projects/jobs/audit records. Empty recon
has one Add Target action and contextual advanced details.

Exactly one Appearance button lives at the bottom of the icon rail. Its menu
contains Light, Dark and ReconX, with immediate update and durable backend storage.
Settings displays the current theme as information. The top-bar duplicate,
notification panel and redundant rail navigation were removed. AI Providers is
directly navigable. Version comes from the Rust snapshot build metadata.

Actual baseline: `artifacts/ux-before-1792391426265` (96 page/size/theme captures).
Refined Windows native matrix: `artifacts/ux-after-1792405586148` (120 captures,
plus provider and raster-scaling captures). Images were actually inspected;
that inspection caught and fixed ReconX theme token specificity, inconsistent
default project selection, excessive provider catalog expansion and a cramped
reverse-integration panel. Original five user-referenced images were not attached;
comparison against those unavailable files is not claimed.

Matrix: 1280×720, 1366×768, 1440×900 and 1920×1080 in all three themes. Automated
checks include overflow, action heights, single theme control, 48 px rail,
HTTP project binding, keyboard persistence/restart and optional panel behavior.
125/150/200% images use actual WebView CDP raster scaling, not physical OS DPI.
The fresh regression logs `artifacts/ux-regression-*.log` cover real native
projects, providers against local protocol fixtures, MCP, tools, jobs, imports,
HTTP/schema/replay, outputs and local recon. Cloud account connectivity is not
claimed by those fixture tests.

Remaining UX work includes native visual acceptance for newly implemented pages,
Linux GUI, physical DPI/accessibility inspection, and further page-by-page
consolidation of legacy structural controls. The shared-control migration does not
claim every custom tab, data-row or drag handle is a standard action button.
