import { Button } from "../components/controls";
import { useWorkspaceProject } from "../workspace";
import { useCallback, useEffect, useState } from "react";
import { request, type Snapshot } from "../api";
import { Heading, Modal, type Perform } from "../components/ui";
import JobHistory from "../components/JobHistory";
import type { AnalysisJob } from "../research";

interface Tool {
  name: string;
  description?: string;
  inputSchema: unknown;
  outputSchema?: unknown;
  annotations?: {
    readOnlyHint?: boolean;
    openWorldHint?: boolean;
    destructiveHint?: boolean;
  };
}
interface Config {
  id: string;
  name: string;
  transport: string;
  executable: string | null;
  args: string[];
  working_directory: string | null;
  endpoint: string | null;
  environment: string[];
  bearer_auth: boolean;
  enabled: boolean;
  trusted: boolean;
  timeout_ms: number;
  allowed_tools: string[];
}
interface Server {
  config: Config;
  revision: string;
  tools: Tool[];
  last_connected_at: string | null;
}
interface Preview {
  fingerprint: string;
  server_id: string;
  tool: string;
  input: unknown;
  warning: string;
}
const blank = (): Config => ({
  id: "",
  name: "",
  transport: "stdio",
  executable: "",
  args: [],
  working_directory: null,
  endpoint: null,
  environment: [],
  bearer_auth: false,
  enabled: true,
  trusted: false,
  timeout_ms: 10000,
  allowed_tools: [],
});

