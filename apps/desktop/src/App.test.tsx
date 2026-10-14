import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, cleanup, act } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";
import type { Settings, Snapshot, AppTheme } from "./api";
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
// jsdom has no native modal dialog implementation. Native tests exercise the
// actual WebView2 showModal/focus behavior separately.
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
let current: Snapshot;
function operation(args: unknown) {
  return (
    args as {
      request?: { op: string; theme: AppTheme; settings: Settings };
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
it("global Add Target saves a typed configuration without changing existing settings", async () => {
  const user = userEvent.setup();
  current.projects = [
    {
      id: "fixture-project",
      name: "Fixture",
      description: "",
      created_at: "2026-10-14",
      updated_at: "2026-10-14",
      archived: false,
    },
  ];
  const requests: unknown[] = [];
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    const req = operation(args);
    if (command === "service" && req?.op === "target_create") {
      requests.push(args);
      return { id: "new-target" };
    }
    return mockService(command, args);
  });
  render(<App />);
  await screen.findByText("Workspace overview");
  await user.click(
    screen.getByRole("button", { name: "Quick actions" }),
  );
  await user.click(
    screen.getByRole("menuitem", { name: "Add Target" }),
  );
  await user.type(screen.getByLabelText("Target name"), "Owned lab");
  await user.type(
    screen.getByLabelText("Target value"),
    "http://localhost:8080",
  );
  await user.selectOptions(screen.getByLabelText("Target type"), "url");
  await user.type(screen.getByLabelText("Tags / groups"), "lab, internal");
  await user.click(screen.getByLabelText(/I confirm I have permission/));
  await user.click(
    screen.getByRole("button", { name: "Save target" }),
  );
  await waitFor(() =>
    expect(
      screen.queryByRole("dialog", { name: "Add Target" }),
    ).not.toBeInTheDocument(),
  );
  expect(requests[0]).toMatchObject({
    request: {
      op: "target_create",
      input: {
        project_id: "fixture-project",
        kind: "url",
        value: "http://localhost:8080",
        tags: ["lab", "internal"],
        authorized: true,
        mode: "passive_auto",
      },
    },
  });
  expect(current.settings.theme).toBe("dark");
});
it("target validation errors keep the dialog and user input available for correction", async () => {
  const user = userEvent.setup();
  current.projects = [
    {
      id: "fixture-project",
      name: "Fixture",
      description: "",
      created_at: "2026-10-14",
      updated_at: "2026-10-14",
      archived: false,
    },
  ];
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command === "service" && operation(args)?.op === "target_create")
      throw { code: "INVALID_TARGET", message: "Invalid IPv4 address" };
    return mockService(command, args);
  });
  render(<App />);
  await screen.findByText("Workspace overview");
  await user.click(
    screen.getByRole("button", { name: "+ Add Target" }),
  );
  await user.type(screen.getByLabelText("Target name"), "Lab IP");
  await user.selectOptions(screen.getByLabelText("Target type"), "ipv4");
  await user.type(screen.getByLabelText("Target value"), "invalid");
  await user.click(
    screen.getByRole("button", { name: "Save target" }),
  );
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Invalid IPv4 address",
  );
  expect(screen.getByLabelText("Target value")).toHaveValue("invalid");
  expect(
    screen.getByRole("dialog", { name: "Add Target" }),
  ).toBeInTheDocument();
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
    await waitFor(() => expect(current.settings.theme).toBe("reconx"));
    await user.click(screen.getByRole("button", { name: "Settings" }));
    expect(screen.getByLabelText("Appearance")).toHaveValue("reconx");
    expect(
      [...screen.getByLabelText("Appearance").querySelectorAll("option")].map(
        (option) => option.value,
      ),
    ).toEqual(["light", "dark", "reconx"]);
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
  it("keeps explicit appearance unchanged when operating system appearance changes", async () => {
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
    await user.selectOptions(screen.getByLabelText("Appearance"), "reconx");
    expect(document.documentElement.dataset.theme).toBe("reconx");
    await act(async () => {
      media.matches = true;
      change?.();
    });
    expect(document.documentElement.dataset.theme).toBe("reconx");
    await user.selectOptions(screen.getByLabelText("Appearance"), "light");
    expect(media.addEventListener).not.toHaveBeenCalled();
    expect(window.matchMedia).not.toHaveBeenCalled();
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
      for (const mode of ["Light", "ReconX", "Dark"]) {
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
    await waitFor(() => expect(writes).toEqual(["light", "reconx", "dark"]));
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
