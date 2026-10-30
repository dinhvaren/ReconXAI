# Research session execution scope

ReconX AI 0.1.0 has persistent target-bound research sessions, typed plans,
one-shot approval, existing Job Engine execution, evidence review and reports.
Manual mode selects supported local operations. AI Assisted mode requests a typed
proposal after previewed model/context consent. Authorized Auto currently excludes
network steps rather than assuming standing network permission.

Session/project/target IDs survive restart and are shared by Desktop/CLI. Planning
and evidence interpretation are separate consent boundaries. Available capabilities
depend on actual tools, target compatibility and policy. AI prose cannot add
executable handlers or override Rust approval/scope checks. Findings remain manual
and reviewable unless their existing independent verification requirements pass.

The current implementation is not an iterative autonomous pentest engine. There
is no PLAN→EXECUTE→EVALUATE loop that automatically selects subsequent offensive
actions. Schema020 adds reviewed offline typed coordination, dependency jobs,
job-boundary pause/resume, explicit bounded retries and invocation/wall-time
limits. Generic subprocess suspension, cloud token/cost budgets, automatic
hypothesis validation and standing authorization for network work remain absent.
See MULTI_AGENT_ARCHITECTURE.md and DAG_SCHEDULER.md for exact semantics.

The graph provides an inspectable record of what was planned, approved,
executed and preserved. Reviewing that graph does not cause a continuation or data
transmission. Existing network capabilities still require their explicit reviewed
request/plan boundaries. Local-fixture tests are distinct from live provider or
external target verification.

See AI_RESEARCH_SESSIONS.md, RECON_TOOL_ADAPTERS.md, EXPLORATION_GRAPH.md and the
current execution ledger for implemented controls and actual acceptance evidence.
