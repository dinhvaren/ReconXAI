import { useEffect, useRef, useState } from "react";
import { request, errorText, type Snapshot } from "../api";
import { Button } from "../components/controls";
import { Heading, Modal } from "../components/ui";
import ReconAssetInventory from "../components/ReconAssetInventory";
import ResearchGraph from "../components/ResearchGraph";
import AgentRuntimePanel from "../components/AgentRuntimePanel";
import JobHistory from "../components/JobHistory";
import { selectPaths, type AnalysisJob } from "../research";
import { useWorkspace } from "../workspace";

interface Reply {
  model: string;
  input_tokens?: number;
  output_tokens?: number;
  text: string;
}
interface Preview {
  request: { target_id: string; provider_id: string | null; mode: string };
  project_id: string;
  target_name: string;
  target_kind: string;
  model: string | null;
  gateway: string | null;
  context: string;
  fingerprint: string;
  limitations: string[];
  recon_reviews?: {
    config: { tool: string; max_assets: number; timeout_secs: number };
    target: string;
    permissions: string[];
    limits: string[];
  }[];
  capabilities: {
    id: string;
    name: string;
    available: boolean;
    reason: string;
    network_requests: number | null;
  }[];
}
interface Evidence {
  id: string;
  title: string;
  kind: string;
  sha256: string;
}
interface Report {
  id: string;
  title: string;
  sha256: string;
}
interface Session {
  runtime_enabled?: boolean;
  plan: {
    id: string;
    preview: Preview;
    proposal: { steps: { capability_id: string; reason: string }[] };
    planning_usage: Reply | null;
    created_at: string;
  };
  status: string;
  jobs: AnalysisJob[];
  evidence: Evidence[];
  findings: { id: string; input: { title: string; status: string } }[];
  reports: Report[];
  interpretation: Reply | null;
}
interface ReportView {
  report: Report;
  document: { text: string; redactions: number; warnings: string[] };
}

