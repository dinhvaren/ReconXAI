import { useCallback, useEffect, useState, useRef } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { request, errorText, type Snapshot } from "../api";
import {
  selectPaths,
  type ImportedFile,
  type AnalysisJob,
  type AnalysisRecord,
  type Section,
} from "../research";
import { Heading, Modal, type Perform } from "../components/ui";
const address = (n: number | null) =>
  n === null ? "—" : `0x${n.toString(16)}`;
function Sections({ rows }: { rows: Section[] }) {
  return (
    <div className="table-scroll">
      <table className="data-table">
        <thead>
          <tr>
            <th>Name</th>
            <th>Address</th>
            <th>Size</th>
            <th>File offset</th>
            <th>Entropy</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((s, i) => (
            <tr key={i}>
              <td>{s.name || `Segment ${i}`}</td>
              <td className="technical">{address(s.address)}</td>
              <td>{s.size.toLocaleString()}</td>
              <td className="technical">{address(s.file_offset)}</td>
              <td>{s.entropy?.toFixed(3) ?? "—"}</td>
            </tr>
          ))}
        </tbody>
      </table>
      {!rows.length && (
        <p className="empty compact">No entries in this format.</p>
      )}
    </div>
  );
}
export default function ReversePage({
  data,
  projectId,
  initialFileId,
  perform,
  busy,
  requestImport = 0,
}: {
  data: Snapshot;
  projectId?: string;
  initialFileId?: string;
  perform: Perform;
  busy: boolean;
  requestImport?: number;
}) {
  const [files, setFiles] = useState<ImportedFile[]>([]);
  const [jobs, setJobs] = useState<AnalysisJob[]>([]);
  const [selected, setSelected] = useState(initialFileId ?? "");
  const [record, setRecord] = useState<AnalysisRecord | null>(null);
  const [search, setSearch] = useState("");
  const [tab, setTab] = useState("Metadata");
  const [stringSearch, setStringSearch] = useState("");
  const [authorized, setAuthorized] = useState(false);
  const [localError, setLocalError] = useState("");
  const [importSummary, setImportSummary] = useState("");
  const [remove, setRemove] = useState(false);
  const [comparison, setComparison] = useState("");
  const [compareId, setCompareId] = useState("");
  const [chooseImport, setChooseImport] = useState(false);
  const [detailLoading, setDetailLoading] = useState(false);
  const load = useCallback(async () => {
    const [f, j] = await Promise.all([
      request<ImportedFile[]>("files_list", { project: projectId || null }),
      request<AnalysisJob[]>("jobs_list", { project: projectId || null }),
    ]);
    setFiles(f);
    setJobs(j);
    setSelected((s) => (f.some((f) => f.id === s) ? s : (f[0]?.id ?? "")));
  }, [projectId]);
  useEffect(() => {
    void load().catch((e) => setLocalError(errorText(e)));
  }, [load, data.audit[0]?.id]);
  useEffect(() => {
    if (initialFileId) setSelected(initialFileId);
  }, [initialFileId]);
  useEffect(() => {
    if (requestImport) setChooseImport(true);
  }, [requestImport]);
  const activeJob = jobs.find(
    (j) => j.file_id === selected && j.status === "running",
  );
  const completed = jobs.find(
    (j) => j.file_id === selected && j.status === "completed",
  );
  useEffect(() => {
    setRecord(null);
    setComparison("");
    setStringSearch("");
    if (!selected) return;
    let active = true;
    setDetailLoading(true);
    request<AnalysisRecord>("file_analysis", { id: selected })
      .then((r) => {
        if (active) setRecord(r);
      })
      .catch((e) => {
        if (active && (e as { code?: string })?.code !== "NOT_ANALYZED")
          setLocalError(errorText(e));
      })
      .finally(() => {
        if (active) setDetailLoading(false);
      });
    return () => {
      active = false;
    };
  }, [selected, completed?.id]);
  useEffect(() => {
    if (!jobs.some((j) => j.status === "running")) return;
    const timer = setInterval(() => {
      void load().catch((e) => setLocalError(errorText(e)));
    }, 750);
    return () => clearInterval(timer);
  }, [jobs, load]);
  const importPaths = useCallback(
    async (paths: string[], folder = false) => {
      if (!projectId) {
        setLocalError("Select or create a project before importing files.");
        return;
      }
      setLocalError("");
      setImportSummary("");
      await perform(async () => {
        let imported = 0;
        const failures: string[] = [];
        let firstId = "";
        for (const path of paths.slice(0, 100)) {
          try {
            if (folder) {
              const result = await request<{
                imported: ImportedFile[];
                failures: { name: string; error_code: string }[];
              }>("file_import", {
                project: projectId,
                path,
                authorized,
                folder,
              });
              imported += result.imported.length;
              firstId ||= result.imported[0]?.id ?? "";
              failures.push(
                ...result.failures.map((f) => `${f.name}: ${f.error_code}`),
              );
            } else {
              const file = await request<ImportedFile>("file_import", {
                project: projectId,
                path,
                authorized,
                folder,
              });
              imported++;
              firstId ||= file.id;
            }
          } catch (e) {
            failures.push(errorText(e));
          }
        }
        if (paths.length > 100)
          failures.push("Only the first 100 selected paths were processed.");
        await load();
        if (firstId) setSelected(firstId);
        setImportSummary(
          `${imported} file(s) imported${failures.length ? `; ${failures.length} failed` : ""}.`,
        );
        setLocalError(failures.join("\n"));
        setChooseImport(false);
      });
    },
    [projectId, authorized, perform, load],
  );
  const dropImport = useRef(importPaths);
  useEffect(() => {
    dropImport.current = importPaths;
  }, [importPaths]);
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    // Native drop events provide local paths; browser File objects are not substitutes.
    try {
      void getCurrentWebviewWindow()
        .onDragDropEvent((event) => {
          if (event.payload.type === "drop" && !disposed)
            void dropImport.current(event.payload.paths);
        })
        .then((fn) => {
          if (disposed) fn();
          else unlisten = fn;
        })
        .catch(() => {});
    } catch {
      /* No native bridge in unit-test DOM. */
    }
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
  const pick = (folder = false) =>
    void (async () => {
      try {
        const paths = await selectPaths(folder);
        if (paths.length) await importPaths(paths, folder);
      } catch (e) {
        setLocalError(errorText(e));
      }
    })();
  const file = files.find((f) => f.id === selected);
  const analysis = record?.analysis;
  const visibleFiles = files.filter((f) =>
    `${f.name} ${f.format} ${f.sha256}`
      .toLowerCase()
      .includes(search.toLowerCase()),
  );
  const matchingStrings =
    analysis?.strings.filter((s) =>
      s.text.toLowerCase().includes(stringSearch.toLowerCase()),
    ) ?? [];
  const fileJobs = jobs.filter((j) => j.file_id === selected);
  return (
    <>
      <Heading
        eyebrow="Local static analysis"
        title="Reverse Engineering"
        description="Import owned files, inspect real binary structure, and retain analysis history."
        action={
          <button
            className="button"
            disabled={busy || !projectId}
            onClick={() => pick()}
          >
            Import File
          </button>
        }
      />
      <section className="panel import-controls">
        <div className="inline-actions">
          <button
            className="button secondary"
            disabled={busy || !projectId}
            onClick={() => pick(true)}
          >
            Import Folder
          </button>
          <span className="muted">
            Drop local files into this workspace · 64 MiB per file
          </span>
        </div>
        <label className="checkbox-row">
          <input
            type="checkbox"
            checked={authorized}
            onChange={(e) => setAuthorized(e.target.checked)}
          />
          I have permission to analyze the imported local files.
        </label>
        <p className="muted">
          Without confirmation, files can be imported but analysis stays
          blocked. Existing target policies are preserved. Folder import reads
          up to 100 direct entries.
        </p>
        {!projectId && <p>Select a project in the toolbar first.</p>}
      </section>
      {localError && (
        <p role="alert" className="banner error break-value">
          {localError}
        </p>
      )}
      {importSummary && (
        <p role="status" className="banner">
          {importSummary}
        </p>
      )}
      <div className="reverse-layout">
        <aside className="panel file-explorer">
          <h2>Imported files</h2>
          <label>
            Search files
            <input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Filename, format or SHA-256"
            />
          </label>
          {!files.length && (
            <p className="muted">
              Import a file to begin. Samples are copied into the private
              application workspace and never executed.
            </p>
          )}
          {visibleFiles.map((f) => (
            <button
              key={f.id}
              className={`file-history-item ${selected === f.id ? "active" : ""}`}
              onClick={() => {
                setSelected(f.id);
                setTab("Metadata");
              }}
            >
              <b>{f.name}</b>
              <span>
                {f.format} · {f.size.toLocaleString()} bytes
              </span>
              <small>{new Date(f.created_at).toLocaleString()}</small>
            </button>
          ))}
        </aside>
        <section className="panel binary-workspace">
          {file ? (
            <>
              <div className="panel-heading">
                <div>
                  <h2>{file.name}</h2>
                  <p>
                    {file.format} ·{" "}
                    {analysis?.architecture ?? "Architecture not available"}
                  </p>
                </div>
                <div className="inline-actions">
                  <button
                    className="button"
                    disabled={busy || !!activeJob}
                    onClick={() =>
                      void perform(async () => {
                        const job = await request<AnalysisJob>("file_analyze", {
                          id: file.id,
                        });
                        setJobs((current) => [job, ...current]);
                        await load();
                      }, "Static analysis started")
                    }
                  >
                    {activeJob ? "Analyzing…" : "Analyze File"}
                  </button>
                  <button
                    className="text-button"
                    disabled={busy || !!activeJob}
                    onClick={() => setRemove(true)}
                  >
                    Remove
                  </button>
                </div>
              </div>
              {activeJob && (
                <div role="status" className="analysis-progress">
                  <b>Static analysis running</b>
                  <p>{activeJob.logs.at(-1)}</p>
                  <button
                    className="button secondary"
                    onClick={() =>
                      void perform(async () => {
                        await request("job_cancel", { id: activeJob.id });
                        await load();
                      }, "Analysis cancelled")
                    }
                  >
                    Cancel analysis
                  </button>
                </div>
              )}
              {detailLoading && <p role="status">Loading saved analysis…</p>}
              {!analysis && !detailLoading && (
                <div className="empty compact">
                  <h3>File imported</h3>
                  <p className="technical break-value">SHA-256 {file.sha256}</p>
                  <p>
                    Start analysis to save sections, imports, exports, strings
                    and hex data.
                  </p>
                </div>
              )}
              {analysis && (
                <>
                  <div
                    className="research-tabs"
                    role="tablist"
                    aria-label="Binary analysis views"
                  >
                    {[
                      "Metadata",
                      "Sections",
                      "Segments",
                      "Imports",
                      "Exports",
                      "Strings",
                      "Hex",
                      "Package",
                      "History",
                      "Compare",
                    ].map((name) => (
                      <button
                        key={name}
                        role="tab"
                        aria-selected={tab === name}
                        onClick={() => setTab(name)}
                      >
                        {name}
                      </button>
                    ))}
                  </div>
                  <div
                    className="binary-content"
                    role="tabpanel"
                    aria-label={tab}
                  >
                    {tab === "Metadata" && (
                      <>
                        <dl className="metadata-grid">
                          <dt>Content format</dt>
                          <dd>{analysis.format}</dd>
                          <dt>Architecture</dt>
                          <dd>{analysis.architecture ?? "Not available"}</dd>
                          <dt>Size</dt>
                          <dd>{analysis.size.toLocaleString()} bytes</dd>
                          <dt>SHA-256</dt>
                          <dd className="technical break-value">
                            {analysis.sha256}
                          </dd>
                          <dt>Entry point</dt>
                          <dd className="technical">
                            {address(analysis.entry_point)}
                          </dd>
                          <dt>Entropy</dt>
                          <dd>{analysis.entropy.toFixed(4)} bits / byte</dd>
                          <dt>Analysis saved</dt>
                          <dd>
                            {new Date(record!.created_at).toLocaleString()}
                          </dd>
                          <dt>Analysis job</dt>
                          <dd className="technical break-value">
                            {record!.job_id}
                          </dd>
                        </dl>
                        {analysis.warnings.map((w) => (
                          <p className="muted" key={w}>
                            {w}
                          </p>
                        ))}
                      </>
                    )}
                    {tab === "Sections" && (
                      <Sections rows={analysis.sections} />
                    )}
                    {tab === "Segments" && (
                      <Sections rows={analysis.segments} />
                    )}
                    {(tab === "Imports" ||
                      tab === "Exports" ||
                      tab === "Package") && (
                      <div className="technical-list">
                        {(tab === "Imports"
                          ? analysis.imports
                          : tab === "Exports"
                            ? analysis.exports
                            : analysis.package_entries
                        ).map((s, i) => (
                          <p className="technical break-value" key={i}>
                            {s}
                          </p>
                        ))}
                        {!(
                          tab === "Imports"
                            ? analysis.imports
                            : tab === "Exports"
                              ? analysis.exports
                              : analysis.package_entries
                        ).length && (
                          <p className="muted">
                            No entries are available in this file.
                          </p>
                        )}
                      </div>
                    )}
                    {tab === "Strings" && (
                      <>
                        <label>
                          Search strings
                          <input
                            value={stringSearch}
                            onChange={(e) => setStringSearch(e.target.value)}
                          />
                        </label>
                        <p className="muted">
                          Showing {Math.min(matchingStrings.length, 200)} of{" "}
                          {matchingStrings.length} matches; narrow the search
                          for more. Strings are local evidence, never sent to AI
                          automatically.
                        </p>
                        <div className="table-scroll">
                          <table className="data-table">
                            <thead>
                              <tr>
                                <th>Offset</th>
                                <th>Encoding</th>
                                <th>String</th>
                              </tr>
                            </thead>
                            <tbody>
                              {matchingStrings.slice(0, 200).map((s, i) => (
                                <tr key={i}>
                                  <td className="technical">
                                    {address(s.offset)}
                                  </td>
                                  <td>{s.encoding}</td>
                                  <td className="technical break-value">
                                    {s.text}
                                  </td>
                                </tr>
                              ))}
                            </tbody>
                          </table>
                        </div>
                      </>
                    )}
                    {tab === "Hex" && (
                      <>
                        <p className="muted">
                          First 512 bytes of the managed sample.
                        </p>
                        <pre className="hex-preview">
                          {analysis.hex_preview}
                        </pre>
                      </>
                    )}
                    {tab === "History" && <JobHistory jobs={fileJobs} />}
                    {tab === "Compare" && (
                      <>
                        <label>
                          Compare with imported file
                          <select
                            value={compareId}
                            onChange={(e) => setCompareId(e.target.value)}
                          >
                            <option value="">Select another file</option>
                            {files
                              .filter((f) => f.id !== selected)
                              .map((f) => (
                                <option key={f.id} value={f.id}>
                                  {f.name} · {f.sha256.slice(0, 8)}
                                </option>
                              ))}
                          </select>
                        </label>
                        <button
                          className="button secondary"
                          disabled={!compareId || busy}
                          onClick={() =>
                            void perform(async () =>
                              setComparison(
                                JSON.stringify(
                                  await request("files_compare", {
                                    a: selected,
                                    b: compareId,
                                  }),
                                  null,
                                  2,
                                ),
                              ),
                            )
                          }
                        >
                          Compare binaries
                        </button>
                        {comparison && (
                          <pre className="hex-preview">{comparison}</pre>
                        )}
                      </>
                    )}
                  </div>
                </>
              )}
              {!analysis && !!fileJobs.length && <JobHistory jobs={fileJobs} />}
            </>
          ) : (
            <div className="empty">
              <h2>No imported sample</h2>
              <p>
                Select a project, then import an EXE, ELF, Mach-O, APK, ZIP or
                raw binary.
              </p>
            </div>
          )}
        </section>
      </div>
      <section className="panel">
        <h3>Integration capabilities</h3>
        <p className="muted">
          Disassembly, decompilation, Ghidra, YARA and AI function analysis are
          not connected in this build. Metadata and strings shown above come
          from the local static parser.
        </p>
      </section>
      <Modal
        title="Import local file"
        open={chooseImport}
        onClose={() => setChooseImport(false)}
      >
        <p>
          Select a project and record permission before starting local analysis.
        </p>
        <label className="checkbox-row">
          <input
            type="checkbox"
            checked={authorized}
            onChange={(e) => setAuthorized(e.target.checked)}
          />
          I have permission to analyze the imported local files.
        </label>
        <button
          className="button"
          disabled={!projectId || busy}
          onClick={() => pick()}
        >
          Choose local files
        </button>
      </Modal>
      <Modal
        title="Remove imported file"
        open={remove}
        onClose={() => setRemove(false)}
      >
        <p>
          Remove {file?.name} and its analysis references? The original file and
          cached sample remain untouched; job history is retained.
        </p>
        <button
          className="button"
          disabled={busy}
          onClick={() =>
            void perform(async () => {
              await request("file_remove", { id: selected });
              setRemove(false);
              await load();
            }, "Imported file removed")
          }
        >
          Remove imported file
        </button>
      </Modal>
    </>
  );
}
export function JobHistory({ jobs }: { jobs: AnalysisJob[] }) {
  return (
    <div className="job-history">
      {jobs.map((j) => (
        <details key={j.id}>
          <summary>
            <b>{j.status}</b> · {new Date(j.created_at).toLocaleString()} ·{" "}
            {j.kind}
          </summary>
          <p className="technical break-value">{j.id}</p>
          {j.error_code && <p>{j.error_code}</p>}
          {j.logs.map((log, i) => (
            <p key={i}>{log}</p>
          ))}
        </details>
      ))}
      {!jobs.length && <p className="muted">No analysis jobs recorded.</p>}
    </div>
  );
}
export function JobsPage({
  data,
  perform,
  busy,
}: {
  data: Snapshot;
  perform: Perform;
  busy: boolean;
}) {
  const [jobs, setJobs] = useState<AnalysisJob[]>([]);
  const [error, setError] = useState("");
  const load = useCallback(async () => {
    try {
      setJobs(await request("jobs_list", { project: null }));
      setError("");
    } catch (e) {
      setError(errorText(e));
    }
  }, []);
  useEffect(() => {
    void load();
  }, [load, data.audit[0]?.id]);
  useEffect(() => {
    if (!jobs.some((j) => j.status === "running")) return;
    const timer = setInterval(() => void load(), 750);
    return () => clearInterval(timer);
  }, [jobs, load]);
  return (
    <>
      <Heading
        eyebrow="Execution history"
        title="Analysis Jobs"
        description="Persistent local static-analysis jobs shared with the CLI."
      />
      {error && <p role="alert">{error}</p>}
      <section className="panel">
        <JobHistory jobs={jobs} />
        {jobs
          .filter((j) => j.status === "running")
          .map((j) => (
            <button
              className="button secondary"
              disabled={busy}
              key={j.id}
              onClick={() =>
                void perform(async () => {
                  await request("job_cancel", { id: j.id });
                  await load();
                })
              }
            >
              Cancel job {j.id.slice(0, 8)}
            </button>
          ))}
      </section>
    </>
  );
}
