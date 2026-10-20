import { beforeEach, afterEach, it, expect, vi } from "vitest";
import {
  render,
  screen,
  waitFor,
  cleanup,
  within,
  act,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { selectPaths } from "./research";
import HttpPage from "./pages/HttpPage";
import type { Snapshot } from "./api";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(), Channel: class {} }));
vi.mock("./research", () => ({ selectPaths: vi.fn(), useJobEvents: () => {} }));
let drop:
  ((e: { payload: { type: string; paths: string[] } }) => void) | undefined;
vi.mock("@tauri-apps/api/webviewWindow", () => ({
  getCurrentWebviewWindow: () => ({
    onDragDropEvent: (fn: typeof drop) => {
      drop = fn;
      return Promise.resolve(() => {});
    },
  }),
}));
const data: Snapshot = {
  projects: [
    {
      id: "p",
      name: "Local fixture",
      description: "",
      created_at: "",
      updated_at: "",
      archived: false,
    },
  ],
  targets: [],
  providers: [],
  audit: [],
  database_path: "isolated",
  version: "0.1",
  settings: {
    theme: "reconx",
    security_mode: "passive_auto",
    default_provider_id: null,
    fallback_provider_id: null,
  },
};
const empty = {
  imports: [],
  traffic: [],
  operations: [],
  endpoints: [],
  total: 0,
  offset: 0,
  page_size: 100,
  comparisons: [],
};
let imported: Record<string, unknown> | undefined,
  reject = false;
