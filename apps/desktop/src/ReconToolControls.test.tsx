import { beforeEach, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import ReconToolControls from "./components/ReconToolControls";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
beforeEach(() => {
  HTMLDialogElement.prototype.showModal=function(){this.setAttribute("open","");};
  HTMLDialogElement.prototype.close=function(){this.removeAttribute("open");};
  cleanup();
  vi.clearAllMocks();
});
const plan = {
  config: {
    target_id: "owned",
    tool: "httpx",
    max_assets: 4,
    timeout_secs: 10,
  },
  target: "http://127.0.0.1:9000/",
  version: "fixture",
  fingerprint: "reviewed",
  permissions: ["Explicit GET approval"],
  limits: ["No redirects"],
};
it("does not run a real-capability request before exact plan approval", async () => {
  const calls: string[] = [];
  vi.mocked(invoke).mockImplementation(async (_c, args) => {
    const r = (
      args as {
        request: {
          op: string;
          config: unknown;
          plan: unknown;
          approved: boolean;
        };
      }
    ).request;
    calls.push(r.op);
    if (r.op === "extensions_catalog")
      return [{ manifest: { id: "httpx", name: "HTTPX" }, status: "ready" }];
    if (r.op === "recon_tool_plan") return plan;
    if (r.op === "recon_tool_run") {
      expect(r.plan).toEqual(plan);
      expect(r.approved).toBe(true);
      return { id: "job" };
    }
    throw Error("Unexpected operation");
  });
  const queued = vi.fn();
  render(
    <ReconToolControls target="owned" navigate={() => {}} onQueued={queued} />,
  );
  await userEvent.click(screen.getByText("Manual managed-tool execution"));
  await waitFor(() =>
    expect(
      screen.getByRole("button", { name: "Review tool plan" }),
    ).toBeEnabled(),
  );
  await userEvent.click(
    screen.getByRole("button", { name: "Review tool plan" }),
  );
  const run = await screen.findByRole("button", { name: "Run approved recon" });
  expect(run).toBeDisabled();
  expect(calls).not.toContain("recon_tool_run");
  await userEvent.click(
    screen.getByLabelText(
      "I approve this destination, passive data sharing and resource limits.",
    ),
  );
  await userEvent.click(run);
  await waitFor(() => expect(queued).toHaveBeenCalledOnce());
});
it("ignores a plan response arriving after the active target changes", async () => {
  let resolve!: (v: unknown) => void;
  vi.mocked(invoke).mockImplementation(async (_c, args) => {
    const r = (args as { request: { op: string } }).request;
    if (r.op === "extensions_catalog")
      return [{ manifest: { id: "httpx", name: "HTTPX" }, status: "ready" }];
    if (r.op === "recon_tool_plan")
      return new Promise((r) => {
        resolve = r;
      });
    throw Error("No execution permitted");
  });
  const props = { navigate: () => {}, onQueued: () => {} };
  const view = render(<ReconToolControls target="owned" {...props} />);
  await userEvent.click(screen.getByText("Manual managed-tool execution"));
  await waitFor(() =>
    expect(
      screen.getByRole("button", { name: "Review tool plan" }),
    ).toBeEnabled(),
  );
  await userEvent.click(
    screen.getByRole("button", { name: "Review tool plan" }),
  );
  view.rerender(<ReconToolControls target="different" {...props} />);
  await act(async () => resolve(plan));
  expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
});
