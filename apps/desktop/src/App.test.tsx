import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, cleanup, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";
import type { Settings, Snapshot, ThemePreference } from "./api";
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  Channel: class {
    onmessage = () => {};
  },
}));
const snapshot: Snapshot = {
  projects: [],
  providers: [],
  audit: [],
  database_path: "test.db",
  version: "0.1.0",
  settings: {
    security_mode: "passive_auto",
    default_provider_id: null,
    fallback_provider_id: null,
    theme: "dark",
  },
};
let current: Snapshot;
function operation(args: unknown) {
  return (
    args as {
      request?: { op: string; theme: ThemePreference; settings: Settings };
    }
  )?.request;
}
async function mockService(command: string, args: unknown) {
  if (command !== "service") return null;
  const request = operation(args)!;
  if (request.op === "set_theme") {
    current.settings.theme = request.theme;
    return structuredClone(current.settings);
  }
  if (request.op === "save_settings") {
    current.settings = structuredClone(request.settings);
    return null;
  }
  if (request.op === "tools_list") return [];
  return structuredClone(current);
}
beforeEach(() => {
  cleanup();
  current = structuredClone(snapshot);
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(mockService);
  vi.stubGlobal(
    "matchMedia",
    vi.fn(() => ({
      matches: false,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    })),
  );
});
const menuChoice = (mode: string) =>
  screen.getByRole("menuitemradio", { name: new RegExp("^" + mode) });
describe("Desktop interactions", () => {
  it("opens keyboard search and keeps navigation usable from the rail", async () => {
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
  it("synchronizes both menus with immediate persisted Settings appearance", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Workspace overview");
    await user.click(screen.getByRole("button", { name: "Topbar appearance" }));
    expect(menuChoice("Dark")).toHaveAttribute("aria-checked", "true");
    await user.click(menuChoice("Light"));
    expect(document.documentElement.dataset.theme).toBe("light");
    await waitFor(() => expect(current.settings.theme).toBe("light"));
    await user.click(
      screen.getByRole("button", { name: "Sidebar appearance" }),
    );
    expect(menuChoice("Light")).toHaveAttribute("aria-checked", "true");
    await user.keyboard("{End}{Enter}");
    await waitFor(() => expect(current.settings.theme).toBe("system"));
    await user.click(screen.getByRole("button", { name: "Settings" }));
    expect(screen.getByLabelText("Appearance")).toHaveValue("system");
    await user.selectOptions(screen.getByLabelText("Appearance"), "dark");
    expect(document.documentElement.dataset.theme).toBe("dark");
    await waitFor(() => expect(current.settings.theme).toBe("dark"));
    await user.click(screen.getByRole("button", { name: "Topbar appearance" }));
    expect(menuChoice("Dark")).toHaveAttribute("aria-checked", "true");
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Topbar appearance" }),
    ).toHaveFocus();
  });
  it("follows OS changes only while System is selected", async () => {
    const user = userEvent.setup();
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
    render(<App />);
    await screen.findByText("Workspace overview");
    await user.click(screen.getByRole("button", { name: "Settings" }));
    await user.selectOptions(screen.getByLabelText("Appearance"), "system");
    expect(document.documentElement.dataset.theme).toBe("light");
    await act(async () => {
      media.matches = true;
      change?.();
    });
    expect(document.documentElement.dataset.theme).toBe("dark");
    await user.selectOptions(screen.getByLabelText("Appearance"), "light");
    expect(media.removeEventListener).toHaveBeenCalled();
    expect(document.documentElement.dataset.theme).toBe("light");
  });
  it("preserves unsaved mode edits while theme is saved independently", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Workspace overview");
    await user.click(screen.getByRole("button", { name: "Settings" }));
    await user.selectOptions(
      screen.getByLabelText("Security mode"),
      "lab_research",
    );
    await user.click(screen.getByRole("button", { name: "Topbar appearance" }));
    await user.click(menuChoice("Light"));
    await waitFor(() => expect(current.settings.theme).toBe("light"));
    expect(current.settings.security_mode).toBe("passive_auto");
    expect(screen.getByLabelText("Security mode")).toHaveValue("lab_research");
    await user.click(screen.getByRole("button", { name: "Save preferences" }));
    await waitFor(() =>
      expect(current.settings.security_mode).toBe("lab_research"),
    );
    expect(current.settings.theme).toBe("light");
  });
  it("rolls back a failed theme write and reports the persistence error", async () => {
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (operation(args)?.op === "set_theme")
        throw { message: "Database is read-only" };
      return mockService(command, args);
    });
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Workspace overview");
    await user.click(screen.getByRole("button", { name: "Topbar appearance" }));
    await user.click(menuChoice("Light"));
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Appearance was not saved. Database is read-only",
    );
    expect(document.documentElement.dataset.theme).toBe("dark");
    expect(current.settings.theme).toBe("dark");
  });
  it("serializes rapid selections so a delayed write cannot win over the latest mode", async () => {
    let release!: () => void;
    const gate = new Promise<void>((resolve) => {
      release = resolve;
    });
    const writes: string[] = [];
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      const request = operation(args);
      if (request?.op === "set_theme") {
        writes.push(request.theme);
        if (writes.length === 1) await gate;
      }
      return mockService(command, args);
    });
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Workspace overview");
    try {
      for (const mode of ["Light", "System", "Dark"]) {
        await user.click(
          screen.getByRole("button", { name: "Topbar appearance" }),
        );
        await user.click(menuChoice(mode));
      }
      expect(document.documentElement.dataset.theme).toBe("dark");
      expect(writes).toEqual(["light"]);
    } finally {
      release();
    }
    await waitFor(() => expect(writes).toEqual(["light", "system", "dark"]));
    await waitFor(() => expect(current.settings.theme).toBe("dark"));
  });
  it("creates projects through native IPC", async () => {
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
  it("reports project backend errors without claiming success", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Workspace overview");
    await user.click(screen.getByRole("button", { name: "Projects" }));
    await user.type(screen.getByLabelText("Project name"), "Duplicate");
    vi.mocked(invoke).mockImplementation(async (command, args) => {
      if (operation(args)?.op === "create_project")
        throw { message: "Name already exists" };
      return mockService(command, args);
    });
    await user.click(screen.getByRole("button", { name: "Create project" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Name already exists",
    );
    expect(screen.queryByText("Project created")).not.toBeInTheDocument();
  });
  it("labels future modules without exposing pretend execution controls", async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByText("Workspace overview");
    await user.click(screen.getByRole("button", { name: "MCP Hub" }));
    expect(await screen.findByText("NOT STARTED")).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /Invoke tool|Connect server/ }),
    ).not.toBeInTheDocument();
  });
});