beforeEach(() => {
  Object.defineProperties(HTMLDialogElement.prototype, {
    showModal: {
      configurable: true,
      value: function (this: HTMLDialogElement) {
        this.setAttribute("open", "");
      },
    },
    close: {
      configurable: true,
      value: function (this: HTMLDialogElement) {
        this.removeAttribute("open");
      },
    },
  });
  imported = undefined;
  reject = false;
  drop = undefined;
  vi.mocked(selectPaths).mockReset();
  vi.mocked(invoke).mockImplementation(async (_command, args) => {
    const value = (args as { request: Record<string, unknown> }).request;
    if (value.op === "http_workspace") return structuredClone(empty);
    if (value.op === "http_import") {
      if (reject)
        throw { message: "Malformed HAR; original records unchanged" };
      imported = value;
      return { id: "imported" };
    }
    return null;
  });
});
afterEach(cleanup);
const perform = async (fn: () => Promise<unknown>) => {
  try {
    await fn();
  } catch {
    /* App performs user-visible error handling; dialog also reports it. */
  }
};
it("queues the selected capture/operation through the shared backend and shows inconclusive schema evidence honestly", async () => {
  const user = userEvent.setup();
  let queued: Record<string, unknown> | undefined;
  const message = {
    headers: [],
    cookies: [],
    body: {
      mime_type: "application/json",
      preview: { id: 1 },
      decoded_size: 8,
      encoded: false,
    },
    http_version: "HTTP/1.1",
  };
  const result = {
    checks: 3,
    network_requests: 0,
    coverage: "partial checks",
    observations: [
      {
        classification: "insufficient_evidence",
        rule: "redacted_value",
        message: "Value was redacted; no type verdict",
        capture_pointer: "/log/entries/0/response/content/text",
        instance_pointer: "/token",
        openapi_pointer: "/components/schemas/User/properties/token",
        capture_evidence_id: "har-evidence",
        openapi_evidence_id: "spec-evidence",
      },
    ],
  };
  vi.mocked(invoke).mockImplementation(async (_command, args) => {
    const value = (args as { request: Record<string, unknown> }).request;
    if (value.op === "http_workspace")
      return {
        ...empty,
        total: 1,
        traffic: [
          {
            id: "capture",
            import_id: "har",
            project_id: "p",
            target_id: null,
            method: "GET",
            url: "https://api.example.test/api/users/1",
            host: "api.example.test",
            path: "/api/users/1",
            status: 200,
            identity: "alice",
            role: "reader",
            started_at: "",
            elapsed_ms: 1,
            authentication: [],
            query_parameters: [],
            request: message,
            response: message,
          },
        ],
        operations: [
          {
            id: "operation",
            import_id: "spec",
            target_id: null,
            method: "GET",
            path: "/users/{id}",
            summary: "Owned API",
          },
        ],
        analysis_jobs: queued
          ? [
              {
                exchange_id: "capture",
                job: {
                  id: "job",
                  kind: "offline_schema_review",
                  status: "completed",
                  progress: 100,
                  stage: "Completed",
                  result,
                  logs: ["Offline"],
                },
              },
            ]
          : [],
      };
    if (value.op === "http_schema_enqueue") {
      queued = value;
      return { id: "job" };
    }
    if (value.op === "http_note") return "";
    return null;
  });
  render(
    <HttpPage
      page="API Security"
      data={data}
      projectId="p"
      busy={false}
      perform={perform}
      navigate={() => {}}
    />,
  );
  await screen.findByRole("heading", { name: "Traffic captures (1)" });
  await user.click(
    document.querySelector(".http-explorer button.http-item") as HTMLElement,
  );
  const button = screen.getByRole("button", {
    name: "Check documented schema",
  });
  expect(button).toBeDisabled();
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Schema operation" }),
    "operation",
  );
  expect(queued).toBeUndefined();
  const formats = screen.getByRole("checkbox", {
    name: "Assert supported formats",
  });
  expect(formats).not.toBeChecked();
  await user.click(formats);
  await user.click(button);
  await screen.findByRole("region", { name: "Offline schema observations" });
  expect(queued).toEqual({
    op: "http_schema_enqueue",
    exchange: "capture",
    operation: "operation",
    options: { assert_formats: true },
  });
  expect(screen.getByText("Insufficient evidence")).toBeInTheDocument();
  await user.click(screen.getByText("Source evidence and schema location"));
  expect(screen.getByText("har-evidence")).toBeInTheDocument();
  expect(screen.getByText("spec-evidence")).toBeInTheDocument();
  expect(
    screen.getByRole("button", { name: "Edit request / replay to loopback" }),
  ).toBeEnabled();
});
it("requires a project and never renders fabricated traffic", async () => {
  render(
    <HttpPage
      page="Web Security"
      data={data}
      busy={false}
      perform={perform}
      navigate={() => {}}
    />,
  );
  expect(screen.getByRole("button", { name: "Import HAR" })).toBeDisabled();
  expect(
    screen.getByRole("heading", { name: "Select a research project" }),
  ).toBeInTheDocument();
  expect(invoke).not.toHaveBeenCalled();
});
it("shows the backend identity matrix beyond the current capture page without treating statuses as permissions", async () => {
  const user = userEvent.setup();
  vi.mocked(invoke).mockImplementation(async (_command, args) => {
    const value = (args as { request: Record<string, unknown> }).request;
    if (value.op === "http_workspace")
      return {
        ...empty,
        authorization_matrix: [
          {
            origin: "https://api.example.test:8443",
            method: "GET",
            path: "/users/{userId}",
            identity: "alice",
            role: "reader",
            capture_count: 105,
            observed_paths: 2,
            statuses: [200, 403],
            body_variants: 3,
            capture_ids: [],
          },
        ],
      };
    return null;
  });
  render(
    <HttpPage
      page="API Security"
      data={data}
      projectId="p"
      busy={false}
      perform={perform}
      navigate={() => {}}
    />,
  );
  await screen.findByRole("heading", { name: "Traffic captures (0)" });
  await user.click(
    screen.getByRole("tab", { name: "Authorization comparison" }),
  );
  const matrix = screen.getByRole("table", {
    name: "Captured authorization observations",
  });
  expect(within(matrix).getByText("Status 200, 403")).toBeInTheDocument();
  expect(
    within(matrix).getByText("105 capture(s) · 2 path(s)"),
  ).toBeInTheDocument();
  expect(
    within(matrix).getByText(
      "GET https://api.example.test:8443/users/{userId}",
    ),
  ).toBeInTheDocument();
  expect(
    screen.getByText(/does not authenticate an identity/),
  ).toBeInTheDocument();
});
it("native picker selection is reviewed, recoverable parser errors retain input, and retry uses the typed service", async () => {
  const user = userEvent.setup();
  vi.mocked(selectPaths).mockResolvedValue(["D:/owned-fixture.har"]);
  render(
    <HttpPage
      page="Web Security"
      data={data}
      projectId="p"
      busy={false}
      perform={perform}
      navigate={() => {}}
    />,
  );
  await user.click(screen.getByRole("button", { name: "Import HAR" }));
  const dialog = screen.getByRole("dialog", {
    name: "Review offline HTTP import",
  });
  expect(imported).toBeUndefined();
  await user.type(
    within(dialog).getByLabelText("Captured identity label"),
    "alice",
  );
  reject = true;
  await user.click(
    within(dialog).getByRole("button", { name: "Import selected files" }),
  );
  await waitFor(() =>
    expect(within(dialog).getByRole("alert")).toHaveTextContent(
      "Malformed HAR",
    ),
  );
  expect(within(dialog).getByLabelText("Captured identity label")).toHaveValue(
    "alice",
  );
  reject = false;
  await user.click(
    within(dialog).getByRole("button", { name: "Import selected files" }),
  );
  await waitFor(() =>
    expect(imported).toMatchObject({
      op: "http_import",
      kind: "har",
      path: "D:/owned-fixture.har",
      input: { project_id: "p", target_id: null, identity: "alice" },
    }),
  );
  await waitFor(() =>
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
  );
  expect(
    screen.getByText("No matching captures. Import a HAR file to begin."),
  ).toBeInTheDocument();
});
it("native drop only opens an import review and does not replay or import without confirmation", async () => {
  render(
    <HttpPage
      page="API Security"
      data={data}
      projectId="p"
      busy={false}
      perform={perform}
      navigate={() => {}}
    />,
  );
  await waitFor(() => expect(drop).toBeDefined());
  await act(async () =>
    drop?.({ payload: { type: "drop", paths: ["D:/owned-api.yaml"] } }),
  );
  const dialog = screen.getByRole("dialog", {
    name: "Review offline HTTP import",
  });
  expect(within(dialog).getByLabelText("HTTP import format")).toHaveValue(
    "openapi",
  );
  expect(imported).toBeUndefined();
  expect(
    vi
      .mocked(invoke)
      .mock.calls.every(
        ([, args]) =>
          (args as { request: { op: string } }).request.op === "http_workspace",
      ),
  ).toBe(true);
});
