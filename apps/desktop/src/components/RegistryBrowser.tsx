import { useState } from "react";
import { request, errorText } from "../api";
import {
  Button,
  Checkbox,
  EmptyState,
  LoadingState,
  Toolbar,
} from "./controls";
import { StatusIndicator } from "./ui";
export interface RegistryEntry {
  id: string;
  name: string;
  description: string;
  publisher: string;
  version: string;
  repository: string | null;
  packages: Record<string, unknown>[];
  transports: string[];
  status: string;
  trust: string;
  schema_supported: boolean;
  server_json: Record<string, unknown>;
  registry_metadata: unknown;
}
export interface RegistryPage {
  api_version: string;
  entries: RegistryEntry[];
  next_cursor: string | null;
  fetched_at: string;
  expires_at: string;
  cached: boolean;
  stale: boolean;
  warning: string | null;
}
export default function RegistryBrowser({ search }: { search: string }) {
  const [page, setPage] = useState<RegistryPage>();
  const [selected, setSelected] = useState<RegistryEntry>();
  const [versions, setVersions] = useState<RegistryPage>();
  const [busy, setBusy] = useState(false);
  const [offline, setOffline] = useState(false);
  const [runtime, setRuntime] = useState("");
  const [error, setError] = useState("");
  const load = async (cursor?: string, refresh = false) => {
    setBusy(true);
    setError("");
    setVersions(undefined);
    try {
      const value = await request<RegistryPage>("registry_search", {
        query: {
          search,
          cursor: cursor ?? null,
          runtime: runtime || null,
          refresh,
          offline,
        },
      });
      setPage(value);
      setSelected(value.entries[0]);
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };
  const versionHistory = async () => {
    if (!selected) return;
    setBusy(true);
    setError("");
    try {
      setVersions(
        await request<RegistryPage>("registry_versions", {
          id: selected.id,
          offline,
        }),
      );
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <section aria-label="Official MCP registry">
      <Toolbar label="Registry catalog actions">
        <label>
          Package runtime
          <select
            aria-label="Registry runtime"
            value={runtime}
            onChange={(e) => setRuntime(e.target.value)}
          >
            <option value="">All declared runtimes</option>
            {["npm", "pypi", "cargo", "oci", "mcpb", "nuget", "remote"].map(
              (r) => (
                <option key={r}>{r}</option>
              ),
            )}
          </select>
        </label>
        <Checkbox
          checked={offline}
          onChange={(e) => setOffline(e.target.checked)}
          label="Offline cache only"
        />
        <Button
          variant="primary"
          disabled={busy}
          disabledReason="A catalog request is in progress"
          onClick={() => void load()}
        >
          Search MCP Registry
        </Button>
        <Button
          disabled={busy || offline}
          disabledReason={
            busy
              ? "A catalog request is in progress"
              : "Disable offline browsing to refresh"
          }
          onClick={() => void load(undefined, true)}
        >
          Refresh catalog
        </Button>
        <Button
          variant="ghost"
          disabled={busy}
          disabledReason="A catalog request is in progress"
          onClick={async () => {
            setBusy(true);
            try {
              await request("registry_clear_cache");
              setPage(undefined);
              setSelected(undefined);
              setVersions(undefined);
            } catch (e) {
              setError(errorText(e));
            } finally {
              setBusy(false);
            }
          }}
        >
          Clear metadata cache
        </Button>
      </Toolbar>
      <p className="registry-note">
        Search sends only your search text to registry.modelcontextprotocol.io.
        Registry entries are unreviewed metadata; listing does not grant
        installation or tool permissions. Platform compatibility is not
        guaranteed.
      </p>
      {error && (
        <div className="banner error" role="alert">
          {error}
        </div>
      )}
      {page && (
        <p role="status" className="registry-note">
          {page.cached ? "Cached metadata" : "Fetched metadata"} ·{" "}
          {page.api_version} · {page.fetched_at}
          {page.stale ? " · Stale; not a latest-version check" : ""}
          {page.warning ? ` · ${page.warning}` : ""}
        </p>
      )}
      {busy && <LoadingState label="Reading registry metadata" />}
      {!page && !busy && (
        <EmptyState
          title="Search the official MCP Registry"
          description="Choose a runtime or browse all entries. Previously fetched searches can be opened offline."
          icon="plug"
        />
      )}
      {page && (
        <div className="extension-workspace">
          <section
            className="panel extension-list"
            aria-label="Registry results"
          >
            {page.entries.length ? (
              page.entries.map((entry) => (
                <button
                  key={`${entry.id}:${entry.version}`}
                  className={`extension-row ${selected?.id === entry.id ? "selected" : ""}`}
                  onClick={() => {
                    setSelected(entry);
                    setVersions(undefined);
                  }}
                >
                  <span>
                    <strong>{entry.name}</strong>
                    <small>
                      {entry.publisher} · {entry.version}
                    </small>
                    <span>{entry.description}</span>
                    <small>Official registry entry · Unreviewed</small>
                  </span>
                </button>
              ))
            ) : (
              <EmptyState
                title="No matching registry entries"
                description="Runtime filters apply to this page. Continue pagination to inspect further results."
                icon="plug"
              />
            )}
            {page.next_cursor && (
              <Button
                disabled={busy}
                disabledReason="A catalog request is in progress"
                onClick={() => void load(page.next_cursor ?? undefined)}
              >
                Next registry page
              </Button>
            )}
          </section>
          {selected && (
            <section
              className="panel extension-details"
              aria-label="Registry entry details"
            >
              <div className="panel-heading">
                <h2>{selected.name}</h2>
                <StatusIndicator label={selected.status} />
              </div>
              <p>{selected.description}</p>
              <dl className="extension-metadata">
                <dt>Server identifier</dt>
                <dd>
                  <code>{selected.id}</code>
                </dd>
                <dt>Publisher namespace</dt>
                <dd>{selected.publisher}</dd>
                <dt>Version</dt>
                <dd>{selected.version}</dd>
                <dt>Transports</dt>
                <dd>{selected.transports.join(", ") || "Not declared"}</dd>
                <dt>Source</dt>
                <dd>Official registry metadata · Unreviewed</dd>
                <dt>Schema</dt>
                <dd>
                  {selected.schema_supported
                    ? "Recognized 2025-12-11 server.json"
                    : "Unrecognized schema; automatic installation unavailable"}
                </dd>
                <dt>Platform support</dt>
                <dd>Not established by catalog presence</dd>
              </dl>
              <h3>Package metadata</h3>
              {selected.packages.length ? (
                selected.packages.map((p, i) => (
                  <div className="extension-dependency" key={i}>
                    <strong>
                      {String(p.registryType)} · {String(p.identifier)}
                    </strong>
                    <span>Version {String(p.version ?? "Not pinned")}</span>
                    <span>
                      Runtime {String(p.runtimeHint ?? "Not declared")}
                    </span>
                  </div>
                ))
              ) : (
                <p>Remote-only or no installable package declared.</p>
              )}
              <p className="registry-note">
                Manual Setup Required for unreviewed packages. Automatic package
                installation will only be offered by a reviewed installer
                adapter; metadata descriptions are never executed.
              </p>
              <Button
                disabled={busy}
                disabledReason="A catalog request is in progress"
                onClick={() => void versionHistory()}
              >
                Load version history
              </Button>
              {versions && (
                <div aria-label="Registry version history">
                  <p>
                    {versions.cached ? "Cached" : "Fetched"} versions
                    {versions.stale ? " · stale" : ""}
                  </p>
                  <ul>
                    {versions.entries.map((v) => (
                      <li key={v.version}>
                        {v.version} · {v.status}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
              <details>
                <summary>Original server.json</summary>
                <pre className="registry-json">
                  {JSON.stringify(selected.server_json, null, 2)}
                </pre>
              </details>
            </section>
          )}
        </div>
      )}
    </section>
  );
}
