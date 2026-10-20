import { useEffect, useState } from "react";
import type { ExtensionEntry, ExtensionPlan } from "../extensions";
import type { AnalysisJob } from "../research";
import { request, errorText } from "../api";
import { Button, Checkbox, Dropdown, Toolbar } from "./controls";
import { Modal, StatusIndicator } from "./ui";

export default function ManagedMcpDetails({
  entry,
  refresh,
  onOpenMcp,
  onOpenJobs,
}: {
  entry: ExtensionEntry;
  refresh: () => Promise<void>;
  onOpenMcp?: () => void;
  onOpenJobs: () => void;
}) {
  const [plan, setPlan] = useState<ExtensionPlan>();
  const [version, setVersion] = useState(entry.available_version);
  const [managed, setManaged] = useState(false);
  const [consent, setConsent] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [permissions, setPermissions] = useState(
    entry.managed?.allowed_tools ?? [],
  );
  const [history, setHistory] = useState<AnalysisJob[]>([]);
  const m = entry.managed!;
  useEffect(() => {
    setPermissions(entry.managed?.allowed_tools ?? []);
  }, [entry.managed?.allowed_tools]);
  useEffect(() => {
    void request<AnalysisJob[]>("managed_mcp_history", {
      id: entry.manifest.id,
    })
      .then(setHistory)
      .catch((e) => setError(errorText(e)));
  }, [entry.manifest.id, m.active_job?.status]);
  useEffect(() => {
    if (!m.active_job) return;
    const timer = setInterval(() => {
      void refresh().catch((e) => setError(errorText(e)));
    }, 1000);
    return () => clearInterval(timer);
  }, [m.active_job, refresh]);
  const run = async (action: () => Promise<void>) => {
    setBusy(true);
    setError("");
    setNotice("");
    try {
      await action();
    } catch (e) {
      setError(errorText(e));
    } finally {
      await refresh().catch((e) => setError(errorText(e)));
      setBusy(false);
    }
  };
  const review = () =>
    run(async () => {
      setPlan(
        await request<ExtensionPlan>("managed_mcp_plan", {
          id: entry.manifest.id,
          version,
          managed_runtime: managed,
        }),
      );
      setConsent(false);
    });
  const installed = !!entry.installed && entry.installed.status !== "failed";
  return (
    <section
      className="panel extension-details"
      aria-label="Managed MCP details"
    >
      <div className="panel-heading">
        <h2>{entry.manifest.name}</h2>
        <StatusIndicator
          label={entry.status.replaceAll("_", " ")}
          tone={entry.status === "ready" ? "good" : "neutral"}
        />
      </div>
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
      <p>{entry.manifest.description}</p>
      <Toolbar label="Managed MCP actions">
        {m.active_job ? (
          <Button
            disabled={busy}
            disabledReason="An operation is in progress"
            onClick={() =>
              void run(async () => {
                await request("job_cancel", { id: m.active_job!.id });
              })
            }
          >
            Cancel installation
          </Button>
        ) : (
          <>
            {!installed || entry.update_available ? (
              <Button
                variant="primary"
                disabled={busy || !entry.supported}
                disabledReason={
                  busy
                    ? "An operation is in progress"
                    : "This package supports Windows/Linux x64"
                }
                onClick={() => void review()}
              >
                {installed ? "Review update" : "Install MCP"}
              </Button>
            ) : (
              <Button
                variant="primary"
                disabled={!onOpenMcp}
                disabledReason="Open MCP Hub through the application navigation"
                onClick={onOpenMcp}
              >
                Open MCP Hub
              </Button>
            )}
            {installed && (
              <Dropdown
                label="Manage MCP"
                actions={[
                  {
                    label: entry.installed?.enabled
                      ? "Disable MCP"
                      : "Enable MCP",
                    disabledReason: busy
                      ? "An operation is in progress"
                      : undefined,
                    run: () =>
                      void run(async () => {
                        await request("managed_mcp_enable", {
                          enabled: !entry.installed?.enabled,
                        });
                      }),
                  },
                  {
                    label: "Run MCP health check",
                    disabledReason: busy
                      ? "An operation is in progress"
                      : !entry.installed?.enabled
                        ? "Enable the extension to negotiate MCP"
                        : undefined,
                    run: () =>
                      void run(async () => {
                        await request("extensions_doctor", {
                          id: entry.manifest.id,
                        });
                        setNotice(
                          "Package integrity, MCP initialize and tool discovery passed.",
                        );
                      }),
                  },
                  {
                    label: "Review version / reinstall",
                    disabledReason: busy
                      ? "An operation is in progress"
                      : undefined,
                    run: () => void review(),
                  },
                  {
                    label: "Uninstall MCP",
                    disabledReason: busy
                      ? "An operation is in progress"
                      : undefined,
                    run: () =>
                      void run(async () => {
                        await request("managed_mcp_remove");
                        setNotice(
                          "Owned package removed; shared runtime, cache and research data retained.",
                        );
                      }),
                  },
                ]}
              />
            )}
          </>
        )}
        <Button variant="ghost" onClick={onOpenJobs}>
          Open jobs
        </Button>
      </Toolbar>
      <dl className="extension-metadata">
        <dt>Publisher</dt>
        <dd>{entry.manifest.publisher}</dd>
        <dt>Trust</dt>
        <dd>Reviewed package lock; third-party executable code</dd>
        <dt>Version</dt>
        <dd>
          {entry.installed?.version ?? "Not installed"} · Available{" "}
          {entry.available_version}
        </dd>
        <dt>Platforms</dt>
        <dd>
          {entry.manifest.operating_systems.join(", ")} /{" "}
          {entry.manifest.architectures.join(", ")}
        </dd>
        <dt>Runtime</dt>
        <dd>{m.runtime_requirement}</dd>
        <dt>Package</dt>
        <dd>
          <code>{m.package_name}</code>
        </dd>
        <dt>Last health check</dt>
        <dd>{entry.installed?.last_health_check ?? "Not tested"}</dd>
        <dt>Source</dt>
        <dd>
          <a href={entry.source_repository} target="_blank" rel="noreferrer">
            Source repository
          </a>
        </dd>
      </dl>
      {!m.active_job && (
        <details>
          <summary>Installation options</summary>
          <label>
            Reviewed package version
            <select
              aria-label="MCP package version"
              value={version}
              onChange={(e) => {
                setVersion(e.target.value);
                setPlan(undefined);
              }}
            >
              <option>2.0.0</option>
              <option>1.0.0</option>
            </select>
          </label>
          <Checkbox
            label="Use isolated managed Node.js even when a compatible runtime is installed"
            checked={managed}
            onChange={(e) => {
              setManaged(e.target.checked);
              setPlan(undefined);
            }}
          />
        </details>
      )}
      {m.active_job && (
        <div role="status">
          <strong>{m.active_job.status}</strong>
          <p>{m.active_job.logs.at(-1)}</p>
          <code>{m.active_job.id}</code>
        </div>
      )}
      <h3>Tool permissions</h3>
      <p>
        Discovery does not invoke tools. Only read-only, closed-world tools can
        be enabled. Every invocation needs a separate input approval in MCP Hub.
      </p>
      {m.tools.length ? (
        <>
          <div className="mcp-permission-list">
            {m.tools.map((t) => {
              const safe =
                t.annotations?.readOnlyHint === true &&
                t.annotations?.openWorldHint === false &&
                t.annotations?.destructiveHint !== true;
              return (
                <Checkbox
                  key={t.name}
                  label={
                    t.name +
                    (safe
                      ? ""
                      : " — unavailable under current execution policy")
                  }
                  disabled={
                    !safe || busy || !entry.installed?.enabled || !!m.active_job
                  }
                  checked={permissions.includes(t.name)}
                  onChange={(e) =>
                    setPermissions((p) =>
                      e.target.checked
                        ? [...p, t.name]
                        : p.filter((n) => n !== t.name),
                    )
                  }
                />
              );
            })}
          </div>
          <Button
            disabled={busy || !entry.installed?.enabled || !!m.active_job}
            disabledReason="Enable the extension and finish installation first"
            onClick={() =>
              void run(async () => {
                await request("managed_mcp_tools", {
                  tools: permissions,
                  approved: true,
                });
                setNotice(
                  "Selected tool permissions saved; invocation still requires exact-input approval.",
                );
              })
            }
          >
            Approve selected tool permissions
          </Button>
        </>
      ) : (
        <p>No verified tool discovery yet.</p>
      )}
      <details>
        <summary>Permissions and isolation</summary>
        <ul>
          {entry.manifest.permissions.map((p) => (
            <li key={p}>
              <code>{p}</code>
            </li>
          ))}
        </ul>
        <p>
          The approved server runs at your user privileges. Clearing inherited
          credentials and denying tool calls does not provide an
          operating-system sandbox.
        </p>
      </details>
      {history.length > 0 && (
        <details>
          <summary>Installation history ({history.length})</summary>
          <ul className="extension-history">
            {history.map((j) => (
              <li key={j.id}>
                <code>{j.id}</code>
                <span>{j.status}</span>
                <span>{j.error_code}</span>
              </li>
            ))}
          </ul>
        </details>
      )}
      {plan && (
        <Modal
          open
          title="Approve MCP installation"
          dismissDisabled={busy}
          onClose={() => {
            if (!busy) setPlan(undefined);
          }}
        >
          <div className="extension-install-body managed-install-body">
            <p>{plan.network_access}</p>
            <ol>
              {plan.actions.map((a) => (
                <li key={a}>{a}</li>
              ))}
            </ol>
            <dl className="extension-metadata">
              <dt>Version</dt>
              <dd>{plan.package_plan?.version}</dd>
              <dt>Runtime</dt>
              <dd>
                {plan.package_plan?.runtime.managed
                  ? "Managed portable"
                  : "Existing"}{" "}
                Node.js {plan.package_plan?.runtime.version}
              </dd>
              <dt>Runtime SHA-256</dt>
              <dd>
                <code>{plan.package_plan?.runtime.sha256}</code>
              </dd>
              <dt>Free space required</dt>
              <dd>
                {Math.ceil(
                  (plan.package_plan?.minimum_free_bytes ?? 0) / 1024 / 1024,
                )}{" "}
                MiB
              </dd>
            </dl>
            <details>
              <summary>
                Pinned dependency graph (
                {plan.package_plan?.dependencies.length})
              </summary>
              <ul>
                {plan.package_plan?.dependencies.map((d) => (
                  <li key={d.path}>
                    <code>
                      {d.path}@{d.version}
                    </code>
                    <small>{d.source}</small>
                    <code>{d.integrity}</code>
                  </li>
                ))}
              </ul>
            </details>
            <Checkbox
              label="I approve this exact dependency plan and user-level MCP execution"
              checked={consent}
              onChange={(e) => setConsent(e.target.checked)}
            />
            <Toolbar label="MCP installation approval">
              <Button
                variant="primary"
                disabled={busy || !consent}
                disabledReason={
                  busy
                    ? "An operation is in progress"
                    : "Review and approve the dependency and process permissions"
                }
                onClick={() =>
                  void run(async () => {
                    const job = await request<AnalysisJob>(
                      "managed_mcp_install",
                      {
                        approval: {
                          id: entry.manifest.id,
                          version: plan.package_plan!.version,
                          managed_runtime: plan.package_plan!.runtime.managed,
                          fingerprint: plan.fingerprint,
                          approved: consent,
                        },
                      },
                    );
                    setPlan(undefined);
                    setNotice(
                      `Installation queued: ${job.id}. Ready requires actual MCP discovery.`,
                    );
                  })
                }
              >
                Approve and install MCP
              </Button>
              <Button
                disabled={busy}
                disabledReason="Wait for the operation to finish"
                onClick={() => setPlan(undefined)}
              >
                Cancel
              </Button>
            </Toolbar>
          </div>
        </Modal>
      )}
    </section>
  );
}