export default function ResearchPage({
  data,
  providerId,
  navigate,
}: {
  data: Snapshot;
  providerId?: string;
  navigate: (page: string) => void;
}) {
  const workspace = useWorkspace()!;
  const project = workspace.selection.project_id,
    target = workspace.selection.target_id;
  const [provider, setProvider] = useState(
    providerId ?? data.settings.default_provider_id ?? "",
  );
  const [mode, setMode] = useState("ai_assisted"),
    [preview, setPreview] = useState<Preview | null>(null);
  const [consent, setConsent] = useState(false),
    [approved, setApproved] = useState(false),
    [network, setNetwork] = useState(false);
  const [session, setSession] = useState<Session | null>(null),
    [history, setHistory] = useState<Session[]>([]);
  const [busy, setBusy] = useState(false),
    [error, setError] = useState("");
  const [context, setContext] = useState(""),
    [correlateConsent, setCorrelateConsent] = useState(false);
  const [title, setTitle] = useState(""),
    [details, setDetails] = useState("");
  const [report, setReport] = useState<ReportView | null>(null),
    [reviewed, setReviewed] = useState(false),
    [format, setFormat] = useState("pdf");
  const [evidence, setEvidence] = useState<{
    evidence: Evidence;
    text: string | null;
  } | null>(null);
  const generation = useRef(0),
    active = useRef(true);
  const selection = useRef({ project, target });
  selection.current = { project, target };
  const act = async (task: () => Promise<void>) => {
    const ticket = generation.current;
    setBusy(true);
    setError("");
    try {
      await task();
    } catch (e) {
      if (ticket === generation.current) setError(errorText(e));
    } finally {
      if (active.current) setBusy(false);
    }
  };
  const accept = (value: Session, allowUnselectedHistory = false) => {
    if (
      value.plan.preview.project_id !== selection.current.project ||
      (value.plan.preview.request.target_id !== selection.current.target &&
        !(allowUnselectedHistory && selection.current.target === null))
    )
      return;
    setSession(value);
    setHistory((previous) =>
      previous.map((s) => (s.plan.id === value.plan.id ? value : s)),
    );
    localStorage.setItem(
      `reconx.research.${value.plan.preview.request.target_id}`,
      value.plan.id,
    );
  };
  const loadHistory = async (restore = false) => {
    const before = selection.current;
    if (!before.project) {
      setHistory([]);
      return;
    }
    const ids = await request<string[]>("research_list", {
      project: before.project,
    });
    const values = await Promise.all(
      ids.slice(0, 20).map((id) => request<Session>("research_show", { id })),
    );
    if (
      selection.current.project !== before.project ||
      selection.current.target !== before.target
    )
      return;
    const matching = values.filter(
      (s) =>
        !before.target || s.plan.preview.request.target_id === before.target,
    );
    setHistory(matching);
    if (restore && before.target) {
      const preferred = localStorage.getItem(
        `reconx.research.${before.target}`,
      );
      const saved =
        matching.find((s) => s.plan.id === preferred) ?? matching[0];
      if (saved) accept(saved);
    }
  };
  useEffect(() => {
    active.current = true;
    const ticket = ++generation.current;
    setPreview(null);
    setConsent(false);
    setApproved(false);
    setNetwork(false);
    setContext("");
    setSession(null);
    setEvidence(null);
    setReport(null);
    setHistory([]);
    setError("");
    setTitle("");
    setDetails("");
    void loadHistory(true).catch((e) => {
      if (ticket === generation.current) setError(errorText(e));
    });
    return () => {
      active.current = false;
      generation.current += 1;
    };
  }, [project, target]);
  useEffect(() => {
    if (
      !session ||
      ![
        "queued",
        "running",
        "cancelling",
        "pause_requested",
        "paused",
      ].includes(session.status)
    )
      return;
    let closed = false;
    const ticket = generation.current;
    const timer = setInterval(
      () =>
        void request<Session>("research_show", { id: session.plan.id })
          .then((s) => {
            if (!closed && ticket === generation.current) accept(s, true);
          })
          .catch((e) => {
            if (!closed) setError(errorText(e));
          }),
      1000,
    );
    return () => {
      closed = true;
      clearInterval(timer);
    };
  }, [session?.plan.id, session?.status]);
  const selectedTarget = data.targets?.find(
    (t) => t.id === target && t.project_id === project,
  );
  const complete = session?.status === "completed";
  const sessionTargetExists = Boolean(
    session &&
    data.targets?.some(
      (t) =>
        t.id === session.plan.preview.request.target_id &&
        t.project_id === project,
    ),
  );
  const historicalOnly = Boolean(
    session &&
    (!sessionTargetExists || target !== session.plan.preview.request.target_id),
  );
  const sessionId = session?.plan.id;
  const refresh = async () => {
    const ticket = generation.current;
    if (sessionId) {
      const value = await request<Session>("research_show", { id: sessionId });
      if (ticket === generation.current) accept(value, true);
    }
    await loadHistory();
  };
  return (
    <>
      <Heading
        title="AI Research"
        eyebrow="Target workspace"
        description="Review a typed plan, run available capabilities and keep the resulting evidence together."
      />
      {error && (
        <div className="banner error" role="alert">
          {error}
        </div>
      )}
      <section className="panel records-panel research-context">
        <h2>Research context</h2>
        <div className="records-toolbar">
          <label>
            Project
            <select
              aria-label="Research project"
              value={project ?? ""}
              disabled={busy || workspace.saving}
              onChange={(e) =>
                void workspace.select(e.target.value || null).catch(() => {})
              }
            >
              <option value="">Select project</option>
              {data.projects
                .filter((p) => !p.archived)
                .map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name}
                  </option>
                ))}
            </select>
          </label>
          <label>
            Target
            <select
              aria-label="Research target"
              value={target ?? ""}
              disabled={busy || workspace.saving}
              onChange={(e) =>
                void workspace
                  .select(project, e.target.value || null)
                  .catch(() => {})
              }
            >
              <option value="">Select target</option>
              {data.targets
                ?.filter((t) => t.project_id === project)
                .map((t) => (
                  <option key={t.id} value={t.id}>
                    {t.name} · {t.kind}
                  </option>
                ))}
            </select>
          </label>
          <label>
            Model
            <select
              aria-label="Research model"
              value={provider}
              disabled={busy}
              onChange={(e) => {
                setProvider(e.target.value);
                setPreview(null);
                setConsent(false);
                setContext("");
                setCorrelateConsent(false);
              }}
            >
              <option value="">No model selected</option>
              {data.providers
                .filter((p) => p.enabled)
                .map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name} · {p.model}
                  </option>
                ))}
            </select>
          </label>
          <label>
            Mode
            <select
              aria-label="Research mode"
              value={mode}
              disabled={busy}
              onChange={(e) => {
                setMode(e.target.value);
                setPreview(null);
                setConsent(false);
              }}
            >
              <option value="manual">Manual</option>
              <option value="ai_assisted">AI Assisted</option>
              <option value="authorized_auto">
                Authorized Auto (offline only)
              </option>
            </select>
          </label>
        </div>
        <p>
          {selectedTarget
            ? `${selectedTarget.kind} · ${selectedTarget.authorized ? "Authorization recorded" : "Authorization missing"} · ${selectedTarget.scope_status}`
            : "Select a target to discover the capabilities available for it."}
        </p>
        <Button
          variant="primary"
          disabled={
            busy ||
            !target ||
            workspace.saving ||
            (mode !== "manual" && !provider)
          }
          disabledReason="Select a target and a configured model, or choose Manual mode"
          onClick={() =>
            void act(async () => {
              const p = await request<Preview>("research_preview", {
                input: {
                  target_id: target,
                  provider_id: provider || null,
                  mode,
                },
              });
              if (p.request.target_id === selection.current.target) {
                setPreview(p);
                setConsent(false);
              }
            })
          }
        >
          Review research context
        </Button>
        <Button variant="secondary" onClick={() => navigate("AI Providers")}>
          Configure models
        </Button>
      </section>
      {preview && (
        <section className="panel records-panel">
          <h2>Available capabilities</h2>
          <ul>
            {preview.capabilities.map((c) => (
              <li key={c.id}>
                <strong>{c.name}</strong> —{" "}
                {c.available ? "Available" : "Unavailable"}. {c.reason}
                {c.network_requests !== null &&
                  c.network_requests > 0 &&
                  ` Maximum active URL probes: ${c.network_requests}.`}
              </li>
            ))}
          </ul>
          <Button variant="secondary" onClick={() => navigate("Extensions")}>
            Open Marketplace
          </Button>
          <details>
            <summary>Policy and current limits</summary>
            <ul>
              {preview.limitations.map((l) => (
                <li key={l}>{l}</li>
              ))}
            </ul>
          </details>
          {mode !== "manual" && (
            <>
              <p>
                Planning destination: <strong>{preview.gateway}</strong> ·{" "}
                {preview.model}
              </p>
              <pre className="research-preview">{preview.context}</pre>
              <label className="checkbox">
                <input
                  type="checkbox"
                  checked={consent}
                  onChange={(e) => setConsent(e.target.checked)}
                />
                I approve sending exactly this planning context to the selected
                model.
              </label>
            </>
          )}
          <Button
            variant="primary"
            disabled={busy || (mode !== "manual" && !consent)}
            disabledReason="Review and approve the displayed AI context"
            onClick={() =>
              void act(async () => {
                accept(
                  await request<Session>("research_plan", {
                    review: {
                      request: preview.request,
                      fingerprint: preview.fingerprint,
                      context: preview.context,
                      consent,
                    },
                  }),
                );
                setPreview(null);
                setApproved(false);
                setNetwork(false);
                await loadHistory();
              })
            }
          >
            {busy
              ? "Working…"
              : mode === "manual"
                ? "Create manual plan"
                : "Generate research plan"}
          </Button>
        </section>
      )}
      {history.length > 0 && (
        <section className="panel records-panel">
          <label>
            Saved sessions
            <select
              aria-label="Research history"
              value={sessionId ?? ""}
              disabled={busy}
              onChange={(e) =>
                void act(async () => {
                  const ticket = generation.current;
                  const value = await request<Session>("research_show", {
                    id: e.target.value,
                  });
                  if (ticket !== generation.current) return;
                  accept(value, true);
                  setContext("");
                  setApproved(false);
                  setNetwork(false);
                })
              }
            >
              <option value="">Select session</option>
              {history.map((s) => (
                <option key={s.plan.id} value={s.plan.id}>
                  {s.plan.created_at} · {s.plan.preview.target_name} ·{" "}
                  {s.status}
                </option>
              ))}
            </select>
          </label>
        </section>
      )}
      {session && (
        <>
          <section
            className="panel records-panel"
            aria-label="Research session"
          >
            <h2>
              Research session · <span role="status">{session.status}</span>
            </h2>
            <p>
              {session.plan.preview.target_name} ·{" "}
              {session.plan.preview.model ?? "Manual"} ·{" "}
              <code>{session.plan.id}</code>
            </p>
            {historicalOnly && (
              <p role="status">
                {sessionTargetExists
                  ? "No active session target is selected."
                  : "This target no longer exists."}{" "}
                Session history is read-only; stored evidence, graphs and
                existing reports remain available.
              </p>
            )}
            <details>
              <summary>Explore stored provenance</summary>
              <ResearchGraph
                session={session.plan.id}
                revision={
                  session.jobs
                    .map((j) => `${j.id}:${j.status}:${j.finished_at}`)
                    .join("|") +
                  session.findings.length +
                  ":" +
                  session.reports.length
                }
              />
            </details>
            <ol>
              {session.plan.proposal.steps.map((step, i) => (
                <li key={step.capability_id}>
                  <strong>
                    {session.plan.preview.capabilities.find(
                      (c) => c.id === step.capability_id,
                    )?.name ?? step.capability_id}
                  </strong>
                  <p>{step.reason}</p>
                  {session.jobs[i] && (
                    <span>
                      {session.jobs[i].status} · {session.jobs[i].stage}
                    </span>
                  )}
                </li>
              ))}
            </ol>
            {session.plan.planning_usage && (
              <p>
                Planning tokens: input{" "}
                {session.plan.planning_usage.input_tokens ?? "not reported"} /
                output{" "}
                {session.plan.planning_usage.output_tokens ?? "not reported"}.
                Cost: unavailable.
              </p>
            )}
            {session.jobs.some((j) => j.kind.startsWith("recon_")) &&
              project && (
                <ReconAssetInventory project={project} target={target} />
              )}
            {session.status === "awaiting_approval" && !historicalOnly && (
              <>
                <label className="checkbox">
                  <input
                    type="checkbox"
                    checked={approved}
                    onChange={(e) => setApproved(e.target.checked)}
                  />
                  I approve the listed capabilities and resource limits for this
                  target.
                </label>
                {session.plan.proposal.steps.some(
                  (s) => s.capability_id === "local.http_head",
                ) && (
                  <div aria-label="Network execution review">
                    <p>
                      Current registered destination:{" "}
                      <code>HEAD {selectedTarget?.value}</code>. Request budget:
                      1; timeout: 5 seconds; redirects, proxy and retries
                      disabled; HTTPS certificate verification enabled.
                      Execution revalidates the plan against the current target
                      and scope. Changes require a new plan.
                    </p>
                    <label className="checkbox">
                      <input
                        type="checkbox"
                        checked={network}
                        onChange={(e) => setNetwork(e.target.checked)}
                      />
                      I separately approve one HEAD request to the authorized
                      owned-loopback target.
                    </label>
                  </div>
                )}
                {session.plan.proposal.steps.some((s) =>
                  ["httpx.probe", "subfinder.passive"].includes(
                    s.capability_id,
                  ),
                ) && (
                  <div aria-label="Recon execution review">
                    {(session.plan.preview.recon_reviews ?? [])
                      .filter((p) =>
                        session.plan.proposal.steps.some((s) =>
                          s.capability_id.startsWith(p.config.tool + "."),
                        ),
                      )
                      .map((p) => (
                        <div key={p.config.tool}>
                          <p>
                            <strong>{p.config.tool}</strong> ·{" "}
                            <code>{p.target}</code>{" "}
                            {p.config.tool === "subfinder"
                              ? "Selected passive providers: 2 requests/second, 120 second deadline; no exact source request-count guarantee."
                              : `Maximum ${p.config.max_assets} URL probes; per-probe timeout ${p.config.timeout_secs}s.`}
                          </p>
                          <ul>
                            {p.permissions.map((x) => (
                              <li key={x}>{x}</li>
                            ))}
                            {p.limits.map((x) => (
                              <li key={x}>{x}</li>
                            ))}
                          </ul>
                        </div>
                      ))}
                    <label className="checkbox">
                      <input
                        type="checkbox"
                        checked={network}
                        onChange={(e) => setNetwork(e.target.checked)}
                      />
                      I separately approve the displayed recon network
                      operations and passive-source data sharing.
                    </label>
                  </div>
                )}
                <Button
                  variant="primary"
                  disabled={
                    busy ||
                    !approved ||
                    (session.plan.proposal.steps.some((s) =>
                      [
                        "local.http_head",
                        "httpx.probe",
                        "subfinder.passive",
                      ].includes(s.capability_id),
                    ) &&
                      !network)
                  }
                  disabledReason="Approve the plan and any network operation"
                  onClick={() =>
                    void act(async () => {
                      accept(
                        await request<Session>("research_approve", {
                          id: sessionId,
                          fingerprint: session.plan.preview.fingerprint,
                          approved,
                          network_approved: network,
                        }),
                      );
                    })
                  }
                >
                  Execute approved plan
                </Button>
              </>
            )}
            {!session.runtime_enabled &&
              ["awaiting_approval", "queued", "running"].includes(
                session.status,
              ) && (
                <Button
                  variant="secondary"
                  disabled={busy}
                  onClick={() =>
                    void act(async () => {
                      await request("research_cancel", { id: sessionId });
                      await refresh();
                    })
                  }
                >
                  Cancel research
                </Button>
              )}
            <p>
              {session.runtime_enabled
                ? "Offline DAG pause finishes current jobs before stopping new claims."
                : "Legacy batches do not pause; the reviewed offline DAG supports job-boundary pause/resume."}{" "}
              Completed jobs and results remain persistent.
            </p>
            <AgentRuntimePanel
              session={session.plan.id}
              status={session.status}
              enabled={!!session.runtime_enabled}
              readOnly={historicalOnly}
              eligible={session.plan.proposal.steps.every((s) =>
                [
                  "local.scope_review",
                  "local.metadata",
                  "local.strings",
                  "yara-x.signatures",
                ].includes(s.capability_id),
              )}
              changed={refresh}
            />
            <JobHistory jobs={session.jobs} />
          </section>
          <section className="panel records-panel">
            <h2>Evidence</h2>
            {session.evidence.length ? (
              <ul>
                {session.evidence.map((e) => (
                  <li key={e.id}>
                    <Button
                      variant="secondary"
                      disabled={busy}
                      onClick={() =>
                        void act(async () =>
                          setEvidence(
                            await request("evidence_show", { id: e.id }),
                          ),
                        )
                      }
                    >
                      {e.title}
                    </Button>{" "}
                    <code>{e.sha256.slice(0, 16)}</code> · {e.kind}
                  </li>
                ))}
              </ul>
            ) : (
              <p>
                Evidence appears after a capability produces an actual result.
              </p>
            )}
            {complete && provider && !historicalOnly && (
              <Button
                disabled={busy}
                onClick={() =>
                  void act(async () => {
                    setContext(
                      await request<string>("research_evidence_preview", {
                        id: sessionId,
                      }),
                    );
                    setCorrelateConsent(false);
                  })
                }
              >
                Preview AI evidence context
              </Button>
            )}
            {context && (
              <>
                <p>
                  Destination:{" "}
                  {data.providers.find((p) => p.id === provider)?.base_url}.
                  Credentials are excluded; review all remaining evidence before
                  transmission.
                </p>
                <pre className="research-preview">{context}</pre>
                <label className="checkbox">
                  <input
                    type="checkbox"
                    checked={correlateConsent}
                    onChange={(e) => setCorrelateConsent(e.target.checked)}
                  />
                  Approve this evidence context for the selected model.
                </label>
                <Button
                  disabled={busy || !correlateConsent}
                  disabledReason="Review and approve this evidence context"
                  onClick={() =>
                    void act(async () => {
                      await request("research_interpret", {
                        review: {
                          id: sessionId,
                          provider_id: provider,
                          gateway: data.providers.find((p) => p.id === provider)
                            ?.base_url,
                          model: data.providers.find((p) => p.id === provider)
                            ?.model,
                          preview: context,
                          consent: correlateConsent,
                        },
                      });
                      setContext("");
                      await refresh();
                    })
                  }
                >
                  Correlate evidence
                </Button>
              </>
            )}
            {session.interpretation && (
              <>
                <h3>AI interpretation · requires review</h3>
                <pre className="research-preview">
                  {session.interpretation.text}
                </pre>
              </>
            )}
          </section>
          {complete && (
            <section className="panel records-panel">
              <h2>Findings and report</h2>
              <p>
                Record only a supported hypothesis. The saved finding remains
                Needs Verification.
              </p>
              <label>
                Finding title
                <input
                  aria-label="Research finding title"
                  disabled={historicalOnly}
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  maxLength={300}
                />
              </label>
              <label>
                Technical details
                <textarea
                  aria-label="Research finding details"
                  disabled={historicalOnly}
                  value={details}
                  onChange={(e) => setDetails(e.target.value)}
                  rows={4}
                />
              </label>
              <Button
                disabled={
                  busy || historicalOnly || !title.trim() || !details.trim()
                }
                disabledReason={
                  historicalOnly
                    ? "Select this session's existing target to create a finding"
                    : "Enter an evidence-supported title and details"
                }
                onClick={() =>
                  void act(async () => {
                    await request("research_finding", {
                      id: sessionId,
                      title,
                      details,
                    });
                    setTitle("");
                    setDetails("");
                    await refresh();
                  })
                }
              >
                Save reviewable finding
              </Button>
              <ul>
                {session.findings.map((f) => (
                  <li key={f.id}>
                    {f.input.title} · {f.input.status}
                  </li>
                ))}
              </ul>
              <Button
                variant="primary"
                disabled={busy || historicalOnly}
                disabledReason={
                  historicalOnly
                    ? "History is read-only; open an existing report"
                    : undefined
                }
                onClick={() =>
                  void act(async () => {
                    setReport(
                      await request("research_report", { id: sessionId }),
                    );
                    setReviewed(false);
                    await refresh();
                  })
                }
              >
                Prepare research report
              </Button>
              {session.reports.map((r) => (
                <Button
                  key={r.id}
                  disabled={busy}
                  onClick={() =>
                    void act(async () => {
                      setReport(await request("report_show", { id: r.id }));
                      setReviewed(false);
                    })
                  }
                >
                  {r.title}
                </Button>
              ))}
            </section>
          )}
        </>
      )}
      <Modal
        open={!!evidence}
        title="Evidence preview"
        onClose={() => setEvidence(null)}
      >
        {evidence && (
          <>
            <p>
              {evidence.evidence.title} · SHA-256{" "}
              <code>{evidence.evidence.sha256}</code>
            </p>
            <pre className="research-preview">
              {evidence.text ??
                "Binary artifact; use Evidence Explorer to export and verify it."}
            </pre>
          </>
        )}
      </Modal>
      <Modal
        open={!!report}
        title="Research report preview"
        onClose={() => setReport(null)}
        dismissDisabled={busy}
      >
        {report && (
          <>
            <p>
              {report.document.redactions} redactions.{" "}
              {report.document.warnings.join(" ")}
            </p>
            <pre className="research-preview">{report.document.text}</pre>
            <label className="checkbox">
              <input
                type="checkbox"
                checked={reviewed}
                onChange={(e) => setReviewed(e.target.checked)}
              />
              I reviewed this exact report snapshot.
            </label>
            <select
              aria-label="Research report format"
              value={format}
              onChange={(e) => setFormat(e.target.value)}
            >
              {["pdf", "markdown", "html", "json"].map((f) => (
                <option key={f}>{f}</option>
              ))}
            </select>
            <Button
              variant="primary"
              disabled={busy || !reviewed}
              disabledReason="Review this report snapshot before export"
              onClick={() =>
                void act(async () => {
                  const path = (
                    await selectPaths(
                      false,
                      true,
                      `research-report.${format === "markdown" ? "md" : format}`,
                    )
                  )[0];
                  if (path) {
                    await request("report_export", {
                      id: report.report.id,
                      format,
                      path,
                      reviewed_sha256: report.report.sha256,
                    });
                    setReport(null);
                  }
                })
              }
            >
              Export reviewed report
            </Button>
          </>
        )}
      </Modal>
    </>
  );
}
