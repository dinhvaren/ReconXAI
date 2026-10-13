import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, cleanup } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  Channel: class {
    onmessage = () => {};
  },
}));
const snapshot = {
  projects: [],
  providers: [],
  settings: {
    security_mode: "passive_auto",
    default_provider_id: null,
    fallback_provider_id: null,
  },
  audit: [],
  database_path: "test.db",
  version: "0.1.0",
};
beforeEach(() => {
  cleanup();
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockResolvedValue(snapshot);
});
describe("Desktop interactions", () => {
  it("creates projects through native IPC and refreshes persisted state", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Workspace overview");
    await user.click(screen.getByRole("button", { name: "Projects" }));
    await user.type(screen.getByLabelText("Project name"), "Evidence review");
    await user.type(screen.getByLabelText("Description"), "Local notes");
    await user.click(screen.getByRole("button", { name: "Create project" }));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("service", {
        request: {
          op: "create_project",
          name: "Evidence review",
          description: "Local notes",
        },
      }),
    );
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Project created",
    );
  });
  it("shows backend failures without claiming successful creation", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Workspace overview");
    await user.click(screen.getByRole("button", { name: "Projects" }));
    await user.type(screen.getByLabelText("Project name"), "Duplicate");
    vi.mocked(invoke).mockRejectedValueOnce({
      code: "DUPLICATE_NAME",
      message: "Name already exists",
    });
    await user.click(screen.getByRole("button", { name: "Create project" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Name already exists",
    );
    expect(screen.queryByText("Project created")).not.toBeInTheDocument();
  });
  it("persists settings and clearly labels future modules", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Workspace overview");
    await user.click(screen.getByRole("button", { name: "Settings" }));
    await user.selectOptions(
      screen.getByLabelText("Security mode"),
      "lab_research",
    );
    await user.click(screen.getByRole("button", { name: "Save preferences" }));
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("service", {
        request: {
          op: "save_settings",
          settings: {
            security_mode: "lab_research",
            default_provider_id: null,
            fallback_provider_id: null,
          },
        },
      }),
    );
    await user.click(screen.getByRole("button", { name: "MCP Hub" }));
    expect(await screen.findByText("NOT STARTED")).toBeInTheDocument();
  });
});
