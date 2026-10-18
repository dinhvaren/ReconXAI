import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { request, errorText, type Snapshot } from "../api";
import { selectPaths, useJobEvents } from "../research";
import { Heading, Modal, type Perform } from "../components/ui";
import type {
  ApiOperation,
  HttpComparison,
  HttpExchange,
  HttpMessage,
  HttpWorkspace,
} from "../http";
import "../http.css";

const pretty = (value: unknown) => JSON.stringify(value, null, 2);
function JsonTree({ value }: { value: unknown }) {
  let count = 0;
  const render = (v: unknown, depth: number): React.ReactNode => {
    if (++count > 500 || depth > 16)
      return <span>Additional structure omitted; use raw text.</span>;
    if (v === null || typeof v !== "object")
      return <span>{JSON.stringify(v)}</span>;
    return (
      <ul className="http-json-tree">
        {Object.entries(v)
          .slice(0, 60)
          .map(([key, child]) => (
            <li key={key}>
              <details>
                <summary>
                  <code>{key}</code> · {child === null ? "null" : typeof child}
                </summary>
                {render(child, depth + 1)}
              </details>
            </li>
          ))}
      </ul>
    );
  };
  return render(value, 0);
}
function Pairs({ rows }: { rows: { name: string; value: string }[] }) {
  return rows.length ? (
    <div className="table-scroll">
      <table className="data-table">
        <thead>
          <tr>
            <th>Name</th>
            <th>Value</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((r, i) => (
            <tr key={i}>
              <td className="technical">{r.name}</td>
              <td>{r.value}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  ) : (
    <p className="helper">None recorded.</p>
  );
}
export default function HttpPage({
  page,
  data,
  projectId,
  perform,
  busy,
  navigate,
}: {
  page: "Web Security" | "API Security";
  data: Snapshot;
  projectId?: string;
  perform: Perform;
  busy: boolean;
  navigate: (page: string) => void;
}) {
  const [project, setProject] = useState(projectId ?? "");
  const [target, setTarget] = useState("");
  const [workspace, setWorkspace] = useState<HttpWorkspace | null>(null);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const requestVersion = useRef(0);
  const [search, setSearch] = useState("");
  const [offset, setOffset] = useState(0);
  const [view, setView] = useState("Traffic");
  const [selected, setSelected] = useState("");
  const [tab, setTab] = useState("Headers");
  const [direction, setDirection] = useState("response");
  const [jsonView, setJsonView] = useState(false);
  const [width, setWidth] = useState(43);
  const [sort, setSort] = useState("time");
  const [pending, setPending] = useState<string[]>([]);
  const [kind, setKind] = useState("har");
  const [title, setTitle] = useState("Offline traffic import");
  const [identity, setIdentity] = useState("");
  const [role, setRole] = useState("");
  const [note, setNote] = useState("");
  const [right, setRight] = useState("");
  const [owner, setOwner] = useState("");
  const [comparison, setComparison] = useState<HttpComparison | null>(null);
  const [aiReview, setAiReview] = useState<{
    job_id: string;
    preview: string;
  } | null>(null);
  const [aiProvider, setAiProvider] = useState("");
  const [consent, setConsent] = useState(false);
  const [aiReply, setAiReply] = useState("");
  useEffect(() => {
    setProject(projectId ?? "");
    setTarget("");
    setOffset(0);
    setSelected("");
  }, [projectId]);
  const load = useCallback(async () => {
    if (!project) {
      setWorkspace(null);
      return;
    }
    setLoading(true);
    const version = ++requestVersion.current;
    try {
      const result = await request<HttpWorkspace>("http_workspace", {
        project,
        target: target || null,
        search,
        offset,
      });
      if (version === requestVersion.current) {
        setWorkspace(result);
        setError("");
      }
    } catch (e) {
      if (version === requestVersion.current) setError(errorText(e));
    } finally {
      if (version === requestVersion.current) setLoading(false);
    }
  }, [project, target, search, offset]);
  useJobEvents((job) => {
    if (job.project_id === project && job.kind === "offline_http_review")
      void load();
  });
  useEffect(() => {
    requestVersion.current += 1;
    setWorkspace(null);
    setSelected("");
    setComparison(null);
  }, [project, target]);
  useEffect(() => {
    let cancelled = false;
    const timer = setTimeout(() => {
      if (!cancelled) void load();
    }, 150);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [load]);
  const drop = useRef((paths: string[]) => {
    setPending(paths);
  });
  drop.current = (paths) => {
    if (!project) {
      setError("Select a project before importing files.");
      return;
    }
    if (paths.length > 10) {
      setError("Import at most 10 files at a time.");
      return;
    }
    setKind(
      paths.every((p) => p.toLowerCase().endsWith(".har")) ? "har" : "openapi",
    );
    setPending(paths);
    setError("");
  };
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    try {
      void getCurrentWebviewWindow()
        .onDragDropEvent((e) => {
          if (!disposed && e.payload.type === "drop")
            drop.current(e.payload.paths);
        })
        .then((fn) => {
          if (disposed) fn();
          else unlisten = fn;
        })
        .catch(() => {});
    } catch {
      /* Unit-test DOM has no native bridge. */
    }
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
  const pick = async (type: string) => {
    try {
      const paths = await selectPaths(false);
      if (paths.length) {
        drop.current(paths);
        setKind(type);
      }
    } catch (e) {
      setError(errorText(e));
    }
  };
  const exchange = workspace?.traffic.find((e) => e.id === selected);
  const targetName = (id: string | null) =>
    data.targets?.find((t) => t.id === id)?.name ??
    (id ? "Associated target unavailable" : "No associated target");
  const operation = workspace?.operations.find((o) => o.id === selected);
  useEffect(() => {
    let disposed = false;
    setNote("");
    if (exchange)
      void request<string>("http_note", { id: exchange.id, note: null })
        .then((text) => {
          if (!disposed) setNote(text);
        })
        .catch((e) => {
          if (!disposed) setError(errorText(e));
        });
    return () => {
      disposed = true;
    };
  }, [exchange?.id]);
  const act = (fn: () => Promise<void>, message?: string) =>
    void perform(async () => {
      setError("");
      try {
        await fn();
        await load();
      } catch (e) {
        setError(errorText(e));
        throw e;
      }
    }, message);
  const traffic = [...(workspace?.traffic ?? [])].sort((a, b) =>
    sort === "status"
      ? a.status - b.status
      : sort === "endpoint"
        ? `${a.host}${a.path}`.localeCompare(`${b.host}${b.path}`)
        : b.started_at.localeCompare(a.started_at),
  );
  const message: HttpMessage | undefined =
    exchange?.[direction as "request" | "response"];
  const renderOperation = (op: ApiOperation) => (
    <>
      <h2>
        {op.method} <code>{op.path}</code>
      </h2>
      <p>{op.summary || "No description supplied."}</p>
      <p className="helper">
        Documentation is imported data, not observed behavior.
      </p>
      <pre className="http-pre">{pretty(op)}</pre>
    </>
  );
  return (
    <>
      <Heading
        eyebrow="Offline research"
        title={page}
        description="Inspect captured traffic and API definitions. Imports never send requests or upload evidence."
        action={
          <div className="form-actions">
            <button
              className="button secondary"
              disabled={busy || !project}
              onClick={() => void pick("har")}
            >
              Import HAR
            </button>
            <button
              className="button primary"
              disabled={busy || !project}
              onClick={() => void pick("openapi")}
            >
              Import OpenAPI
            </button>
          </div>
        }
      />
      <section
        className="panel http-controls"
        aria-label="HTTP workspace controls"
      >
        <label>
          Research project
          <select
            aria-label="HTTP project"
            value={project}
            onChange={(e) => {
              setProject(e.target.value);
              setTarget("");
              setOffset(0);
              setSelected("");
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
          Associated target
          <select
            aria-label="HTTP target"
            value={target}
            onChange={(e) => {
              setTarget(e.target.value);
              setOffset(0);
              setSelected("");
            }}
          >
            <option value="">All project captures / no target on import</option>
            {data.targets
              ?.filter((t) => t.project_id === project)
              .map((t) => (
                <option key={t.id} value={t.id}>
                  {t.name}
                </option>
              ))}
          </select>
        </label>
        <button
          className="button secondary"
          disabled={!project || busy || loading}
          onClick={() => void load()}
        >
          Refresh HTTP workspace
        </button>
        <button
          className="button secondary"
          disabled={!workspace || busy}
          onClick={() =>
            act(async () => {
              const paths = await selectPaths(
                false,
                true,
                "reconx-http-redacted.json",
              );
              if (paths[0])
                await request("http_export", { project, path: paths[0] });
            }, "Redacted HTTP export saved")
          }
        >
          Export redacted inventory
        </button>
      </section>
      {error && <p role="alert">{error}</p>}
      {!project ? (
        <div className="panel empty-state">
          <h2>Select a research project</h2>
          <p>
            Choose an existing project, then import a HAR capture or OpenAPI
            definition. No sample traffic is generated.
          </p>
        </div>
      ) : (
        <>
          <div
            role="tablist"
            aria-label="HTTP workspace views"
            className="http-tabs"
          >
            {[
              "Traffic",
              "Endpoints",
              "API definitions",
              "Imports",
              "Authorization comparison",
            ].map((v) => (
              <button
                key={v}
                role="tab"
                aria-selected={view === v}
                onClick={() => {
                  setView(v);
                  setSelected("");
                }}
              >
                {v}
              </button>
            ))}
          </div>
          {loading && <p role="status">Loading saved HTTP records…</p>}
          {workspace && (
            <>
              <div className="http-toolbar">
                <label>
                  Search imported traffic
                  <input
                    aria-label="Search imported traffic"
                    type="search"
                    value={search}
                    placeholder="Host, method, status, body text or identity"
                    onChange={(e) => {
                      setSearch(e.target.value);
                      setOffset(0);
                    }}
                  />
                </label>
                <label>
                  Sort captures
                  <select
                    value={sort}
                    onChange={(e) => setSort(e.target.value)}
                  >
                    <option value="time">Timestamp</option>
                    <option value="endpoint">Endpoint</option>
                    <option value="status">Status</option>
                  </select>
                </label>
                <label>
                  Explorer width
                  <input
                    aria-label="HTTP explorer width"
                    type="range"
                    min="30"
                    max="55"
                    value={width}
                    onChange={(e) => setWidth(Number(e.target.value))}
                  />
                </label>
              </div>
              <p className="helper">
                Drag files into this workspace to review an offline import.
                Credential values are masked; non-JSON bodies are withheld. Raw
                originals remain restricted evidence.
              </p>
              <div
                className="http-panes"
                style={{
                  gridTemplateColumns: `minmax(220px,${width}%) minmax(0,1fr)`,
                }}
              >
                <section
                  className="panel http-explorer"
                  aria-label="Endpoint Explorer"
                >
                  {view === "Traffic" && (
                    <>
                      <h2>Traffic captures ({workspace.total})</h2>
                      {traffic.length === 0 ? (
                        <p>No matching captures. Import a HAR file to begin.</p>
                      ) : (
                        traffic.map((e) => (
                          <button
                            key={e.id}
                            className={`http-item ${selected === e.id ? "active" : ""}`}
                            onClick={() => {
                              setSelected(e.id);
                              setComparison(null);
                            }}
                          >
                            <span>
                              <b>{e.method}</b> {e.status || "No response"}
                            </span>
                            <code>
                              {new URL(e.url).origin}
                              {e.path}
                            </code>
                            <small>
                              {e.identity || "Unlabelled identity"} ·{" "}
                              {e.elapsed_ms.toFixed(1)} ms
                            </small>
                          </button>
                        ))
                      )}
                      <div className="form-actions">
                        <button
                          className="button ghost"
                          disabled={offset === 0}
                          onClick={() => setOffset(Math.max(0, offset - 100))}
                        >
                          Previous captures
                        </button>
                        <button
                          className="button ghost"
                          disabled={
                            offset + workspace.page_size >= workspace.total
                          }
                          onClick={() => setOffset(offset + 100)}
                        >
                          Next captures
                        </button>
                      </div>
                    </>
                  )}
                  {view === "Endpoints" && (
                    <>
                      <h2>Endpoint inventory</h2>
                      {workspace.endpoints.length === 0 ? (
                        <p>No observed endpoints.</p>
                      ) : (
                        workspace.endpoints.map((e, i) => (
                          <div className="http-item" key={i}>
                            <small>{targetName(e.target_id)}</small>
                            <span>
                              {e.method} · {e.requests} capture(s)
                            </span>
                            <code>
                              {e.origin}
                              {e.path}
                            </code>
                            <small>
                              {e.documented
                                ? "Matches API documentation"
                                : "Undocumented observation"}
                            </small>
                          </div>
                        ))
                      )}
                      <p className="helper">
                        Documentation matches are observations, not verified
                        vulnerabilities.
                      </p>
                    </>
                  )}
                  {view === "API definitions" && (
                    <>
                      <h2>Documented operations</h2>
                      {workspace.operations.length === 0 ? (
                        <p>Import OpenAPI 3.0/3.1 JSON or YAML.</p>
                      ) : (
                        workspace.operations.map((o) => (
                          <button
                            className={`http-item ${selected === o.id ? "active" : ""}`}
                            key={o.id}
                            onClick={() => setSelected(o.id)}
                          >
                            <span>{o.method}</span>
                            <code>{o.path}</code>
                            <small>{o.summary || "No summary"}</small>
                          </button>
                        ))
                      )}
                    </>
                  )}
                  {view === "Imports" && (
                    <>
                      <h2>Import history</h2>
                      {workspace.imports.map((i) => (
                        <article key={i.id} className="http-item">
                          <h3>{i.title}</h3>
                          <p>
                            {i.kind} · {new Date(i.created_at).toLocaleString()}
                          </p>
                          <code>{i.sha256}</code>
                          <button
                            className="button ghost"
                            disabled={busy}
                            onClick={() =>
                              act(async () => {
                                await request("evidence_verify", {
                                  id: i.evidence_id,
                                });
                              }, "Original import integrity verified")
                            }
                          >
                            Verify original integrity
                          </button>
                          <details>
                            <summary>Import metadata and components</summary>
                            <pre className="http-pre">{pretty(i.summary)}</pre>
                          </details>
                        </article>
                      ))}
                    </>
                  )}
                  {view === "Authorization comparison" && (
                    <>
                      <h2>Captured identity matrix</h2>
                      <p className="helper">
                        Labels describe supplied captures. This does not
                        authenticate an identity, replay traffic or prove access
                        control.
                      </p>
                      <p className="helper">
                        Summary covers all imported captures in the selected
                        target, including captures outside the current traffic
                        page. Statuses are recorded observations, not expected
                        permissions.
                      </p>
                      <div className="table-scroll">
                        <table
                          className="data-table http-matrix"
                          aria-label="Captured authorization observations"
                        >
                          <thead>
                            <tr>
                              <th>Endpoint / identity</th>
                              <th>Captured responses</th>
                            </tr>
                          </thead>
                          <tbody>
                            {(workspace.authorization_matrix ?? []).map(
                              (row, i) => (
                                <tr key={i}>
                                  <td>
                                    <small>{targetName(row.target_id)}</small>
                                    <code>
                                      {row.method} {row.origin}
                                      {row.path}
                                    </code>
                                    <div>
                                      {row.identity || "Unlabelled"} ·{" "}
                                      {row.role || "No role"}
                                    </div>
                                  </td>
                                  <td>
                                    <span>
                                      Status {row.statuses.join(", ")}
                                    </span>
                                    <div>
                                      {row.capture_count} capture(s) ·{" "}
                                      {row.observed_paths} path(s)
                                    </div>
                                    <small>
                                      {row.body_variants} distinct response body
                                      hash(es)
                                    </small>
                                  </td>
                                </tr>
                              ),
                            )}
                          </tbody>
                        </table>
                      </div>
                      <h3>Captures on this page</h3>
                      {traffic.map((e) => (
                        <button
                          className="http-item"
                          key={e.id}
                          onClick={() => {
                            setSelected(e.id);
                            setTab("Diff");
                          }}
                        >
                          <span>
                            {e.identity || "Unlabelled"} · {e.role || "No role"}
                          </span>
                          <code>
                            {e.method} {e.path}
                          </code>
                          <small>Captured status {e.status}</small>
                        </button>
                      ))}
                      <h3>Saved comparisons</h3>
                      {workspace.comparisons.map((c) => (
                        <button
                          key={c.id}
                          className="http-item"
                          onClick={() => setComparison(c)}
                        >
                          <span>
                            {c.left_identity || "Capture A"} ↔{" "}
                            {c.right_identity || "Capture B"}
                          </span>
                          <small>{c.status.replaceAll("_", " ")}</small>
                        </button>
                      ))}
                    </>
                  )}
                </section>
                <section
                  className="panel http-inspector"
                  aria-label="HTTP inspector"
                >
                  {operation ? (
                    renderOperation(operation)
                  ) : exchange ? (
                    <>
                      <h2>
                        {exchange.method}{" "}
                        <span>
                          {exchange.host}
                          {exchange.path}
                        </span>
                      </h2>
                      <p className="helper">
                        {exchange.status} · {exchange.elapsed_ms.toFixed(1)} ms
                        · {new Date(exchange.started_at).toLocaleString()}
                      </p>
                      <p>
                        Authentication indicators:{" "}
                        {exchange.authentication.join(", ") || "None captured"}
                      </p>
                      <div
                        className="http-tabs"
                        role="tablist"
                        aria-label="HTTP inspection tabs"
                      >
                        {[
                          "Headers",
                          "Cookies",
                          "Parameters",
                          "Body",
                          "Raw text",
                          "Diff",
                          "Notes",
                        ].map((t) => (
                          <button
                            key={t}
                            role="tab"
                            aria-selected={tab === t}
                            onClick={() => setTab(t)}
                          >
                            {t}
                          </button>
                        ))}
                      </div>
                      {["Headers", "Cookies", "Body"].includes(tab) && (
                        <label>
                          Message
                          <select
                            aria-label="HTTP message direction"
                            value={direction}
                            onChange={(e) => setDirection(e.target.value)}
                          >
                            <option value="request">Request</option>
                            <option value="response">Response</option>
                          </select>
                        </label>
                      )}
                      {tab === "Headers" && (
                        <Pairs rows={message?.headers ?? []} />
                      )}
                      {tab === "Cookies" && (
                        <>
                          <p className="helper">
                            All cookie values are masked.
                          </p>
                          <Pairs rows={message?.cookies ?? []} />
                        </>
                      )}
                      {tab === "Parameters" && (
                        <>
                          <h3>Query parameter names</h3>
                          <pre className="http-pre">
                            {pretty(exchange.query_parameters)}
                          </pre>
                          <h3>Documented path parameter names</h3>
                          <pre className="http-pre">
                            {pretty(exchange.request.path_parameters ?? [])}
                          </pre>
                          <p className="helper">
                            Names are derived from matched OpenAPI templates.
                            Query and path parameter values are not displayed
                            here.
                          </p>
                        </>
                      )}
                      {tab === "Body" && (
                        <>
                          <p className="helper">
                            {message?.body.mime_type || "Unknown content type"}{" "}
                            · {message?.body.decoded_size ?? 0} decoded bytes
                          </p>
                          <label className="inline-check">
                            <input
                              type="checkbox"
                              checked={jsonView}
                              onChange={(e) => setJsonView(e.target.checked)}
                            />
                            JSON tree
                          </label>
                          {jsonView ? (
                            <JsonTree value={message?.body.preview} />
                          ) : (
                            <pre className="http-pre">
                              {pretty(message?.body.preview)}
                            </pre>
                          )}
                        </>
                      )}
                      {tab === "Raw text" && (
                        <>
                          <p className="helper">
                            Redacted normalized capture, not original wire
                            bytes.
                          </p>
                          <pre className="http-pre">{pretty(exchange)}</pre>
                        </>
                      )}
                      {tab === "Notes" && (
                        <>
                          <label>
                            Analysis note
                            <textarea
                              aria-label="HTTP analysis note"
                              value={note}
                              maxLength={16 * 1024}
                              onChange={(e) => setNote(e.target.value)}
                            />
                          </label>
                          <button
                            className="button secondary"
                            disabled={busy}
                            onClick={() =>
                              act(async () => {
                                setNote(
                                  await request<string>("http_note", {
                                    id: exchange.id,
                                    note,
                                  }),
                                );
                              }, "HTTP analysis note saved")
                            }
                          >
                            Save HTTP note
                          </button>
                        </>
                      )}
                      {tab === "Diff" && (
                        <>
                          <p className="helper">
                            Compare retained responses. A difference alone is
                            not a vulnerability.
                          </p>
                          <label>
                            Compare with
                            <select
                              aria-label="Compare HTTP capture"
                              value={right}
                              onChange={(e) => setRight(e.target.value)}
                            >
                              <option value="">
                                Select a different capture
                              </option>
                              {traffic
                                .filter((e) => e.id !== exchange.id)
                                .map((e) => (
                                  <option value={e.id} key={e.id}>
                                    {e.identity || e.id.slice(0, 8)} ·{" "}
                                    {e.method} {e.path} · {e.status}
                                  </option>
                                ))}
                            </select>
                          </label>
                          <label>
                            Ownership field (optional top-level JSON key)
                            <input
                              aria-label="Ownership field"
                              maxLength={100}
                              value={owner}
                              onChange={(e) => setOwner(e.target.value)}
                            />
                          </label>
                          <button
                            className="button secondary"
                            disabled={busy || !right}
                            onClick={() =>
                              act(async () => {
                                setComparison(
                                  await request<HttpComparison>(
                                    "http_compare",
                                    {
                                      input: {
                                        left: exchange.id,
                                        right,
                                        owner_field: owner,
                                      },
                                    },
                                  ),
                                );
                              }, "Offline comparison saved as evidence")
                            }
                          >
                            Compare captured responses
                          </button>
                          {right && (
                            <div className="http-diff">
                              <div>
                                <h3>Selected response</h3>
                                <pre className="http-pre">
                                  {pretty(exchange.response.body.preview)}
                                </pre>
                              </div>
                              <div>
                                <h3>Comparison response</h3>
                                <pre className="http-pre">
                                  {pretty(
                                    traffic.find((e) => e.id === right)
                                      ?.response.body.preview,
                                  )}
                                </pre>
                              </div>
                            </div>
                          )}
                        </>
                      )}
                      <hr />
                      <h3>Local capture analysis</h3>
                      <button
                        className="button secondary"
                        disabled={busy}
                        onClick={() =>
                          act(async () => {
                            await request("http_review_enqueue", {
                              id: exchange.id,
                            });
                          }, "Captured-header review job queued")
                        }
                      >
                        Analyze captured headers
                      </button>
                      {workspace.analysis_jobs
                        ?.filter((j) => j.exchange_id === exchange.id)
                        .map(({ job }) => (
                          <article className="http-item" key={job.id}>
                            <span>Captured-header review · {job.status}</span>
                            <small>
                              {job.stage} · {job.progress}%
                            </small>
                            {job.error_message && (
                              <p role="alert">{job.error_message}</p>
                            )}
                            {["queued", "running"].includes(job.status) && (
                              <button
                                className="button ghost"
                                disabled={busy}
                                onClick={() =>
                                  act(async () => {
                                    await request("job_cancel", { id: job.id });
                                  }, "Cancellation requested")
                                }
                              >
                                Cancel capture review
                              </button>
                            )}
                            <details>
                              <summary>Actual result and execution log</summary>
                              <pre className="http-pre">
                                {pretty({ result: job.result, logs: job.logs })}
                              </pre>
                            </details>
                            <button
                              className="button ghost"
                              disabled={
                                busy ||
                                job.status !== "completed" ||
                                !data.providers.some((p) => p.enabled)
                              }
                              onClick={() =>
                                act(async () => {
                                  const profile = await request<{ id: string }>(
                                    "ai_agent_provider",
                                    { agent: "web" },
                                  );
                                  setAiProvider(profile.id);
                                  setAiReview({
                                    job_id: job.id,
                                    preview: await request<string>(
                                      "job_evidence_preview",
                                      { id: job.id },
                                    ),
                                  });
                                  setConsent(false);
                                  setAiReply("");
                                })
                              }
                            >
                              Review result with AI
                            </button>
                            <p className="helper">
                              AI review needs a configured model and separate
                              approval of the exact preview.
                            </p>
                          </article>
                        ))}
                      <button
                        className="button secondary"
                        disabled
                        title="Active replay is not implemented in this offline workspace"
                      >
                        Replay request — unavailable
                      </button>
                      <p className="helper">
                        Imported credentials and traffic are never automatically
                        sent to a target or AI provider.
                      </p>
                    </>
                  ) : (
                    !comparison && (
                      <>
                        <h2>Inspect your research input</h2>
                        <p>
                          Select a capture or documented operation. Import files
                          to populate this workspace with real records.
                        </p>
                      </>
                    )
                  )}
                  {comparison && (
                    <section aria-label="HTTP comparison result">
                      <h2>Comparison · Needs manual review</h2>
                      <pre className="http-pre">{pretty(comparison)}</pre>
                      <p className="helper">
                        Evidence: <code>{comparison.evidence_id}</code>.
                        Original captures are linked by record ID.
                      </p>
                      <button
                        className="button primary"
                        disabled={busy}
                        onClick={() =>
                          act(async () => {
                            await request("http_comparison_finding", {
                              id: comparison.id,
                            });
                          }, "Finding saved as Needs Verification")
                        }
                      >
                        Create review finding
                      </button>
                      <button
                        className="button ghost"
                        onClick={() => navigate("Findings")}
                      >
                        Open Findings
                      </button>
                      <button
                        className="button ghost"
                        onClick={() => navigate("Reports")}
                      >
                        Open Reports
                      </button>
                    </section>
                  )}
                </section>
              </div>
            </>
          )}
        </>
      )}
      <Modal
        open={!!aiReview}
        title="Review captured evidence with AI"
        dismissDisabled={busy}
        onClose={() => setAiReview(null)}
      >
        {aiReview && (
          <div className="form">
            <label>
              Analysis model
              <select
                aria-label="HTTP analysis model"
                value={aiProvider}
                onChange={(e) => {
                  setAiProvider(e.target.value);
                  setConsent(false);
                }}
              >
                {data.providers
                  .filter((p) => p.enabled)
                  .map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name} · {p.model}
                    </option>
                  ))}
              </select>
            </label>
            <p>
              Selected destination:{" "}
              <code>
                {data.providers.find((p) => p.id === aiProvider)?.base_url}
              </code>
            </p>
            <p className="helper">
              The exact preview below is sent only after approval. Model output
              is unverified interpretation, separate from captured evidence.
            </p>
            <pre className="http-pre">{aiReview.preview}</pre>
            <label className="inline-check">
              <input
                type="checkbox"
                aria-label="Approve HTTP evidence transmission"
                checked={consent}
                onChange={(e) => setConsent(e.target.checked)}
              />
              I approve sending this preview to the selected gateway.
            </label>
            <button
              className="button primary"
              disabled={busy || !consent || !aiProvider}
              onClick={() =>
                act(async () => {
                  const reply = await request<{ text: string }>(
                    "job_interpret",
                    {
                      review: {
                        ...aiReview,
                        provider_id: aiProvider,
                        gateway: data.providers.find((p) => p.id === aiProvider)
                          ?.base_url,
                        consent,
                      },
                    },
                  );
                  setAiReply(reply.text);
                }, "AI interpretation saved as unverified")
              }
            >
              Analyze preview with selected model
            </button>
            {aiReply && (
              <section aria-label="Unverified HTTP AI interpretation">
                <h3>AI interpretation · Unverified</h3>
                <pre className="http-pre">{aiReply}</pre>
              </section>
            )}
            {error && <p role="alert">{error}</p>}
          </div>
        )}
      </Modal>
      <Modal
        open={!!pending.length}
        title="Review offline HTTP import"
        dismissDisabled={busy}
        onClose={() => setPending([])}
      >
        <form
          className="form"
          onSubmit={(e) => {
            e.preventDefault();
            act(async () => {
              for (const path of pending) {
                await request("http_import", {
                  kind,
                  input: {
                    project_id: project,
                    target_id: target || null,
                    title,
                    identity,
                    role,
                  },
                  path,
                });
              }
              setPending([]);
              setOffset(0);
            }, "Offline HTTP files imported; no requests sent");
          }}
        >
          <p>
            Files selected: {pending.length}. Originals may contain credentials
            and will be retained as restricted evidence.
          </p>
          <ul>
            {pending.map((path, i) => (
              <li key={i}>{path.split(/[\\/]/).pop()}</li>
            ))}
          </ul>
          <label>
            Import format
            <select
              aria-label="HTTP import format"
              value={kind}
              onChange={(e) => setKind(e.target.value)}
            >
              <option value="har">HAR 1.2</option>
              <option value="openapi">OpenAPI 3.0 / 3.1 JSON or YAML</option>
            </select>
          </label>
          <label>
            Import title
            <input
              required
              maxLength={180}
              aria-label="HTTP import title"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
            />
          </label>
          {kind === "har" && (
            <>
              <label>
                Identity label (no credentials)
                <input
                  aria-label="Captured identity label"
                  maxLength={100}
                  value={identity}
                  onChange={(e) => setIdentity(e.target.value)}
                />
              </label>
              <label>
                Role label
                <input
                  aria-label="Captured role label"
                  maxLength={100}
                  value={role}
                  onChange={(e) => setRole(e.target.value)}
                />
              </label>
            </>
          )}
          <p className="helper">
            16 MiB per file, 2000 entries/operations. YAML aliases, external
            references and active replay are disabled. The associated project
            and target are those selected above.
          </p>
          {error && <p role="alert">{error}</p>}
          <button className="button primary" disabled={busy || !project}>
            Import selected files
          </button>
        </form>
      </Modal>
    </>
  );
}
