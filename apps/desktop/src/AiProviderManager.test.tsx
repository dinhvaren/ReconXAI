import { readFileSync } from "node:fs";
import { it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  render,
  screen,
  waitFor,
  cleanup,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import AiProviderManager from "./components/AiProviderManager";
import { defaultOptions, type AiManagerState, type ProviderPreset } from "./ai";
import type { Provider, Snapshot } from "./api";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(), Channel: class {} }));
const catalog = JSON.parse(
  readFileSync("../../crates/reconx-ai/src/provider-catalog.json", "utf8"),
) as ProviderPreset[];
const data: Snapshot = {
  projects: [],
  providers: [],
  audit: [],
  database_path: "fixture",
  version: "0.1",
  settings: {
    theme: "reconx",
    security_mode: "passive_auto",
    default_provider_id: null,
    fallback_provider_id: null,
  },
};
let state: AiManagerState, saved: Record<string, unknown> | undefined;
beforeEach(() => {
  state = {
    catalog,
    profiles: [],
    routing: [],
    agents: ["orchestrator", "reverse"],
  };
  saved = undefined;
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
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (_command, args) => {
    const r = (args as { request: Record<string, unknown> }).request;
    if (r.op === "ai_manager") return structuredClone(state);
    if (r.op === "ai_models_discover") return ["installed-local-model"];
    if (r.op === "ai_profile_save") {
      saved = structuredClone(r);
      state.profiles = [
        {
          provider: r.provider as Provider,
          vendor_id: r.vendor_id as string,
          options: r.options as typeof defaultOptions,
          header_names: [],
          credential_state: "stored",
          connection: null,
          models: [],
          models_refreshed_at: null,
        },
      ];
      return null;
    }
    if (r.op === "ai_connection_test") {
      state.profiles[0].connection = {
        state: "connected",
        checked_at: "2026-10-17T00:00:00Z",
        last_success_at: "2026-10-17T00:00:00Z",
        latency_ms: 24,
        message: "Selected model responded",
        error_code: null,
      };
      return state.profiles[0].connection;
    }
    return null;
  });
});
afterEach(cleanup);
const perform = async (action: () => Promise<unknown>) => {
  await action();
};
it("saves a preset/model/API key together and distinguishes Saved from Connected", async () => {
  const user = userEvent.setup();
  render(<AiProviderManager data={data} busy={false} perform={perform} />);
  await user.click(
    await screen.findByRole("button", { name: "Add AI Provider" }),
  );
  const dialog = screen.getByRole("dialog");
  await user.selectOptions(
    within(dialog).getByLabelText("Select Provider"),
    "deepseek",
  );
  expect(
    within(dialog).getByRole("option", { name: "DeepSeek Flash" }),
  ).toBeInTheDocument();
  expect(within(dialog).getByLabelText("Gateway base URL")).not.toBeVisible();
  await user.selectOptions(
    within(dialog).getByLabelText("Select Model"),
    "deepseek-v4-pro",
  );
  const key = within(dialog).getByLabelText("API key");
  expect(key).toHaveAttribute("type", "password");
  await user.type(key, "synthetic-only");
  await user.click(within(dialog).getByLabelText("Show entered key"));
  expect(key).toHaveAttribute("type", "text");
  await user.click(
    within(dialog).getByRole("button", { name: "Save provider" }),
  );
  await waitFor(() =>
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument(),
  );
  expect(saved?.key).toBe("synthetic-only");
  expect((saved?.provider as Provider).model).toBe("deepseek-v4-pro");
  expect(saved?.make_default).toBe(true);
  expect(screen.getByText("Saved · Not tested")).toBeInTheDocument();
  expect(screen.queryByDisplayValue("synthetic-only")).not.toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "Test connection" }));
  await screen.findByText(/Connected \(last test\)/);
  await user.selectOptions(screen.getByLabelText('Provider category'),'local');
  expect(screen.queryByRole('button',{name:'Manage'})).not.toBeInTheDocument();
  expect(screen.getByText('No configured models match this filter.')).toBeInTheDocument();
  await user.selectOptions(screen.getByLabelText('Provider category'),'configured');
  expect(screen.getByRole('button',{name:'Manage'})).toBeInTheDocument();
});
it("discovers models for a keyless local server before saving", async () => {
  const user = userEvent.setup();
  render(<AiProviderManager data={data} busy={false} perform={perform} />);
  await user.click(
    await screen.findByRole("button", { name: "Add AI Provider" }),
  );
  const dialog = screen.getByRole("dialog");
  await user.selectOptions(
    within(dialog).getByLabelText("Select Provider"),
    "lmstudio",
  );
  expect(within(dialog).queryByLabelText("API key")).not.toBeInTheDocument();
  await user.click(
    within(dialog).getByRole("button", { name: "Refresh Models" }),
  );
  await within(dialog).findByRole("option", { name: "installed-local-model" });
  expect(within(dialog).getByLabelText("Select Model")).toHaveValue(
    "installed-local-model",
  );
  await user.click(
    within(dialog).getByRole("button", { name: "Save provider" }),
  );
  await waitFor(() => expect(saved).toBeDefined());
  expect(saved?.key).toBeNull();
  expect((saved?.provider as Provider).requires_key).toBe(false);
});
it("keeps key input and shows a recoverable error when credential storage fails", async () => {
  const user = userEvent.setup();
  render(
    <AiProviderManager
      data={data}
      busy={false}
      perform={async (action) => {
        try {
          await action();
        } catch {
          /* Global app banner also handles this. */
        }
      }}
    />,
  );
  await user.click(
    await screen.findByRole("button", { name: "Add AI Provider" }),
  );
  const dialog = screen.getByRole("dialog");
  await user.type(within(dialog).getByLabelText("API key"), "synthetic");
  vi.mocked(invoke).mockImplementationOnce(async () => {
    throw {
      code: "SECRET_STORE_UNAVAILABLE",
      message: "Secure storage unavailable",
    };
  });
  await user.click(
    within(dialog).getByRole("button", { name: "Save provider" }),
  );
  await within(dialog).findByText("Secure storage unavailable");
  expect(screen.getByRole("dialog")).toBeInTheDocument();
  expect(within(dialog).getByLabelText("API key")).toHaveValue("synthetic");
  expect(saved).toBeUndefined();
});
it("builds the official workspace endpoint without asking for a base URL", async () => {
  const user = userEvent.setup();
  render(<AiProviderManager data={data} busy={false} perform={perform} />);
  await user.click(
    await screen.findByRole("button", { name: "Add AI Provider" }),
  );
  const dialog = screen.getByRole("dialog");
  await user.selectOptions(
    within(dialog).getByLabelText("Select Provider"),
    "dashscope",
  );
  await user.type(
    within(dialog).getByLabelText("Model Studio workspace ID"),
    "owned-workspace",
  );
  await user.type(within(dialog).getByLabelText("API key"), "synthetic");
  await user.click(
    within(dialog).getByRole("button", { name: "Save provider" }),
  );
  await waitFor(() => expect(saved).toBeDefined());
  expect((saved?.provider as Provider).base_url).toBe(
    "https://owned-workspace.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1",
  );
});
