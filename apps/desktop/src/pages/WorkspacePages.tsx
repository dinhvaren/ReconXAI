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
  errorText,
  providerKinds,
  type ToolInventory,
  type ToolManifest,
  type ToolProgress,
  type Snapshot,
  type Provider,
  type Kind,
  type Reply,
  type Settings,
} from "../api";
import { Modal, Heading, Icon as Glyph, type Perform } from "../components/ui";
import { useTheme } from "../theme";
const date = (text: string) => new Date(text).toLocaleString();
export function Projects({
  data,
  busy,
  perform,
  onAddTarget,
}: {
  data: Snapshot;
  busy: boolean;
  perform: Perform;
  onAddTarget: () => void;
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
          <div className="inline-actions">
            <button className="button" onClick={onAddTarget}>
              + Add Target
            </button>
            <button
              className="button secondary"
              onClick={() => setShowImport(!showImport)}
            >
              Import project
            </button>
          </div>
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
export function SettingsPage({
  data,
  busy,
  perform,
}: {
  data: Snapshot;
  busy: boolean;
  perform: Perform;
}) {
  const theme = useTheme();
  const [settings, setSettings] = useState<Omit<Settings, "theme">>(() => ({
    security_mode: data.settings.security_mode,
    default_provider_id: data.settings.default_provider_id,
    fallback_provider_id: data.settings.fallback_provider_id,
  }));
  // Appearance is managed centrally; theme changes must not discard unsaved routing/mode edits.
  useEffect(
    () =>
      setSettings((current) => ({
        ...current,
        security_mode: data.settings.security_mode,
        default_provider_id: data.settings.default_provider_id,
        fallback_provider_id: data.settings.fallback_provider_id,
      })),
    [
      data.settings.security_mode,
      data.settings.default_provider_id,
      data.settings.fallback_provider_id,
    ],
  );
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
                () => theme.savePreferences(settings),
                "Preferences saved",
              );
            }}
          >
            <label>
              Appearance
              <select
                aria-label="Appearance"
                value={theme.preference}
                onChange={(e) =>
                  void theme
                    .setTheme(e.target.value as Settings["theme"])
                    .catch(() => {})
                }
              >
                <option value="light">Light</option>
                <option value="dark">Dark</option>
                <option value="reconx">ReconX</option>
              </select>
            </label>
            <p className="helper">
              Appearance applies and saves immediately. Light, Dark and ReconX
              share one selection across all appearance controls.
            </p>
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
export function ProviderEditor({
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
export function AiWorkspace({
  data,
  busy,
  perform,
  onTools,
}: {
  data: Snapshot;
  busy: boolean;
  perform: Perform;
  onTools: () => void;
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
  const [contextWidth, setContextWidth] = useState(210);
  const [contextTab, setContextTab] = useState("History");
  const selectedProvider = data.providers.find((p) => p.id === id);
  const history = data.audit
    .filter(
      (event) =>
        event.action.startsWith("ai.") || event.action.startsWith("tool."),
    )
    .slice(0, 8);
  return (
    <>
      <Heading
        eyebrow="AI-ASSISTED LOCAL RESEARCH"
        title="AI Workspace"
        action={
          <button className="button secondary" onClick={onTools}>
            Plan local analysis
          </button>
        }
        description="Ask your configured model to explain evidence, review code, or draft remediation."
      />
      <div className="ai-layout">
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
                  {m.meta && (
                    <details className="reasoning-summary">
                      <summary>Response details</summary>
                      <p>{m.meta}</p>
                    </details>
                  )}
                </div>
              ))
            ) : (
              <div className="empty research-notebook-empty">
                <div className="ai-orb">
                  <Glyph name="ai" />
                </div>
                <h2>Research notebook</h2>
                <p>
                  Select a model and bring a question or evidence to examine.
                  <br />
                  AI answers are unverified interpretations and cannot execute
                  actions.
                </p>
                <div className="prompt-hints">
                  <button
                    type="button"
                    onClick={() =>
                      setPrompt(
                        "Explain the following evidence. Separate observations from hypotheses:\n\n",
                      )
                    }
                  >
                    Explain supplied evidence
                  </button>
                  <button
                    type="button"
                    onClick={() =>
                      setPrompt(
                        "Review this remediation and identify assumptions or missing information:\n\n",
                      )
                    }
                  >
                    Review remediation
                  </button>
                  <button
                    type="button"
                    onClick={() =>
                      setPrompt(
                        "Summarize these research notes without inventing findings:\n\n",
                      )
                    }
                  >
                    Summarize notes
                  </button>
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
                    reply = await request<Reply>("chat", {
                      id,
                      prompt: current,
                    });
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
                Submitted text is sent to the selected provider. Keep secrets
                out. Chat is not persisted.
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
        <div
          className="context-resizer"
          role="separator"
          aria-label="Resize research context"
          aria-orientation="vertical"
          aria-valuenow={contextWidth}
          aria-valuemin={175}
          aria-valuemax={280}
          tabIndex={0}
          onKeyDown={(e) => {
            if (e.key === "ArrowLeft")
              setContextWidth((w) => Math.min(280, w + 10));
            if (e.key === "ArrowRight")
              setContextWidth((w) => Math.max(175, w - 10));
          }}
          onPointerDown={(e) => e.currentTarget.setPointerCapture(e.pointerId)}
          onPointerMove={(e) => {
            if (e.currentTarget.hasPointerCapture(e.pointerId))
              setContextWidth((w) =>
                Math.max(175, Math.min(280, w - e.movementX)),
              );
          }}
          onPointerUp={(e) => {
            if (e.currentTarget.hasPointerCapture(e.pointerId))
              e.currentTarget.releasePointerCapture(e.pointerId);
          }}
        />
        <aside
          className="research-context"
          aria-label="Research context"
          style={{ width: contextWidth }}
        >
          <div className="research-context-header">Research context</div>
          <div className="research-context-section">
            <h3>Selected model</h3>
            <p>
              {selectedProvider
                ? `${selectedProvider.name} · ${selectedProvider.model}`
                : "No model selected"}
            </p>
            <p>
              {running
                ? "Text request in progress"
                : "Ready for a text request"}
            </p>
          </div>
          <div className="tabs" role="tablist" aria-label="Context views">
            {["History", "Evidence"].map((tab) => (
              <button
                type="button"
                role="tab"
                className="tab"
                key={tab}
                aria-selected={contextTab === tab}
                onClick={() => setContextTab(tab)}
              >
                {tab}
              </button>
            ))}
          </div>
          <div
            className="research-context-section"
            role="tabpanel"
            aria-label={contextTab}
          >
            {contextTab === "History" ? (
              history.length ? (
                history.map((event) => (
                  <div className="context-event" key={event.id}>
                    {event.action}
                    <small>
                      {event.outcome} ·{" "}
                      {new Date(event.timestamp).toLocaleTimeString([], {
                        hour: "2-digit",
                        minute: "2-digit",
                      })}
                    </small>
                  </div>
                ))
              ) : (
                <p>No model or tool operations recorded yet.</p>
              )
            ) : (
              <p>
                Evidence attachments are not implemented. Only text you submit
                is sent to the selected model.
              </p>
            )}
          </div>
          <div className="research-context-section">
            <h3>Agent tasks & approvals</h3>
            <p>
              Agent execution and workflow approval controls are not available.
              This conversation cannot run tools.
            </p>
          </div>
          <div className="research-context-section">
            <h3>Response quality</h3>
            <p>
              Model answers are unverified interpretations. Review the supplied
              evidence before recording a conclusion.
            </p>
          </div>
        </aside>
      </div>
    </>
  );
}
export function ToolManagerPage({
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
  const [profile, setProfile] = useState("reverse");
  const [profilePlan, setProfilePlan] = useState<ToolManifest[]>([]);
  const [catalog, setCatalog] = useState<
    { id: string; category: string; integration: string }[]
  >([]);
  const [query, setQuery] = useState("");
  const [installing, setInstalling] = useState("");
  const [operationLog, setOperationLog] = useState("");
  async function install(id: string) {
    setInstalling(id);
    try {
      await installTool(id, setProgress);
      await request("tools_doctor", { id });
    } finally {
      setInstalling("");
    }
  }
  const load = useCallback(async () => {
    setInventory(await request<ToolInventory[]>("tools_list"));
    setCatalog(await request("tools_catalog"));
    setLoadError("");
  }, []);
  useEffect(() => {
    load().catch((e) => setLoadError(errorText(e)));
  }, [load]);
  return (
    <>
      <Modal
        title="Tool operation history"
        open={!!operationLog}
        onClose={() => setOperationLog("")}
      >
        <pre className="technical-output">{operationLog}</pre>
      </Modal>
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
      <section className="panel integration-detail">
        <div className="panel-heading">
          <h2>Tool profiles</h2>
        </div>
        <label>
          Profile
          <select
            aria-label="Tool profile"
            value={profile}
            onChange={(e) => {
              setProfile(e.target.value);
              setProfilePlan([]);
            }}
          >
            <option value="core">Core</option>
            <option value="reverse">Reverse Engineering</option>
            {["recon", "web", "api", "mobile", "cloud", "full"].map((p) => (
              <option key={p} value={p}>
                {p} - pending adapters
              </option>
            ))}
          </select>
        </label>
        <button
          className="button secondary"
          disabled={busy}
          onClick={() =>
            void perform(async () =>
              setProfilePlan(
                await request<ToolManifest[]>("tools_profile_plan", {
                  id: profile,
                }),
              ),
            )
          }
        >
          Review profile plan
        </button>
        {profilePlan.length > 0 && (
          <>
            <pre className="technical-output">
              {JSON.stringify(
                profilePlan.map((t) => ({
                  id: t.id,
                  version: t.version,
                  url: t.url,
                  sha256: t.sha256,
                  dependencies: t.dependencies,
                })),
                null,
                2,
              )}
            </pre>
            <button
              className="button primary"
              disabled={busy}
              onClick={() =>
                void perform(async () => {
                  for (const tool of profilePlan) {
                    if (
                      !inventory.some(
                        (t) =>
                          t.manifest.id === tool.id && t.integrity_verified,
                      )
                    )
                      await install(tool.id);
                    else await request("tools_doctor", { id: tool.id });
                  }
                  setProfilePlan([]);
                  await load();
                }, "Profile installed and health checks passed")
              }
            >
              Approve profile installation
            </button>
          </>
        )}
      </section>
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
                {tool.manifest.capabilities.join(", ")} - latest verified
                manifest: {tool.manifest.version}.
              </p>
              <details>
                <summary>Tool details</summary>
                <p className="helper">
                  Platform: {tool.manifest.platform}. Dependencies:{" "}
                  {tool.manifest.dependencies.join(", ") || "None"}. Last health
                  check: {tool.receipt?.last_health_check || "Not checked"}.
                </p>
                <code>{tool.manifest.binary_sha256}</code>
              </details>
            </div>
            <span
              className={tool.status === "tested" ? "badge" : "badge muted"}
            >
              {tool.status.replaceAll("_", " ")}
            </span>
            <div className="card-actions">
              <button
                className="text-button"
                disabled={busy}
                onClick={() =>
                  void perform(async () =>
                    setOperationLog(
                      JSON.stringify(
                        await request("tools_history", {
                          id: tool.manifest.id,
                        }),
                        null,
                        2,
                      ),
                    ),
                  )
                }
              >
                Open logs
              </button>
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
                title={
                  !tool.integrity_verified
                    ? "Install and verify this tool before checking health"
                    : "Check the verified executable version"
                }
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
                className="text-button"
                disabled={busy || tool.integrity_verified}
                title={
                  tool.integrity_verified
                    ? "Installed executable matches the latest pinned verified manifest"
                    : "Reinstall only the managed files, then verify integrity and version"
                }
                onClick={() =>
                  void perform(async () => {
                    await request("tools_uninstall", { id: tool.manifest.id });
                    await install(tool.manifest.id);
                    await load();
                  }, "Managed tool repaired and verified")
                }
              >
                Repair
              </button>
              <button
                disabled={busy || tool.status === "not_installed"}
                title={
                  tool.status === "not_installed"
                    ? "This tool has not been installed"
                    : "Remove only this managed installation"
                }
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
                  await install(plan.id);
                  await load();
                }, "Official release downloaded, verified and installed")
              }
            >
              Install verified release
            </button>
          </div>
        </section>
      )}
      {busy && progress?.phase === "downloading" && installing && (
        <button
          className="button secondary"
          onClick={() => void cancelStream(`tool:${installing}`)}
        >
          Cancel download
        </button>
      )}
      <section className="panel integration-detail">
        <div className="panel-heading">
          <h2>Integration catalog</h2>
        </div>
        <input
          aria-label="Search tool catalog"
          placeholder="Search registered tools"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <p className="helper">
          Only ripgrep and YARA-X have verified installers and fixed local
          adapters. Other entries below are pending integration; no install or
          execution is claimed.
        </p>
        <div className="integration-catalog">
          {catalog
            .filter((t) => t.id.includes(query.toLowerCase()))
            .map((t) => (
              <p key={t.id}>
                <code>{t.id}</code> · {t.category} · {t.integration}
              </p>
            ))}
        </div>
      </section>
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
        rollback and package-manager adapters remain planned. A tested local
        binary does not imply support for any security scanner.
      </p>
    </>
  );
}
export function PlannedModule({ name }: { name: string }) {
  const [view, setView] = useState("Disassembly");
  const content: Record<
    string,
    { icon: string; description: string; capabilities: [string, string][] }
  > = {
    "MCP Hub": {
      icon: "plug",
      description:
        "A central place for server connections, capabilities and permissions.",
      capabilities: [
        ["Server registry", "Stdio and remote transports are planned."],
        [
          "Tool permissions",
          "Capability discovery and approval rules are planned.",
        ],
      ],
    },
    "Reverse Engineering": {
      icon: "binary",
      description: "A technical workspace for local, owned binary samples.",
      capabilities: [
        [
          "Static analysis",
          "PE/ELF metadata, strings and section inventory are planned.",
        ],
        ["AI assistance", "Evidence-based code explanations are planned."],
      ],
    },
    Reconnaissance: {
      icon: "globe",
      description:
        "Target intelligence with explicit scope and request budgets.",
      capabilities: [
        ["Target inventory", "Registration and scope policies are planned."],
        [
          "Passive collection",
          "Normalized intelligence and provenance are planned.",
        ],
      ],
    },
    Findings: {
      icon: "flag",
      description:
        "Reviewable observations linked to independently collected evidence.",
      capabilities: [
        [
          "Verification states",
          "Hypothesis, unverified, verified and refuted records are planned.",
        ],
        [
          "Evidence references",
          "Attachments, provenance and deduplication are planned.",
        ],
      ],
    },
    Reports: {
      icon: "file",
      description: "Evidence-led reporting with clear verification status.",
      capabilities: [
        ["Export formats", "Markdown, HTML, JSON and PDF exports are planned."],
        [
          "Quality controls",
          "Evidence checks and secret redaction are planned.",
        ],
      ],
    },
  };
  const module = content[name] ?? {
    icon: "shield",
    description: "A dedicated workspace for this research discipline.",
    capabilities: [
      [
        "Shared projects",
        "Project and configuration foundations are available.",
      ],
      ["Native analysis", "This module's analysis service is planned."],
    ] as [string, string][],
  };
  return (
    <>
      <Heading
        eyebrow="RESEARCH MODULE"
        title={name}
        description={module.description}
        action={<span className="badge muted">NOT STARTED</span>}
      />
      <section className="module-workspace">
        <div className="module-banner">
          <Glyph name={module.icon} size={28} />
          <div>
            <h2>
              {name === "MCP Hub"
                ? "No gateway connected"
                : name === "Reverse Engineering"
                  ? "Binary analysis workspace"
                  : "Implementation pending"}
            </h2>
            <p>{name} has no execution capabilities in this build.</p>
          </div>
        </div>
        {name === "Reverse Engineering" ? (
          <div className="technical-layout">
            <aside className="binary-explorer">
              <h3>Binary explorer</h3>
              <p>No binary analysis service is connected.</p>
              <h3 style={{ marginTop: 23 }}>Functions</h3>
              <p>
                Function inventory will appear after a native analyzer is
                implemented.
              </p>
            </aside>
            <div>
              <div
                className="tabs"
                role="tablist"
                aria-label="Binary analysis views"
              >
                {["Disassembly", "Decompiled", "Metadata"].map((tab) => (
                  <button
                    role="tab"
                    className="tab"
                    key={tab}
                    aria-selected={view === tab}
                    onClick={() => setView(tab)}
                  >
                    {tab}
                  </button>
                ))}
              </div>
              <div
                className="module-empty-state"
                role="tabpanel"
                aria-label={view}
              >
                <Glyph name="binary" size={35} />
                <h2>No binary loaded</h2>
                <p>
                  The {view.toLowerCase()} view is reserved for a future native
                  analyzer. This build cannot load or inspect binaries.
                </p>
              </div>
              <div className="terminal-panel">
                <span className="terminal-prompt">analysis / status</span>
                {"\n"}Native binary analysis: not implemented{"\n"}Debugger
                integrations: not connected
              </div>
            </div>
          </div>
        ) : name === "MCP Hub" ? (
          <>
            <div className="table-scroll">
              <table>
                <thead>
                  <tr>
                    <th>Server</th>
                    <th>Transport</th>
                    <th>Health</th>
                    <th>Permissions</th>
                  </tr>
                </thead>
                <tbody>
                  <tr>
                    <td colSpan={4}>
                      <div className="empty compact">
                        <Glyph name="plug" />
                        <h3>MCP gateway is not implemented</h3>
                        <p>
                          No servers or discovered capabilities are available.
                          Existing external server configurations have not been
                          imported.
                        </p>
                      </div>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
            <div className="storage-detail">
              <ul className="module-capability-list">
                {module.capabilities.map(([title, description]) => (
                  <li key={title}>
                    <b>{title}</b>
                    {description}
                  </li>
                ))}
              </ul>
            </div>
          </>
        ) : name === "Findings" ? (
          <div className="table-scroll">
            <table>
              <thead>
                <tr>
                  <th>Severity</th>
                  <th>Finding</th>
                  <th>Asset</th>
                  <th>Verification</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td colSpan={4}>
                    <div className="empty large">
                      <Glyph name="flag" />
                      <h3>No findings engine available</h3>
                      <p>
                        This build has no findings database or evidence
                        validation service. No vulnerability records are
                        generated.
                      </p>
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        ) : (
          <div className="module-empty-state">
            <Glyph name={module.icon} size={32} />
            <h2>{name} is planned</h2>
            <p>
              The project, AI settings and audit foundations are functional.
              This module's analysis and execution service has not been
              implemented.
            </p>
            <ul className="module-capability-list">
              {module.capabilities.map(([title, description]) => (
                <li key={title}>
                  <b>{title}</b>
                  {description}
                </li>
              ))}
            </ul>
          </div>
        )}
      </section>
    </>
  );
}
