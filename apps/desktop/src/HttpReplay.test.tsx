import { beforeEach, afterEach, it, expect, vi } from "vitest";
import { render, screen, waitFor, cleanup } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import type { Snapshot } from "./api";
import HttpRequestWorkspace, {
  type DraftInput,
  type Draft,
  type AuthProfile,
} from "./components/HttpRequestWorkspace";
import HttpAuthManager from "./components/HttpAuthManager";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(), Channel: class {} }));
vi.mock("./research", () => ({ useJobEvents: () => {} }));
const data = {
  projects: [],
  targets: [
    {
      id: "target",
      project_id: "project",
      name: "Owned fixture",
      value: "http://127.0.0.1:8765/",
      kind: "rest_api",
      description: "",
      tags: [],
      allowed_scope: [],
      excluded_scope: [],
      authorized: true,
      mode: "lab_research",
      scope_status: "allowed",
      created_at: "",
      updated_at: "",
      last_analyzed_at: null,
    },
  ],
  providers: [],
  audit: [],
  database_path: "isolated",
  version: "0.1",
  settings: {
    theme: "reconx",
    security_mode: "lab_research",
    default_provider_id: null,
    fallback_provider_id: null,
  },
} as Snapshot;
const seed: DraftInput = {
  project_id: "project",
  target_id: "target",
  source_exchange_id: "capture",
  title: "Owned request",
  method: "GET",
  url: "http://127.0.0.1:8765/json",
  headers: [],
  body: "",
  content_type: "",
  auth_profile_id: null,
  timeout_ms: 5000,
  follow_redirects: false,
  proxy: null,
};
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
  vi.mocked(invoke).mockReset();
});
afterEach(cleanup);
it("does not offer a stale draft revision while reopening the editor after a CLI change", async () => {
  let resolveRefresh: (value: Draft[]) => void = () => {};
  let visits = 0;
  vi.mocked(invoke).mockImplementation(async (_cmd, args) => {
    const r = (args as { request: Record<string, unknown> }).request;
    if (r.op === "http_drafts") {
      if (visits++ === 0)
        return [
          {
            ...seed,
            id: "draft",
            revision: 1,
            updated_at: "",
            title: "Old revision",
          },
        ];
      return new Promise<Draft[]>((resolve) => {
        resolveRefresh = resolve;
      });
    }
    return [];
  });
  const props = {
    project: "project",
    target: "target",
    source: null,
    identity: null,
    data,
    onClose: () => {},
    onResponse: () => {},
  };
  const view = render(<HttpRequestWorkspace {...props} open />);
  await screen.findByRole("option", { name: /Old revision/ });
  view.rerender(<HttpRequestWorkspace {...props} open={false} />);
  view.rerender(<HttpRequestWorkspace {...props} open />);
  expect(
    screen.getByRole("combobox", { name: "Saved HTTP request" }),
  ).toBeDisabled();
  expect(
    screen.queryByRole("option", { name: /Old revision/ }),
  ).not.toBeInTheDocument();
  resolveRefresh([
    {
      ...seed,
      id: "draft",
      revision: 2,
      updated_at: "",
      title: "Current revision",
    },
  ]);
  await screen.findByRole("option", { name: /Current revision/ });
  await waitFor(() =>
    expect(
      screen.getByRole("combobox", { name: "Saved HTTP request" }),
    ).toBeEnabled(),
  );
});
it("requires exact reviewed consent, invalidates edited plans and never replays on import or save", async () => {
  const user = userEvent.setup();
  let revision = 0;
  let sent: Record<string, unknown> | undefined;
  vi.mocked(invoke).mockImplementation(async (_cmd, args) => {
    const r = (args as { request: Record<string, unknown> }).request;
    if (
      r.op === "http_drafts" ||
      r.op === "http_auth_profiles" ||
      r.op === "http_replay_history"
    )
      return [];
    if (r.op === "http_draft_seed") return structuredClone(seed);
    if (r.op === "http_draft_save") {
      expect(Object.keys(r.input as DraftInput).sort()).toEqual(
        Object.keys(seed).sort(),
      );
      expect(r.id).toBe(revision ? "draft" : null);
      expect(r.revision).toBe(revision || null);
      return {
        ...(r.input as DraftInput),
        id: "draft",
        revision: ++revision,
        updated_at: "",
      } satisfies Draft;
    }
    if (r.op === "http_replay_preview")
      return {
        draft: { ...seed, id: "draft" },
        destination_address: "127.0.0.1:8765",
        fingerprint: `review-${revision}`,
        request_budget: 1,
        identity: null,
        safeguards: ["One reviewed local request"],
      };
    if (r.op === "http_replay_enqueue") {
      sent = r;
      return { id: "job" };
    }
    return null;
  });
  render(
    <HttpRequestWorkspace
      open
      project="project"
      target="target"
      source="capture"
      identity={null}
      data={data}
      onClose={() => {}}
      onResponse={() => {}}
    />,
  );
  await screen.findByRole("textbox", { name: "HTTP request URL" });
  expect(sent).toBeUndefined();
  await user.click(screen.getByRole("button", { name: "Save request" }));
  await screen.findByText("Request saved. No network request sent.");
  expect(sent).toBeUndefined();
  await user.click(screen.getByRole("button", { name: "Review send" }));
  const approve = await screen.findByRole("checkbox", {
    name: "Approve exact HTTP request",
  });
  expect(
    screen.getByRole("button", { name: "Send approved request" }),
  ).toBeDisabled();
  await user.click(approve);
  await user.type(
    screen.getByRole("textbox", { name: "HTTP request URL" }),
    "?page=1",
  );
  expect(
    screen.queryByRole("button", { name: "Send approved request" }),
  ).not.toBeInTheDocument();
  expect(sent).toBeUndefined();
  await user.click(screen.getByRole("button", { name: "Review send" }));
  await user.click(
    await screen.findByRole("checkbox", { name: "Approve exact HTTP request" }),
  );
  await user.click(
    screen.getByRole("button", { name: "Send approved request" }),
  );
  await waitFor(() =>
    expect(sent).toEqual({
      op: "http_replay_enqueue",
      approval: {
        draft_id: "draft",
        fingerprint: `review-${revision}`,
        approved: true,
      },
    }),
  );
});
it("saves a newly entered credential through the typed service, clears it, and does not mark the identity authenticated", async () => {
  const user = userEvent.setup();
  const profiles: AuthProfile[] = [];
  let saved: Record<string, unknown> | undefined;
  vi.mocked(invoke).mockImplementation(async (_cmd, args) => {
    const r = (args as { request: Record<string, unknown> }).request;
    if (r.op === "http_auth_profiles") return [...profiles];
    if (r.op === "http_auth_save") {
      saved = r;
      const profile = {
        ...(r.input as AuthProfile),
        id: "identity",
        revision: 1,
        origin: "http://127.0.0.1:8765",
        expired: false,
        credential_state: "configured",
        last_check: null,
        cleanup_pending: false,
      };
      profiles.push(profile);
      return profile;
    }
    return null;
  });
  render(
    <HttpAuthManager
      open
      project="project"
      target="target"
      data={data}
      onClose={() => {}}
      onTest={() => {}}
    />,
  );
  await screen.findByText("No identities configured.");
  await user.type(
    screen.getByRole("textbox", { name: "Identity name" }),
    "Alice",
  );
  await user.selectOptions(
    screen.getByRole("combobox", { name: "Identity authentication type" }),
    "bearer",
  );
  const input = screen.getByLabelText("Identity credential");
  await user.type(input, "synthetic-private-credential");
  expect(input).toHaveAttribute("type", "password");
  await user.click(screen.getByRole("button", { name: "Save identity" }));
  await screen.findByText("Alice · user");
  expect(saved?.secret).toBe("synthetic-private-credential");
  expect(
    screen.getByText("No authentication check has run."),
  ).toBeInTheDocument();
  expect(
    screen.queryByLabelText("Identity credential"),
  ).not.toBeInTheDocument();
  expect(
    vi
      .mocked(invoke)
      .mock.calls.some(
        ([, a]) =>
          (a as { request: { op: string } }).request.op ===
          "http_replay_enqueue",
      ),
  ).toBe(false);
});
