import { useState } from "react";
import { request, errorText } from "../api";
import { activeJob, type AnalysisJob } from "../research";
import { Modal } from "./ui";

export default function JobHistory({
  jobs,
  onCancel,
  onOpenFile,
}: {
  jobs: AnalysisJob[];
  onCancel?: (id: string) => void;
  onOpenFile?: (id: string, project: string) => void;
}) {
  const [result, setResult] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState("");
  async function showResult(id: string) {
    setLoading(id);
    setError("");
    try {
      setResult(JSON.stringify(await request("job_result", { id }), null, 2));
    } catch (e) {
      setError(errorText(e));
    } finally {
      setLoading("");
    }
  }
  return (
    <div className="job-history">
      {error && <p role="alert">{error}</p>}
      {jobs.map((job) => (
        <article
          className="job-entry"
          key={job.id}
          data-job-id={job.id}
          data-job-status={job.status}
        >
          <div className="inline-actions">
            <span className="badge">{job.status}</span>
            <b>{job.kind.replaceAll("_", " ")}</b>
            <small>{new Date(job.created_at).toLocaleString()}</small>
          </div>
          <p>
            {job.stage}
            {job.cancel_requested && activeJob(job)
              ? " · waiting for worker cancellation"
              : ""}
          </p>
          <progress
            aria-label={`Progress ${job.id}`}
            max={100}
            value={job.progress ?? 0}
          />
          <div className="inline-actions">
            {onCancel && activeJob(job) && (
              <button
                className="button secondary"
                disabled={job.cancel_requested}
                onClick={() => onCancel(job.id)}
                aria-label={`Cancel job ${job.id}`}
              >
                {job.cancel_requested ? "Cancellation requested" : "Cancel job"}
              </button>
            )}
            {job.status === "completed" && (
              <button
                className="button secondary"
                disabled={loading === job.id}
                onClick={() => void showResult(job.id)}
              >
                View result
              </button>
            )}
            {onOpenFile && job.file_id && (
              <button
                className="text-button"
                onClick={() => onOpenFile(job.file_id!, job.project_id)}
              >
                Open file
              </button>
            )}
          </div>
          <details>
            <summary>Execution log</summary>
            <p className="technical break-value">{job.id}</p>
            <p className="muted">
              Started:{" "}
              {job.started_at
                ? new Date(job.started_at).toLocaleString()
                : "Not started"}{" "}
              · Finished:{" "}
              {job.finished_at
                ? new Date(job.finished_at).toLocaleString()
                : "Not finished"}{" "}
              · Timeout: {job.timeout_ms / 1000}s
            </p>
            {job.error_code && (
              <p role="status">
                {job.error_code}: {job.error_message}
              </p>
            )}
            <ol className="technical">
              {job.logs.map((log, i) => (
                <li key={i}>{log}</li>
              ))}
            </ol>
          </details>
        </article>
      ))}
      {!jobs.length && <p className="muted">No jobs match this view.</p>}
      <Modal title="Job result" open={!!result} onClose={() => setResult("")}>
        <pre className="hex-preview job-result-preview">
          {result.slice(0, 32768)}
        </pre>
        {result.length > 32768 && (
          <p>
            Preview limited to 32 KB. Open the file workspace or use reconx jobs
            result for the full retained result.
          </p>
        )}
      </Modal>
    </div>
  );
}
