import { useCallback, useEffect, useState } from "react";
import { request, type Snapshot, type Reply } from "../api";
import { Heading, Modal, type Perform } from "../components/ui";
import JobHistory from "../components/JobHistory";
import type { ImportedFile } from "../research";
interface Plan {
  id: string;
  file_id: string;
  proposal: { steps: { capability_id: string; reason: string }[] };
  created_at: string;
}
interface StoredPlan {
  plan: Plan;
  executed_at: string | null;
  jobs: string[];
}
interface Capability {
  id: string;
  name: string;
  available: boolean;
  ai_plannable: boolean;
}

export function OrchestrationPage({
  data,
  projectId,
  busy,
  perform,
}: {
  data: Snapshot;
  projectId?: string;
  busy: boolean;
  perform: Perform;
}) {
  const [project, setProject] = useState(projectId ?? ""),
    [files, setFiles] = useState<ImportedFile[]>([]),
    [file, setFile] = useState("");
  const [provider, setProvider] = useState(
      data.settings.default_provider_id ?? "",
    ),
    [plans, setPlans] = useState<StoredPlan[]>([]),
    [caps, setCaps] = useState<Capability[]>([]);
  const [preview, setPreview] = useState(""),
    [consent, setConsent] = useState(false),
    [error, setError] = useState("");
  const [evidence, setEvidence] = useState<{
      id: string;
      text: string;
      gateway: string;
      provider: string;
    } | null>(null),
    [reply, setReply] = useState("");
  useEffect(() => {
    let active = true;
    void request<{ id: string }>("ai_agent_provider", { agent: "orchestrator" })
      .then((p) => {
        if (active && p?.id) setProvider(p.id);
      })
      .catch(() => {});
    return () => {
      active = false;
    };
  }, []);
  const config = data.providers.find((p) => p.id === provider);
  const load = useCallback(async () => {
    if (!project) {
      setPlans([]);
      setFiles([]);
      setCaps([]);
      return;
    }
    const [f, p, c] = await Promise.all([
      request<ImportedFile[]>("files_list", { project }),
      request<StoredPlan[]>("plans_list", { project }),
      request<Capability[]>("capabilities", { project }),
    ]);
    setFiles(f);
    setPlans(p);
    setCaps(c);
    setError("");
  }, [project]);
  useEffect(() => {
    void load().catch((e) => setError(e.message ?? String(e)));
    const t = setInterval(
      () => void load().catch((e) => setError(e.message ?? String(e))),
      3000,
    );
    return () => clearInterval(t);
  }, [load]);
  useEffect(() => {
    setPreview("");
    setConsent(false);
  }, [file, provider, project]);
  const jobs = (data.jobs ?? []).filter((j) =>
    plans.some((p) => p.jobs.includes(j.id)),
  );
  return (
    <>
      <Heading
        eyebrow="AI assisted · Local analysis"
        title="Agent Workflows"
        description="Prepare a structured local-file plan, approve its capabilities, then inspect actual Job Engine results. No model-generated shell commands."
      />
      {error && (
        <p role="alert" className="connection-result">
          {error}
        </p>
      )}
      <section className="panel integration-detail">
        <div className="panel-heading">
          <h2>Local analysis planner</h2>
          <span className="badge muted">Approval required</span>
        </div>
        <div className="integration-controls">
          <label>
            Project
            <select
              aria-label="Workflow project"
              value={project}
              onChange={(e) => {
                setProject(e.target.value);
                setFile("");
              }}
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
            Imported file
            <select
              aria-label="Workflow file"
              value={file}
              onChange={(e) => setFile(e.target.value)}
            >
              <option value="">Select imported file</option>
              {files.map((f) => (
                <option key={f.id} value={f.id}>
                  {f.name} · {f.format}
                </option>
              ))}
            </select>
          </label>
          <label>
            AI provider
            <select
              aria-label="Workflow provider"
              value={provider}
              onChange={(e) => setProvider(e.target.value)}
            >
              <option value="">Select provider</option>
              {data.providers
                .filter((p) => p.enabled)
                .map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name} · {p.model}
                  </option>
                ))}
            </select>
          </label>
        </div>
        <p className="helper">
          Available:{" "}
          {caps
            .filter((c) => c.available && c.ai_plannable)
            .map((c) => c.name)
            .join(", ") || "Choose a project"}
          . MCP tools remain available for separately reviewed manual calls in
          MCP Hub.
        </p>
        <button
          className="button secondary"
          disabled={busy || !file || !config}
          onClick={() =>
            void perform(async () => {
              setPreview(
                await request<string>("plan_preview", { file_id: file }),
              );
              setConsent(false);
            })
          }
        >
          Preview planning context
        </button>
        {preview && config && (
          <>
            <pre className="technical-output">{preview}</pre>
            <p className="helper">
              Destination: {config.base_url}. Only the displayed metadata and
              capability definitions are transmitted; no binary bytes, paths or
              notes. No fallback gateway is used.
            </p>
            <label className="checkbox-label">
              <input
                aria-label="Approve planning metadata transmission"
                type="checkbox"
                checked={consent}
                onChange={(e) => setConsent(e.target.checked)}
              />
              I approve transmitting this exact planning context to this
              provider.
            </label>
            <button
              className="button primary"
              disabled={busy || !consent}
              onClick={() =>
                void perform(async () => {
                  await request("plan_create", {
                    review: {
                      file_id: file,
                      provider_id: provider,
                      gateway: config.base_url,
                      preview,
                      consent,
                    },
                  });
                  setPreview("");
                  setConsent(false);
                  await load();
                }, "Structured plan validated and saved for review")
              }
            >
              Generate structured plan
            </button>
          </>
        )}
      </section>
      <section className="panel integration-detail">
        <div className="panel-heading">
          <h2>Saved plans</h2>
          <span className="badge muted">{plans.length}</span>
        </div>
        {!plans.length && (
          <p className="helper">No plan has been generated for this project.</p>
        )}
        {plans.map((p) => (
          <article className="integration-plan" key={p.plan.id}>
            <p>
              {files.find((f) => f.id === p.plan.file_id)?.name ??
                "Imported file removed"}
              <span className="helper">
                {" "}
                · {new Date(p.plan.created_at).toLocaleString()}
              </span>
            </p>
            <ol>
              {p.plan.proposal.steps.map((step) => (
                <li key={step.capability_id}>
                  <code>{step.capability_id}</code>
                  <p className="helper">{step.reason}</p>
                </li>
              ))}
            </ol>
            <button
              className="button secondary"
              disabled={busy || !!p.executed_at}
              onClick={() =>
                void perform(async () => {
                  await request("plan_approve", {
                    id: p.plan.id,
                    approved: true,
                  });
                  await load();
                }, "Reviewed plan queued through the shared Job Engine")
              }
            >
              {p.executed_at
                ? "Approval consumed"
                : "Approve local plan and queue"}
            </button>
          </article>
        ))}
      </section>
      <section className="panel integration-detail">
        <div className="panel-heading">
          <h2>Tool execution history</h2>
        </div>
        <JobHistory
          jobs={jobs}
          onCancel={(id) =>
            void perform(async () => {
              await request("job_cancel", { id });
              await load();
            })
          }
        />
        {jobs
          .filter((j) => j.status === "completed")
          .map((j) => (
            <div className="integration-evidence" key={j.id}>
              <span>{j.kind.replaceAll("_", " ")}</span>
              <button
                className="text-button"
                disabled={busy || !config}
                onClick={() =>
                  void perform(async () => {
                    if (!config) return;
                    const text = await request<string>("job_evidence_preview", {
                      id: j.id,
                    });
                    const saved = await request<Reply[]>(
                      "job_interpretations",
                      { id: j.id },
                    );
                    setReply(saved[0]?.text ?? "");
                    setConsent(false);
                    setEvidence({
                      id: j.id,
                      text,
                      gateway: config.base_url,
                      provider,
                    });
                  })
                }
              >
                Review evidence with AI
              </button>
            </div>
          ))}
        <p className="helper">
          Model plans and explanations are unverified suggestions. Tool results
          are persisted observations. Autonomous network research, decompilation
          and offensive execution are unavailable.
        </p>
      </section>
      <Modal
        title="Review selected job evidence"
        open={!!evidence}
        onClose={() => setEvidence(null)}
      >
        {evidence && (
          <>
            <pre className="technical-output">{evidence.text}</pre>
            <p className="helper">Destination: {evidence.gateway}</p>
            <label className="checkbox-label">
              <input
                aria-label="Approve evidence transmission"
                type="checkbox"
                checked={consent}
                onChange={(e) => setConsent(e.target.checked)}
              />
              I approve sending this displayed evidence to the selected
              provider.
            </label>
            <button
              className="button primary"
              disabled={busy || !consent}
              onClick={() =>
                void perform(async () => {
                  const response = await request<Reply>("job_interpret", {
                    review: {
                      job_id: evidence.id,
                      provider_id: evidence.provider,
                      gateway: evidence.gateway,
                      preview: evidence.text,
                      consent,
                    },
                  });
                  setReply(response.text);
                  setConsent(false);
                }, "AI interpretation saved")
              }
            >
              Interpret evidence
            </button>
            {reply && (
              <div className="connection-result">
                <p className="helper">AI interpretation · unverified</p>
                <p>{reply}</p>
              </div>
            )}
          </>
        )}
      </Modal>
    </>
  );
}
