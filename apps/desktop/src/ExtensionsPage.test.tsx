import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeAll, expect, it, vi } from "vitest";
import ExtensionsPage from "./pages/ExtensionsPage";
const invoke = vi.hoisted(() => vi.fn());
beforeAll(() => {
  HTMLDialogElement.prototype.showModal = function () {
    this.setAttribute("open", "");
  };
  HTMLDialogElement.prototype.close = function () {
    this.removeAttribute("open");
  };
});
vi.mock("./api", () => ({
  request: invoke,
  installNativeExtension: (approval: unknown) =>
    invoke("extensions_install", { approval }),
  errorText: (e: unknown) => String(e),
}));
const entry = {
  manifest: {
    id: "reconx.yara-signatures",
    name: "YARA-X File Signatures",
    publisher: "ReconX AI",
    version: "0.1.0",
    description: "Fixed real file signatures",
    category: "Reverse Engineering",
    compatible_reconx: "0.1.0",
    operating_systems: ["windows", "linux"],
    architectures: ["x86_64"],
    required_runtime: "Job Engine",
    permissions: ["read:explicitly-imported-file"],
    required_tools: ["yara-x"],
  },
  status: "not_installed",
  supported: true,
  installed: null,
  dependencies: [],
  update_available: false,
  available_version: "0.1.0",
  installation_size: null,
  source_repository: "https://github.com/dinhvaren/ReconXAI",
};
afterEach(() => {
  cleanup();
  invoke.mockReset();
});
it("search selection binds the installation plan to the visible extension", async () => {
  const user = userEvent.setup();
  const httpx = {
    ...entry,
    manifest: {
      ...entry.manifest,
      id: "httpx",
      name: "ProjectDiscovery httpx",
      description: "HTTP probing",
    },
  };
  invoke.mockImplementation(async (op: string) => {
    if (op === "extensions_catalog") return [entry, httpx];
    if (op === "extensions_history" || op === "files_list") return [];
    if (op === "extensions_plan")
      return {
        extension: httpx,
        fingerprint: "reviewed-httpx",
        actions: [],
        requires_admin: false,
        network_access: "Pinned download",
      };
    throw Error(op);
  });
  render(
    <ExtensionsPage
      projectId="owned"
      onOpenFiles={vi.fn()}
      onOpenJobs={vi.fn()}
    />,
  );
  await screen.findByRole("button", { name: "Install extension" });
  await user.type(
    screen.getByRole("searchbox", { name: "Search extensions" }),
    "httpx",
  );
  await user.click(screen.getByRole("button", { name: "Install extension" }));
  await screen.findByRole("button", { name: "Approve and install" });
  expect(
    invoke.mock.calls.find(([op]) => op === "extensions_plan")?.[1],
  ).toEqual({ id: "httpx" });
});
it("requires review and consent, and keeps failed dependency checks visibly unsuccessful", async () => {
  const user = userEvent.setup();
  invoke.mockImplementation(async (op: string) => {
    if (op === "extensions_catalog") return [entry];
    if (op === "extensions_history" || op === "files_list") return [];
    if (op === "extensions_plan")
      return {
        extension: entry,
        fingerprint: "reviewed-plan",
        actions: ["Verify pinned dependency"],
        requires_admin: false,
        network_access: "Pinned download",
      };
    if (op === "extensions_install")
      throw Error("TOOL_NOT_INSTALLED: Missing verified dependency");
    throw Error(op);
  });
  render(
    <ExtensionsPage
      projectId="owned"
      onOpenFiles={vi.fn()}
      onOpenJobs={vi.fn()}
    />,
  );
  await user.click(
    await screen.findByRole("button", { name: "Install extension" }),
  );
  const install = await screen.findByRole("button", {
    name: "Approve and install",
  });
  expect(install).toBeDisabled();
  await user.click(install);
  expect(invoke.mock.calls.some(([op]) => op === "extensions_install")).toBe(
    false,
  );
  await user.click(
    screen.getByRole("checkbox", {
      name: "I approve these dependencies and extension permissions",
    }),
  );
  await user.click(install);
  await waitFor(() =>
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Missing verified dependency",
    ),
  );
  expect(screen.queryByText("ready", { exact: true })).not.toBeInTheDocument();
  expect(invoke).toHaveBeenCalledWith("extensions_install", {
    approval: {
      id: "reconx.yara-signatures",
      fingerprint: "reviewed-plan",
      approved: true,
    },
  });
});
it("filters discovery and reports absent updates without fabricated releases", async () => {
  const user = userEvent.setup();
  invoke.mockImplementation(async (op: string) =>
    op === "extensions_catalog" ? [entry] : [],
  );
  render(<ExtensionsPage onOpenFiles={vi.fn()} onOpenJobs={vi.fn()} />);
  await screen.findByRole("button", { name: "Install extension" });
  await user.type(
    screen.getByRole("searchbox", { name: "Search extensions" }),
    "unknown-package",
  );
  expect(screen.getByText("No matching extensions")).toBeVisible();
  await user.clear(
    screen.getByRole("searchbox", { name: "Search extensions" }),
  );
  await user.click(screen.getByRole("tab", { name: "Updates available" }));
  expect(
    screen.getByText(
      "No compatible extension update is available in this catalog.",
    ),
  ).toBeVisible();
  expect(screen.queryByRole("button", { name: "Run signatures" })).not.toBeInTheDocument();
});
