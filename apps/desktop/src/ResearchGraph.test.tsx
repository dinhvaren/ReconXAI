import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeAll, expect, it, vi } from "vitest";
import ResearchGraph from "./components/ResearchGraph";
const invoke = vi.hoisted(() => vi.fn());
vi.mock("./api", () => ({
  request: invoke,
  errorText: (e: unknown) => String(e),
}));
beforeAll(() => {
  HTMLDialogElement.prototype.showModal = function () {
    this.setAttribute("open", "");
  };
  HTMLDialogElement.prototype.close = function () {
    this.removeAttribute("open");
  };
});
afterEach(() => {
  cleanup();
  invoke.mockReset();
});
const page = (label: string) => ({
  revision: 1,
  source_sha256: "fixture-hash",
  total_nodes: 1,
  total_edges: 0,
  matched_nodes: 1,
  next_offset: null,
  edges_truncated: false,
  nodes: [
    {
      id: label,
      graph: "asset",
      kind: "target",
      label,
      source_table: "targets",
      source_id: label,
      attributes: { status: "observed_unverified" },
    },
  ],
  edges: [],
});
it("filters on the backend, supports keyboard pan/zoom and never executes a capability", async () => {
  const user = userEvent.setup();
  invoke.mockResolvedValue(page("Owned graph fixture"));
  render(<ResearchGraph session="session-a" revision="completed" />);
  await screen.findByRole("button", { name: "target: Owned graph fixture" });
  expect(
    screen.getByRole("button", { name: "Next graph page" }),
  ).toBeDisabled();
  await user.click(screen.getByRole("button", { name: "Zoom in graph" }));
  expect(screen.getByText(/125%/)).toBeVisible();
  const canvas = screen.getByRole("group", { name: "Provenance graph canvas" });
  const before = canvas.getAttribute("viewBox");
  fireEvent.keyDown(canvas, { key: "ArrowRight" });
  expect(canvas.getAttribute("viewBox")).not.toBe(before);
  await user.selectOptions(screen.getByLabelText("Graph type"), "asset");
  await waitFor(() =>
    expect(invoke).toHaveBeenLastCalledWith(
      "research_graph",
      expect.objectContaining({
        id: "session-a",
        query: expect.objectContaining({
          graph: "asset",
          limit: 100,
          offset: 0,
        }),
      }),
    ),
  );
  expect(invoke.mock.calls.every(([op]) => op === "research_graph")).toBe(true);
});
it("late results from another session cannot replace nodes or record inspection", async () => {
  const user = userEvent.setup();
  let old: (value: unknown) => void = () => {};
  invoke.mockImplementation((op: string, data: { id: string }) =>
    op === "research_graph_record"
      ? Promise.resolve({ record: { id: "new" } })
      : data.id === "old"
        ? new Promise((resolve) => {
            old = resolve;
          })
        : Promise.resolve(page("New session target")),
  );
  const view = render(<ResearchGraph session="old" revision="one" />);
  await waitFor(() => expect(invoke).toHaveBeenCalled());
  view.rerender(<ResearchGraph session="new" revision="two" />);
  await screen.findByRole("button", { name: "target: New session target" });
  old(page("Old session target"));
  await new Promise((resolve) => setTimeout(resolve, 10));
  expect(
    screen.queryByRole("button", { name: "target: Old session target" }),
  ).not.toBeInTheDocument();
  await user.click(
    screen.getByRole("button", { name: "Inspect linked record" }),
  );
  await screen.findByRole("dialog", { name: "Linked provenance record" });
  expect(invoke).toHaveBeenCalledWith("research_graph_record", {
    id: "new",
    node: "New session target",
  });
});
