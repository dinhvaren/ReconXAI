# Agent architecture status

Per-agent model routing settings exist in the AI Provider Manager. Typed local
planning and target research sessions use the real Core services. These features
do not constitute a completed multi-agent runtime.

Current useful responsibilities are implemented as Core operations: plan
validation, scope checks, job execution, evidence integrity checking, finding
review and report export. Their inputs/results are typed and persisted by their
existing services. The graph links these actual operations without inventing
agent assignments or agent messages.

No new chat personas, synthetic workers, duplicate task queue or decorative agent
status cards are added. A general Planner/Worker/Review message protocol, DAG
scheduler, agent token/cost budgeting, retries/backoff and pause/resume are not
implemented. Model profiles without required capabilities must not be presented
as ready for tool execution. Mock-model tests do not verify real cloud accounts.

Future non-offensive analysis coordination can reuse the Job Engine and explicitly
bounded local capabilities. Its acceptance must prove real independent work,
dependency handling, restart recovery, evidence references and no policy bypass.
It must not infer that a model interpretation is a verified security finding or
silently send project data to fallback providers.

This module remains PARTIAL/NOT_IMPLEMENTED at the runtime layer; see the current
machine-readable inventories. The implemented provenance graph is documented
separately in EXPLORATION_GRAPH.md.
