import { Button } from "../components/controls";
import JobHistory from "../components/JobHistory";
import FileResearchPanel from "../components/FileResearchPanel";
import StartResearchButton from "../components/StartResearchButton";
import { useWorkspace } from "../workspace";
import { useJobEvents, activeJob as jobIsActive } from "../research";
import { useCallback, useEffect, useState, useRef } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import {
  request,
  streamImport,
  cancelStream,
  errorText,
  type Snapshot,
  type FileImportProgress,
} from "../api";
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
  const workspace = useWorkspace();
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
  const [importProgress, setImportProgress] =
    useState<FileImportProgress | null>(null);
  const [importing, setImporting] = useState(false);
  const [limitMiB, setLimitMiB] = useState(8192);
  const activeImport = useRef<string | null>(null),
    importCancelled = useRef(false);
  const currentProject = useRef(projectId);
  currentProject.current = projectId;
  useEffect(() => {
    currentProject.current = projectId;
    return () => {
      currentProject.current = undefined;
      importCancelled.current = true;
      if (activeImport.current?.startsWith("import:"))
        void cancelStream(activeImport.current).catch(() => {});
    };
  }, [projectId]);
  const loadVersion = useRef(0);
  const load = useCallback(async () => {
    const version = ++loadVersion.current;
    const [f, j] = await Promise.all([
      request<ImportedFile[]>("files_list", { project: projectId || null }),
      request<AnalysisJob[]>("jobs_list", { project: projectId || null }),
    ]);
    if (version !== loadVersion.current || currentProject.current !== projectId)
      return;
    setFiles(f);
    setJobs(j);
    setSelected((s) => {
      if (workspace && !workspace.selection.target_id) return "";
      if (workspace?.selection.target_id)
        return (
          f.find(
            (file) =>
              file.id === s && file.target_id === workspace.selection.target_id,
          )?.id ??
          f.find((file) => file.target_id === workspace.selection.target_id)
            ?.id ??
          ""
        );
      return f.some((file) => file.id === s) ? s : (f[0]?.id ?? "");
    });
  }, [projectId, workspace?.selection.target_id]);
  useEffect(() => {
    void load().catch((e) => setLocalError(errorText(e)));
  }, [load, data.audit[0]?.id]);
  useEffect(() => {
    if (initialFileId) setSelected(initialFileId);
  }, [initialFileId]);
  useEffect(() => {
    if (requestImport) setChooseImport(true);
  }, [requestImport]);
  const activeJob = jobs.find((j) => j.file_id === selected && jobIsActive(j));
  useJobEvents((job) => {
    if (!projectId || job.project_id === projectId)
      setJobs((current) => [job, ...current.filter((j) => j.id !== job.id)]);
  });
  const completed = jobs.find(
    (j) => j.file_id === selected && j.status === "completed",
  );
  useEffect(() => {
    setRecord(null);
    setTab("Metadata");
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
    if (!jobs.some(jobIsActive)) return;
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
      if (activeImport.current) {
        setLocalError("An import is already in progress");
        return;
      }
      if (!Number.isInteger(limitMiB) || limitMiB < 1 || limitMiB > 65536) {
        setLocalError("Choose an import limit from 1 to 65536 MiB");
        return;
      }
      activeImport.current = "batch";
      importCancelled.current = false;
      setImporting(true);
      setImportProgress(null);
      setLocalError("");
      setImportSummary("");
      try {
        await perform(async () => {
          let imported = 0;
          const failures: string[] = [];
          let firstId = "";
          let firstTarget: string | null = null;
          for (const path of paths.slice(0, 100)) {
            if (importCancelled.current) break;
            const requestId = crypto.randomUUID();
            activeImport.current = `import:${requestId}`;
            try {
              if (folder) {
                const result = await streamImport<{
                  imported: ImportedFile[];
                  failures: { name: string; error_code: string }[];
                }>(
                  projectId,
                  path,
                  authorized,
                  folder,
                  limitMiB * 1024 * 1024,
                  requestId,
                  setImportProgress,
                );
                imported += result.imported.length;
                firstId ||= result.imported[0]?.id ?? "";
                firstTarget ||= result.imported[0]?.target_id ?? null;
                failures.push(
                  ...result.failures.map((f) => `${f.name}: ${f.error_code}`),
                );
              } else {
                const file = await streamImport<ImportedFile>(
                  projectId,
                  path,
                  authorized,
                  folder,
                  limitMiB * 1024 * 1024,
                  requestId,
                  setImportProgress,
                );
                imported++;
                firstId ||= file.id;
                firstTarget ||= file.target_id;
              }
            } catch (e) {
              failures.push(errorText(e));
              if ((e as { code?: string })?.code === "CANCELLED") {
                importCancelled.current = true;
                break;
              }
            }
          }
          if (paths.length > 100)
            failures.push("Only the first 100 selected paths were processed.");
          await load();
          if (currentProject.current !== projectId) return;
          if (firstId) {
            if (workspace) await workspace.select(projectId, firstTarget);
            setSelected(firstId);
          }
          setImportSummary(
            importCancelled.current
              ? "Import interrupted. Completed files are retained in the list below."
              : `${imported} file(s) imported${failures.length ? `; ${failures.length} failed` : ""}.`,
          );
          setLocalError(failures.join("\n"));
          setChooseImport(false);
        });
      } finally {
        activeImport.current = null;
        setImporting(false);
        setImportProgress(null);
      }
    },
    [projectId, authorized, perform, load, limitMiB],
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
          <Button
            className="button"
            disabled={busy || !projectId}
            onClick={() => pick()}
            disabledReason={
              busy ? "An operation is in progress" : "Select a project first"
            }
          >
            Import File
          </Button>
        }
      />
      <section className="panel import-controls">
        <div className="inline-actions">
          <Button
            className="button secondary"
            disabled={busy || !projectId}
            onClick={() => pick(true)}
            disabledReason={
              busy ? "An operation is in progress" : "Select a project first"
            }
          >
            Import Folder
          </Button>
          <span className="muted">
            Drop local files into this workspace · streamed import; bounded deep
            parsing
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
        <label>
          Import limit (MiB)
          <input
            aria-label="Import limit (MiB)"
            type="number"
            min={1}
            max={65536}
            value={limitMiB}
            disabled={importing}
            onChange={(e) => setLimitMiB(Number(e.target.value))}
          />
        </label>
        <p>
          Files above 64 MiB retain their complete bytes and hash. Metadata jobs
          stream integrity and inspect a bounded prefix; deep structure and YARA
          analysis remain limited.
        </p>
        {importing && (
          <div role="status">
            <p>{importProgress?.stage ?? "Preparing streaming import"}</p>
            {importProgress && (
              <>
                <progress
                  max={Math.max(1, importProgress.total)}
                  value={importProgress.completed}
                />
                <span>
                  {importProgress.completed.toLocaleString()} /{" "}
                  {importProgress.total.toLocaleString()} bytes (current file)
                </span>
              </>
            )}
            <Button
              onClick={() => {
                importCancelled.current = true;
                if (activeImport.current?.startsWith("import:"))
                  void cancelStream(activeImport.current).catch((e) =>
                    setLocalError(errorText(e)),
                  );
              }}
            >
              Cancel import
            </Button>
          </div>
        )}
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
                if (workspace)
                  void workspace
                    .select(f.project_id, f.target_id)
                    .catch(() => {});
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
                  <StartResearchButton
                    projectId={file.project_id}
                    targetId={file.target_id ?? undefined}
                  />
                  <p>
                    {file.format} ·{" "}
                    {analysis?.architecture ?? "Architecture not available"}
                  </p>
                </div>
                <div className="inline-actions">
                  <Button
                    className="button"
                    disabled={busy || !!activeJob}
                    onClick={() =>
                      void perform(async () => {
                        const job = await request<AnalysisJob>("file_analyze", {
                          id: file.id,
                        });
                        setJobs((current) => [
                          job,
                          ...current.filter((j) => j.id !== job.id),
                        ]);
                        await load();
                      }, "Static analysis queued")
                    }
                    disabledReason={
                      busy
                        ? "An operation is in progress"
                        : "Wait for the active analysis job to finish"
                    }
                  >
                    {activeJob ? "Analyzing…" : "Analyze File"}
                  </Button>
                  <Button
                    className="text-button"
                    disabled={busy || !!activeJob}
                    onClick={() => setRemove(true)}
                    disabledReason={
                      busy
                        ? "An operation is in progress"
                        : "Wait for the active analysis job to finish"
                    }
                  >
                    Remove
                  </Button>
                </div>
              </div>
              {activeJob && (
                <div role="status" className="analysis-progress">
                  <b>
                    {activeJob.status === "queued"
                      ? "Static analysis queued"
                      : "Static analysis running"}
                  </b>
                  <progress
                    aria-label="Analysis progress"
                    max={100}
                    value={activeJob.progress}
                  />
                  <p>{activeJob.stage}</p>
                  <Button
                    className="button secondary"
                    disabled={activeJob.cancel_requested}
                    onClick={() =>
                      void perform(async () => {
                        await request("job_cancel", { id: activeJob.id });
                        await load();
                      }, "Cancellation requested")
                    }
                    disabledReason={"Cancellation has already been requested"}
                  >
                    {activeJob.cancel_requested
                      ? "Cancellation requested"
                      : "Cancel analysis"}
                  </Button>
                </div>
              )}
              {detailLoading && <p role="status">Loading saved analysis…</p>}
              {!analysis && !detailLoading && (
                <div className="empty compact">
                  <h3>File imported</h3>
                  <p className="technical break-value">SHA-256 {file.sha256}</p>
                  <p>
                    {file.size > 64 * 1024 * 1024
                      ? "Start bounded metadata analysis. Deep structure remains deferred for this file size."
                      : "Start analysis to save sections, imports, exports, strings and hex data."}
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
                        disabled={
                          file.size > 64 * 1024 * 1024 &&
                          [
                            "Sections",
                            "Segments",
                            "Imports",
                            "Exports",
                            "Package",
                          ].includes(name)
                        }
                        title={
                          file.size > 64 * 1024 * 1024 &&
                          [
                            "Sections",
                            "Segments",
                            "Imports",
                            "Exports",
                            "Package",
                          ].includes(name)
                            ? "Deep structure is deferred under the independent parser budget"
                            : undefined
                        }
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
                          <dt>Source path</dt>
                          <dd className="technical break-value">
                            {file.source_path || "Not available"}
                          </dd>
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
                        <Button
                          className="button secondary"
                          disabled={
                            !compareId ||
                            busy ||
                            file.size > 64 * 1024 * 1024 ||
                            (files.find((f) => f.id === compareId)?.size ?? 0) >
                              64 * 1024 * 1024
                          }
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
                          disabledReason={
                            file.size > 64 * 1024 * 1024 ||
                            (files.find((f) => f.id === compareId)?.size ?? 0) >
                              64 * 1024 * 1024
                              ? "Byte comparison currently supports files up to 64 MiB; complete large artifacts remain preserved"
                              : !compareId
                                ? "Select a file to compare"
                                : "An operation is in progress"
                          }
                        >
                          Compare binaries
                        </Button>
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
        <FileResearchPanel
          file={file}
          analysisId={record?.id}
          data={data}
          perform={perform}
          busy={busy}
        />
      </div>
      <details className="module-details">
        <summary>Available analysis integrations</summary>
        <p className="muted">
          Disassembly, decompilation and function analysis require a verified
          external analyzer. Local metadata and strings use bounded parsers;
          large files show streamed metadata with explicit limits. Installed
          YARA-X signatures retain their separate 64 MiB analysis budget.
        </p>
      </details>
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
        <Button
          className="button"
          disabled={!projectId || busy}
          onClick={() => pick()}
          disabledReason={
            !projectId
              ? "Select a project first"
              : "An operation is in progress"
          }
        >
          Choose local files
        </Button>
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
        <Button
          className="button"
          disabled={busy}
          onClick={() =>
            void perform(async () => {
              await request("file_remove", { id: selected });
              setRemove(false);
              await load();
            }, "Imported file removed")
          }
          disabledReason={"An operation is in progress"}
        >
          Remove imported file
        </Button>
      </Modal>
    </>
  );
}
