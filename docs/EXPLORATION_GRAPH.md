# Research provenance graphs

The schema019 implementation materializes two related views from existing SQLite
records. It does not execute jobs, ask a model for follow-up actions, or create
assets from model prose. This is a bounded provenance feature, not autonomous
pentesting or a complete knowledge-graph engine.

## Stored model

`research_graph_nodes` stores a stable opaque ID, session, graph category, node
kind, actual source table/record ID, redacted label and bounded attributes.
`research_graph_edges` stores directed, typed relations and composite foreign keys
that prevent edges from linking nodes in another session. `research_graph_snapshots`
stores revision, projection hash, update time and counts. Existing projects,
sessions, jobs, inventory, evidence, findings and reports remain authoritative.
No second job queue/database is introduced.

Asset nodes represent real projects, registered targets, imported files and
inventory rows supported by completed-job observations. Exploration nodes
represent persisted sessions/plans, recorded approval, jobs, actual started
invocations, observations, evidence, manual findings and reports. An unexecuted
plan has no invented invocation or observation. Model interpretation stays
unverified evidence and is never converted into an asset or a Verified finding.

IDs derive from session, graph, node kind and persistent source identity; an
observation also includes the real asset identity. Display names do not determine
identity. Historical HTTP metadata comes from the completed job result rather
than the inventory row's mutable latest-job pointer. Typed edges include
BELONGS_TO_PROJECT, HAS_PLAN, INVESTIGATES, APPROVED_BY, DECOMPOSES_INTO,
EXECUTED_BY, DEPENDS_ON, PRODUCED, ABOUT, SUPPORTED_BY, HAS_FINDING and HAS_REPORT.

## Materialization and limits

Viewing/refreshing a graph reads stored session records and atomically upserts a
projection. Identical content preserves revision and IDs. Changed projections
retain existing node creation metadata and remove only obsolete derived graph
rows. They never remove the original observations/evidence. The page and export
read a consistent SQLite transaction. The projection is refreshed on review/UI
job-state changes; this is not an always-running graph worker.

- At most 5,000 nodes and 20,000 edges per session projection.
- At most 8 KiB metadata per node; no raw artifacts embedded.
- 1–200 nodes per backend page; Desktop requests 100.
- At most 1,000 connecting edges rendered for the current node page.
- At most 16 MiB per complete JSON export, 32 KiB per linked-record preview.

Limits fail explicitly rather than creating partial success. `source_sha256` is
the hash of the canonical redacted projection, not an independent integrity check
of every artifact. Evidence nodes retain original SHA-256 and explicitly state
that evidence integrity must be checked separately.

Labels strip URL authentication, query and fragment, and use common secret
redaction. Local import locations are omitted from graph target metadata; file
nodes expose name/hash/size/format rather than source paths. Export contains
redacted metadata only and never sends data to a model. Linked job/evidence/finding
inspection uses the existing Core record and redaction services; returned tool
text remains untrusted plain text.

## Desktop and CLI

Research Session → **Explore stored provenance** opens the graph. Users can filter
asset/exploration category and node type, search labels/IDs, page results, zoom,
pan by pointer or keyboard, focus the current page's connected neighborhood,
inspect source records and export JSON. A running task is visually distinct.
The layout is a deterministic grid; positions do not assert a causal relationship.
Only stored directed edges assert a relationship. Neighborhood filtering applies
to the loaded page and does not claim to expand the entire database graph.

```text
reconx graph nodes <session-id> --graph asset --limit 100
reconx graph nodes <session-id> --kind evidence --search <text>
reconx graph edges <session-id> --offset 0
reconx graph export <session-id> --output research-graph.json
```

Use `--json` for machine-readable output. `graph edges` returns edges whose
endpoints are in its bounded node page; complete bounded export includes all
stored nodes/edges. Exports refuse to overwrite an existing file.

## Verification and gaps

See the current master/test ledger for actual outcomes. Tests cover no execution
on review, real owned HTTP observations, real Job Engine evidence, manual finding
links, default redaction, stable IDs/revisions, pagination, cross-project/session
isolation, restart persistence, export and migration backup preservation.

The current graph is session-scoped, not a global cross-project intelligence
graph. It does not implement graph inference, agent decisions, semantic
clustering, complete state-transition history or speculative hypothesis nodes.
Refreshing a session after its target/file has been removed is not supported;
existing source/evidence records are preserved. Graph refresh is bounded and may
fail explicitly for very large sessions. These gaps keep the broader graph/module
release requirement PARTIAL until independently accepted.
