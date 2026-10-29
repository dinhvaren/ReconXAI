import { useCallback, useEffect, useState } from "react";
import {
  errorText,
  request,
  installNativeExtension,
  type ToolProgress,
} from "../api";
import type { ExtensionEntry, ExtensionPlan } from "../extensions";
import type { AnalysisJob, ImportedFile } from "../research";
import {
  Button,
  EmptyState,
  LoadingState,
  SearchField,
  Tabs,
  Toolbar,
} from "../components/controls";
import { Heading, Icon, Modal, StatusIndicator } from "../components/ui";
import ManagedMcpDetails from "../components/ManagedMcpDetails";
import RegistryBrowser from "../components/RegistryBrowser";

export default function ExtensionsPage({
  projectId,
  onOpenFiles,
  onOpenJobs,
  onOpenMcp,
  onOpenRecon,
}: {
  projectId?: string;
  onOpenFiles: () => void;
  onOpenJobs: () => void;
  onOpenMcp?: () => void;
  onOpenRecon?: () => void;
}) {
  const [entries, setEntries] = useState<ExtensionEntry[]>([]);
  const [files, setFiles] = useState<ImportedFile[]>([]);
  const [installProgress, setInstallProgress] = useState<ToolProgress | null>(
    null,
  );
  const [history, setHistory] = useState<AnalysisJob[]>([]);
  const [selectedId, setSelected] = useState("");
  const [query, setQuery] = useState("");
  const [view, setView] = useState("discover");
  const [category, setCategory] = useState("");
  const [file, setFile] = useState("");
  const [plan, setPlan] = useState<ExtensionPlan>();
  const [consent, setConsent] = useState(false);
  const [busy, setBusy] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const refresh = useCallback(async () => {
    const catalog = await request<ExtensionEntry[]>("extensions_catalog");
    setEntries(catalog);
    setSelected((current) => current || catalog[0]?.manifest.id || "");
    setFiles(
      projectId
        ? await request<ImportedFile[]>("files_list", { project: projectId })
        : [],
    );
  }, [projectId]);
  useEffect(() => {
    let active = true;
    setLoading(true);
    void refresh()
      .catch((e) => {
        if (active) setError(errorText(e));
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [refresh]);
  const needle = query.toLowerCase();
  const visible = entries
    .filter(
      (e) =>
        `${e.manifest.name} ${e.manifest.description} ${e.manifest.id}`
          .toLowerCase()
          .includes(needle) &&
        (!category ||
          e.manifest.category.toLowerCase() === category.toLowerCase()) &&
        (view !== "installed" || e.installed) &&
        (view !== "updates" || e.update_available),
    )
    .sort((a, b) =>
      view === "recent"
        ? (b.installed?.updated_at ?? "").localeCompare(
            a.installed?.updated_at ?? "",
          )
        : a.manifest.name.localeCompare(b.manifest.name),
    );
  const selected = visible.some((e) => e.manifest.id === selectedId)
    ? selectedId
    : (visible[0]?.manifest.id ?? "");
  const entry = visible.find((e) => e.manifest.id === selected);
  const loadHistory = useCallback(async () => {
    if (selected)
      setHistory(
        await request<AnalysisJob[]>("extensions_history", { id: selected }),
      );
  }, [selected]);
  useEffect(() => {
    void loadHistory().catch((e) => setError(errorText(e)));
  }, [loadHistory]);
  // The shared worker updates the same durable jobs read by CLI. Poll only while
  // there is an actual invocation in progress, and stop when this page unmounts.
  useEffect(() => {
    if (!history.some((j) => ["queued", "running"].includes(j.status))) return;
    const timer = setInterval(
      () => void loadHistory().catch((e) => setError(errorText(e))),
      1000,
    );
    return () => clearInterval(timer);
  }, [history, loadHistory]);
  useEffect(() => {
    setFile("");
  }, [projectId]);
  const run = async (action: () => Promise<void>) => {
    setBusy(true);
    setError("");
    setNotice("");
    try {
      await action();
    } catch (e) {
      setError(errorText(e));
    } finally {
      try {
        await refresh();
        await loadHistory();
      } catch (e) {
        setError(errorText(e));
      }
      setBusy(false);
    }
  };
  const reviewInstall = () =>
    run(async () => {
      setPlan(
        await request<ExtensionPlan>("extensions_plan", { id: selected }),
      );
      setConsent(false);
    });
  return (
    <>
      <Heading
        eyebrow="Integrations"
        title="Extensions"
        description="Install reviewed adapters and manage their shared dependencies."
        action={
          <Button
            disabled={busy}
            disabledReason="An extension operation is in progress"
            onClick={() => void run(refresh)}
          >
            Refresh
          </Button>
        }
      />
      {error && (
        <div role="alert" className="banner error">
          {error}
        </div>
      )}
      {notice && (
        <div role="status" className="banner success">
          {notice}
        </div>
      )}
      <Toolbar label="Extension discovery">
        <label>
          Search extensions
          <SearchField
            aria-label="Search extensions"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search extensions"
          />
        </label>
        <label>
          Category
          <select
            aria-label="Extension category"
            value={category}
            onChange={(e) => setCategory(e.target.value)}
          >
            <option value="">All available categories</option>
            <option value="mcp">MCP</option>
            {[...new Set(entries.map((e) => e.manifest.category))]
              .filter((c) => c.toLowerCase() !== "mcp")
              .map((c) => (
                <option key={c}>{c}</option>
              ))}
          </select>
        </label>
      </Toolbar>
      <Tabs
        label="Extension views"
        value={view}
        onChange={setView}
        items={[
          { id: "discover", label: "Discover" },
          { id: "installed", label: "Installed" },
          { id: "updates", label: "Updates available" },
        ]}
      />
      {category === "mcp" && view === "discover" && (
        <RegistryBrowser search={query} />
      )}
      {loading ? (
        <LoadingState label="Loading extension catalog" />
      ) : (
        <div className="extension-workspace">
          <section
            className="panel extension-list"
            aria-label="Extension results"
          >
            {visible.length ? (
              visible.map((e) => (
                <button
                  key={e.manifest.id}
                  className={`extension-row ${selected === e.manifest.id ? "selected" : ""}`}
                  onClick={() => setSelected(e.manifest.id)}
                  aria-pressed={selected === e.manifest.id}
                >
                  <Icon name="blocks" />
                  <span>
                    <strong>{e.manifest.name}</strong>
                    <small>
                      {e.manifest.publisher} · {e.manifest.version}
                    </small>
                    <span>{e.manifest.description}</span>
                    <small>{e.status.replaceAll("_", " ")}</small>
                  </span>
                </button>
              ))
            ) : (
              <EmptyState
                title="No matching extensions"
                description={
                  view === "updates"
                    ? "No compatible extension update is available in this catalog."
                    : "Change the search or category to view available adapters."
                }
                icon="blocks"
              />
            )}
          </section>
          {entry?.managed ? (
            <ManagedMcpDetails
              key={entry.manifest.id}
              entry={entry}
              refresh={refresh}
              onOpenMcp={onOpenMcp}
              onOpenJobs={onOpenJobs}
            />
          ) : (
            entry && (
              <section
                className="panel extension-details"
                aria-label="Extension details"
              >
                <div className="panel-heading">
                  <h2>{entry.manifest.name}</h2>
                  <StatusIndicator
                    label={entry.status.replaceAll("_", " ")}
                    tone={entry.status === "ready" ? "good" : "neutral"}
                  />
                </div>
                <p>{entry.manifest.description}</p>
                <Toolbar label="Extension actions">
                  {!entry.installed || entry.installed.status === "failed" ? (
                    <Button
                      variant="primary"
                      disabled={busy || !entry.supported}
                      disabledReason={
                        busy
                          ? "An extension operation is in progress"
                          : "No verified dependency for this OS and architecture"
                      }
                      onClick={() => void reviewInstall()}
                    >
                      {entry.installed ? "Review repair" : "Install extension"}
                    </Button>
                  ) : (
                    <>
                      <Button
                        disabled={busy || entry.status === "installing"}
                        disabledReason="Wait for the current installation to finish"
                        onClick={() =>
                          void run(async () => {
                            await request("extensions_enable", {
                              id: selected,
                              enabled: !entry.installed?.enabled,
                            });
                            setNotice(
                              entry.installed?.enabled
                                ? "Extension disabled; its active jobs receive cancellation."
                                : "Extension enabled.",
                            );
                          })
                        }
                      >
                        {entry.installed.enabled
                          ? "Disable extension"
                          : "Enable extension"}
                      </Button>
                      <Button
                        disabled={busy}
                        disabledReason="An extension operation is in progress"
                        onClick={() =>
                          void run(async () => {
                            await request("extensions_doctor", {
                              id: selected,
                            });
                            setNotice(
                              selected === "reconx.yara-signatures"
                                ? "Executable integrity, version and real benign signature scan passed."
                                : "Verified binary and adapter health checks passed.",
                            );
                          })
                        }
                      >
                        Run health check
                      </Button>
                    </>
                  )}
                  {entry.update_available && (
                    <Button
                      disabled={busy}
                      disabledReason="An extension operation is in progress"
                      onClick={() => void reviewInstall()}
                    >
                      Review update
                    </Button>
                  )}
                  {entry.installed && (
                    <Button
                      variant="destructive"
                      disabled={busy || entry.status === "installing"}
                      disabledReason="Wait for the current installation to finish"
                      onClick={() =>
                        void run(async () => {
                          await request("extensions_remove", { id: selected });
                          setNotice(
                            "Extension removed. Shared tools and evidence were retained.",
                          );
                        })
                      }
                    >
                      Uninstall extension
                    </Button>
                  )}
                </Toolbar>
                <dl className="extension-metadata">
                  <dt>Publisher</dt>
                  <dd>
                    {entry.manifest.publisher} · First-party compiled adapter
                  </dd>
                  <dt>Version</dt>
                  <dd>
                    {entry.manifest.version} · ReconX{" "}
                    {entry.manifest.compatible_reconx}
                  </dd>
                  <dt>Platforms</dt>
                  <dd>
                    {entry.manifest.operating_systems.join(", ")} ·{" "}
                    {entry.manifest.architectures.join(", ")}
                  </dd>
                  <dt>Runtime</dt>
                  <dd>{entry.manifest.required_runtime}</dd>
                  <dt>Source</dt>
                  <dd>
                    <a
                      href={entry.source_repository}
                      target="_blank"
                      rel="noreferrer"
                    >
                      Source repository
                    </a>
                  </dd>
                  <dt>Download size</dt>
                  <dd>
                    {entry.installation_size === null
                      ? "Not provided by the pinned manifest"
                      : entry.installation_size}
                  </dd>
                  <dt>Last health check</dt>
                  <dd>{entry.installed?.last_health_check ?? "Not checked"}</dd>
                </dl>
                <h3>Permissions</h3>
                <ul>
                  {entry.manifest.permissions.map((p) => (
                    <li key={p}>
                      <code>{p}</code>
                    </li>
                  ))}
                </ul>
                <h3>Shared dependencies</h3>
                {entry.dependencies.map((d) => (
                  <div className="extension-dependency" key={d.manifest.id}>
                    <strong>
                      {d.manifest.name} {d.manifest.version}
                    </strong>
                    <span>{d.status.replaceAll("_", " ")}</span>
                    <code>SHA-256 {d.manifest.sha256}</code>
                  </div>
                ))}
                {["httpx", "subfinder"].includes(entry.manifest.id) ? (
                  <div>
                    <h3>Target-bound recon</h3>
                    <p>
                      Open Reconnaissance or Start AI Research to review scope,
                      passive-source disclosure and active probe budgets.
                    </p>
                    <Button
                      variant="primary"
                      disabled={!onOpenRecon}
                      disabledReason="Open the target-bound Recon workspace from the main navigation"
                      onClick={onOpenRecon}
                    >
                      Open Reconnaissance
                    </Button>
                  </div>
                ) : (
                  <>
                    <h3>Run file signatures</h3>
                    <label>
                      Imported file
                      <select
                        aria-label="Extension input file"
                        value={file}
                        onChange={(e) => setFile(e.target.value)}
                      >
                        <option value="">Select an imported file</option>
                        {files.map((f) => (
                          <option key={f.id} value={f.id}>
                            {f.name}
                          </option>
                        ))}
                      </select>
                    </label>
                    <Toolbar label="Extension invocation">
                      <Button
                        variant="primary"
                        disabled={busy || entry.status !== "ready" || !file}
                        disabledReason={
                          busy
                            ? "An extension operation is in progress"
                            : entry.status !== "ready"
                              ? "Install, health-check and enable this extension"
                              : "Select an explicitly imported project file"
                        }
                        onClick={() =>
                          void run(async () => {
                            const job = await request<AnalysisJob>(
                              "extensions_invoke",
                              { id: selected, file_id: file },
                            );
                            setNotice(`Signature job queued: ${job.id}`);
                          })
                        }
                      >
                        Run signatures
                      </Button>
                      <Button variant="ghost" onClick={onOpenFiles}>
                        Import a file
                      </Button>
                      <Button variant="ghost" onClick={onOpenJobs}>
                        Open jobs
                      </Button>
                    </Toolbar>
                  </>
                )}
                {history.length > 0 && (
                  <>
                    <h3>Invocation history</h3>
                    <ul className="extension-history">
                      {history.map((j) => (
                        <li key={j.id}>
                          <code>{j.id}</code>
                          <span>{j.status}</span>
                          {j.error_code && <span>{j.error_code}</span>}
                        </li>
                      ))}
                    </ul>
                  </>
                )}
                <details>
                  <summary>Integration boundaries</summary>
                  <p>
                    Native adapters use compiled, fixed tool arguments. The MCP
                    category also offers reviewed npm packages and untrusted
                    Registry discovery. General community installation and OS
                    sandboxing remain unavailable. Subprocesses use fixed
                    arguments, cleared credentials, pinned executable hashes,
                    bounded output and timeouts; this is not an operating-system
                    sandbox. Uninstall retains shared tools and all project
                    data.
                  </p>
                </details>
              </section>
            )
          )}
        </div>
      )}
      {plan && (
        <Modal
          open
          dismissDisabled={busy}
          title="Review extension installation"
          onClose={() => {
            if (!busy) setPlan(undefined);
          }}
        >
          <div className="extension-install-body">
            <p>
              {plan.extension.manifest.name} {plan.extension.manifest.version}
            </p>
            <ul>
              {plan.actions.map((a) => (
                <li key={a}>{a}</li>
              ))}
            </ul>
            <p>{plan.network_access}</p>
            <p>
              Administrator privileges:{" "}
              {plan.requires_admin ? "Required" : "Not required"}. Installation
              size: {plan.extension.installation_size ?? "Unavailable"}.
            </p>
            <h3>Required permissions</h3>
            <ul>
              {plan.extension.manifest.permissions.map((p) => (
                <li key={p}>
                  <code>{p}</code>
                </li>
              ))}
            </ul>
            {plan.extension.dependencies.map((d) => (
              <details key={d.manifest.id}>
                <summary>
                  {d.manifest.name} {d.manifest.version} — verified download
                </summary>
                <p className="mono">{d.manifest.url}</p>
                <p className="mono">Archive SHA-256: {d.manifest.sha256}</p>
                <p className="mono">
                  Executable SHA-256: {d.manifest.binary_sha256}
                </p>
              </details>
            ))}
            <label className="check">
              <input
                type="checkbox"
                checked={consent}
                disabled={busy}
                onChange={(e) => setConsent(e.target.checked)}
              />
              I approve these dependencies and extension permissions
            </label>
            <Button
              variant="primary"
              disabled={busy || !consent}
              disabledReason={
                busy
                  ? "The approved installation is running"
                  : "Review and approve the installation plan first"
              }
              onClick={() =>
                void run(async () => {
                  try {
                    setInstallProgress(null);
                    await installNativeExtension(
                      {
                        id: plan.extension.manifest.id,
                        fingerprint: plan.fingerprint,
                        approved: true,
                      },
                      setInstallProgress,
                    );
                  } catch (e) {
                    setPlan(undefined);
                    throw e;
                  }
                  setPlan(undefined);
                  setNotice(
                    plan.extension.manifest.id === "reconx.yara-signatures"
                      ? "Extension installed. Integrity, version and actual benign signature scan passed."
                      : "Extension installed. Verified binary and adapter health checks passed.",
                  );
                })
              }
            >
              {busy ? "Installing and checking…" : "Approve and install"}
            </Button>
            {busy && installProgress && (
              <div role="status">
                {installProgress.phase} ·{" "}
                {installProgress.downloaded.toLocaleString()} bytes
                {installProgress.total ? (
                  <progress
                    aria-label="Extension download progress"
                    max={installProgress.total}
                    value={installProgress.downloaded}
                  />
                ) : null}
              </div>
            )}
          </div>
        </Modal>
      )}
    </>
  );
}
