import { Button } from "../components/controls";
import { useCallback, useEffect, useState } from "react";
import { request, errorText, type Snapshot } from "../api";
import {
  activeJob,
  jobKinds,
  useJobEvents,
  type AnalysisJob,
  type ImportedFile,
  type Target,
  type JobKind,
} from "../research";
import { Heading, type Perform } from "../components/ui";
import JobHistory from "../components/JobHistory";

export default function JobsPage({
  data,
  perform,
  busy,
  onOpenFile,
}: {
  data: Snapshot;
  perform: Perform;
  busy: boolean;
  onOpenFile: (id: string, project: string) => void;
}) {
  const [jobs, setJobs] = useState<AnalysisJob[]>([]);
  const [files, setFiles] = useState<ImportedFile[]>([]);
  const [targets, setTargets] = useState<Target[]>([]);
  const [status, setStatus] = useState("");
  const [project, setProject] = useState("");
  const [query, setQuery] = useState("");
  const [kind, setKind] = useState<JobKind>("local_static_binary");
  const [fileId, setFileId] = useState("");
  const [targetId, setTargetId] = useState("");
  const [timeout, setTimeoutSeconds] = useState(60);
  const [error, setError] = useState("");
  const load = useCallback(async () => {
    try {
      const [j, f, t] = await Promise.all([
        request<AnalysisJob[]>("jobs_list", { project: null }),
        request<ImportedFile[]>("files_list", { project: null }),
        request<Target[]>("targets_list", { project: null }),
      ]);
      setJobs(j);
      setFiles(f);
      setTargets(t);
      setError("");
    } catch (e) {
      setError(errorText(e));
    }
  }, []);
  useEffect(() => {
    void load();
  }, [load, data.audit[0]?.id]);
  useJobEvents((job) =>
    setJobs((current) =>
      [job, ...current.filter((j) => j.id !== job.id)].sort((a, b) =>
        b.created_at.localeCompare(a.created_at),
      ),
    ),
  );
  // Persisted reads also include CLI-created work and expired-lease recovery.
  useEffect(() => {
    const timer = setInterval(() => void load(), 1500);
    return () => clearInterval(timer);
  }, [load]);
  const visible = jobs.filter(
    (j) =>
      (!project || j.project_id === project) &&
      (!status || (status === "active" ? activeJob(j) : j.status === status)) &&
      `${j.id} ${j.kind} ${j.stage} ${j.error_code ?? ""}`
        .toLowerCase()
        .includes(query.toLowerCase()),
  );
  const scopeReview = kind === "target_registration_review";
  return (
    <>
      <Heading
        eyebrow="Job Center"
        title="Analysis Jobs"
        description="Durable local execution, shared with CLI. Two jobs maximum across clients; no sample execution or network scanning."
      />
      {error && <p role="alert">{error}</p>}
      <section className="panel">
        <div className="panel-heading">
          <h2>Queue local task</h2>
        </div>
        <form
          className="form job-controls"
          onSubmit={(e) => {
            e.preventDefault();
            void perform(async () => {
              await request("job_enqueue", {
                input: {
                  kind,
                  file_id: scopeReview ? null : fileId,
                  target_id: scopeReview ? targetId : null,
                  timeout_ms: Math.round(timeout * 1000),
                },
              });
              await load();
            }, "Local job queued");
          }}
        >
          <label>
            Job type
            <select
              aria-label="Job type"
              value={kind}
              onChange={(e) => setKind(e.target.value as JobKind)}
            >
              {jobKinds.map(([id, label]) => (
                <option key={id} value={id}>
                  {label}
                </option>
              ))}
            </select>
          </label>
          {scopeReview ? (
            <label>
              Job target
              <select
                required
                aria-label="Job target"
                value={targetId}
                onChange={(e) => setTargetId(e.target.value)}
              >
                <option value="">Choose a target</option>
                {targets.map((t) => (
                  <option key={t.id} value={t.id}>
                    {t.name}
                  </option>
                ))}
              </select>
            </label>
          ) : (
            <label>
              Job file
              <select
                required
                aria-label="Job file"
                value={fileId}
                onChange={(e) => setFileId(e.target.value)}
              >
                <option value="">Choose an imported file</option>
                {files.map((f) => (
                  <option key={f.id} value={f.id}>
                    {f.name}
                  </option>
                ))}
              </select>
            </label>
          )}
          <label>
            Timeout (seconds)
            <input
              type="number"
              min="0.001"
              max="3600"
              step="0.001"
              required
              aria-label="Timeout (seconds)"
              value={timeout}
              onChange={(e) => setTimeoutSeconds(Number(e.target.value))}
            />
          </label>
          <Button
            className="button"
            disabled={busy}
            disabledReason={"An operation is in progress"}
          >
            Queue job
          </Button>
        </form>
        <p className="helper">
          Progress records completed parsing stages and processed bytes.
          Cancellation and timeout are cooperative; no timer-generated results.
        </p>
      </section>
      <section className="panel">
        <div className="form job-controls">
          <label>
            Search jobs
            <input
              aria-label="Search jobs"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Job ID, type, stage or error"
            />
          </label>
          <label>
            Job status
            <select
              aria-label="Job status"
              value={status}
              onChange={(e) => setStatus(e.target.value)}
            >
              {[
                ["", "All jobs"],
                ["active", "Active jobs"],
                ["queued", "Queued"],
                ["running", "Running"],
                ["completed", "Completed"],
                ["failed", "Failed"],
                ["cancelled", "Cancelled"],
              ].map(([v, l]) => (
                <option key={v} value={v}>
                  {l}
                </option>
              ))}
            </select>
          </label>
          <label>
            Job project
            <select
              aria-label="Job project"
              value={project}
              onChange={(e) => setProject(e.target.value)}
            >
              <option value="">All projects</option>
              {data.projects.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
          </label>
        </div>
        <JobHistory
          jobs={visible}
          onOpenFile={onOpenFile}
          onCancel={(id) =>
            void perform(async () => {
              await request("job_cancel", { id });
              await load();
            }, "Cancellation requested")
          }
        />
      </section>
    </>
  );
}
