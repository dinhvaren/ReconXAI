import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type FormEvent,
} from "react";
import {
  request,
  streamChat,
  cancelStream,
  installTool,
  type ToolInventory,
  type ToolManifest,
  type ToolProgress,
  errorText,
  providerKinds,
  type Snapshot,
  type Provider,
  type Kind,
  type Project,
  type Reply,
  type Settings,
} from "./api";

const navigation = [
  ["Workspace", "Dashboard", "Projects", "Targets & Scope"],
  [
    "Analysis",
    "Reconnaissance",
    "Web Security",
    "API Security",
    "Network Security",
    "Reverse Engineering",
    "Mobile Security",
    "Cloud & Containers",
    "CI/CD Security",
  ],
  [
    "Intelligence",
    "AI Workspace",
    "Agent Workflows",
    "MCP Hub",
    "Tool Manager",
  ],
  ["Research", "Findings", "Evidence", "Reports", "Settings"],
];
const implemented = new Set([
  "Dashboard",
  "Projects",
  "AI Workspace",
  "Settings",
  "Tool Manager",
]);
function Glyph({ name = "grid" }: { name?: string }) {
  const paths: Record<string, string> = {
    grid: "M3 3h7v7H3z M14 3h7v7h-7z M3 14h7v7H3z M14 14h7v7h-7z",
    folder: "M3 7V4h6l3 3h9v13H3z",
    ai: "M12 3l2.4 6.6L21 12l-6.6 2.4L12 21l-2.4-6.6L3 12l6.6-2.4z",
    settings:
      "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8 M12 2v3 M12 19v3 M2 12h3 M19 12h3 M5 5l2 2 M17 17l2 2 M5 19l2-2 M17 7l2-2",
    shield: "M12 2L3 6v6c0 5 9 10 9 10s9-5 9-10V6z M8 12l3 3 5-6",
  };
  return (
    <svg
      aria-hidden="true"
      width="18"
      height="18"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinejoin="round"
      strokeLinecap="round"
    >
      <path d={paths[name] ?? paths.shield} />
    </svg>
  );
}
const date = (text: string) => new Date(text).toLocaleString();
type Perform = (
  action: () => Promise<unknown>,
  message?: string,
) => Promise<void>;

