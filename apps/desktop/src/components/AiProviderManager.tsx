import { Button } from "./controls";
import { useEffect, useState } from "react";
import {
  request,
  errorText,
  providerKinds,
  type Provider,
  type Snapshot,
} from "../api";
import {
  defaultOptions,
  type AiManagerState,
  type AiProfile,
  type ProviderPreset,
  type AgentRoute,
  type GatewayOptions,
} from "../ai";
import { Modal, type Perform } from "./ui";
const autoName = (v: ProviderPreset, model: string) =>
  (
    v.name +
    " - " +
    (v.models.find((m) => m.id === model)?.name ?? model)
  ).slice(0, 90);
const ready = (p: AiProfile) =>
  ["stored", "not_required"].includes(p.credential_state);
export default function AiProviderManager({
  data,
  busy,
  perform,
}: {
  data: Snapshot;
  busy: boolean;
  perform: Perform;
}) {
  const [state, setState] = useState<AiManagerState | null>(null),
    [error, setError] = useState(""),
    [search, setSearch] = useState(""),
    [filter, setFilter] = useState("all");
  const [editor, setEditor] = useState<{
      vendor: ProviderPreset;
      profile?: AiProfile;
    } | null>(null),
    [removing, setRemoving] = useState<AiProfile | null>(null);
  const [routes, setRoutes] = useState<AgentRoute[]>([]),
    [routingEnabled, setRoutingEnabled] = useState(false);
  const receive = (next: AiManagerState) => {
    if (next?.catalog) {
      setState(next);
      setRoutes(next.routing);
      setRoutingEnabled(next.routing.length > 0);
      setError("");
    }
  };
  const reload = async () =>
    receive(await request<AiManagerState>("ai_manager"));
  useEffect(() => {
    let active = true;
    void request<AiManagerState>("ai_manager")
      .then((next) => {
        if (active) receive(next);
      })
      .catch((e) => {
        if (active) setError(errorText(e));
      });
    return () => {
      active = false;
    };
  }, [data.providers, data.settings.default_provider_id]);
  const act = (action: () => Promise<unknown>, message: string) =>
    void perform(async () => {
      await action();
      await reload();
    }, message);
  const catalog = state?.catalog ?? [],
    profiles = state?.profiles ?? [];
  const matches = (s: string) => s.toLowerCase().includes(search.toLowerCase());
  const vendors = catalog.filter(
    (v) =>
      matches(v.name) &&
      (filter === "all" ||
        v.category === filter ||
        (filter === "configured" &&
          profiles.some((p) => p.vendor_id === v.id)) ||
        (filter === "unconfigured" &&
          !profiles.some((p) => p.vendor_id === v.id))),
  );
  const visibleProfiles = profiles.filter((profile) => {
    const vendor = catalog.find((v) => v.id === profile.vendor_id);
    return (
      matches(profile.provider.name + " " + (vendor?.name ?? "")) &&
      filter !== "unconfigured" &&
      (filter === "all" ||
        filter === "configured" ||
        vendor?.category === filter)
    );
  });
  return (
    <section
      className="panel ai-provider-manager"
      id="ai-providers"
      aria-label="AI Providers"
    >
      <div className="panel-heading">
        <div>
          <h2>AI Providers</h2>
          <p className="helper">
            Choose a provider and model, paste your key, then save. Your model
            becomes available in AI Workspace immediately.
          </p>
        </div>
        <Button
          className="button primary"
          disabled={busy || !catalog.length}
          onClick={() => setEditor({ vendor: catalog[0] })}
          disabledReason={
            busy
              ? "An operation is in progress"
              : "The provider catalog has not loaded"
          }
        >
          Add AI Provider
        </Button>
      </div>
      {error && <p role="alert">{error}</p>}
      {!state && !error && <p role="status">Loading provider catalog…</p>}
      <div className="form-grid">
        <label>
          Search providers
          <input
            type="search"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Provider or platform"
          />
        </label>
        <label>
          Provider category
          <select value={filter} onChange={(e) => setFilter(e.target.value)}>
            <option value="all">All providers</option>
            <option value="cloud">Cloud providers</option>
            <option value="aggregator">Inference platforms</option>
            <option value="local">Local providers</option>
            <option value="custom">Custom gateways</option>
            <option value="configured">Configured</option>
            <option value="unconfigured">Not configured</option>
          </select>
        </label>
      </div>
      <h3>Configured models</h3>
      {!profiles.length && (
        <p className="helper">
          No models configured. Add a cloud provider or discover a local model.
        </p>
      )}
      {!!profiles.length && !visibleProfiles.length && (
        <p className="helper">No configured models match this filter.</p>
      )}
      <div className="provider-list">
        {visibleProfiles.map((profile) => {
          const p = profile.provider,
            vendor =
              catalog.find((v) => v.id === profile.vendor_id) ??
              catalog.find((v) => v.id === "custom");
          const status = !p.enabled
            ? "Disabled"
            : !ready(profile)
              ? profile.credential_state === "store_unavailable"
                ? "Credential store unavailable"
                : "API key or custom headers missing"
              : profile.connection?.state === "connected"
                ? "Connected (last test)"
                : profile.connection?.state === "failed"
                  ? "Connection test failed"
                  : "Saved · Not tested";
          return (
            <article className="provider-row" key={p.id}>
              <div className="provider-info">
                <h3 aria-label={p.name}>
                  {p.name}
                  {data.settings.default_provider_id === p.id && (
                    <span className="badge">Default</span>
                  )}
                </h3>
                <p>
                  {vendor?.name} · <code>{p.model}</code>
                </p>
                <p className="helper">
                  {status}
                  {profile.connection &&
                    ` · ${profile.connection.latency_ms} ms`}
                </p>
                {profile.connection && (
                  <p className="helper">
                    {profile.connection.message}
                    {profile.connection.last_success_at &&
                      ` · Last successful test: ${new Date(profile.connection.last_success_at).toLocaleString()}`}
                  </p>
                )}
                <small>
                  {profile.credential_state === "stored"
                    ? "API key stored securely"
                    : profile.credential_state === "not_required"
                      ? "No API key required"
                      : "Credential not ready"}
                </small>
                {vendor &&
                  ((p.base_url !== vendor.base_url &&
                    !vendor.endpoint_template) ||
                    p.kind !== vendor.kind) && (
                    <small>Custom endpoint or protocol override</small>
                  )}
              </div>
              <div className="provider-actions">
                <Button
                  className="text-button"
                  disabled={busy}
                  onClick={() => vendor && setEditor({ vendor, profile })}
                  disabledReason={"An operation is in progress"}
                >
                  Manage
                </Button>
                <Button
                  className="text-button"
                  disabled={busy || !p.enabled || !ready(profile)}
                  onClick={() =>
                    act(
                      () => request("ai_connection_test", { id: p.id }),
                      "Connection test finished; review the result",
                    )
                  }
                  disabledReason={
                    busy
                      ? "An operation is in progress"
                      : !p.enabled
                        ? "Enable this provider first"
                        : "Store the required provider credential first"
                  }
                >
                  Test connection
                </Button>
                <Button
                  className="text-button"
                  disabled={
                    busy ||
                    !p.enabled ||
                    !ready(profile) ||
                    data.settings.default_provider_id === p.id
                  }
                  onClick={() =>
                    act(
                      () =>
                        request("save_settings", {
                          settings: {
                            ...data.settings,
                            default_provider_id: p.id,
                            fallback_provider_id:
                              data.settings.fallback_provider_id === p.id
                                ? null
                                : data.settings.fallback_provider_id,
                          },
                        }),
                      "Default model saved",
                    )
                  }
                  disabledReason={
                    busy
                      ? "An operation is in progress"
                      : !p.enabled
                        ? "Enable this provider first"
                        : !ready(profile)
                          ? "Store the required provider credential first"
                          : "This provider is already the default"
                  }
                >
                  Set default
                </Button>
                <Button
                  className="text-button"
                  disabled={busy}
                  onClick={() =>
                    act(
                      () =>
                        request("save_provider", {
                          provider: { ...p, enabled: !p.enabled },
                        }),
                      p.enabled ? "Profile disabled" : "Profile enabled",
                    )
                  }
                  disabledReason={"An operation is in progress"}
                >
                  {p.enabled ? "Disable" : "Enable"}
                </Button>
                <Button
                  className="text-button danger"
                  disabled={busy}
                  onClick={() => setRemoving(profile)}
                  disabledReason={"An operation is in progress"}
                >
                  Remove
                </Button>
              </div>
            </article>
          );
        })}
      </div>
      <details open={filter !== "all" || !!search}>
        <summary>Browse provider catalog ({vendors.length})</summary>
        <div className="catalog-grid">
          {vendors.map((v) => (
            <button
              type="button"
              className="catalog-choice"
              key={v.id}
              disabled={busy}
              onClick={() => setEditor({ vendor: v })}
            >
              <strong>{v.name}</strong>
              <span>
                {v.category === "local"
                  ? "Local inference"
                  : v.kind === "compatible"
                    ? "OpenAI-compatible API"
                    : providerKinds[v.kind].label}
              </span>
              <small>
                {v.models.length
                  ? `${v.models.length} built-in model presets`
                  : "Discover models from your server"}
              </small>
            </button>
          ))}
        </div>
      </details>
      {!!profiles.length && (
        <details className="agent-routing">
          <summary>Agent model routing</summary>
          <p className="helper">
            Current agents request text-based structured plans. Native tool
            calling or vision is not assumed. Evidence transmission still
            requires review of the selected destination.
          </p>
          <label className="inline-check">
            <input
              type="checkbox"
              checked={!routingEnabled}
              onChange={(e) => {
                if (e.target.checked) {
                  setRoutes([]);
                  setRoutingEnabled(false);
                  act(
                    () => request("ai_routing_save", { routes: [] }),
                    "All agents use the default model",
                  );
                } else setRoutingEnabled(true);
              }}
            />
            Use Default Model for All Agents
          </label>
          <div className="form-grid">
            {state?.agents.map((agent) => (
              <label key={agent}>
                {agent.replaceAll("_", " ")} agent
                <select
                  aria-label={`${agent} agent model`}
                  disabled={!routingEnabled || busy}
                  value={
                    routes.find((r) => r.agent_id === agent)?.provider_id ?? ""
                  }
                  onChange={(e) => {
                    const id = e.target.value;
                    setRoutes((old) => [
                      ...old.filter((r) => r.agent_id !== agent),
                      ...(id ? [{ agent_id: agent, provider_id: id }] : []),
                    ]);
                  }}
                >
                  <option value="">Use default model</option>
                  {profiles
                    .filter((p) => p.provider.enabled && ready(p))
                    .map((p) => (
                      <option key={p.provider.id} value={p.provider.id}>
                        {p.provider.name} · {p.provider.model}
                      </option>
                    ))}
                </select>
              </label>
            ))}
          </div>
          <Button
            className="button secondary"
            disabled={busy}
            onClick={() =>
              act(
                () => request("ai_routing_save", { routes }),
                "Agent routing saved",
              )
            }
            disabledReason={"An operation is in progress"}
          >
            Save agent routing
          </Button>
        </details>
      )}
      {editor && (
        <ProviderSetup
          key={editor.profile?.provider.id ?? editor.vendor.id}
          initial={editor}
          catalog={catalog}
          busy={busy}
          defaultOn={!data.settings.default_provider_id}
          perform={perform}
          close={() => setEditor(null)}
          reload={reload}
        />
      )}
      <Modal
        open={!!removing}
        title="Remove AI model profile"
        dismissDisabled={busy}
        onClose={() => setRemoving(null)}
      >
        {removing && (
          <>
            <p>
              Remove {removing.provider.name} and its stored API key and
              headers? Projects and evidence are preserved.
            </p>
            <Button
              className="button danger"
              disabled={busy}
              onClick={() =>
                act(async () => {
                  await request("delete_provider", {
                    id: removing.provider.id,
                  });
                  setRemoving(null);
                }, "Provider removed")
              }
              disabledReason={"An operation is in progress"}
            >
              Confirm removal
            </Button>
          </>
        )}
      </Modal>
    </section>
  );
}
function ProviderSetup({
  initial,
  catalog,
  busy,
  defaultOn,
  perform,
  close,
  reload,
}: {
  initial: { vendor: ProviderPreset; profile?: AiProfile };
  catalog: ProviderPreset[];
  busy: boolean;
  defaultOn: boolean;
  perform: Perform;
  close: () => void;
  reload: () => Promise<void>;
}) {
  const [vendor, setVendor] = useState(initial.vendor);
  const [provider, setProvider] = useState<Provider>(
    initial.profile?.provider ?? {
      id: crypto.randomUUID(),
      name: autoName(initial.vendor, initial.vendor.models[0]?.id ?? ""),
      kind: initial.vendor.kind,
      base_url: initial.vendor.base_url,
      model: initial.vendor.models[0]?.id ?? "",
      enabled: true,
      timeout_secs: 120,
      max_output_tokens: 4096,
      requires_key: initial.vendor.requires_key,
    },
  );
  const [options, setOptions] = useState<GatewayOptions>(
    initial.profile?.options ?? defaultOptions,
  );
  const [key, setKey] = useState(""),
    [showKey, setShowKey] = useState(false),
    [makeDefault, setMakeDefault] = useState(defaultOn),
    [error, setError] = useState(""),
    [refreshing, setRefreshing] = useState(false);
  const [live, setLive] = useState(initial.profile?.models ?? []),
    [manual, setManual] = useState(
      !initial.vendor.models.some(
        (m) => m.id === initial.profile?.provider.model,
      ) && !!initial.profile,
    );
  const [workspace, setWorkspace] = useState(
    initial.profile?.provider.base_url.match(
      /^https:\/\/([a-zA-Z0-9-]+)\.ap-southeast-1\.maas/,
    )?.[1] ?? "",
  );
  const [replaceHeaders, setReplaceHeaders] = useState(false),
    [headers, setHeaders] = useState("");
  const update = (patch: Partial<Provider>) =>
    setProvider((p) => ({ ...p, ...patch }));
  const changeModel = (model: string) =>
    update({
      model,
      ...(!initial.profile && provider.name === autoName(vendor, provider.model)
        ? { name: autoName(vendor, model) }
        : {}),
    });
  const choose = (id: string) => {
    const v = catalog.find((v) => v.id === id)!;
    setVendor(v);
    setProvider((p) => ({
      ...p,
      name: autoName(v, v.models[0]?.id ?? ""),
      kind: v.kind,
      base_url: v.base_url,
      model: v.models[0]?.id ?? "",
      requires_key: v.requires_key,
    }));
    setOptions(defaultOptions);
    setLive([]);
    setKey("");
    setShowKey(false);
    setWorkspace("");
    setManual(false);
    setReplaceHeaders(false);
    setHeaders("");
    setError("");
  };
  const refresh = async () => {
    setError("");
    setRefreshing(true);
    try {
      const unchanged =
        initial.profile &&
        JSON.stringify(provider) === JSON.stringify(initial.profile.provider) &&
        JSON.stringify(options) === JSON.stringify(initial.profile.options) &&
        !key;
      const models = unchanged
        ? await request<string[]>("ai_models_refresh", { id: provider.id })
        : await request<string[]>("ai_models_discover", {
            provider: { ...provider, model: provider.model || "discovery" },
            key: key || null,
            options,
          });
      setLive(models);
      if (!provider.model && models.length) changeModel(models[0]);
    } catch (e) {
      setError(errorText(e));
    } finally {
      setRefreshing(false);
    }
  };
  const hasKey =
    initial.profile?.key_configured ??
    initial.profile?.credential_state === "stored";
  const selected = vendor.models.find((m) => m.id === provider.model),
    presets = new Set(vendor.models.map((m) => m.id));
  return (
    <Modal
      open
      title={initial.profile ? "Manage AI Provider" : "Add AI Provider"}
      dismissDisabled={busy || refreshing}
      onClose={close}
    >
      <form
        className="form provider-setup"
        onSubmit={(e) => {
          e.preventDefault();
          setError("");
          void perform(async () => {
            try {
              let values: Record<string, string> | null = null;
              if (replaceHeaders) {
                const parsed: unknown = JSON.parse(headers || "{}");
                if (
                  !parsed ||
                  typeof parsed !== "object" ||
                  Array.isArray(parsed) ||
                  Object.values(parsed).some((v) => typeof v !== "string")
                )
                  throw new Error(
                    "Custom headers must be a JSON object with string values",
                  );
                values = parsed as Record<string, string>;
              }
              await request("ai_profile_save", {
                provider,
                vendor_id: vendor.id,
                key: key || null,
                make_default: makeDefault,
                options,
                headers: values,
              });
              setKey("");
              setHeaders("");
              await reload();
              close();
            } catch (e) {
              setError(errorText(e));
              throw e;
            }
          }, "AI provider saved securely");
        }}
      >
        <label>
          Select Provider
          <select
            aria-label="Select Provider"
            value={vendor.id}
            disabled={!!initial.profile || busy || refreshing}
            onChange={(e) => choose(e.target.value)}
          >
            {catalog.map((v) => (
              <option key={v.id} value={v.id}>
                {v.name}
              </option>
            ))}
          </select>
        </label>
        {vendor.endpoint_template && (
          <label>
            Model Studio workspace ID (Singapore)
            <input
              aria-label="Model Studio workspace ID"
              required
              pattern="[a-zA-Z0-9-]+"
              maxLength={100}
              value={workspace}
              onChange={(e) => {
                setWorkspace(e.target.value);
                update({
                  base_url: vendor.endpoint_template!.replace(
                    "{workspace}",
                    e.target.value.trim(),
                  ),
                });
              }}
            />
          </label>
        )}
        <label>
          Select Model
          <select
            aria-label="Select Model"
            value={provider.model}
            disabled={busy || refreshing}
            onChange={(e) => {
              changeModel(e.target.value);
              setManual(false);
            }}
          >
            <option value="">Choose a model</option>
            {vendor.models.length > 0 && (
              <optgroup label="Built-in presets · official documentation">
                {vendor.models.map((m) => (
                  <option key={m.id} value={m.id}>
                    {m.name}
                    {m.deprecated ? " (deprecated)" : ""}
                  </option>
                ))}
              </optgroup>
            )}
            {live.length > 0 && (
              <optgroup label="Discovered from provider">
                {live
                  .filter((id) => !presets.has(id))
                  .map((id) => (
                    <option key={id} value={id}>
                      {id}
                    </option>
                  ))}
              </optgroup>
            )}
            {provider.model &&
              !presets.has(provider.model) &&
              !live.includes(provider.model) && (
                <option value={provider.model}>
                  {provider.model} (saved / custom)
                </option>
              )}
          </select>
        </label>
        <div className="form-actions">
          <Button
            type="button"
            className="button secondary"
            disabled={busy || refreshing || !vendor.discovery || replaceHeaders}
            onClick={() => void refresh()}
            disabledReason={
              busy
                ? "An operation is in progress"
                : refreshing
                  ? "Model discovery is in progress"
                  : !vendor.discovery
                    ? "This provider has no supported model discovery adapter"
                    : "Save header credential changes before discovering models"
            }
          >
            {refreshing ? "Discovering models…" : "Refresh Models"}
          </Button>
          <span className="helper">
            {replaceHeaders
              ? "Save custom headers before refreshing models."
              : vendor.discovery
                ? "Contacts only your configured endpoint."
                : "Curated catalog; live discovery is not documented for this profile."}
          </span>
        </div>
        {selected && (
          <p className="helper">
            {selected.description}
            {selected.capabilities.context_window
              ? ` Context: ${selected.capabilities.context_window.toLocaleString()} tokens.`
              : ""}{" "}
            Unknown capabilities are not assumed.
          </p>
        )}
        {provider.requires_key && (
          <>
            <label>
              API key
              <input
                aria-label="API key"
                type={showKey ? "text" : "password"}
                autoComplete="new-password"
                spellCheck={false}
                maxLength={8192}
                disabled={busy || refreshing}
                required={!initial.profile || !hasKey}
                value={key}
                onChange={(e) => setKey(e.target.value)}
                placeholder={
                  hasKey
                    ? "Stored securely · leave blank to keep"
                    : "Paste your API key"
                }
              />
            </label>
            <label className="inline-check">
              <input
                type="checkbox"
                checked={showKey}
                onChange={(e) => setShowKey(e.target.checked)}
              />
              Show entered key
            </label>
          </>
        )}
        <p className="helper">
          {vendor.setup_note} Saving does not contact the model. Test connection
          sends a short inference check and may incur a small provider charge.
        </p>
        {vendor.category === "custom" && (
          <label>
            Gateway base URL
            <input
              aria-label="Gateway base URL"
              required
              type="url"
              value={provider.base_url}
              onChange={(e) => update({ base_url: e.target.value })}
            />
          </label>
        )}
        <details open={vendor.category === "custom" || undefined}>
          <summary>Advanced Settings</summary>
          <div className="form-grid">
            <label>
              Configuration name
              <input
                required
                maxLength={100}
                value={provider.name}
                onChange={(e) => update({ name: e.target.value })}
              />
            </label>
            {vendor.category !== "custom" && (
              <label>
                Gateway base URL
                <input
                  aria-label="Gateway base URL"
                  required
                  type="url"
                  value={provider.base_url}
                  onChange={(e) => update({ base_url: e.target.value })}
                />
              </label>
            )}
            <label>
              Provider protocol
              <select
                value={provider.kind}
                onChange={(e) =>
                  update({ kind: e.target.value as Provider["kind"] })
                }
              >
                {Object.entries(providerKinds).map(([id, p]) => (
                  <option key={id} value={id}>
                    {p.label}
                  </option>
                ))}
              </select>
            </label>
            <label className="inline-check">
              <input
                type="checkbox"
                checked={manual}
                onChange={(e) => setManual(e.target.checked)}
              />
              Enter custom model ID
            </label>
            {manual && (
              <label>
                Model ID
                <input
                  required
                  maxLength={200}
                  value={provider.model}
                  onChange={(e) => changeModel(e.target.value)}
                />
              </label>
            )}
            <label>
              Timeout in seconds
              <input
                type="number"
                required
                min={1}
                max={300}
                value={provider.timeout_secs}
                onChange={(e) =>
                  update({ timeout_secs: Number(e.target.value) })
                }
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
            <label>
              Authentication header
              <input
                value={options.auth_header ?? ""}
                maxLength={100}
                placeholder="Automatic for this protocol"
                onChange={(e) =>
                  setOptions({
                    ...options,
                    auth_header: e.target.value || null,
                  })
                }
              />
            </label>
            <label>
              Authentication prefix
              <select
                aria-label="Authentication prefix"
                disabled={!options.auth_header}
                value={options.auth_prefix}
                onChange={(e) =>
                  setOptions({ ...options, auth_prefix: e.target.value })
                }
              >
                <option value="Bearer ">Bearer</option>
                <option value="Token ">Token</option>
                <option value="">No prefix</option>
              </select>
            </label>
            <label>
              Proxy URL
              <input
                type="url"
                placeholder="HTTPS or loopback proxy; no embedded credentials"
                value={options.proxy_url ?? ""}
                onChange={(e) =>
                  setOptions({ ...options, proxy_url: e.target.value || null })
                }
              />
            </label>
            <label>
              Model discovery path
              <input
                placeholder="models"
                value={options.discovery_path ?? ""}
                maxLength={200}
                onChange={(e) =>
                  setOptions({
                    ...options,
                    discovery_path: e.target.value || null,
                  })
                }
              />
            </label>
          </div>
          <label className="inline-check">
            <input
              type="checkbox"
              checked={provider.requires_key}
              disabled={[
                "openai",
                "anthropic",
                "gemini",
                "deepseek",
                "openrouter",
              ].includes(provider.kind)}
              onChange={(e) => update({ requires_key: e.target.checked })}
            />
            Requires secure API key
          </label>
          <label className="inline-check">
            <input
              type="checkbox"
              checked={options.streaming}
              onChange={(e) =>
                setOptions({ ...options, streaming: e.target.checked })
              }
            />
            Enable streaming
          </label>
          <label className="inline-check">
            <input
              type="checkbox"
              checked={replaceHeaders}
              onChange={(e) => setReplaceHeaders(e.target.checked)}
            />
            Replace custom headers
          </label>
          {initial.profile && initial.profile.header_names.length > 0 && (
            <p className="helper">
              Stored header names: {initial.profile.header_names.join(", ")}.
              Values are never displayed.
            </p>
          )}
          {replaceHeaders && (
            <label>
              Custom headers (JSON)
              <textarea
                aria-label="Custom headers (JSON)"
                spellCheck={false}
                value={headers}
                onChange={(e) => setHeaders(e.target.value)}
                placeholder={'{"X-Tenant": "value"}'}
                maxLength={8192}
              />
            </label>
          )}
          <p className="helper">
            Custom header values use secure credential storage. An empty object
            removes them. Remove stored credentials before changing the gateway
            destination or protocol.
          </p>
        </details>
        {hasKey && (
          <Button
            className="text-button danger"
            type="button"
            disabled={busy || refreshing}
            onClick={() =>
              void perform(async () => {
                await request("remove_key", { id: provider.id });
                setKey("");
                await reload();
                close();
              }, "Stored API key removed")
            }
            disabledReason={
              busy
                ? "An operation is in progress"
                : "Model discovery is in progress"
            }
          >
            Remove stored API key
          </Button>
        )}
        <label className="inline-check">
          <input
            type="checkbox"
            checked={makeDefault}
            onChange={(e) => setMakeDefault(e.target.checked)}
          />
          Use as default model
        </label>
        {error && <p role="alert">{error}</p>}
        <div className="form-actions">
          <Button
            className="button primary"
            disabled={busy || refreshing || !provider.model}
            disabledReason={
              busy
                ? "An operation is in progress"
                : refreshing
                  ? "Model discovery is in progress"
                  : "Select a model first"
            }
          >
            Save provider
          </Button>
          <Button
            className="button ghost"
            type="button"
            disabled={busy || refreshing}
            onClick={close}
            disabledReason={
              busy
                ? "An operation is in progress"
                : "Model discovery is in progress"
            }
          >
            Cancel
          </Button>
        </div>
      </form>
    </Modal>
  );
}
