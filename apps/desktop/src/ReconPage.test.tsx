import { afterEach, expect, it, vi } from "vitest";
import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import type { Snapshot } from "./api";
import type { AnalysisJob } from "./research";
import ReconPage from "./pages/ReconPage";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
let onJob: ((job: AnalysisJob) => void) | undefined;
vi.mock("./research", () => ({
  useJobEvents: (handler: typeof onJob) => {
    onJob = handler;
  },
  activeJob: (job: AnalysisJob) => ["queued", "running"].includes(job.status),
}));
afterEach(cleanup);
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((fn) => {
    resolve = fn;
  });
  return { promise, resolve };
}

it("a late queued snapshot cannot replace a completed job and its asset", async () => {
  const oldAssets = deferred<unknown[]>(),
    oldJobs = deferred<AnalysisJob[]>();
  const job: AnalysisJob = {
    id: "job-1",
    project_id: "p",
    kind: "local_http_inventory",
    status: "completed",
    stage: "Completed",
    progress: 100,
    logs: [],
    created_at: "2026-10-18T00:00:00Z",
    timeout_ms: 5000,
    target_id: null,
    file_id: null,
    started_at: null,
    finished_at: null,
    cancel_requested: false,
    result: null,
    error_code: null,
    error_message: null,
  };
  const asset = {
    id: "asset-1",
    value: "http://127.0.0.1:18000/",
    target_id: null,
    last_seen: job.created_at,
    job_id: job.id,
    facts: {
      status: 200,
      http_version: "HTTP/1.1",
      reported_server: "owned fixture",
      content_type: "text/plain",
      header_count: 1,
    },
  };
  let assetsCalls = 0,
    jobsCalls = 0;
  vi.mocked(invoke).mockImplementation(async (_command, args) => {
    const { op } = (args as { request: { op: string } }).request;
    if (op === "assets_list")
      return ++assetsCalls === 1 ? oldAssets.promise : [asset];
    if (op === "jobs_list") return ++jobsCalls === 1 ? oldJobs.promise : [job];
    return null;
  });
  const data = {
    projects: [{ id: "p", name: "Owned local fixture", archived: false }],
    settings: { security_mode: "lab_research" },
    targets: [],
    audit: [],
  } as unknown as Snapshot;
  render(
    <ReconPage
      data={data}
      projectId="p"
      busy={false}
      perform={async (fn) => {
        await fn();
      }}
      onAddTarget={() => {}}
      navigate={() => {}}
    />,
  );
  await waitFor(() => expect(jobsCalls).toBe(1));
  await act(async () => {
    onJob?.(job);
  });
  await screen.findByText("completed", { exact: true });
  await act(async () => {
    oldAssets.resolve([]);
    oldJobs.resolve([
      { ...job, status: "queued", stage: "Queued", progress: 0 },
    ]);
    await Promise.all([oldAssets.promise, oldJobs.promise]);
  });
  expect(screen.getByText("completed", { exact: true })).toBeInTheDocument();
  expect(
    screen.getByRole("cell", { name: /owned fixture/ }),
  ).toBeInTheDocument();
});
