import { Button } from "../components/controls";
import { useCallback, useEffect, useRef, useState } from "react";
import { request, errorText, type Snapshot } from "../api";
import { useJobEvents, type AnalysisJob } from "../research";
import { Heading, Modal, type Perform } from "../components/ui";
import { EmptyState } from "../components/controls";
import ReconAssetInventory from "../components/ReconAssetInventory";
import ReconToolControls from "../components/ReconToolControls";
import JobHistory from "../components/JobHistory";
import StartResearchButton from "../components/StartResearchButton";
import {
  useWorkspace,
  useWorkspaceProject,
  useWorkspaceTarget,
} from "../workspace";
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
  const workspace = useWorkspace();
  const [project, setProject] = useWorkspaceProject(
      projectId ?? data.projects[0]?.id ?? "",
    ),
    [target, setTarget] = useWorkspaceTarget(initialTarget ?? "");
  const [jobs, setJobs] = useState<AnalysisJob[]>([]),
    [plan, setPlan] = useState<Plan | null>(null),
    [approved, setApproved] = useState(false),
    [error, setError] = useState("");
  const loadVersion = useRef(0);
  const load = useCallback(async () => {
    const version = ++loadVersion.current;
    if (!project) {
      setJobs([]);
      return;
    }
    const j = await request<AnalysisJob[]>("jobs_list", { project });
    if (version !== loadVersion.current) return;
    setJobs(
      j.filter(
        (j) =>
          (j.kind === "local_http_inventory" || j.kind.startsWith("recon_")) &&
          (!target || j.target_id === target),
      ),
    );
  }, [project, target]);
  useEffect(() => {
    setJobs([]);
    setPlan(null);
    setApproved(false);
    return () => {
      loadVersion.current += 1;
    };
  }, [project, target]);
  useEffect(() => {
    void load().catch((e) => setError(errorText(e)));
  }, [load, data.audit[0]?.id]);
  useEffect(() => {
    if (workspace) return;
    if (projectId) setProject(projectId);
    setTarget(initialTarget ?? "");
  }, [projectId, initialTarget]);
  useJobEvents((j) => {
    if (
      j.project_id === project &&
      (j.kind === "local_http_inventory" || j.kind.startsWith("recon_"))
    )
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
      ["domain", "url", "rest_api", "graphql"].includes(t.kind),
  );
  return (
    <>
      <Heading
        eyebrow="Authorized reconnaissance"
        title="Reconnaissance"
        description="Discover permitted assets, inspect real observations and keep research evidence together."
        action={
          <div className="inline-actions">
            <StartResearchButton />
            {!target && <Button onClick={onAddTarget}>Add Target</Button>}
          </div>
        }
      />
      {error && <p role="alert">{error}</p>}
      <ReconToolControls
        target={target}
        navigate={navigate}
        onQueued={() => void load().catch((e) => setError(errorText(e)))}
      />
      <p>
        AI model:{" "}
        {data.providers?.find((p) => p.id === data.settings.default_provider_id)
          ?.model ?? "Not configured"}
        . Select a model and review the complete workflow in Start AI Research.
      </p>
      <section className="panel records-panel">
        <h2>Research target</h2>
        <div className="records-toolbar">
          <label>
            Inventory project
            <select
              aria-label="Inventory project"
              value={project}
              onChange={(e) => {
                setProject(e.target.value);
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
              <option value="">Select domain / URL / API target</option>
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
              <summary>Legacy loopback HEAD: policy and limits</summary>
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
            <details>
              <summary>Manual loopback HEAD</summary>
              <div className="inline-actions">
                <Button
                  className="button secondary"
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
            </details>
          </>
        )}
      </section>
      {project && (
        <ReconAssetInventory
          project={project}
          target={target}
          revision={jobs.map((j) => j.id + j.status).join(":")}
        />
      )}
      {(!!jobs.length || !!target) && (
        <>
          <section className="panel records-panel">
            <h2>Recent recon jobs</h2>
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
          DNS/RDAP/certificate transparency, historical URLs, crawling and other
          scanner adapters remain unimplemented. HTTPX and Subfinder use
          reviewed Marketplace extensions and separate network approval.
          Registering those target types does not make them executable through
          this page.
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
              className="button secondary"
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
