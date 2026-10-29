import { useEffect, useRef, useState } from "react";
import { request, errorText } from "../api";
import { Button } from "./controls";
import { Modal } from "./ui";
type Plan = {
  config: {
    target_id: string;
    tool: string;
    max_assets: number;
    timeout_secs: number;
  };
  target: string;
  version: string;
  fingerprint: string;
  permissions: string[];
  limits: string[];
};
type Extension = { manifest: { id: string; name: string }; status: string };
export default function ReconToolControls({
  target,
  navigate,
  onQueued,
}: {
  target: string;
  navigate: (page: string) => void;
  onQueued: () => void;
}) {
  const [tools, setTools] = useState<Extension[]>([]),
    [tool, setTool] = useState("httpx"),
    [budget, setBudget] = useState(4),
    [plan, setPlan] = useState<Plan | null>(null),
    [approved, setApproved] = useState(false),
    [busy, setBusy] = useState(false),
    [error, setError] = useState("");
  const active = useRef(target);
  active.current = target;
  useEffect(() => {
    let live = true;
    setPlan(null);
    setApproved(false);
    const load = () =>
      request<Extension[]>("extensions_catalog")
        .then((x) => {
          if (live)
            setTools(
              x.filter((e) => ["httpx", "subfinder"].includes(e.manifest.id)),
            );
        })
        .catch((e) => {
          if (live) setError(errorText(e));
        });
    void load();
    const timer = setInterval(() => void load(), 5000);
    return () => {
      live = false;
      clearInterval(timer);
    };
  }, [target]);
  const ready = tools.find((e) => e.manifest.id === tool)?.status === "ready";
  async function preview() {
    const before = target;
    setBusy(true);
    setError("");
    try {
      const p = await request<Plan>("recon_tool_plan", {
        config: {
          target_id: target,
          tool,
          max_assets: budget,
          timeout_secs: 10,
          technology: true,
          sources: ["crtsh"],
        },
      });
      if (active.current === before) {
        setPlan(p);
        setApproved(false);
      }
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  }
  return (
    <section
      className="panel records-panel"
      aria-label="Recon tool availability"
    >
      <h2>Available tools</h2>
      <p>
        {tools.length
          ? tools
              .map(
                (e) => `${e.manifest.name}: ${e.status.replaceAll("_", " ")}`,
              )
              .join(" · ")
          : "Checking managed tools…"}
      </p>
      <div className="inline-actions">
        <Button variant="ghost" onClick={() => navigate("Extensions")}>
          Manage recon dependencies
        </Button>
      </div>
      {error && <p role="alert">{error}</p>}
      <details>
        <summary>Manual managed-tool execution</summary>
        <div className="records-toolbar">
          <label>
            Adapter
            <select
              aria-label="Recon adapter"
              value={tool}
              onChange={(e) => setTool(e.target.value)}
            >
              <option value="httpx">HTTPX</option>
              <option value="subfinder">Subfinder (passive)</option>
            </select>
          </label>
          <label>
            Maximum HTTP URLs
            <input
              aria-label="Recon asset budget"
              type="number"
              min={1}
              max={8}
              value={budget}
              onChange={(e) => setBudget(Number(e.target.value))}
            />
          </label>
          <Button
            disabled={!target || !ready || busy}
            disabledReason={
              !target
                ? "Select an authorized target"
                : !ready
                  ? "Install and health-check this extension"
                  : "An operation is in progress"
            }
            onClick={() => void preview()}
          >
            Review tool plan
          </Button>
        </div>
      </details>
      <Modal
        title="Approve recon tool plan"
        open={!!plan}
        onClose={() => {
          if (!busy) setPlan(null);
        }}
        dismissDisabled={busy}
      >
        {plan && (
          <div className="records-form">
            <p>
              <strong>
                {plan.config.tool} {plan.version}
              </strong>{" "}
              · <code>{plan.target}</code>
            </p>
            <ul>
              {[...plan.permissions, ...plan.limits].map((x) => (
                <li key={x}>{x}</li>
              ))}
            </ul>
            <pre>{JSON.stringify(plan.config, null, 2)}</pre>
            <label className="checkbox">
              <input
                type="checkbox"
                checked={approved}
                onChange={(e) => setApproved(e.target.checked)}
              />
              I approve this destination, passive data sharing and resource
              limits.
            </label>
            {error && <p role="alert">{error}</p>}
            <Button
              variant="primary"
              disabled={!approved || busy}
              disabledReason="Review and approve the exact plan"
              onClick={() => {
                setBusy(true);
                void request("recon_tool_run", { plan, approved })
                  .then(() => {
                    setPlan(null);
                    onQueued();
                  })
                  .catch((e) => setError(errorText(e)))
                  .finally(() => setBusy(false));
              }}
            >
              Run approved recon
            </Button>
          </div>
        )}
      </Modal>
    </section>
  );
}