export function McpPage({
  data,
  projectId,
  busy,
  perform,
  addRequested = false,
  onAddHandled,
}: {
  data: Snapshot;
  projectId?: string;
  busy: boolean;
  perform: Perform;
  addRequested?: boolean;
  onAddHandled?: () => void;
}) {
  const [servers, setServers] = useState<Server[]>([]),
    [selected, setSelected] = useState("");
  const [states, setStates] = useState<Record<string, string>>({}),
    [history, setHistory] = useState<AnalysisJob[]>([]);
  const [editor, setEditor] = useState<Config | null>(null),
    [args, setArgs] = useState("[]"),
    [allowed, setAllowed] = useState(""),
    [envNames, setEnvNames] = useState("");
  const [toolName, setToolName] = useState(""),
    [input, setInput] = useState("{}"),
    [preview, setPreview] = useState<Preview | null>(null);
  const [project, setProject] = useWorkspaceProject(projectId ?? ""),
    [secret, setSecret] = useState(""),
    [secretName, setSecretName] = useState("bearer"),
    [loadError, setLoadError] = useState("");
  const server = servers.find((s) => s.config.id === selected),
    tool = server?.tools.find((t) => t.name === toolName);
  const load = useCallback(async () => {
    const values = await request<Server[]>("mcp_list");
    setServers(values);
    const pairs = await Promise.all(
      values.map(async (s) => [
        s.config.id,
        await request<string>("mcp_state", { id: s.config.id }),
      ]),
    );
    setStates(Object.fromEntries(pairs));
    if (selected)
      setHistory(await request<AnalysisJob[]>("mcp_history", { id: selected }));
    setLoadError("");
  }, [selected]);
  useEffect(() => {
    void load().catch((e) => setLoadError(String(e.message ?? e)));
    const timer = setInterval(
      () => void load().catch((e) => setLoadError(String(e.message ?? e))),
      3000,
    );
    return () => clearInterval(timer);
  }, [load]);
  useEffect(() => {
    setToolName("");
    setInput("{}");
    setPreview(null);
    setSecret("");
  }, [selected]);
  useEffect(() => {
    setSecretName(
      server?.config.bearer_auth
        ? "bearer"
        : (server?.config.environment[0] ?? ""),
    );
  }, [
    selected,
    server?.config.bearer_auth,
    server?.config.environment.join(","),
  ]);
  function edit(config: Config) {
    setEditor({ ...config });
    setArgs(JSON.stringify(config.args));
    setAllowed(config.allowed_tools.join(", "));
    setEnvNames(config.environment.join(", "));
  }
  useEffect(() => {
    if (addRequested) {
      edit(blank());
      onAddHandled?.();
    }
  }, [addRequested, onAddHandled]);
  return (
    <>
      <Heading
        eyebrow="Connections"
        title="MCP Hub"
        description="Discover capabilities from explicitly trusted servers. Review exact inputs before every invocation."
        action={
          <Button
            className="button primary"
            disabled={busy}
            onClick={() => edit(blank())}
            disabledReason={"An operation is in progress"}
          >
            Add MCP Server
          </Button>
        }
      />
      {loadError && (
        <p role="alert" className="connection-result">
          {loadError}
        </p>
      )}
      <div className="integration-layout">
        <section className="panel integration-list" aria-label="MCP servers">
          <div className="panel-heading">
            <h2>Servers</h2>
            <Button
              className="text-button"
              disabled={busy}
              onClick={() => void perform(load)}
              disabledReason={"An operation is in progress"}
            >
              Refresh
            </Button>
          </div>
          {!servers.length && (
            <p className="helper">
              Add your own executable or Streamable HTTP endpoint. No external
              integrations are preconfigured.
            </p>
          )}
          {servers.map((s) => (
            <button
              key={s.config.id}
              className={`integration-server ${selected === s.config.id ? "selected" : ""}`}
              onClick={() => setSelected(s.config.id)}
            >
              <span>{s.config.name}</span>
              <small>
                {s.config.transport} · {states[s.config.id] ?? "Checking"}
              </small>
              <small>
                {s.tools.length} discovered tools ·{" "}
                {s.config.enabled ? "Enabled" : "Disabled"}
              </small>
            </button>
          ))}
        </section>
        <section className="panel integration-detail">
          {!server ? (
            <p className="helper">
              Select a server to inspect its configuration, permissions and
              tools.
            </p>
          ) : (
            <>
              <div className="panel-heading">
                <h2>{server.config.name}</h2>
                <span className="badge muted">
                  {states[selected] ?? "Checking"}
                </span>
              </div>
              <p className="helper">
                {server.config.transport === "stdio"
                  ? [server.config.executable, ...server.config.args].join(" ")
                  : server.config.endpoint}
              </p>
              <p className="helper">
                Last connected:{" "}
                {server.last_connected_at
                  ? new Date(server.last_connected_at).toLocaleString()
                  : "Never"}
                . Authentication:{" "}
                {server.config.bearer_auth
                  ? "Bearer authentication; OS credential required"
                  : "No HTTP authentication"}
                .
              </p>
              <div className="card-actions">
                <Button
                  className="button primary"
                  disabled={
                    busy || !server.config.enabled || !server.config.trusted
                  }
                  title="Connect only after reviewing and trusting the configured server"
                  onClick={() =>
                    void perform(async () => {
                      await request("mcp_connect", { id: selected });
                      await load();
                    }, "Connected and discovered actual MCP tools")
                  }
                >
                  Connect / Discover
                </Button>
                <Button
                  className="button secondary"
                  disabled={busy || states[selected] !== "connected"}
                  onClick={() =>
                    void perform(async () => {
                      await request("mcp_disconnect", { id: selected });
                      await load();
                    })
                  }
                  disabledReason={
                    busy
                      ? "An operation is in progress"
                      : "Connect this MCP server first"
                  }
                >
                  Disconnect
                </Button>
                <Button
                  className="text-button"
                  disabled={
                    busy || !server.config.enabled || !server.config.trusted
                  }
                  onClick={() =>
                    void perform(async () => {
                      await request("mcp_disconnect", { id: selected });
                      await request("mcp_connect", { id: selected });
                      await load();
                    }, "MCP server reconnected and tools discovered")
                  }
                  disabledReason={
                    busy
                      ? "An operation is in progress"
                      : !server.config.enabled
                        ? "Enable this MCP server first"
                        : "Review this server and grant trust before connecting"
                  }
                >
                  Reconnect
                </Button>
                <Button
                  className="text-button"
                  disabled={busy}
                  onClick={() => edit(server.config)}
                  disabledReason={"An operation is in progress"}
                >
                  Configure / Permissions
                </Button>
                <Button
                  className="text-button danger"
                  disabled={busy}
                  onClick={() =>
                    void perform(async () => {
                      await request("mcp_remove", { id: selected });
                      setSelected("");
                      await load();
                    }, "Server removed; execution history retained")
                  }
                  disabledReason={"An operation is in progress"}
                >
                  Remove server
                </Button>
              </div>
              {!server.config.trusted && (
                <p className="helper">
                  Connection is blocked until you explicitly trust the
                  executable or endpoint in configuration.
                </p>
              )}
              {(server.config.bearer_auth ||
                server.config.environment.length > 0) && (
                <form
                  className="form"
                  onSubmit={(e) => {
                    e.preventDefault();
                    void perform(async () => {
                      await request("mcp_set_secret", {
                        id: selected,
                        name: secretName,
                        key: secret,
                      });
                      setSecret("");
                    }, "MCP credential stored securely");
                  }}
                >
                  <label>
                    Credential name
                    <select
                      aria-label="MCP credential name"
                      value={secretName}
                      onChange={(e) => setSecretName(e.target.value)}
                    >
                      {[
                        ...(server.config.bearer_auth ? ["bearer"] : []),
                        ...server.config.environment,
                      ].map((n) => (
                        <option key={n}>{n}</option>
                      ))}
                    </select>
                  </label>
                  <label>
                    Secret
                    <input
                      aria-label="MCP secret"
                      type="password"
                      autoComplete="new-password"
                      value={secret}
                      onChange={(e) => setSecret(e.target.value)}
                    />
                  </label>
                  <Button
                    className="button secondary"
                    disabled={busy || !secret}
                    disabledReason={
                      busy
                        ? "An operation is in progress"
                        : "Enter the credential first"
                    }
                  >
                    Store credential
                  </Button>
                </form>
              )}
              <div className="panel-heading">
                <h2>Tool explorer</h2>
              </div>
              <label>
                Discovered tool
                <select
                  aria-label="MCP tool"
                  value={toolName}
                  onChange={(e) => {
                    setToolName(e.target.value);
                    setPreview(null);
                    setInput("{}");
                  }}
                >
                  <option value="">Select tool</option>
                  {server.tools.map((t) => (
                    <option key={t.name}>{t.name}</option>
                  ))}
                </select>
              </label>
              {tool && (
                <>
                  <p className="helper">
                    {tool.description ?? "No description provided"} — Server
                    content is untrusted.
                  </p>
                  <details>
                    <summary>Input / output schemas</summary>
                    <pre className="technical-output">
                      {JSON.stringify(
                        {
                          input: tool.inputSchema,
                          output: tool.outputSchema ?? "Not provided",
                          annotations: tool.annotations,
                        },
                        null,
                        2,
                      )}
                    </pre>
                  </details>
                  <p className="helper">
                    Permission:{" "}
                    {server.config.allowed_tools.includes(tool.name)
                      ? "Manually allowlisted; read-only / closed-world checks still apply"
                      : "Denied. Add the reviewed tool name in server permissions to enable calls."}
                  </p>
                  <label>
                    JSON input
                    <textarea
                      aria-label="MCP input"
                      className="code-input"
                      rows={5}
                      value={input}
                      onChange={(e) => {
                        setInput(e.target.value);
                        setPreview(null);
                      }}
                    />
                  </label>
                  <label>
                    Project
                    <select
                      aria-label="MCP project"
                      value={project}
                      onChange={(e) => {
                        setProject(e.target.value);
                        setPreview(null);
                      }}
                    >
                      <option value="">Select project</option>
                      {data.projects
                        .filter((p) => !p.archived)
                        .map((p) => (
                          <option value={p.id} key={p.id}>
                            {p.name}
                          </option>
                        ))}
                    </select>
                  </label>
                  <Button
                    className="button primary"
                    disabled={
                      busy ||
                      !project ||
                      !server.config.allowed_tools.includes(tool.name)
                    }
                    onClick={() =>
                      void perform(async () =>
                        setPreview(
                          await request<Preview>("mcp_preview", {
                            id: selected,
                            tool: tool.name,
                            input: JSON.parse(input),
                          }),
                        ),
                      )
                    }
                    disabledReason={
                      busy
                        ? "An operation is in progress"
                        : !project
                          ? "Select a project first"
                          : "Grant permission for this tool in the MCP server configuration"
                    }
                  >
                    Review invocation
                  </Button>
                </>
              )}
              <div className="panel-heading">
                <h2>Invocation history</h2>
              </div>
              <JobHistory
                jobs={history}
                onCancel={(id) =>
                  void perform(async () => {
                    await request("job_cancel", { id });
                    await load();
                  })
                }
              />
            </>
          )}
        </section>
      </div>
      <Modal
        title="MCP server configuration"
        open={!!editor}
        onClose={() => setEditor(null)}
      >
        {editor && (
          <form
            className="form"
            onSubmit={(e) => {
              e.preventDefault();
              void perform(async () => {
                const array: unknown = JSON.parse(args);
                if (
                  !Array.isArray(array) ||
                  !array.every((a) => typeof a === "string")
                )
                  throw new Error("Arguments must be an array of strings");
                const saved = await request<Server>("mcp_save", {
                  server: {
                    ...editor,
                    args: editor.transport === "stdio" ? array : [],
                    allowed_tools: allowed
                      .split(",")
                      .map((s) => s.trim())
                      .filter(Boolean),
                    environment:
                      editor.transport === "stdio"
                        ? envNames
                            .split(",")
                            .map((s) => s.trim())
                            .filter(Boolean)
                        : [],
                  },
                });
                setSelected(saved.config.id);
                setEditor(null);
                await load();
              }, "MCP configuration saved; connect to discover tools");
            }}
          >
            <label>
              Name
              <input
                aria-label="Server name"
                required
                value={editor.name}
                onChange={(e) => setEditor({ ...editor, name: e.target.value })}
              />
            </label>
            <label>
              Transport
              <select
                aria-label="MCP transport"
                value={editor.transport}
                onChange={(e) =>
                  setEditor({
                    ...editor,
                    transport: e.target.value,
                    executable: null,
                    endpoint: null,
                    working_directory: null,
                    bearer_auth: false,
                  })
                }
              >
                <option value="stdio">stdio</option>
                <option value="streamable_http">Streamable HTTP</option>
              </select>
            </label>
            {editor.transport === "stdio" ? (
              <>
                <label>
                  Absolute executable path
                  <input
                    aria-label="MCP executable"
                    required
                    value={editor.executable ?? ""}
                    onChange={(e) =>
                      setEditor({ ...editor, executable: e.target.value })
                    }
                  />
                </label>
                <label>
                  Arguments (JSON string array)
                  <textarea
                    aria-label="MCP arguments"
                    className="code-input"
                    value={args}
                    onChange={(e) => setArgs(e.target.value)}
                  />
                </label>
                <label>
                  Working directory
                  <input
                    aria-label="MCP working directory"
                    value={editor.working_directory ?? ""}
                    onChange={(e) =>
                      setEditor({
                        ...editor,
                        working_directory: e.target.value || null,
                      })
                    }
                  />
                </label>
                <label>
                  Secret environment variable names (comma separated)
                  <input
                    aria-label="MCP environment names"
                    value={envNames}
                    onChange={(e) => setEnvNames(e.target.value)}
                  />
                </label>
                <p className="helper">
                  Enter names only. Store values separately in the OS credential
                  store. Host environment and AI provider credentials are not
                  forwarded. The child runs with your account privileges;
                  configure an external sandbox if required.
                </p>
              </>
            ) : (
              <>
                <label>
                  Endpoint URL
                  <input
                    aria-label="MCP endpoint"
                    required
                    value={editor.endpoint ?? ""}
                    onChange={(e) =>
                      setEditor({ ...editor, endpoint: e.target.value })
                    }
                  />
                </label>
                <label className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={editor.bearer_auth}
                    onChange={(e) =>
                      setEditor({ ...editor, bearer_auth: e.target.checked })
                    }
                  />
                  Use bearer authentication
                </label>
              </>
            )}
            <label>
              Connection / invocation timeout (ms)
              <input
                aria-label="MCP timeout"
                type="number"
                min={100}
                max={120000}
                value={editor.timeout_ms}
                onChange={(e) =>
                  setEditor({ ...editor, timeout_ms: Number(e.target.value) })
                }
              />
            </label>
            <label>
              Allowed tool names (comma separated)
              <input
                aria-label="MCP allowed tools"
                value={allowed}
                onChange={(e) => setAllowed(e.target.value)}
              />
            </label>
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={editor.enabled}
                onChange={(e) =>
                  setEditor({ ...editor, enabled: e.target.checked })
                }
              />
              Enable server
            </label>
            <label className="checkbox-label">
              <input
                aria-label="Trust MCP server"
                type="checkbox"
                checked={editor.trusted}
                onChange={(e) =>
                  setEditor({ ...editor, trusted: e.target.checked })
                }
              />
              I reviewed and trust this executable / endpoint and its account
              permissions.
            </label>
            <p className="helper">
              Changes disconnect the server and invalidate previous invocation
              approvals. Never place credentials in URLs or arguments. This
              adapter permits manually approved read-only, closed-world tools
              only.
            </p>
            <Button
              className="button primary"
              disabled={busy}
              disabledReason={"An operation is in progress"}
            >
              Save server
            </Button>
          </form>
        )}
      </Modal>
      <Modal
        title="Approve MCP invocation"
        open={!!preview}
        onClose={() => setPreview(null)}
      >
        {preview && (
          <>
            <p className="helper">{preview.warning}</p>
            <pre className="technical-output">
              {JSON.stringify(preview, null, 2)}
            </pre>
            <Button
              className="button primary"
              disabled={busy || !project}
              onClick={() =>
                void perform(async () => {
                  await request("mcp_invoke", {
                    approval: {
                      server_id: preview.server_id,
                      tool: preview.tool,
                      input: preview.input,
                      project_id: project,
                      fingerprint: preview.fingerprint,
                      approved: true,
                    },
                  });
                  setPreview(null);
                  await load();
                }, "Approved MCP job queued; results appear in history")
              }
              disabledReason={
                busy ? "An operation is in progress" : "Select a project first"
              }
            >
              Approve and queue
            </Button>
          </>
        )}
      </Modal>
    </>
  );
}
