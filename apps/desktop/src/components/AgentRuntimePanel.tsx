import { useEffect, useRef, useState } from "react";
import { request, errorText } from "../api";
import { Button } from "./controls";

type Limits = {
  concurrency: number;
  max_invocations: number;
  max_duration_ms: number;
  max_retries: number;
};
type Step = {
  id: string;
  agent_id: string;
  capability_id: string;
  dependencies: string[];
};
type Preview = {
  session_id: string;
  fingerprint: string;
  limits: Limits;
  steps: Step[];
  limitations: string[];
};
type Runtime = {
  session_id: string;
  status: string;
  stop_reason: string | null;
  revision: number;
  invocations: number;
  limits: Limits;
  deadline_ms: number;
  runtime_cost: number | null;
  network_requests_by_runtime: number;
  model_requests_by_runtime: number;
  tasks: {
    id: string;
    agent_id: string;
    capability_id: string;
    state: string;
    job_id: string;
    attempt: number;
    dependencies: string[];
    error_code: string | null;
  }[];
  messages: {
    id: number;
    agent_id: string;
    message_type: string;
    timestamp: string;
    payload: unknown;
    evidence_ids: string[];
  }[];
  next_message: number | null;
};

export default function AgentRuntimePanel({
  session,
  status,
  enabled,
  readOnly,
  eligible,
  changed,
}: {
  session: string;
  status: string;
  enabled: boolean;
  readOnly: boolean;
  eligible: boolean;
  changed: () => Promise<void>;
}) {
  const [limits, setLimits] = useState<Limits>({
    concurrency: 2,
    max_invocations: 24,
    max_duration_ms: 600000,
    max_retries: 1,
  });
  const [preview, setPreview] = useState<Preview | null>(null);
  const [runtime, setRuntime] = useState<Runtime | null>(null);
  const [approved, setApproved] = useState(false),
    [busy, setBusy] = useState(false),
    [error, setError] = useState("");
  const [after, setAfter] = useState(0);
  const generation = useRef(0);
  useEffect(() => {
    ++generation.current;
    setPreview(null);
    setRuntime(null);
    setApproved(false);
    setAfter(0);
    setError("");
    setBusy(false);
    return () => {
      ++generation.current;
    };
  }, [session]);
  useEffect(() => {
    if (!enabled) return;
    let closed = false;
    const ticket = generation.current;
    const load = async () => {
      try {
        const value = await request<Runtime>("agent_runtime_show", {
          id: session,
          after,
        });
        if (!closed && ticket === generation.current)
          setRuntime((previous) =>
            previous &&
            previous.session_id === value.session_id &&
            previous.revision > value.revision
              ? previous
              : value,
          );
      } catch (e) {
        if (!closed && ticket === generation.current) setError(errorText(e));
      }
    };
    void load();
    const timer = setInterval(() => void load(), 1200);
    return () => {
      closed = true;
      clearInterval(timer);
    };
  }, [session, enabled, after]);
  const act = async (op: string, args: Record<string, unknown>) => {
    const ticket = generation.current;
    setBusy(true);
    setError("");
    try {
      const value = await request<Runtime>(op, args);
      if (ticket !== generation.current) return;
      setRuntime(value);
      setPreview(null);
      setApproved(false);
      await changed();
    } catch (e) {
      if (ticket === generation.current) setError(errorText(e));
    } finally {
      if (ticket === generation.current) setBusy(false);
    }
  };
  const updateLimit = (key: keyof Limits, value: number) => {
    setLimits((p) => ({ ...p, [key]: value }));
    setPreview(null);
    setApproved(false);
  };
  if (!enabled && status !== "awaiting_approval") return null;
  return (
    <section className="panel records-panel" aria-label="Offline agent runtime">
      <h3>Offline agent DAG</h3>
      <p>
        Scope review → independent file analysis → evidence integrity → report.
        Network, MCP, model-driven continuation and new model requests are
        excluded from this runtime.
      </p>
      {error && (
        <p role="alert" className="banner error">
          {error}
        </p>
      )}
      {!enabled && (
        <>
          {!eligible && (
            <p>
              This plan contains capabilities outside the offline DAG. Its
              existing reviewed execution flow remains available.
            </p>
          )}
          <div className="records-toolbar">
            <label>
              Concurrent jobs
              <select
                aria-label="Agent concurrency"
                value={limits.concurrency}
                disabled={busy || readOnly}
                onChange={(e) =>
                  updateLimit("concurrency", Number(e.target.value))
                }
              >
                <option value={1}>1</option>
                <option value={2}>2</option>
              </select>
            </label>
            <label>
              Job budget
              <input
                aria-label="Agent invocation budget"
                type="number"
                min={1}
                max={24}
                value={limits.max_invocations}
                disabled={busy || readOnly}
                onChange={(e) =>
                  updateLimit("max_invocations", Number(e.target.value))
                }
              />
            </label>
            <label>
              Wall-time limit (seconds)
              <input
                aria-label="Agent time budget"
                type="number"
                min={1}
                max={3600}
                value={limits.max_duration_ms / 1000}
                disabled={busy || readOnly}
                onChange={(e) =>
                  updateLimit("max_duration_ms", Number(e.target.value) * 1000)
                }
              />
            </label>
            <label>
              Explicit retry limit
              <input
                aria-label="Agent retry limit"
                type="number"
                min={0}
                max={2}
                value={limits.max_retries}
                disabled={busy || readOnly}
                onChange={(e) =>
                  updateLimit("max_retries", Number(e.target.value))
                }
              />
            </label>
          </div>
          <Button
            disabled={busy || readOnly || !eligible}
            disabledReason="Choose a live target and an offline plan"
            onClick={() =>
              void (async () => {
                const ticket = generation.current;
                setBusy(true);
                setError("");
                try {
                  const p = await request<Preview>("agent_runtime_preview", {
                    id: session,
                    limits,
                  });
                  if (ticket === generation.current) {
                    setPreview(p);
                    setApproved(false);
                  }
                } catch (e) {
                  if (ticket === generation.current) setError(errorText(e));
                } finally {
                  if (ticket === generation.current) setBusy(false);
                }
              })()
            }
          >
            Review offline agent DAG
          </Button>
          {preview && (
            <>
              <ol aria-label="Reviewed agent DAG">
                {preview.steps.map((s) => (
                  <li key={s.id}>
                    <strong>{s.agent_id}</strong> · {s.capability_id} · depends
                    on {s.dependencies.join(", ") || "none"}
                  </li>
                ))}
              </ol>
              <ul>
                {preview.limitations.map((t) => (
                  <li key={t}>{t}</li>
                ))}
              </ul>
              <label className="check">
                <input
                  aria-label="Approve offline agent DAG"
                  type="checkbox"
                  checked={approved}
                  onChange={(e) => setApproved(e.target.checked)}
                />
                I approve these offline tasks, dependency edges and resource
                limits.
              </label>
              <Button
                variant="primary"
                disabled={busy || readOnly || !approved}
                disabledReason="Review and approve the complete offline DAG"
                onClick={() =>
                  void act("agent_runtime_start", {
                    review: {
                      session_id: session,
                      fingerprint: preview.fingerprint,
                      limits: preview.limits,
                      approved,
                    },
                  })
                }
              >
                Start offline agent DAG
              </Button>
            </>
          )}
        </>
      )}
      {runtime && (
        <>
          <p role="status">
            Agent DAG · {runtime.status} ·{" "}
            {runtime.tasks.filter((t) => t.state === "COMPLETED").length}/
            {runtime.tasks.length} completed
          </p>
          <p>
            Invocations: {runtime.invocations}/{runtime.limits.max_invocations}{" "}
            · concurrency: {runtime.limits.concurrency} · runtime model
            requests: {runtime.model_requests_by_runtime} · network requests:{" "}
            {runtime.network_requests_by_runtime}
          </p>
          <p>
            Cost:{" "}
            {runtime.runtime_cost === null
              ? "not reported"
              : runtime.runtime_cost}
            . Planning token usage is reported separately; cloud token/cost
            budgets are not implemented here.
          </p>
          {runtime.stop_reason && (
            <p>Stop reason: {runtime.stop_reason.replaceAll("_", " ")}</p>
          )}
          <p>
            Pause finishes current jobs before stopping new claims. Wall-time
            includes pauses. Interrupted work requires explicit retry.
          </p>
          <div className="records-toolbar">
            {runtime.status === "running" && (
              <Button
                disabled={busy || readOnly}
                onClick={() => void act("agent_runtime_pause", { id: session })}
              >
                Pause after current jobs
              </Button>
            )}
            {runtime.status === "paused" && (
              <Button
                disabled={busy || readOnly}
                onClick={() =>
                  void act("agent_runtime_resume", { id: session })
                }
              >
                Resume offline DAG
              </Button>
            )}
            {["failed", "paused"].includes(runtime.status) &&
              runtime.tasks.some((t) =>
                ["FAILED", "BLOCKED"].includes(t.state),
              ) && (
                <Button
                  disabled={busy || readOnly}
                  onClick={() =>
                    void act("agent_runtime_retry", { id: session })
                  }
                >
                  Retry incomplete offline tasks
                </Button>
              )}
            {["running", "pause_requested", "paused"].includes(
              runtime.status,
            ) && (
              <Button
                disabled={busy}
                onClick={() =>
                  void (async () => {
                    const ticket = generation.current;
                    setBusy(true);
                    try {
                      await request("research_cancel", { id: session });
                      if (ticket === generation.current) await changed();
                    } catch (e) {
                      if (ticket === generation.current) setError(errorText(e));
                    } finally {
                      if (ticket === generation.current) setBusy(false);
                    }
                  })()
                }
              >
                Cancel agent DAG
              </Button>
            )}
          </div>
          <div className="table-scroll">
            <table aria-label="Agent task execution">
              <thead>
                <tr>
                  <th>Agent</th>
                  <th>Capability</th>
                  <th>State</th>
                  <th>Attempt</th>
                  <th>Dependencies</th>
                  <th>Job</th>
                </tr>
              </thead>
              <tbody>
                {runtime.tasks.map((t) => (
                  <tr key={t.id}>
                    <td>{t.agent_id}</td>
                    <td>{t.capability_id}</td>
                    <td>
                      {t.state}
                      {t.error_code && ` · ${t.error_code}`}
                    </td>
                    <td>{t.attempt + 1}</td>
                    <td>
                      {t.dependencies
                        .map((d) => d.split(":").at(-1))
                        .join(", ") || "none"}
                    </td>
                    <td>
                      <code>{t.job_id}</code>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <details>
            <summary>Structured agent events</summary>
            <ol>
              {runtime.messages.map((m) => (
                <li key={m.id}>
                  <strong>
                    {m.agent_id} · {m.message_type}
                  </strong>{" "}
                  · {m.timestamp}
                  <pre>{JSON.stringify(m.payload, null, 2)}</pre>
                  {m.evidence_ids.length > 0 && (
                    <p>Evidence: {m.evidence_ids.join(", ")}</p>
                  )}
                </li>
              ))}
            </ol>
            <Button disabled={after === 0} onClick={() => setAfter(0)}>
              First event page
            </Button>
            <Button
              disabled={runtime.next_message === null}
              onClick={() => setAfter(runtime.next_message!)}
            >
              Next event page
            </Button>
          </details>
        </>
      )}
    </section>
  );
}