export default function App() {
  const [page, setPage] = useState("Dashboard");
  const [collapsed, setCollapsed] = useState(false);
  const [data, setData] = useState<Snapshot | null>(null);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [busy, setBusy] = useState(false);
  const refresh = useCallback(
    async () => setData(await request<Snapshot>("snapshot")),
    [],
  );
  useEffect(() => {
    refresh().catch((e) => setError(errorText(e)));
  }, [refresh]);
  const perform: Perform = async (action, message) => {
    setBusy(true);
    setError("");
    setNotice("");
    try {
      await action();
      await refresh();
      if (message) setNotice(message);
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };
  const projects = data?.projects.filter((p) => !p.archived) ?? [];
  return (
    <div className={collapsed ? "workbench collapsed" : "workbench"}>
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark">
            R<span>×</span>
          </div>
          <div className="brand-text">
            <b>
              ReconX <span>AI</span>
            </b>
            <small>RESEARCH WORKBENCH</small>
          </div>
        </div>
        <button
          className="collapse-button"
          onClick={() => setCollapsed(!collapsed)}
          aria-label={collapsed ? "Expand navigation" : "Collapse navigation"}
        >
          {collapsed ? "›" : "‹"}
          <span>Collapse navigation</span>
        </button>
        <nav aria-label="Primary navigation">
          {navigation.map(([group, ...items]) => (
            <div className="nav-group" key={group}>
              <p>{group}</p>
              {items.map((item) => (
                <button
                  key={item}
                  title={item}
                  className={page === item ? "nav-item active" : "nav-item"}
                  onClick={() => {
                    setPage(item);
                    setNotice("");
                    setError("");
                  }}
                >
                  <Glyph
                    name={
                      item === "Projects"
                        ? "folder"
                        : item === "AI Workspace"
                          ? "ai"
                          : item === "Settings"
                            ? "settings"
                            : item === "Dashboard"
                              ? "grid"
                              : "shield"
                    }
                  />
                  <span>{item}</span>
                  {!implemented.has(item) && <i title="Planned module" />}
                </button>
              ))}
            </div>
          ))}
        </nav>
        <div className="sidebar-footer">
          <span className="status-dot" />
          <div>
            <b>Local workspace</b>
            <small>v{data?.version ?? "0.1.0"} · Foundation</small>
          </div>
        </div>
      </aside>
      <div className="workspace">
        <header className="topbar">
          <div className="breadcrumb">
            WORKSPACE <span>/</span> <b>{page}</b>
          </div>
          <div className="top-actions">
            <span className="mode-chip">
              <Glyph name="shield" />
              {data?.settings.security_mode.replaceAll("_", " ") ??
                "passive auto"}
            </span>
            <button
              className="button ghost small"
              disabled={busy}
              onClick={() => void perform(refresh, "Workspace refreshed")}
            >
              ↻ Refresh
            </button>
          </div>
        </header>
        <main>
          {error && (
            <div role="alert" className="banner error">
              {error}
              <button aria-label="Dismiss error" onClick={() => setError("")}>
                ×
              </button>
            </div>
          )}
          {notice && (
            <div role="status" className="banner success">
              {notice}
            </div>
          )}
          {!data ? (
            <div className="empty large">
              <Glyph name="shield" />
              <h1>Connecting to the workspace</h1>
              <p>
                The desktop requires its native Rust backend. Open ReconX AI
                with the Tauri development or build command.
              </p>
            </div>
          ) : (
            <>
              {page === "Dashboard" && (
                <Dashboard data={data} projects={projects} navigate={setPage} />
              )}
              {page === "Projects" && (
                <Projects data={data} busy={busy} perform={perform} />
              )}
              {page === "Settings" && (
                <SettingsPage data={data} busy={busy} perform={perform} />
              )}
              {page === "AI Workspace" && (
                <AiWorkspace data={data} busy={busy} perform={perform} />
              )}
              {page === "Tool Manager" && (
                <ToolManagerPage busy={busy} perform={perform} />
              )}
              {!implemented.has(page) && <PlannedModule name={page} />}
            </>
          )}
        </main>
        <footer className="workspace-footer">
          <span>
            <span className="status-dot" /> SQLite workspace{" "}
            {data ? "connected" : "connecting"}
          </span>
          <span>Local analysis · Human control · Traceable changes</span>
        </footer>
      </div>
    </div>
  );
}
function Heading({
  eyebrow,
  title,
  description,
  action,
}: {
  eyebrow: string;
  title: string;
  description: string;
  action?: React.ReactNode;
}) {
  return (
    <div className="page-heading">
      <div>
        <p className="eyebrow">{eyebrow}</p>
        <h1>{title}</h1>
        <p className="subtitle">{description}</p>
      </div>
      {action}
    </div>
  );
}
function Dashboard({
  data,
  projects,
  navigate,
}: {
  data: Snapshot;
  projects: Project[];
  navigate: (s: string) => void;
}) {
  const completed = data.audit.filter((e) => e.outcome === "completed").length;
  return (
    <>
      <Heading
        eyebrow="YOUR RESEARCH, CONNECTED"
        title="Workspace overview"
        description="Projects, model connections, and an auditable research history."
        action={
          <button
            className="button primary"
            onClick={() => navigate("Projects")}
          >
            + Create project
          </button>
        }
      />
      <div className="stats-grid">
        {[
          [
            "Active projects",
            projects.length,
            "Persistent in your local workspace",
            "folder",
          ],
          [
            "AI providers",
            data.providers.length,
            `${data.providers.filter((p) => p.enabled).length} enabled configurations`,
            "ai",
          ],
          [
            "Recent completed actions",
            completed,
            "From the latest 100 audit events",
            "grid",
          ],
          [
            "Execution capabilities",
            0,
            "Workflow execution is planned",
            "shield",
          ],
        ].map(([label, count, detail, icon]) => (
          <div className="stat-card" key={String(label)}>
            <div className="stat-top">
              <span>{label}</span>
              <Glyph name={String(icon)} />
            </div>
            <strong>{count}</strong>
            <small>{detail}</small>
          </div>
        ))}
      </div>
      <div className="dashboard-grid">
        <section className="panel">
          <div className="panel-heading">
            <h2>Recent projects</h2>
            <button
              className="text-button"
              onClick={() => navigate("Projects")}
            >
              View all ↗
            </button>
          </div>
          {projects.length === 0 ? (
            <div className="empty">
              <Glyph name="folder" />
              <h3>Your workspace starts here</h3>
              <p>Create a project to keep your research organized.</p>
              <button
                className="button secondary"
                onClick={() => navigate("Projects")}
              >
                Create your first project
              </button>
            </div>
          ) : (
            <div className="project-list">
              {projects.slice(0, 5).map((p) => (
                <button
                  className="project-row"
                  key={p.id}
                  onClick={() => navigate("Projects")}
                >
                  <div className="project-avatar">
                    <Glyph name="folder" />
                  </div>
                  <div>
                    <b>{p.name}</b>
                    <p>{p.description || "No description"}</p>
                  </div>
                  <span className="badge">Active</span>
                </button>
              ))}
            </div>
          )}
        </section>
        <section className="panel">
          <div className="panel-heading">
            <h2>Connection overview</h2>
            <span className="badge muted">Configuration</span>
          </div>
          <div className="connection-list">
            <div>
              <span>Project database</span>
              <span className="state ready">Connected</span>
            </div>
            <div>
              <span>AI gateway</span>
              <span className="state">
                {data.providers.length
                  ? "Configured · test in Settings"
                  : "Not configured"}
              </span>
            </div>
            <div>
              <span>MCP gateway</span>
              <span className="state">Planned</span>
            </div>
            <div>
              <span>Managed tools</span>
              <span className="state">Inventory in Tool Manager</span>
            </div>
          </div>
          <div className="panel-note">
            Configure a cloud provider or local model in Settings. Connectivity
            is verified only when you run a connection test.
          </div>
          <button
            className="text-button padded"
            onClick={() => navigate("Settings")}
          >
            Open provider settings ↗
          </button>
        </section>
      </div>
      <section className="panel history-panel">
        <div className="panel-heading">
          <h2>Recent activity</h2>
          <span className="badge muted">Local audit trail</span>
        </div>
        <AuditTable data={data} />
      </section>
    </>
  );
}
function AuditTable({ data }: { data: Snapshot }) {
  return data.audit.length ? (
    <div className="table-scroll">
      <table>
        <thead>
          <tr>
            <th>Action</th>
            <th>Record</th>
            <th>Outcome</th>
            <th>Time</th>
          </tr>
        </thead>
        <tbody>
          {data.audit.slice(0, 12).map((e) => (
            <tr key={e.id}>
              <td className="mono">{e.action}</td>
              <td className="mono muted-text">{e.entity_id.slice(0, 12)}</td>
              <td>
                <span
                  className={
                    e.outcome === "completed" ? "badge" : "badge warning"
                  }
                >
                  {e.outcome}
                </span>
              </td>
              <td className="muted-text">{date(e.timestamp)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  ) : (
    <div className="empty compact">
      <p>No actions recorded yet.</p>
    </div>
  );
}
function Projects({
  data,
  busy,
  perform,
}: {
  data: Snapshot;
  busy: boolean;
  perform: Perform;
}) {
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [editing, setEditing] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [archived, setArchived] = useState(false);
  const [importText, setImportText] = useState("");
  const [showImport, setShowImport] = useState(false);
  const save = (event: FormEvent) => {
    event.preventDefault();
    void perform(
      async () => {
        if (editing)
          await request("update_project", { id: editing, name, description });
        else await request("create_project", { name, description });
        setName("");
        setDescription("");
        setEditing(null);
      },
      editing ? "Project updated" : "Project created",
    );
  };
  const exportProject = (id: string) =>
    void perform(async () => {
      const result = await request<{
        format_version: number;
        name: string;
        description: string;
      }>("export_project", { id });
      const blob = new Blob([JSON.stringify(result, null, 2)], {
        type: "application/json",
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "reconx-project.json";
      a.click();
      setTimeout(() => URL.revokeObjectURL(url), 1000);
    }, "Project exported. Review descriptions before sharing.");
  const projects = data.projects.filter(
    (p) =>
      (archived || !p.archived) &&
      `${p.name} ${p.description}`.toLowerCase().includes(search.toLowerCase()),
  );
  return (
    <>
      <Heading
        eyebrow="PROJECT MANAGEMENT"
        title="Projects"
        description="A durable home for your research notes and workspace history."
        action={
          <button
            className="button secondary"
            onClick={() => setShowImport(!showImport)}
          >
            Import project
          </button>
        }
      />
      <div className="projects-layout">
        <section className="panel">
          <div className="panel-heading">
            <h2>{editing ? "Edit project" : "Create project"}</h2>
          </div>
          <form className="form" onSubmit={save}>
            <label>
              Project name
              <input
                required
                maxLength={120}
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="e.g. Internal application review"
              />
            </label>
            <label>
              Description
              <textarea
                maxLength={8192}
                rows={5}
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Describe the purpose of this project. Keep secrets out of notes."
              />
            </label>
            <div className="form-actions">
              <button className="button primary" disabled={busy}>
                {busy ? "Saving…" : editing ? "Save project" : "Create project"}
              </button>
              {editing && (
                <button
                  type="button"
                  className="button ghost"
                  onClick={() => {
                    setEditing(null);
                    setName("");
                    setDescription("");
                  }}
                >
                  Cancel edit
                </button>
              )}
            </div>
            <p className="helper">
              Project changes are recorded in the local audit trail.
            </p>
          </form>
        </section>
        <section className="panel">
          <div className="panel-heading">
            <h2>
              Project library <span className="count">{projects.length}</span>
            </h2>
          </div>
          <div className="filters">
            <input
              aria-label="Search projects"
              placeholder="Search projects…"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
            <label className="inline-check">
              <input
                type="checkbox"
                checked={archived}
                onChange={(e) => setArchived(e.target.checked)}
              />
              Show archived
            </label>
          </div>
          {projects.length ? (
            <div className="project-library">
              {projects.map((p) => (
                <article className="library-card" key={p.id}>
                  <div className="library-title">
                    <Glyph name="folder" />
                    <h3>{p.name}</h3>
                    <span className={p.archived ? "badge muted" : "badge"}>
                      {p.archived ? "Archived" : "Active"}
                    </span>
                  </div>
                  <p>{p.description || "No description"}</p>
                  <small>Updated {date(p.updated_at)}</small>
                  <div className="card-actions">
                    <button
                      disabled={busy}
                      className="text-button"
                      onClick={() => {
                        setEditing(p.id);
                        setName(p.name);
                        setDescription(p.description);
                      }}
                    >
                      Edit
                    </button>
                    <button
                      disabled={busy}
                      className="text-button"
                      onClick={() => exportProject(p.id)}
                    >
                      Export
                    </button>
                    <button
                      disabled={busy}
                      className="text-button"
                      onClick={() =>
                        void perform(
                          () =>
                            request("archive_project", {
                              id: p.id,
                              archived: !p.archived,
                            }),
                          p.archived ? "Project restored" : "Project archived",
                        )
                      }
                    >
                      {p.archived ? "Restore" : "Archive"}
                    </button>
                  </div>
                </article>
              ))}
            </div>
          ) : (
            <div className="empty">
              <Glyph name="folder" />
              <h3>No projects found</h3>
              <p>Create a project or change your search.</p>
            </div>
          )}
        </section>
      </div>
      {showImport && (
        <section className="panel import-panel">
          <div className="panel-heading">
            <h2>Import project JSON</h2>
          </div>
          <form
            className="form"
            onSubmit={(e) => {
              e.preventDefault();
              void perform(async () => {
                await request("import_project", {
                  project: JSON.parse(importText),
                });
                setImportText("");
                setShowImport(false);
              }, "Project imported");
            }}
          >
            <label>
              Exported project
              <textarea
                rows={6}
                required
                value={importText}
                onChange={(e) => setImportText(e.target.value)}
                placeholder='{"format_version":1,"name":"Research","description":""}'
              />
            </label>
            <button className="button primary" disabled={busy}>
              Import as a new project
            </button>
          </form>
        </section>
      )}
    </>
  );
}
function SettingsPage({
  data,
  busy,
  perform,
}: {
  data: Snapshot;
  busy: boolean;
  perform: Perform;
}) {
  const [settings, setSettings] = useState<Settings>(data.settings);
  useEffect(() => setSettings(data.settings), [data.settings]);
  const [editor, setEditor] = useState<Provider | null>(null);
  const [keyId, setKeyId] = useState<string | null>(null);
  const [key, setKey] = useState("");
  const [result, setResult] = useState("");
  function addProvider() {
    setEditor({
      id: crypto.randomUUID(),
      name: "",
      kind: "openai",
      base_url: providerKinds.openai.url,
      model: "",
      enabled: true,
      timeout_secs: 60,
      max_output_tokens: 2048,
      requires_key: true,
    });
  }
  return (
    <>
      <Heading
        eyebrow="WORKSPACE CONFIGURATION"
        title="Settings"
        description="Choose your models and manage your shared desktop and CLI configuration."
      />
      <div className="settings-grid">
        <section className="panel">
          <div className="panel-heading">
            <h2>Workspace preferences</h2>
          </div>
          <form
            className="form"
            onSubmit={(e) => {
              e.preventDefault();
              void perform(
                () => request("save_settings", { settings }),
                "Preferences saved",
              );
            }}
          >
            <label>
              Security mode
              <select
                value={settings.security_mode}
                onChange={(e) =>
                  setSettings({
                    ...settings,
                    security_mode: e.target.value as Settings["security_mode"],
                  })
                }
              >
                <option value="passive_auto">Passive Auto</option>
                <option value="authorized_auto">Authorized Auto</option>
                <option value="lab_research">Lab Research</option>
              </select>
            </label>
            <p className="helper">
              Modes are saved preferences. This foundation only exposes local
              project operations and AI text; active workflows are unavailable
              in every mode.
            </p>
            <label>
              Default model configuration
              <select
                value={settings.default_provider_id ?? ""}
                onChange={(e) =>
                  setSettings({
                    ...settings,
                    default_provider_id: e.target.value || null,
                  })
                }
              >
                <option value="">No default provider</option>
                {data.providers.map((p) => (
                  <option value={p.id} key={p.id}>
                    {p.name} · {p.model}
                  </option>
                ))}
              </select>
            </label>
            <label>
              Fallback model configuration
              <select
                aria-label="Fallback model configuration"
                value={settings.fallback_provider_id ?? ""}
                onChange={(e) =>
                  setSettings({
                    ...settings,
                    fallback_provider_id: e.target.value || null,
                  })
                }
              >
                <option value="">No fallback</option>
                {data.providers
                  .filter((p) => p.id !== settings.default_provider_id)
                  .map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name} · {p.model}
                    </option>
                  ))}
              </select>
            </label>
            <p className="helper">
              Fallback applies only to the default model when a connection fails
              before a request can be sent. HTTP failures, timeouts and
              interrupted answers are not retried. Saving this choice authorizes
              transmission to the fallback gateway.
            </p>
            <button className="button primary" disabled={busy}>
              Save preferences
            </button>
          </form>
        </section>
        <section className="panel">
          <div className="panel-heading">
            <h2>Local storage</h2>
            <span className="badge">Connected</span>
          </div>
          <div className="storage-detail">
            <p>Desktop and CLI use the same database.</p>
            <code>{data.database_path}</code>
            <p>
              SQLite with versioned migrations, concurrent access, and
              persistent audit records.
            </p>
            <div className="security-note">
              <Glyph name="shield" />
              <span>
                API keys live in OS credential storage. No plaintext fallback is
                enabled.
              </span>
            </div>
          </div>
        </section>
      </div>
      <section className="panel provider-panel">
        <div className="panel-heading">
          <div>
            <h2>AI provider configurations</h2>
            <p className="helper">
              Name configurations for reasoning, local private analysis, or
              report writing.
            </p>
          </div>
          <button className="button secondary" onClick={addProvider}>
            + Add provider
          </button>
        </div>
        {data.providers.length ? (
          <div className="provider-list">
            {data.providers.map((p) => (
              <article className="provider-row" key={p.id}>
                <div className="project-avatar">
                  <Glyph name="ai" />
                </div>
                <div className="provider-info">
                  <b>{p.name}</b>
                  <small>
                    {providerKinds[p.kind].label} · {p.model}
                  </small>
                  <code>{p.base_url}</code>
                </div>
                <span className={p.enabled ? "badge" : "badge muted"}>
                  {p.enabled ? "Enabled" : "Disabled"}
                </span>
                <div className="card-actions">
                  <button
                    className="text-button"
                    disabled={busy}
                    onClick={() => {
                      setEditor(p);
                      setResult("");
                    }}
                  >
                    Edit
                  </button>
                  <button
                    className="text-button"
                    disabled={busy}
                    onClick={() => {
                      setKeyId(p.id);
                      setKey("");
                      setResult("");
                    }}
                  >
                    Credential
                  </button>
                  <button
                    className="text-button"
                    disabled={busy || !p.enabled}
                    onClick={() =>
                      void perform(async () => {
                        const models = await request<string[]>("models", {
                          id: p.id,
                        });
                        setResult(
                          `${p.name}: connected · ${models.length} models · ${models.slice(0, 12).join(", ")}`,
                        );
                      }, "Connection test completed")
                    }
                  >
                    Test connection
                  </button>
                  <button
                    className="text-button danger"
                    disabled={busy}
                    onClick={() =>
                      void perform(
                        () => request("delete_provider", { id: p.id }),
                        "Provider deleted",
                      )
                    }
                  >
                    Delete
                  </button>
                </div>
              </article>
            ))}
          </div>
        ) : (
          <div className="empty">
            <Glyph name="ai" />
            <h3>Connect your preferred model</h3>
            <p>
              Add a cloud provider, Ollama, LM Studio, or a compatible gateway.
            </p>
            <button className="button secondary" onClick={addProvider}>
              Add your first provider
            </button>
          </div>
        )}
        {result && (
          <div className="connection-result" role="status">
            {result}
          </div>
        )}
      </section>
      {editor && (
        <ProviderEditor
          initial={editor}
          busy={busy}
          perform={perform}
          close={() => setEditor(null)}
        />
      )}
      {keyId && (
        <section className="panel credential-panel">
          <div className="panel-heading">
            <h2>Secure credential</h2>
            <button
              className="text-button"
              onClick={() => {
                setKeyId(null);
                setKey("");
              }}
            >
              Close
            </button>
          </div>
          <form
            className="form"
            onSubmit={(e) => {
              e.preventDefault();
              void perform(async () => {
                await request("set_key", { id: keyId, key });
                setKey("");
                setKeyId(null);
              }, "Credential stored securely");
            }}
          >
            <label>
              API key
              <input
                type="password"
                autoComplete="new-password"
                required
                value={key}
                onChange={(e) => setKey(e.target.value)}
              />
            </label>
            <p className="helper">
              The credential is sent only to the saved gateway. Remove it before
              changing the gateway destination or protocol.
            </p>
            <div className="form-actions">
              <button className="button primary" disabled={busy}>
                Store credential
              </button>
              <button
                type="button"
                className="button ghost"
                disabled={busy}
                onClick={() =>
                  void perform(async () => {
                    setResult(
                      (await request<boolean>("credential_status", {
                        id: keyId,
                      }))
                        ? "Credential is configured"
                        : "No credential configured",
                    );
                  })
                }
              >
                Check status
              </button>
              <button
                type="button"
                className="button ghost danger"
                disabled={busy}
                onClick={() =>
                  void perform(async () => {
                    await request("remove_key", { id: keyId });
                    setKey("");
                  }, "Credential removed")
                }
              >
                Remove credential
              </button>
            </div>
          </form>
        </section>
      )}
    </>
  );
}
function ProviderEditor({
  initial,
  busy,
  perform,
  close,
}: {
  initial: Provider;
  busy: boolean;
  perform: Perform;
  close: () => void;
}) {
  const [provider, setProvider] = useState(initial);
  useEffect(() => setProvider(initial), [initial]);
  const update = (patch: Partial<Provider>) =>
    setProvider({ ...provider, ...patch });
  return (
    <section className="panel provider-editor">
      <div className="panel-heading">
        <h2>Provider configuration</h2>
        <button className="text-button" onClick={close}>
          Close editor
        </button>
      </div>
      <form
        className="form"
        onSubmit={(e) => {
          e.preventDefault();
          void perform(async () => {
            await request("save_provider", { provider });
            close();
          }, "Provider configuration saved");
        }}
      >
        <div className="form-grid">
          <label>
            Configuration name
            <input
              required
              maxLength={100}
              value={provider.name}
              onChange={(e) => update({ name: e.target.value })}
              placeholder="e.g. Local Private Model"
            />
          </label>
          <label>
            Provider protocol
            <select
              value={provider.kind}
              onChange={(e) => {
                const kind = e.target.value as Kind;
                update({
                  kind,
                  base_url: providerKinds[kind].url,
                  requires_key: providerKinds[kind].key,
                });
              }}
            >
              {Object.entries(providerKinds).map(([kind, p]) => (
                <option key={kind} value={kind}>
                  {p.label}
                </option>
              ))}
            </select>
          </label>
          <label>
            Gateway base URL
            <input
              required
              type="url"
              value={provider.base_url}
              onChange={(e) => update({ base_url: e.target.value })}
            />
          </label>
          <label>
            Model ID
            <input
              required
              maxLength={200}
              value={provider.model}
              onChange={(e) => update({ model: e.target.value })}
              placeholder="Enter an available model ID"
            />
          </label>
          <label>
            Timeout in seconds
            <input
              type="number"
              required
              min={1}
              max={300}
              value={provider.timeout_secs}
              onChange={(e) => update({ timeout_secs: Number(e.target.value) })}
            />
          </label>
          <label>
            Maximum output tokens
            <input
              type="number"
              required
              min={1}
              max={32768}
              value={provider.max_output_tokens}
              onChange={(e) =>
                update({ max_output_tokens: Number(e.target.value) })
              }
            />
          </label>
        </div>
        <div className="form-actions">
          <label className="inline-check">
            <input
              type="checkbox"
              checked={provider.enabled}
              onChange={(e) => update({ enabled: e.target.checked })}
            />
            Enabled
          </label>
          <label className="inline-check">
            <input
              type="checkbox"
              checked={provider.requires_key}
              onChange={(e) => update({ requires_key: e.target.checked })}
            />
            Requires secure API key
          </label>
        </div>
        <p className="helper">
          Saving a gateway authorizes AI requests to that destination when you
          select Test or Send. HTTPS is required except for loopback addresses.
          Streaming is available in AI Workspace. Proxies and automatic retries
          are not implemented.
        </p>
        <button className="button primary" disabled={busy}>
          Save provider
        </button>
      </form>
    </section>
  );
}
function AiWorkspace({
  data,
  busy,
  perform,
}: {
  data: Snapshot;
  busy: boolean;
  perform: Perform;
}) {
  const [id, setId] = useState(
    data.settings.default_provider_id ??
      data.providers.find((p) => p.enabled)?.id ??
      "",
  );
  const [prompt, setPrompt] = useState("");
  const [stream, setStream] = useState(true);
  const [running, setRunning] = useState<string | null>(null);
  const active = useRef<string | null>(null);
  const [cancelError, setCancelError] = useState("");
  useEffect(
    () => () => {
      if (active.current) void cancelStream(active.current).catch(() => {});
    },
    [],
  );
  const [messages, setMessages] = useState<
    { role: string; text: string; meta?: string }[]
  >([]);
  const enabled = data.providers.filter((p) => p.enabled);
  return (
    <>
      <Heading
        eyebrow="AI-ASSISTED LOCAL RESEARCH"
        title="AI Workspace"
        description="Ask your configured model to explain evidence, review code, or draft remediation."
      />
      <section className="panel chat-panel">
        <div className="panel-heading">
          <div className="chat-heading">
            <Glyph name="ai" />
            <h2>Research assistant</h2>
          </div>
          <select
            aria-label="AI provider"
            value={id}
            onChange={(e) => setId(e.target.value)}
          >
            <option value="">Select a provider</option>
            {enabled.map((p) => (
              <option value={p.id} key={p.id}>
                {p.name} · {p.model}
              </option>
            ))}
          </select>
        </div>
        <div className="chat-messages" aria-live="polite">
          {messages.length ? (
            messages.map((m, i) => (
              <div
                key={i}
                className={
                  m.role === "You"
                    ? "chat-message user"
                    : "chat-message assistant"
                }
              >
                <span className="eyebrow">{m.role}</span>
                <pre>{m.text}</pre>
                {m.meta && <small>{m.meta}</small>}
              </div>
            ))
          ) : (
            <div className="empty large">
              <div className="ai-orb">
                <Glyph name="ai" />
              </div>
              <h2>Turn evidence into understanding</h2>
              <p>
                Connect a model in Settings, then submit text for analysis.
                <br />
                AI answers are unverified interpretations and cannot execute
                actions.
              </p>
              <div className="prompt-hints">
                <span>Explain supplied evidence</span>
                <span>Review remediation</span>
                <span>Summarize research notes</span>
              </div>
            </div>
          )}
        </div>
        <form
          className="chat-composer"
          onSubmit={(e) => {
            e.preventDefault();
            void perform(async () => {
              const current = prompt;
              setMessages((m) => [
                ...m,
                { role: "You", text: current },
                {
                  role: "AI · unverified",
                  text: "",
                  meta: "Waiting for model…",
                },
              ]);
              setPrompt("");
              const updateLast = (text: string, meta: string) =>
                setMessages((m) =>
                  m.map((item, i) =>
                    i === m.length - 1 ? { ...item, text, meta } : item,
                  ),
                );
              try {
                let reply: Reply;
                if (stream) {
                  const requestId = crypto.randomUUID();
                  active.current = requestId;
                  setRunning(requestId);
                  setCancelError("");
                  reply = await streamChat(id, current, requestId, (text) =>
                    updateLast(text, "Streaming · unverified"),
                  );
                } else {
                  reply = await request<Reply>("chat", { id, prompt: current });
                }
                updateLast(
                  reply.text,
                  `${reply.fallback_used ? "Fallback · " : ""}${reply.model} · ${reply.latency_ms} ms · ${reply.input_tokens ?? "?"} input / ${reply.output_tokens ?? "?"} output tokens`,
                );
              } catch (error) {
                setMessages((m) =>
                  m.map((item, i) =>
                    i === m.length - 1
                      ? {
                          ...item,
                          meta: "Interrupted · partial output is unverified",
                        }
                      : item,
                  ),
                );
                throw error;
              } finally {
                active.current = null;
                setRunning(null);
              }
            });
          }}
        >
          <textarea
            aria-label="Message"
            required
            rows={3}
            maxLength={32768}
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            placeholder="Describe the evidence you want to understand…"
          />
          <div className="composer-footer">
            <small>
              Submitted text is sent to the selected provider. Keep secrets out.
              Chat is not persisted.
            </small>
            <label className="inline-check">
              <input
                type="checkbox"
                checked={stream}
                disabled={busy}
                onChange={(e) => setStream(e.target.checked)}
              />
              Stream output
            </label>
            {running && (
              <button
                type="button"
                className="button secondary"
                onClick={() =>
                  void cancelStream(running).catch((e) =>
                    setCancelError(errorText(e)),
                  )
                }
              >
                Cancel request
              </button>
            )}
            <button
              className="button primary"
              disabled={
                busy || !enabled.some((p) => p.id === id) || !prompt.trim()
              }
            >
              {busy ? "Waiting for model…" : "Send message ↗"}
            </button>
          </div>
          {cancelError && (
            <p role="alert" className="helper">
              {cancelError}
            </p>
          )}
        </form>
      </section>
    </>
  );
}
function ToolManagerPage({
  busy,
  perform,
}: {
  busy: boolean;
  perform: Perform;
}) {
  const [inventory, setInventory] = useState<ToolInventory[]>([]);
  const [plan, setPlan] = useState<ToolManifest | null>(null);
  const [progress, setProgress] = useState<ToolProgress | null>(null);
  const [loadError, setLoadError] = useState("");
  const load = useCallback(async () => {
    setInventory(await request<ToolInventory[]>("tools_list"));
    setLoadError("");
  }, []);
  useEffect(() => {
    load().catch((e) => setLoadError(errorText(e)));
  }, [load]);
  return (
    <>
      <Heading
        eyebrow="MANAGED LOCAL UTILITIES"
        title="Tool Manager"
        description="Review pinned official releases, verify integrity, and manage portable local tools."
        action={
          <button
            className="button secondary"
            disabled={busy}
            onClick={() => void perform(load)}
          >
            Refresh inventory
          </button>
        }
      />
      {loadError && (
        <p role="alert" className="banner error">
          {loadError}
        </p>
      )}
      <section className="panel">
        <div className="panel-heading">
          <h2>Tool inventory</h2>
          <span className="badge muted">No administrator access</span>
        </div>
        {inventory.map((tool) => (
          <article
            className="provider-row"
            style={{ padding: "22px" }}
            key={tool.manifest.id}
          >
            <div className="project-avatar">
              <Glyph name="folder" />
            </div>
            <div className="provider-info">
              <b>{tool.manifest.name}</b>
              <small>
                {tool.manifest.version} · {tool.manifest.platform}
              </small>
              <p className="helper">
                Local text utility. Security scanner adapters are not
                implemented.
              </p>
            </div>
            <span
              className={tool.status === "tested" ? "badge" : "badge muted"}
            >
              {tool.status.replaceAll("_", " ")}
            </span>
            <div className="card-actions">
              <button
                disabled={busy}
                className="text-button"
                onClick={() =>
                  void perform(async () =>
                    setPlan(
                      await request<ToolManifest>("tools_plan", {
                        id: tool.manifest.id,
                      }),
                    ),
                  )
                }
              >
                Review install plan
              </button>
              <button
                disabled={busy || !tool.integrity_verified}
                className="text-button"
                onClick={() =>
                  void perform(async () => {
                    await request("tools_doctor", { id: tool.manifest.id });
                    await load();
                  }, "Pinned executable integrity and version verified")
                }
              >
                Health check
              </button>
              <button
                disabled={busy || tool.status === "not_installed"}
                className="text-button danger"
                onClick={() =>
                  void perform(async () => {
                    await request("tools_uninstall", { id: tool.manifest.id });
                    await load();
                    setPlan(null);
                  }, "Managed tool uninstalled")
                }
              >
                Uninstall
              </button>
            </div>
          </article>
        ))}
      </section>
      {plan && (
        <section className="panel provider-editor">
          <div className="panel-heading">
            <h2>Installation plan · {plan.name}</h2>
            <span className="badge muted">Pinned release</span>
          </div>
          <div className="storage-detail">
            <p>
              Version {plan.version} · {plan.platform} · portable installation
              in the application data directory.
            </p>
            <p>Official release</p>
            <code>{plan.source}</code>
            <p>Archive SHA-256</p>
            <code>{plan.sha256}</code>
            <p>Executable SHA-256</p>
            <code>{plan.binary_sha256}</code>
            <p>
              Both hashes must match before installation. The health check
              invokes only the verified executable with a fixed version flag.
              Dependency resolution is trivial for this adapter because it has
              no external dependencies.
            </p>
            <button
              disabled={
                busy ||
                inventory.some(
                  (t) =>
                    t.manifest.id === plan.id && t.status !== "not_installed",
                )
              }
              className="button primary"
              onClick={() =>
                void perform(async () => {
                  setProgress(null);
                  await installTool(plan.id, setProgress);
                  await load();
                }, "Official release downloaded, verified and installed")
              }
            >
              Install verified release
            </button>
          </div>
        </section>
      )}
      {progress && (
        <div className="connection-result" role="status">
          {progress.phase} · {(progress.downloaded / 1024 / 1024).toFixed(2)}{" "}
          MiB{" "}
          {progress.total
            ? "/ " + (progress.total / 1024 / 1024).toFixed(2) + " MiB"
            : ""}
        </div>
      )}
      <p className="helper" style={{ marginTop: 18 }}>
        Discovery of unmanaged executables, profile installation, updates,
        rollback and installer cancellation remain planned. A tested ripgrep
        binary does not imply support for any security scanner.
      </p>
    </>
  );
}
function PlannedModule({ name }: { name: string }) {
  return (
    <>
      <Heading
        eyebrow="MODULE ROADMAP"
        title={name}
        description="This module is scheduled for a later development phase."
      />
      <section className="panel">
        <div className="empty large">
          <Glyph name="shield" />
          <span className="badge muted">NOT STARTED</span>
          <h2>Implementation pending</h2>
          <p>
            {name} has no execution capabilities in this build.
            <br />
            The shared project, settings, provider, and audit foundations are
            available.
          </p>
          <p className="helper">
            See docs/ROADMAP.md and docs/IMPLEMENTATION_STATUS.md for the
            remaining work.
          </p>
        </div>
      </section>
    </>
  );
}
