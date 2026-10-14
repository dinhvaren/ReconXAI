import { useEffect, useState, useCallback } from "react";
import { request, errorText, type Snapshot } from "../api";
import {
  targetKinds,
  localFileKinds,
  selectPaths,
  type Target,
  type ImportedFile,
  type AnalysisJob,
  type ScopeDecision,
} from "../research";
import { Heading, Modal, type Perform } from "../components/ui";
export default function TargetsPage({
  data,
  projectId,
  onAdd,
  onEdit,
  onReverse,
  perform,
  busy,
}: {
  data: Snapshot;
  projectId?: string;
  onAdd: () => void;
  onEdit: (target: Target) => void;
  onReverse: (fileId: string, projectId: string) => void;
  perform: Perform;
  busy: boolean;
}) {
  const [targets, setTargets] = useState<Target[]>([]);
  const [search, setSearch] = useState("");
  const [kind, setKind] = useState("");
  const [filterProject, setFilterProject] = useState(projectId ?? "");
  const [selected, setSelected] = useState<Target | null>(null);
  const [scope, setScope] = useState<ScopeDecision | null>(null);
  const [candidate, setCandidate] = useState("");
  const [files, setFiles] = useState<ImportedFile[]>([]);
  const [jobs, setJobs] = useState<AnalysisJob[]>([]);
  const [remove, setRemove] = useState<Target | null>(null);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);
  const load = useCallback(async () => {
    setLoading(true);
    try {
      const rows = await request<Target[]>("targets_list", {
        project: filterProject || null,
      });
      setTargets(rows);
      setSelected((s) => rows.find((t) => t.id === s?.id) ?? null);
      setError("");
    } catch (e) {
      setError(errorText(e));
    } finally {
      setLoading(false);
    }
  }, [filterProject]);
  useEffect(() => {
    setFilterProject(projectId ?? "");
  }, [projectId]);
  useEffect(() => {
    void load();
  }, [load, data.audit[0]?.id]);
  useEffect(() => {
    setScope(null);
    setFiles([]);
    setJobs([]);
    setCandidate("");
    if (!selected) return;
    let active = true;
    Promise.all([
      request<ScopeDecision>("scope_validate", {
        id: selected.id,
        candidate: null,
      }),
      request<ImportedFile[]>("files_list", { project: selected.project_id }),
      request<AnalysisJob[]>("jobs_list", { project: selected.project_id }),
    ])
      .then(([decision, f, j]) => {
        if (active) {
          setScope(decision);
          setFiles(f.filter((f) => f.target_id === selected.id));
          setJobs(j.filter((j) => j.target_id === selected.id));
        }
      })
      .catch((e) => {
        if (active) setError(errorText(e));
      });
    return () => {
      active = false;
    };
  }, [selected]);
  const visible = targets.filter(
    (t) =>
      (!kind || t.kind === kind) &&
      `${t.name} ${t.value} ${t.tags.join(" ")} ${t.description}`
        .toLowerCase()
        .includes(search.toLowerCase()),
  );
  const transfer = (save: boolean) =>
    void perform(async () => {
      const paths = await selectPaths(false, save);
      if (!paths[0]) return;
      if (save)
        await request("targets_export_file", {
          project: filterProject || null,
          path: paths[0],
        });
      else await request("targets_import_file", { path: paths[0] });
      await load();
    });
  return (
    <>
      <Heading
        eyebrow="Research inputs"
        title="Targets & Scope"
        description="Register assets, group them with tags, and define authorization boundaries."
        action={
          <button className="button" onClick={onAdd}>
            + Add Target
          </button>
        }
      />
      <section className="panel target-inventory">
        <div className="inventory-toolbar">
          <label>
            Search targets
            <input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Name, value, tag or group"
            />
          </label>
          <label>
            Filter target type
            <select value={kind} onChange={(e) => setKind(e.target.value)}>
              <option value="">All types</option>
              {targetKinds.map(([id, label]) => (
                <option key={id} value={id}>
                  {label}
                </option>
              ))}
            </select>
          </label>
          <label>
            Filter target project
            <select
              value={filterProject}
              onChange={(e) => setFilterProject(e.target.value)}
            >
              <option value="">All projects</option>
              {data.projects.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
          </label>
          <button
            className="button secondary"
            disabled={busy}
            onClick={() => transfer(false)}
          >
            Import targets
          </button>
          <button
            className="button secondary"
            disabled={busy || !targets.length}
            onClick={() => transfer(true)}
          >
            Export targets
          </button>
        </div>
        {error && (
          <p role="alert" className="banner error">
            {error}
          </p>
        )}
        {loading && <p role="status">Loading target inventory…</p>}
        {!loading && !visible.length && (
          <div className="empty">
            <h2>No matching targets</h2>
            <p>Add a target or import a version-1 JSON target list.</p>
          </div>
        )}
        {!!visible.length && (
          <div className="table-scroll">
            <table className="data-table">
              <thead>
                <tr>
                  <th>Target / scope value</th>
                  <th>Type</th>
                  <th>Project</th>
                  <th>Scope status</th>
                  <th>Last analysis</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {visible.map((t) => (
                  <tr
                    key={t.id}
                    className={selected?.id === t.id ? "selected-row" : ""}
                  >
                    <td>
                      <button
                        className="text-button target-name"
                        onClick={() => setSelected(t)}
                      >
                        {t.name}
                      </button>
                      <small className="break-value">{t.value}</small>
                      {!!t.tags.length && <small>{t.tags.join(" · ")}</small>}
                    </td>
                    <td>{targetKinds.find(([id]) => id === t.kind)?.[1]}</td>
                    <td>
                      {data.projects.find((p) => p.id === t.project_id)?.name}
                    </td>
                    <td>
                      <span className="badge">
                        {t.scope_status.replaceAll("_", " ")}
                      </span>
                      {t.excluded_scope.length > 0 && (
                        <small>Has exclusions</small>
                      )}
                    </td>
                    <td>
                      {t.last_analyzed_at
                        ? new Date(t.last_analyzed_at).toLocaleString()
                        : "Not analyzed"}
                    </td>
                    <td>
                      <div className="inline-actions">
                        <button
                          className="text-button"
                          onClick={() => onEdit(t)}
                        >
                          Edit
                        </button>
                        <button
                          className="text-button"
                          onClick={() => setRemove(t)}
                        >
                          Delete
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>
      {selected && (
        <section className="panel target-details">
          <div className="panel-heading">
            <h2>{selected.name}</h2>
            <button className="text-button" onClick={() => setSelected(null)}>
              Close details
            </button>
          </div>
          <p className="break-value">{selected.value}</p>
          <p>{selected.description || "No description recorded."}</p>
          <div className="form-columns">
            <div>
              <h3>Allowed scope</h3>
              {selected.allowed_scope.map((r) => (
                <p className="technical break-value" key={r}>
                  {r}
                </p>
              ))}
            </div>
            <div>
              <h3>Excluded scope</h3>
              {selected.excluded_scope.length ? (
                selected.excluded_scope.map((r) => (
                  <p className="technical break-value" key={r}>
                    {r}
                  </p>
                ))
              ) : (
                <p className="muted">No exclusions recorded.</p>
              )}
            </div>
          </div>
          <form
            className="inventory-toolbar"
            onSubmit={(e) => {
              e.preventDefault();
              void perform(async () =>
                setScope(
                  await request("scope_validate", {
                    id: selected.id,
                    candidate: candidate || null,
                  }),
                ),
              );
            }}
          >
            <label>
              Value to validate
              <input
                value={candidate}
                onChange={(e) => setCandidate(e.target.value)}
                placeholder={selected.value}
              />
            </label>
            <button className="button secondary" disabled={busy}>
              Validate scope
            </button>
          </form>
          {scope && (
            <p role="status" className="scope-result">
              <b>{scope.status.replaceAll("_", " ")}</b> — {scope.reason}
            </p>
          )}
          <h3>Available analysis</h3>
          {localFileKinds.includes(selected.kind) ? (
            <button
              className="button"
              disabled={busy || scope?.status !== "allowed"}
              onClick={() =>
                void perform(async () => {
                  const file = await request<ImportedFile>(
                    "file_import_target",
                    { id: selected.id },
                  );
                  onReverse(file.id, file.project_id);
                })
              }
            >
              Import for static analysis
            </button>
          ) : (
            <p className="muted">
              Target registration is available. This target type has no
              execution adapter in this build.
            </p>
          )}
          <h3>Imported assets</h3>
          {files.length ? (
            files.map((f) => (
              <button
                key={f.id}
                className="file-history-item"
                onClick={() => onReverse(f.id, f.project_id)}
              >
                <b>{f.name}</b>
                <span>
                  {f.format} · {f.size.toLocaleString()} bytes
                </span>
              </button>
            ))
          ) : (
            <p className="muted">No imported assets linked to this target.</p>
          )}
          <h3>Analysis history</h3>
          {jobs.length ? (
            jobs.map((j) => (
              <p key={j.id} className="break-value">
                {new Date(j.created_at).toLocaleString()} · {j.kind} ·{" "}
                <b>{j.status}</b>
                {j.error_code && ` · ${j.error_code}`}
              </p>
            ))
          ) : (
            <p className="muted">No analysis jobs recorded.</p>
          )}
          <p className="muted">
            Evidence, findings and target-aware AI correlation are not
            implemented yet.
          </p>
        </section>
      )}
      <Modal
        title="Delete target"
        open={!!remove}
        onClose={() => setRemove(null)}
      >
        <p>
          Delete {remove?.name} from the inventory? Imported files and job
          history are retained with their target link removed.
        </p>
        <button
          className="button"
          disabled={busy}
          onClick={() =>
            void perform(async () => {
              await request("target_remove", { id: remove!.id });
              setRemove(null);
              await load();
            }, "Target removed")
          }
        >
          Delete target
        </button>
      </Modal>
    </>
  );
}
