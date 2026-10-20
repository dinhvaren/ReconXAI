import { render, screen, cleanup, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeAll, afterEach, it, expect, vi } from "vitest";
import ManagedMcpDetails from "./components/ManagedMcpDetails";
import RegistryBrowser from "./components/RegistryBrowser";
import type { ExtensionEntry, ExtensionPlan } from "./extensions";
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
const entry = {
  manifest: {
    id: "reconx.mcp-everything",
    name: "MCP Everything",
    description: "Reviewed server",
    permissions: ["execute:user"],
    operating_systems: ["windows", "linux"],
    architectures: ["x86_64"],
  },
  available_version: "2.0.0",
  status: "not_installed",
  supported: true,
  installed: null,
  managed: {
    allowed_tools: [],
    tools: [],
    active_job: null,
    package_name: "@modelcontextprotocol/server-everything",
    runtime_requirement: "Node >=24",
  },
} as unknown as ExtensionEntry;
it("requires exact MCP install approval and never marks an enqueued job Ready", async () => {
  const user = userEvent.setup();
  const plan = {
    extension: entry,
    fingerprint: "exact-plan",
    actions: ["Verify all archives"],
    network_access: "No OS sandbox",
    package_plan: {
      version: "2.0.0",
      runtime: {
        managed: true,
        version: "24.22.0",
        sha256: "verified-runtime",
      },
      dependencies: [],
      minimum_free_bytes: 536870912,
    },
  } as unknown as ExtensionPlan;
  invoke.mockImplementation(async (op: string) => {
    if (op === "managed_mcp_history") return [];
    if (op === "managed_mcp_plan") return plan;
    if (op === "managed_mcp_install")
      return { id: "real-queue-id", status: "queued" };
    throw Error(op);
  });
  render(
    <ManagedMcpDetails
      entry={entry}
      refresh={vi.fn().mockResolvedValue(undefined)}
      onOpenJobs={vi.fn()}
    />,
  );
  await user.click(screen.getByRole("button", { name: "Install MCP" }));
  const install = await screen.findByRole("button", {
    name: "Approve and install MCP",
  });
  expect(install).toBeDisabled();
  await user.click(
    screen.getByRole("checkbox", {
      name: "I approve this exact dependency plan and user-level MCP execution",
    }),
  );
  await user.click(install);
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith("managed_mcp_install", {
      approval: {
        id: entry.manifest.id,
        version: "2.0.0",
        managed_runtime: true,
        fingerprint: "exact-plan",
        approved: true,
      },
    }),
  );
  expect(
    await screen.findByText(/Installation queued: real-queue-id/),
  ).toBeVisible();
  expect(screen.queryByText("ready", { exact: true })).not.toBeInTheDocument();
});
it("disables tools that lack explicit read-only and closed-world annotations", () => {
  const installed = {
    ...entry,
    status: "ready",
    installed: { enabled: true, version: "2.0.0" },
    managed: {
      ...entry.managed!,
      tools: [
        {
          name: "echo",
          annotations: { readOnlyHint: true, openWorldHint: false },
        },
        { name: "unsafe" },
      ],
    },
  } as ExtensionEntry;
  invoke.mockResolvedValue([]);
  render(
    <ManagedMcpDetails
      entry={installed}
      refresh={vi.fn().mockResolvedValue(undefined)}
      onOpenJobs={vi.fn()}
    />,
  );
  expect(screen.getByRole("checkbox", { name: "echo" })).toBeEnabled();
  expect(screen.getByRole("checkbox", { name: /unsafe/ })).toBeDisabled();
});
it("keeps Registry metadata untrusted and sends the opaque pagination cursor unchanged", async () => {
  const user = userEvent.setup();
  const row = {
    id: "io.example/server",
    name: "Example MCP",
    description: "Community entry",
    publisher: "io.example",
    version: "1",
    packages: [],
    transports: ["stdio"],
    trust: "official_registry_entry_unreviewed",
    schema_supported: false,
    server_json: {},
    registry_metadata: {},
  };
  const page = {
    api_version: "v0.1",
    entries: [row],
    next_cursor: "opaque+/=?cursor",
    cached: false,
    stale: false,
    fetched_at: "now",
    expires_at: "later",
    warning: null,
  };
  invoke.mockResolvedValue(page);
  render(<RegistryBrowser search="example" />);
  await user.click(
    screen.getByRole("button", { name: /Search MCP Registry/i }),
  );
  expect((await screen.findAllByText("Example MCP")).length).toBeGreaterThan(0);
  await user.click(screen.getByRole("button", { name: /Next registry page/i }));
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith("registry_search", {
      query: {
        search: "example",
        cursor: "opaque+/=?cursor",
        runtime: null,
        offline: false,
        refresh: false,
      },
    }),
  );
  expect(screen.getByText(/Manual setup required/i)).toBeVisible();
});
