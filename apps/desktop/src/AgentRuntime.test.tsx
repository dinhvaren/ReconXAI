import { beforeEach, expect, it, vi } from "vitest";
import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import AgentRuntimePanel from "./components/AgentRuntimePanel";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
beforeEach(() => {
  cleanup();
  vi.clearAllMocks();
});
const limits = {
  concurrency: 2,
  max_invocations: 24,
  max_duration_ms: 600000,
  max_retries: 1,
};
const preview = {
  session_id: "s1",
  fingerprint: "actual-reviewed-fingerprint",
  limits,
  steps: [
    {
      id: "scope",
      agent_id: "recon",
      capability_id: "local.scope_review",
      dependencies: [],
    },
  ],
  limitations: ["No network/MCP/model-driven continuation"],
};
const runtime = {
  session_id: "s1",
  revision: 2,
  status: "running",
  stop_reason: null,
  limits,
  invocations: 1,
  deadline_ms: Date.now() + 600000,
  runtime_cost: null,
  model_requests_by_runtime: 0,
  network_requests_by_runtime: 0,
  tasks: [
    {
      id: "s1:scope",
      agent_id: "recon",
      capability_id: "local.scope_review",
      state: "RUNNING",
      job_id: "job1",
      attempt: 0,
      dependencies: [],
      error_code: null,
    },
  ],
  messages: [],
  next_message: null,
};
const props = {
  session: "s1",
  status: "awaiting_approval",
  enabled: false,
  eligible: true,
  readOnly: false,
  changed: vi.fn(async () => {}),
};
it("requires explicit approval of the fingerprinted DAG and invalidates review after a budget change", async () => {
  const calls: Record<string, unknown>[] = [];
  vi.mocked(invoke).mockImplementation(async (_op, args) => {
    const r = (args as { request: Record<string, unknown> }).request;
    calls.push(r);
    if (r.op === "agent_runtime_preview")
      return { ...preview, limits: r.limits };
    if (r.op === "agent_runtime_start") return runtime;
    throw Error(String(r.op));
  });
  render(<AgentRuntimePanel {...props} />);
  expect(calls).toHaveLength(0);
  await userEvent.click(
    screen.getByRole("button", { name: "Review offline agent DAG" }),
  );
  expect(
    await screen.findByRole("button", { name: "Start offline agent DAG" }),
  ).toBeDisabled();
  await userEvent.click(screen.getByLabelText("Approve offline agent DAG"));
  await userEvent.clear(screen.getByLabelText("Agent invocation budget"));
  await userEvent.type(screen.getByLabelText("Agent invocation budget"), "20");
  expect(
    screen.queryByRole("button", { name: "Start offline agent DAG" }),
  ).not.toBeInTheDocument();
  await userEvent.click(
    screen.getByRole("button", { name: "Review offline agent DAG" }),
  );
  await screen.findByLabelText("Approve offline agent DAG");
  await userEvent.click(screen.getByLabelText("Approve offline agent DAG"));
  await userEvent.click(
    screen.getByRole("button", { name: "Start offline agent DAG" }),
  );
  const start = calls.find((c) => c.op === "agent_runtime_start");
  expect(start?.review).toEqual({
    session_id: "s1",
    fingerprint: preview.fingerprint,
    limits: { ...limits, max_invocations: 20 },
    approved: true,
  });
  expect(await screen.findByText(/Cost: not reported/)).toBeInTheDocument();
  expect(calls.some((c) => String(c.op).includes("chat"))).toBe(false);
});
it("does not execute or request a preview for a historical or unsupported plan", () => {
  render(<AgentRuntimePanel {...props} readOnly eligible={false} />);
  expect(
    screen.getByRole("button", { name: "Review offline agent DAG" }),
  ).toBeDisabled();
  expect(invoke).not.toHaveBeenCalled();
});
it("discards a late preview when the selected session changes", async () => {
  let release!: (value: typeof preview) => void;
  vi.mocked(invoke).mockImplementation(
    () =>
      new Promise((resolve) => {
        release = resolve;
      }),
  );
  const view = render(<AgentRuntimePanel {...props} />);
  await userEvent.click(
    screen.getByRole("button", { name: "Review offline agent DAG" }),
  );
  view.rerender(<AgentRuntimePanel {...props} session="s2" />);
  await act(async () => release(preview));
  expect(
    screen.queryByRole("button", { name: "Start offline agent DAG" }),
  ).not.toBeInTheDocument();
  expect(
    screen.queryByRole("list", { name: "Reviewed agent DAG" }),
  ).not.toBeInTheDocument();
});
it("sends a real pause request and displays resume only after the backend reaches paused", async () => {
  const calls: string[] = [];
  vi.mocked(invoke).mockImplementation(async (_op, args) => {
    const r = (args as { request: { op: string } }).request;
    calls.push(r.op);
    if (r.op === "agent_runtime_show") return runtime;
    if (r.op === "agent_runtime_pause")
      return { ...runtime, revision: 3, status: "paused" };
    throw Error(r.op);
  });
  render(<AgentRuntimePanel {...props} enabled status="running" />);
  await userEvent.click(
    await screen.findByRole("button", { name: "Pause after current jobs" }),
  );
  await waitFor(() => expect(calls).toContain("agent_runtime_pause"));
  expect(
    await screen.findByRole("button", { name: "Resume offline DAG" }),
  ).toBeInTheDocument();
});
