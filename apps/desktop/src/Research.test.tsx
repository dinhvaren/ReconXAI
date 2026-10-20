import { beforeEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import {
  WorkspaceProvider,
  useWorkspace,
  type WorkspaceSelection,
} from "./workspace";
import ResearchPage from "./pages/ResearchPage";
import type { Snapshot } from "./api";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
beforeEach(() => {
  cleanup();
  localStorage.clear();
  vi.clearAllMocks();
});
function Controls() {
  const w = useWorkspace()!;
  return (
    <>
      <output data-testid="selection">
        {w.selection.project_id}/{w.selection.target_id}
      </output>
      <button onClick={() => void w.select("p2", "t2")}>
        Change workspace
      </button>
    </>
  );
}
it("ignores an old context read that returns after a user selection was saved", async () => {
  let release!: (s: WorkspaceSelection) => void;
  let reads = 0;
  let persisted: WorkspaceSelection = {
    revision: 0,
    project_id: "p1",
    target_id: "t1",
  };
  vi.mocked(invoke).mockImplementation(async (_command, args) => {
    const r = (
      args as { request: { op: string; project: string; target: string } }
    ).request;
    if (r.op === "workspace_selection") {
      if (++reads === 1)
        return new Promise<WorkspaceSelection>((resolve) => {
          release = resolve;
        });
      return persisted;
    }
    persisted = { revision: 1, project_id: r.project, target_id: r.target };
    return persisted;
  });
  render(
    <WorkspaceProvider>
      <Controls />
    </WorkspaceProvider>,
  );
  await userEvent.click(screen.getByText("Change workspace"));
  await waitFor(() => expect(persisted.project_id).toBe("p2"));
  await act(async () =>
    release({ revision: 0, project_id: "p1", target_id: "t1" }),
  );
  expect(screen.getByTestId("selection")).toHaveTextContent("p2/t2");
});
const data = {
  projects: [{ id: "p1", name: "Fixture project", archived: false }],
  targets: [
    {
      id: "t1",
      project_id: "p1",
      name: "Owned fixture",
      kind: "url",
      authorized: true,
      scope_status: "allowed",
    },
  ],
  providers: [
    {
      id: "model",
      name: "Local fixture model",
      model: "mock",
      base_url: "http://127.0.0.1:9999/v1",
      enabled: true,
    },
  ],
  settings: { default_provider_id: "model" },
} as unknown as Snapshot;
it("restores the persisted target session when WebView localStorage has no session hint", async () => {
  vi.mocked(invoke).mockImplementation(async (_command, args) => {
    const r = (args as { request: { op: string } }).request;
    if (r.op === "workspace_selection")
      return { revision: 1, project_id: "p1", target_id: "t1" };
    if (r.op === "research_list") return ["persisted-session"];
    if (r.op === "research_show")
      return {
        plan: {
          id: "persisted-session",
          created_at: "2026-10-20",
          planning_usage: null,
          preview: {
            project_id: "p1",
            target_name: "Owned fixture",
            model: "mock",
            request: { target_id: "t1" },
            capabilities: [],
          },
          proposal: { steps: [] },
        },
        status: "completed",
        jobs: [],
        evidence: [],
        findings: [],
        reports: [],
        interpretation: null,
      };
    throw new Error(`Unexpected ${r.op}`);
  });
  render(
    <WorkspaceProvider>
      <ResearchPage data={data} navigate={() => {}} />
    </WorkspaceProvider>,
  );
  expect(
    await screen.findByRole("button", {
      name: "Prepare research report",
    }),
  ).toBeInTheDocument();
  expect(localStorage.getItem("reconx.research.t1")).toBe("persisted-session");
});
it("opens a target-bound research context without sending any AI or HTTP request before consent", async () => {
  const operations: string[] = [];
  vi.mocked(invoke).mockImplementation(async (_command, args) => {
    const r = (args as { request: { op: string } }).request;
    operations.push(r.op);
    if (r.op === "workspace_selection")
      return { revision: 1, project_id: "p1", target_id: "t1" };
    if (r.op === "research_list") return [];
    if (r.op === "research_preview")
      return {
        request: { target_id: "t1", provider_id: "model", mode: "ai_assisted" },
        project_id: "p1",
        target_name: "Owned fixture",
        target_kind: "url",
        model: "mock",
        gateway: "http://127.0.0.1:9999/v1",
        context: "Reviewed metadata only",
        fingerprint: "fp",
        capabilities: [
          {
            id: "local.scope_review",
            name: "Offline scope",
            available: true,
            reason: "No network",
            network_requests: 0,
          },
        ],
        limitations: ["Bounded research only"],
      };
    throw new Error(`Unexpected ${r.op}`);
  });
  render(
    <WorkspaceProvider>
      <ResearchPage data={data} navigate={() => {}} />
    </WorkspaceProvider>,
  );
  await waitFor(() =>
    expect(screen.getByLabelText("Research target")).toHaveValue("t1"),
  );
  await userEvent.click(screen.getByText("Review research context"));
  expect(await screen.findByText("Reviewed metadata only")).toBeInTheDocument();
  expect(
    screen.getByRole("button", { name: "Generate research plan" }),
  ).toBeDisabled();
  expect(operations).not.toContain("research_plan");
  expect(operations).not.toContain("research_approve");
});
