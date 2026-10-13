import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, cleanup, act } from "@testing-library/react";
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
    theme: "dark",
  },
  audit: [],
  database_path: "test.db",
  version: "0.1.0",
};
beforeEach(() => {
  cleanup();
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (_command, args) =>
    (args as { request: { op: string } }).request.op === "tools_list"
      ? []
      : snapshot,
  );
  vi.stubGlobal(
    "matchMedia",
    vi.fn(() => ({
      matches: false,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    })),
  );
});
describe("Desktop interactions", () => {
  it("opens search by keyboard and keeps module navigation usable from the rail", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Workspace overview");
    await user.keyboard("{Control>}k{/Control}");
    const search = screen.getByRole("combobox", { name: "Search workspace" });
    expect(search).toHaveFocus();
    await user.type(search, "Reports");
    expect(screen.getByRole("option", { name: /Reports/ })).toHaveTextContent(
      "Planned module",
    );
    await user.keyboard("{Enter}");
    expect(
      await screen.findByRole("heading", { name: "Reports" }),
    ).toBeInTheDocument();
    expect(search).toHaveValue("");
    expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
    await user.click(
      screen.getByRole("button", { name: "Collapse navigation" }),
    );
    await user.click(screen.getByRole("button", { name: "Open AI Workspace" }));
    expect(
      await screen.findByRole("heading", { name: "AI Workspace" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Send message/ })).toBeDisabled();
  });
  it("quick theme switching persists through the shared service", async () => {
    const user = userEvent.setup();
    let current = structuredClone(snapshot);
    vi.mocked(invoke).mockImplementation(async (_command, args) => {
      const request = (
        args as { request: { op: string; settings: typeof snapshot.settings } }
      ).request;
      if (request.op === "save_settings") {
        current = { ...current, settings: request.settings };
        return null;
      }
      return current;
    });
    render(<App />);
    await screen.findByText("Workspace overview");
    await user.click(
      screen.getByRole("button", { name: "Switch to light mode" }),
    );
    await waitFor(() =>
      expect(document.documentElement.dataset.theme).toBe("light"),
    );
    expect(current.settings.theme).toBe("light");
    await user.click(
      screen.getByRole("button", { name: "Switch to dark mode" }),
    );
    await waitFor(() =>
      expect(document.documentElement.dataset.theme).toBe("dark"),
    );
    expect(current.settings.theme).toBe("dark");
  });
  it("system appearance responds to operating system changes", async () => {
    const user = userEvent.setup();
    let current = structuredClone(snapshot);
    let change: (() => void) | undefined;
    const media = {
      matches: false,
      addEventListener: vi.fn((_event, listener) => {
        change = listener;
      }),
      removeEventListener: vi.fn(),
    };
    vi.stubGlobal(
      "matchMedia",
      vi.fn(() => media),
    );
    vi.mocked(invoke).mockImplementation(async (_command, args) => {
      const request = (
        args as { request: { op: string; settings: typeof snapshot.settings } }
      ).request;
      if (request.op === "save_settings") {
        current = { ...current, settings: request.settings };
        return null;
      }
      return current;
    });
    render(<App />);
    await screen.findByText("Workspace overview");
    await user.click(screen.getByRole("button", { name: "Settings" }));
    await user.selectOptions(screen.getByLabelText("Appearance"), "system");
    await user.click(screen.getByRole("button", { name: "Save preferences" }));
    await waitFor(() =>
      expect(document.documentElement.dataset.theme).toBe("light"),
    );
    expect(current.settings.theme).toBe("system");
    await act(async () => {
      media.matches = true;
      change?.();
    });
    expect(document.documentElement.dataset.theme).toBe("dark");
  });
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
            theme: "dark",
          },
        },
      }),
    );
    await user.click(screen.getByRole("button", { name: "MCP Hub" }));
    expect(await screen.findByText("NOT STARTED")).toBeInTheDocument();
  });
});
