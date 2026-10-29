import { useEffect, useRef, useState } from "react";
import { request, errorText } from "../api";
type Asset = {
  id: string;
  target_id: string | null;
  value: string;
  kind: string;
  last_seen: string;
  job_id: string;
  facts: Record<string, unknown>;
};
export default function ReconAssetInventory({
  project,
  target,
  revision = "",
}: {
  project: string;
  target?: string | null;
  revision?: string;
}) {
  const [assets, setAssets] = useState<Asset[]>([]),
    [error, setError] = useState("");
  const generation = useRef(0);
  useEffect(() => {
    const ticket = ++generation.current;
    setAssets([]);
    if (!project) return;
    let active = true;
    let reads = 0;
    const load = async () => {
      const read = ++reads;
      try {
        const rows = await request<Asset[]>("assets_list", { project });
        if (active && ticket === generation.current && read === reads) {
          setAssets(rows.filter((a) => !target || a.target_id === target));
          setError("");
        }
      } catch (e) {
        if (active) setError(errorText(e));
      }
    };
    void load();
    const timer = setInterval(() => void load(), 2000);
    return () => {
      active = false;
      clearInterval(timer);
    };
  }, [project, target, revision]);
  const text = (value: unknown) =>
    typeof value === "string" || typeof value === "number"
      ? String(value)
      : Array.isArray(value)
        ? value.join(", ")
        : "Not observed";
  return (
    <section className="panel records-panel" aria-label="Recon asset inventory">
      <h2>Discovered assets</h2>
      {error && <p role="alert">{error}</p>}
      {!assets.length ? (
        <p>No persisted observations for this target.</p>
      ) : (
        <div className="table-scroll">
          <table>
            <thead>
              <tr>
                <th>Host / URL</th>
                <th>HTTP / IP</th>
                <th>Technology / TLS</th>
                <th>Source / evidence</th>
                <th>Observed</th>
              </tr>
            </thead>
            <tbody>
              {assets.map((a) => (
                <tr key={a.id}>
                  <td className="break-value">
                    {a.value}
                    <small className="record-meta">{a.kind}</small>
                  </td>
                  <td>
                    {text(a.facts.status)}
                    <small className="record-meta">{text(a.facts.ip)}</small>
                  </td>
                  <td>
                    {text(a.facts.technology ?? a.facts.reported_server)}
                    <small className="record-meta">
                      {a.facts.tls
                        ? "TLS certificate observed; trust not verified"
                        : "TLS not observed"}
                    </small>
                  </td>
                  <td>
                    {text(a.facts.sources ?? a.facts.source_tool)}
                    <small className="record-meta break-value">
                      {text(a.facts.evidence_id)}
                    </small>
                  </td>
                  <td>
                    {new Date(a.last_seen).toLocaleString()}
                    <small className="record-meta break-value">
                      {a.job_id}
                    </small>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      <p>
        Passive names and technology hints require validation. HTTP availability
        does not establish a vulnerability.
      </p>
    </section>
  );
}
