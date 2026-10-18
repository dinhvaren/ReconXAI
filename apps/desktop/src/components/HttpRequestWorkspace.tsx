import { useEffect, useRef, useState } from "react";
import { request, errorText, type Snapshot } from "../api";
import { useJobEvents, type AnalysisJob } from "../research";
import { Modal } from "./ui";

export interface AuthProfile {
  id: string;
  revision: number;
  project_id: string;
  target_id: string;
  name: string;
  role: string;
  kind: string;
  header_name: string;
  origin: string;
  expired: boolean;
  credential_state: string;
  last_check: { status: number; authentication: string } | null;
  cleanup_pending: boolean;
}
export interface DraftInput {
  project_id: string;
  target_id: string;
  source_exchange_id: string | null;
  title: string;
  method: string;
  url: string;
  headers: { name: string; value: string }[];
  content_type: string;
  body: string;
  auth_profile_id: string | null;
  timeout_ms: number;
  follow_redirects: boolean;
  proxy: string | null;
}
export interface Draft extends DraftInput {
  id: string;
  revision: number;
  updated_at: string;
}
interface Plan {
  draft: Draft;
  destination_address: string;
  request_budget: number;
  state_changing: boolean;
  identity: AuthProfile | null;
  fingerprint: string;
  safeguards: string[];
}
const pretty = (value: unknown) => JSON.stringify(value, null, 2);
export default function HttpRequestWorkspace({
  open,
  project,
  target,
  source,
  identity,
  data,
  onClose,
  onResponse,
}: {
  open: boolean;
  project: string;
  target: string;
  source: string | null;
  identity: AuthProfile | null;
  data: Snapshot;
  onClose: () => void;
  onResponse: (id: string) => void;
}) {
  const [draft, setDraft] = useState<DraftInput | null>(null);
  const [drafts, setDrafts] = useState<Draft[]>([]),
    [profiles, setProfiles] = useState<AuthProfile[]>([]),
    [history, setHistory] = useState<AnalysisJob[]>([]);
  const [loading, setLoading] = useState(true);
  const [headers, setHeaders] = useState("[]"),
    [error, setError] = useState(""),
    [notice, setNotice] = useState(""),
    [busy, setBusy] = useState(false),
    [plan, setPlan] = useState<Plan | null>(null),
    [consent, setConsent] = useState(false);
  const version = useRef(0);
  useEffect(() => {
    const current = ++version.current;
    setLoading(true);
    setDrafts([]);
    setProfiles([]);
    if (!open) return;
    setDraft(null);
    setHistory([]);
    setPlan(null);
    setError("");
    setNotice("");
    setConsent(false);
    void (async () => {
      const [requests, identities] = await Promise.all([
        request<Draft[]>("http_drafts", { project, target: target || null }),
        request<AuthProfile[]>("http_auth_profiles", { project, target: null }),
      ]);
      if (current !== version.current) return;
      setDrafts(requests);
      setProfiles(identities);
      if (source) {
        const seed = await request<DraftInput>("http_draft_seed", {
          exchange: source,
          target: target || null,
        });
        if (current !== version.current) return;
        setDraft(seed);
        setHeaders(pretty(seed.headers));
        setNotice(
          "Imported credentials, query values and withheld bodies are not copied. Review and re-enter permitted values before saving.",
        );
      } else if (identity) {
        const t = data.targets?.find((t) => t.id === identity.target_id);
        if (t) {
          setDraft({
            project_id: project,
            target_id: t.id,
            source_exchange_id: null,
            title: `Check ${identity.name}`,
            method: "GET",
            url: t.value,
            headers: [],
            content_type: "",
            body: "",
            auth_profile_id: identity.id,
            timeout_ms: 5000,
            follow_redirects: false,
            proxy: null,
          });
          setHeaders("[]");
        }
      }
    })()
      .catch((e) => {
        if (current === version.current) setError(errorText(e));
      })
      .finally(() => {
        if (current === version.current) setLoading(false);
      });
    return () => {
      version.current += 1;
    };
  }, [open, project, target, source, identity?.id]);
  useJobEvents((job) => {
    if (
      !open ||
      !loaded.current ||
      job.kind !== "manual_http_replay" ||
      job.project_id !== project
    )
      return;
    const current = version.current;
    const draftId = loaded.current.id;
    void request<AnalysisJob[]>("http_replay_history", { id: draftId })
      .then((rows) => {
        if (current === version.current && draftId === loaded.current?.id)
          setHistory(rows);
      })
      .catch((e) => {
        if (current === version.current) setError(errorText(e));
      });
  });
  const edit = (change: Partial<DraftInput>) => {
    setDraft((d) => (d ? { ...d, ...change } : d));
    setPlan(null);
    setConsent(false);
    setNotice("");
  };
  const act = async (fn: () => Promise<void>) => {
    setBusy(true);
    setError("");
    try {
      await fn();
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };
  const save = async () => {
    if (!draft) throw Error("Select a request first");
    const parsed: unknown = JSON.parse(headers);
    if (
      !Array.isArray(parsed) ||
      parsed.some(
        (h) => !h || typeof h.name !== "string" || typeof h.value !== "string",
      )
    )
      throw Error("Headers must be an array of name/value objects");
    // Keep a loaded request ID/revision while editing; save uses optimistic concurrency.
    const value = await request<Draft>("http_draft_save", {
      id: loaded.current?.id ?? null,
      revision: loaded.current?.revision ?? null,
      input: {
        project_id: draft.project_id,
        target_id: draft.target_id,
        source_exchange_id: draft.source_exchange_id,
        title: draft.title,
        method: draft.method,
        url: draft.url,
        headers: parsed,
        content_type: draft.content_type,
        body: draft.body,
        auth_profile_id: draft.auth_profile_id,
        timeout_ms: draft.timeout_ms,
        follow_redirects: draft.follow_redirects,
        proxy: draft.proxy,
      },
    });
    loaded.current = value;
    setDraft(value);
    setNotice("Request saved. No network request sent.");
    setDrafts(
      await request<Draft[]>("http_drafts", {
        project,
        target: target || null,
      }),
    );
    return value;
  };
  const loaded = useRef<Draft | null>(null);
  useEffect(() => {
    loaded.current = null;
  }, [open, source, identity?.id, project, target]);
  return (
    <Modal
      title="Manual loopback HTTP request"
      open={open}
      onClose={onClose}
      dismissDisabled={busy}
    >
      <div className="form http-dialog-content">
        <p>
          One explicitly approved request to an owned loopback service. Proxy
          and redirect following are disabled. TLS verification stays enabled.
        </p>
        {error && <p role="alert">{error}</p>}
        {notice && <p role="status">{notice}</p>}
        <label className="field">
          <span>Saved requests</span>
          <select
            aria-label="Saved HTTP request"
            value={loaded.current?.id ?? ""}
            disabled={busy || loading}
            onChange={(e) =>
              void act(async () => {
                const value = drafts.find((d) => d.id === e.target.value);
                if (!value) return;
                loaded.current = value;
                setDraft(value);
                setHeaders(pretty(value.headers));
                setPlan(null);
                setConsent(false);
                setHistory(
                  await request<AnalysisJob[]>("http_replay_history", {
                    id: value.id,
                  }),
                );
              })
            }
          >
            <option value="">Select a saved draft</option>
            {drafts.map((d) => (
              <option key={d.id} value={d.id}>
                {d.title} · {d.method} {d.url}
              </option>
            ))}
          </select>
        </label>
        {!draft && (
          <p className="helper">
            Select an imported capture and choose Edit request, choose a saved
            draft, or start an identity check from Authentication profiles.
          </p>
        )}
        {draft && (
          <>
            <div className="form-grid">
              <label>
                Request title
                <input
                  aria-label="HTTP request title"
                  value={draft.title}
                  disabled={busy}
                  onChange={(e) => edit({ title: e.target.value })}
                />
              </label>
              <label>
                Method
                <select
                  aria-label="HTTP request method"
                  value={draft.method}
                  disabled={busy}
                  onChange={(e) => edit({ method: e.target.value })}
                >
                  {[
                    "GET",
                    "POST",
                    "PUT",
                    "PATCH",
                    "DELETE",
                    "HEAD",
                    "OPTIONS",
                  ].map((m) => (
                    <option key={m}>{m}</option>
                  ))}
                </select>
              </label>
            </div>
            <label className="field">
              <span>URL — scheme, host, port, path and query</span>
              <input
                aria-label="HTTP request URL"
                value={draft.url}
                disabled={busy}
                onChange={(e) => edit({ url: e.target.value })}
              />
            </label>
            <p className="helper">
              Target:{" "}
              {data.targets?.find((t) => t.id === draft.target_id)?.name ??
                draft.target_id}
              . POST, PUT, PATCH and DELETE require Lab Research for both app
              and target.
            </p>
            <div className="form-grid">
              <label>
                Test identity
                <select
                  aria-label="HTTP test identity"
                  value={draft.auth_profile_id ?? ""}
                  disabled={busy}
                  onChange={(e) =>
                    edit({ auth_profile_id: e.target.value || null })
                  }
                >
                  <option value="">Anonymous — no credentials</option>
                  {profiles
                    .filter((p) => p.target_id === draft.target_id)
                    .map((p) => (
                      <option
                        key={p.id}
                        value={p.id}
                        disabled={p.expired || p.credential_state === "missing"}
                      >
                        {p.name} · {p.role}
                        {p.expired ? " · Expired" : ""}
                      </option>
                    ))}
                </select>
              </label>
              <label>
                Timeout (ms)
                <input
                  type="number"
                  min={500}
                  max={10000}
                  aria-label="HTTP request timeout"
                  value={draft.timeout_ms}
                  disabled={busy}
                  onChange={(e) => edit({ timeout_ms: Number(e.target.value) })}
                />
              </label>
            </div>
            <label className="field">
              <span>Public headers (JSON name/value array)</span>
              <textarea
                className="technical"
                aria-label="HTTP request headers"
                rows={3}
                value={headers}
                disabled={busy}
                onChange={(e) => {
                  setHeaders(e.target.value);
                  edit({});
                }}
              />
            </label>
            <p className="helper">
              Accept, Cache-Control, If-None-Match, If-Modified-Since and
              X-Request-ID are supported. Cookies and authentication headers
              come only from the selected secure identity.
            </p>
            <label className="field">
              <span>Content-Type</span>
              <input
                aria-label="HTTP request content type"
                value={draft.content_type}
                disabled={busy}
                placeholder="application/json"
                onChange={(e) => edit({ content_type: e.target.value })}
              />
            </label>
            <label className="field">
              <span>Request body — JSON, URL-encoded form or raw UTF-8</span>
              <textarea
                className="technical"
                aria-label="HTTP request body"
                rows={7}
                value={draft.body}
                disabled={busy}
                onChange={(e) => edit({ body: e.target.value })}
              />
            </label>
            <p className="helper">
              Drafts reject credentials and redacted placeholders. Multipart
              uploads are not implemented. Redirect responses are captured,
              never followed.
            </p>
            <div className="form-actions">
              <button
                className="button secondary"
                disabled={busy}
                onClick={() =>
                  void act(async () => {
                    await save();
                  })
                }
              >
                Save request
              </button>
              <button
                className="button primary"
                disabled={busy}
                onClick={() =>
                  void act(async () => {
                    const saved = await save();
                    setPlan(
                      await request<Plan>("http_replay_preview", {
                        id: saved.id,
                      }),
                    );
                    setConsent(false);
                  })
                }
              >
                Review send
              </button>
            </div>
            {plan && (
              <section aria-label="HTTP execution review" className="panel">
                <h3>Review one actual HTTP request</h3>
                <p>
                  <strong>{plan.draft.method}</strong>{" "}
                  <code>{plan.draft.url}</code>
                </p>
                <p>
                  Pinned socket: <code>{plan.destination_address}</code> ·
                  Budget: {plan.request_budget}
                </p>
                <p>
                  Identity: {plan.identity?.name ?? "Anonymous"} · Stored
                  credentials remain hidden.
                </p>
                <ul>
                  {plan.safeguards.map((s) => (
                    <li key={s}>{s}</li>
                  ))}
                </ul>
                <label className="inline-check">
                  <input
                    type="checkbox"
                    aria-label="Approve exact HTTP request"
                    checked={consent}
                    onChange={(e) => setConsent(e.target.checked)}
                  />
                  I own this local service and approve this exact request and
                  any state change.
                </label>
                <button
                  className="button primary"
                  disabled={busy || !consent}
                  onClick={() =>
                    void act(async () => {
                      await request("http_replay_enqueue", {
                        approval: {
                          draft_id: plan.draft.id,
                          fingerprint: plan.fingerprint,
                          approved: true,
                        },
                      });
                      setPlan(null);
                      setConsent(false);
                      setNotice(
                        "Approved request queued. Cancellation cannot undo an action already received by the service.",
                      );
                      setHistory(
                        await request<AnalysisJob[]>("http_replay_history", {
                          id: plan.draft.id,
                        }),
                      );
                    })
                  }
                >
                  Send approved request
                </button>
              </section>
            )}
            <section aria-label="HTTP replay history">
              <h3>Actual execution history</h3>
              {!history.length && (
                <p className="helper">
                  No request has run for this saved draft.
                </p>
              )}
              {history.map((job) => {
                const result = job.result as {
                  exchange_id?: string;
                  observations?: { status: number; elapsed_ms: number };
                } | null;
                return (
                  <article className="http-item" key={job.id}>
                    <strong>{job.status}</strong>
                    <p>
                      {job.stage} · {job.progress}%
                    </p>
                    {job.error_message && (
                      <p role="alert">{job.error_message}</p>
                    )}
                    {result?.observations && (
                      <p>
                        HTTP {result.observations.status} ·{" "}
                        {result.observations.elapsed_ms.toFixed(1)} ms
                      </p>
                    )}
                    {["queued", "running"].includes(job.status) && (
                      <button
                        className="button secondary"
                        disabled={busy}
                        onClick={() =>
                          void act(async () => {
                            await request("job_cancel", { id: job.id });
                            setHistory(
                              await request<AnalysisJob[]>(
                                "http_replay_history",
                                { id: loaded.current?.id },
                              ),
                            );
                          })
                        }
                      >
                        Cancel request
                      </button>
                    )}
                    {result?.exchange_id && (
                      <button
                        className="button primary"
                        onClick={() => {
                          onResponse(result.exchange_id!);
                          onClose();
                        }}
                      >
                        Open captured response
                      </button>
                    )}
                    {job.status === "completed" && (
                      <button
                        className="button secondary"
                        disabled={busy}
                        onClick={() =>
                          void act(async () => {
                            const evidence = await request<{ id: string }>(
                              "evidence_from_job",
                              { id: job.id },
                            );
                            setNotice(`Result evidence saved: ${evidence.id}`);
                          })
                        }
                      >
                        Save result evidence
                      </button>
                    )}
                    <details>
                      <summary>Result and execution log</summary>
                      <pre className="http-pre">
                        {pretty({ result: job.result, logs: job.logs })}
                      </pre>
                    </details>
                  </article>
                );
              })}
            </section>
          </>
        )}
      </div>
    </Modal>
  );
}
