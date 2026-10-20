import { Button } from "../components/controls";
import { useCallback, useEffect, useRef, useState } from "react";
import { request, errorText, type Snapshot } from "../api";
import { useJobEvents, type AnalysisJob } from "../research";
import { Heading, Modal, type Perform } from "../components/ui";
import { EmptyState } from "../components/controls";
import JobHistory from "../components/JobHistory";
interface Plan {
  target_id: string;
  project_id: string;
  url: string;
  method: string;
  request_budget: number;
  timeout_ms: number;
  fingerprint: string;
  permissions: string[];
  limitations: string[];
}
interface Asset {
  id: string;
  target_id: string | null;
  value: string;
  last_seen: string;
  job_id: string;
  facts: {
    status: number;
    http_version: string;
    reported_server: string | null;
    content_type: string | null;
    header_count: number;
  };
}
export default function ReconPage({
  data,
  projectId,
  initialTarget,
  perform,
  busy,
  onAddTarget,
  navigate,
}: {
  data: Snapshot;
  projectId?: string;
  initialTarget?: string;
  perform: Perform;
  busy: boolean;
  onAddTarget: () => void;
  navigate: (page: string, project?: string) => void;
}) {
  const [project, setProject] = useState(
      projectId ?? data.projects[0]?.id ?? "",
    ),
    [target, setTarget] = useState(initialTarget ?? "");
  const [assets, setAssets] = useState<Asset[]>([]),
    [jobs, setJobs] = useState<AnalysisJob[]>([]),
    [plan, setPlan] = useState<Plan | null>(null),
    [approved, setApproved] = useState(false),
    [error, setError] = useState("");
  const loadVersion = useRef(0);
  const load = useCallback(async () => {
    const version = ++loadVersion.current;
    if (!project) {
      setAssets([]);
      setJobs([]);
      return;
    }
    const [a, j] = await Promise.all([
      request<Asset[]>("assets_list", { project }),
      request<AnalysisJob[]>("jobs_list", { project }),
    ]);
    if (version !== loadVersion.current) return;
    setAssets(a);
    setJobs(j.filter((j) => j.kind === "local_http_inventory"));
  }, [project]);
  useEffect(() => {
    setAssets([]);
    setJobs([]);
    setPlan(null);
    setApproved(false);
    return () => {
      loadVersion.current += 1;
    };
  }, [project]);
  useEffect(() => {
    void load().catch((e) => setError(errorText(e)));
  }, [load, data.audit[0]?.id]);
  useEffect(() => {
    if (projectId) setProject(projectId);
    setTarget(initialTarget ?? "");
  }, [projectId, initialTarget]);
  useJobEvents((j) => {
    if (j.project_id === project && j.kind === "local_http_inventory")
      void load().catch((e) => setError(errorText(e)));
  });
  const act = (fn: () => Promise<unknown>, message: string) =>
    perform(async () => {
      try {
        await fn();
        await load();
        setError("");
      } catch (e) {
        setError(errorText(e));
        throw e;
      }
    }, message);
  const targets = (data.targets ?? []).filter(
    (t) =>
      t.project_id === project &&
      ["url", "rest_api", "graphql"].includes(t.kind),
  );
  return (
    <>
      <Heading
        eyebrow="Owned local inventory"
        title="Reconnaissance"
        description="Plan and review an owned local HTTP observation."
        action={
          target ? (
            <Button className="button primary" onClick={onAddTarget}>
              + Add Target
            </Button>
          ) : undefined
        }
      />
      {error && <p role="alert">{error}</p>}
      <section className="panel records-panel">
        <h2>Plan local HTTP inventory</h2>
        <div className="records-toolbar">
          <label>
            Inventory project
            <select
              aria-label="Inventory project"
              value={project}
              onChange={(e) => {
                setProject(e.target.value);
                setTarget("");
              }}
            >
              <option value="">Choose project</option>
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
            Inventory target
            <select
              aria-label="Inventory target"
              value={target}
              onChange={(e) => setTarget(e.target.value)}
            >
              <option value="">Select URL / API target</option>
              {targets.map((t) => (
                <option key={t.id} value={t.id}>
                  {t.name}
                </option>
              ))}
            </select>
          </label>
        </div>
        {!target ? (
          <EmptyState
            title="No target selected"
            icon="radar"
            description="Select an existing target or add a new target to begin reconnaissance."
            action={
              <Button variant="primary" onClick={onAddTarget}>
                Add Target
              </Button>
            }
            secondary={
              <Button
                variant="ghost"
                onClick={() => navigate("Targets & Scope", project)}
              >
                Open Targets
              </Button>
            }
          />
        ) : (
          <>
            <details className="module-details">
              <summary>Execution policy and adapter limits</summary>
              <p>
                Requirements: an authorized loopback URL, matching scope, and
                Authorized Auto or Lab Research selected in both Settings and
                the target. The current application mode is{" "}
                {data.settings.security_mode.replaceAll("_", " ")}. This
                operation always requires manual approval.
              </p>
              <p>
                No external tools, authentication, proxy, redirects or
                response-body download. At most 20 approved requests per project
                per minute; two running jobs across clients.
              </p>
            </details>
            <div className="inline-actions">
              <Button
                className="button primary"
                disabled={!target || busy}
                onClick={() =>
                  void act(async () => {
                    setPlan(
                      await request<Plan>("recon_preview", { id: target }),
                    );
                    setApproved(false);
                  }, "Local execution plan ready")
                }
                disabledReason={
                  !target
                    ? "Select a target first"
                    : "An operation is in progress"
                }
              >
                Preview local HEAD plan
              </Button>
              <Button
                className="button secondary"
                onClick={() => navigate("Settings", project)}
              >
                Open security settings
              </Button>
            </div>
          </>
        )}
      </section>
      {(!!assets.length || !!jobs.length || !!target) && (
        <>
          <section className="panel records-panel">
            <h2>Observed endpoints</h2>
            <p>
              Actual completed observations. HTTP status and reported server
              names do not establish a vulnerability.
            </p>
            <div className="table-scroll">
              <table>
                <thead>
                  <tr>
                    <th>Endpoint</th>
                    <th>HTTP</th>
                    <th>Reported metadata</th>
                    <th>Last observed</th>
                  </tr>
                </thead>
                <tbody>
                  {assets.map((a) => (
                    <tr key={a.id}>
                      <td className="break-value">
                        {a.value}
                        <small className="record-meta">{a.id}</small>
                      </td>
                      <td>
                        {a.facts.status}
                        <small className="record-meta">
                          {a.facts.http_version}
                        </small>
                      </td>
                      <td>
                        {a.facts.reported_server ?? "Not available"}
                        <small className="record-meta">
                          {a.facts.content_type ?? "Content type not supplied"}
                        </small>
                      </td>
                      <td>
                        {new Date(a.last_seen).toLocaleString()}
                        <small className="record-meta">Job {a.job_id}</small>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            {!assets.length && (
              <p>
                No observed endpoints yet. Nothing is contacted until you
                approve a plan.
              </p>
            )}
            <div className="inline-actions">
              <Button
                className="button secondary"
                onClick={() => void act(load, "Inventory refreshed")}
              >
                Refresh inventory
              </Button>
              <Button
                className="button secondary"
                onClick={() => navigate("Evidence", project)}
              >
                Open inventory evidence
              </Button>
              <Button
                className="button secondary"
                onClick={() => navigate("Reports", project)}
              >
                Prepare research report
              </Button>
            </div>
          </section>
          <section className="panel records-panel">
            <h2>Local inventory jobs</h2>
            <JobHistory
              jobs={jobs}
              onCancel={(id) =>
                void act(
                  () => request("job_cancel", { id }),
                  "Cancellation requested",
                )
              }
            />
          </section>
        </>
      )}
      <details className="module-details">
        <summary>Available reconnaissance adapters</summary>
        <p>
          DNS/RDAP/certificate transparency, historical URLs, crawling and
          external scanner adapters remain unimplemented. Registering those
          target types does not make them executable through this page.
        </p>
      </details>
      <Modal
        dismissDisabled={busy}
        title="Approve local HTTP plan"
        open={!!plan}
        onClose={() => {
          if (!busy) setPlan(null);
        }}
      >
        {plan && (
          <div className="records-form">
            {error && <p role="alert">{error}</p>}
            <p className="technical break-value">
              {plan.method} {plan.url}
            </p>
            <p>
              {plan.request_budget} request maximum · {plan.timeout_ms} ms
              timeout
            </p>
            <ul>
              {plan.permissions.map((p) => (
                <li key={p}>{p}</li>
              ))}
              {plan.limitations.map((p) => (
                <li key={p}>{p}</li>
              ))}
            </ul>
            <p className="technical break-value">
              Approval fingerprint: {plan.fingerprint}
            </p>
            <label className="check">
              <input
                type="checkbox"
                checked={approved}
                onChange={(e) => setApproved(e.target.checked)}
              />
              I approve this exact request to my authorized local service.
            </label>
            <Button
              className="button primary"
              disabled={busy || !approved}
              onClick={() =>
                void act(async () => {
                  await request("recon_enqueue", {
                    id: plan.target_id,
                    fingerprint: plan.fingerprint,
                    approved,
                  });
                  setPlan(null);
                }, "Approved local inventory queued")
              }
              disabledReason={
                busy
                  ? "An operation is in progress"
                  : "Review and approve the current plan or report"
              }
            >
              Approve and queue HEAD
            </Button>
          </div>
        )}
      </Modal>
    </>
  );
}
