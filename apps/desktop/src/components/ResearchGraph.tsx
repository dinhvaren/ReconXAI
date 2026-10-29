import { useEffect, useRef, useState } from "react";
import { request, errorText } from "../api";
import { selectPaths } from "../research";
import { Button } from "./controls";
import { Modal } from "./ui";
import "./ResearchGraph.css";

type Node = {
  id: string;
  graph: string;
  kind: string;
  label: string;
  source_table: string;
  source_id: string;
  attributes: Record<string, unknown>;
};
type Edge = { id: string; source: string; target: string; relation: string };
type Page = {
  revision: number;
  source_sha256: string;
  total_nodes: number;
  total_edges: number;
  matched_nodes: number;
  next_offset: number | null;
  nodes: Node[];
  edges: Edge[];
  edges_truncated: boolean;
};
export default function ResearchGraph({
  session,
  revision,
}: {
  session: string;
  revision: string;
}) {
  const [page, setPage] = useState<Page | null>(null),
    [error, setError] = useState("");
  const [graph, setGraph] = useState(""),
    [kind, setKind] = useState(""),
    [search, setSearch] = useState("");
  const [offset, setOffset] = useState(0),
    [refresh, setRefresh] = useState(0),
    [loading, setLoading] = useState(false);
  const [selected, setSelected] = useState(""),
    [record, setRecord] = useState<unknown | null>(null);
  const [zoom, setZoom] = useState(1),
    [pan, setPan] = useState({ x: 0, y: 0 });
  const [neighborsOnly, setNeighborsOnly] = useState(false);
  const selectedRef = useRef({ session, node: selected });
  selectedRef.current = { session, node: selected };
  const generation = useRef(0),
    drag = useRef<{ x: number; y: number; px: number; py: number } | null>(
      null,
    );
  useEffect(() => {
    setOffset(0);
    setPage(null);
    setSelected("");
    setRecord(null);
  }, [session, graph, kind, search]);
  useEffect(() => {
    setPan({ x: 0, y: 0 });
    setZoom(1);
    setNeighborsOnly(false);
  }, [session, graph, kind, search, offset]);
  useEffect(() => {
    const ticket = ++generation.current;
    setLoading(true);
    setError("");
    const timer = setTimeout(() => {
      void request<Page>("research_graph", {
        id: session,
        query: { graph, kind, search, offset, limit: 100 },
      })
        .then((value) => {
          if (ticket !== generation.current) return;
          setPage(value);
          setSelected((current) =>
            value.nodes.some((n) => n.id === current)
              ? current
              : (value.nodes[0]?.id ?? ""),
          );
        })
        .catch((e) => {
          if (ticket === generation.current) setError(errorText(e));
        })
        .finally(() => {
          if (ticket === generation.current) setLoading(false);
        });
    }, 150);
    return () => {
      generation.current++;
      clearTimeout(timer);
    };
  }, [session, revision, graph, kind, search, offset, refresh]);
  const active = page?.nodes.find((n) => n.id === selected);
  const connected = new Set([
    selected,
    ...(page?.edges
      .filter((e) => e.source === selected || e.target === selected)
      .flatMap((e) => [e.source, e.target]) ?? []),
  ]);
  const visibleNodes =
    page?.nodes.filter((n) => !neighborsOnly || connected.has(n.id)) ?? [];
  const visibleEdges =
    page?.edges.filter(
      (e) =>
        !neighborsOnly || (connected.has(e.source) && connected.has(e.target)),
    ) ?? [];
  const positions = new Map(
    page?.nodes.map((n, i) => [
      n.id,
      { x: 30 + (i % 4) * 245, y: 30 + Math.floor(i / 4) * 90 },
    ]),
  );
  const inspect = async () => {
    if (!active) return;
    const ticket = generation.current;
    try {
      const value = await request("research_graph_record", {
        id: session,
        node: active.id,
      });
      if (
        ticket === generation.current &&
        selectedRef.current.session === session &&
        selectedRef.current.node === active.id
      )
        setRecord(value);
    } catch (e) {
      if (ticket === generation.current) setError(errorText(e));
    }
  };
  return (
    <section className="panel records-panel" aria-label="Research provenance">
      <h2>Exploration &amp; asset graph</h2>
      <p>
        Stored records only. Opening this graph makes no tool or AI requests.
        Hypotheses are not verified findings.
      </p>
      <div className="graph-toolbar">
        <label>
          Graph
          <select
            aria-label="Graph type"
            value={graph}
            onChange={(e) => setGraph(e.target.value)}
          >
            <option value="">Both graphs</option>
            <option value="asset">Assets</option>
            <option value="exploration">Exploration</option>
          </select>
        </label>
        <label>
          Node type
          <select
            aria-label="Graph node type"
            value={kind}
            onChange={(e) => setKind(e.target.value)}
          >
            {[
              "",
              "project",
              "target",
              "file",
              "domain",
              "url",
              "session",
              "plan",
              "approval",
              "task",
              "tool_invocation",
              "observation",
              "evidence",
              "finding",
              "report",
            ].map((k) => (
              <option key={k} value={k}>
                {k || "All types"}
              </option>
            ))}
          </select>
        </label>
        <label>
          Search
          <input
            aria-label="Search graph nodes"
            value={search}
            maxLength={128}
            onChange={(e) => setSearch(e.target.value)}
          />
        </label>
        <Button disabled={loading} onClick={() => setRefresh((v) => v + 1)}>
          Refresh records
        </Button>
        <Button
          disabled={!page || loading}
          onClick={() =>
            void (async () => {
              const id = session;
              const ticket = generation.current;
              try {
                const path = (
                  await selectPaths(false, true, "research-graph.json")
                )[0];
                if (path && ticket === generation.current) {
                  await request("research_graph_export", { id, path });
                  if (ticket === generation.current) setError("");
                }
              } catch (e) {
                if (ticket === generation.current) setError(errorText(e));
              }
            })()
          }
        >
          Export graph
        </Button>
      </div>
      {error && <p role="alert">{error}</p>}
      {loading && <p role="status">Loading stored provenance…</p>}
      {page && (
        <>
          <p>
            {page.matched_nodes} matching nodes · {page.total_nodes} total ·{" "}
            {page.total_edges} edges · revision {page.revision}. This page shows{" "}
            {page.nodes.length} nodes and their connecting edges.
          </p>
          <div className="graph-toolbar">
            <Button
              aria-label="Zoom out graph"
              onClick={() => setZoom((v) => Math.max(0.25, v - 0.25))}
            >
              −
            </Button>
            <Button
              aria-label="Zoom in graph"
              onClick={() => setZoom((v) => Math.min(3, v + 0.25))}
            >
              +
            </Button>
            <Button
              onClick={() => {
                setPan({ x: 0, y: 0 });
                setZoom(1);
              }}
            >
              Reset graph view
            </Button>
            <Button
              disabled={!active}
              onClick={() => setNeighborsOnly((v) => !v)}
            >
              {neighborsOnly
                ? "Show whole graph page"
                : "Focus connected nodes"}
            </Button>
            <span>
              {Math.round(zoom * 100)}% · Drag background or focus the canvas
              and use arrow keys to pan.
            </span>
          </div>
          <svg
            className="research-graph-canvas"
            role="group"
            aria-label="Provenance graph canvas"
            tabIndex={0}
            viewBox={`${pan.x} ${pan.y} ${1000 / zoom} ${500 / zoom}`}
            onKeyDown={(e) => {
              const v: Record<string, [number, number]> = {
                ArrowLeft: [-40, 0],
                ArrowRight: [40, 0],
                ArrowUp: [0, -40],
                ArrowDown: [0, 40],
              };
              if (v[e.key]) {
                e.preventDefault();
                const [x, y] = v[e.key];
                setPan((p) => ({ x: p.x + x / zoom, y: p.y + y / zoom }));
              }
            }}
            onPointerDown={(e) => {
              if ((e.target as Element).closest("[data-graph-node]")) return;
              e.currentTarget.setPointerCapture(e.pointerId);
              drag.current = {
                x: e.clientX,
                y: e.clientY,
                px: pan.x,
                py: pan.y,
              };
            }}
            onPointerMove={(e) => {
              const d = drag.current;
              if (d) {
                const r = e.currentTarget.getBoundingClientRect();
                setPan({
                  x: d.px - ((e.clientX - d.x) * 1000) / r.width / zoom,
                  y: d.py - ((e.clientY - d.y) * 500) / r.height / zoom,
                });
              }
            }}
            onPointerUp={() => {
              drag.current = null;
            }}
            onPointerCancel={() => {
              drag.current = null;
            }}
          >
            {visibleEdges.map((e) => {
              const a = positions.get(e.source),
                b = positions.get(e.target);
              return a && b ? (
                <line
                  key={e.id}
                  x1={a.x + 105}
                  y1={a.y + 30}
                  x2={b.x + 105}
                  y2={b.y + 30}
                >
                  <title>{e.relation}</title>
                </line>
              ) : null;
            })}
            {visibleNodes.map((n) => {
              const p = positions.get(n.id)!;
              return (
                <g
                  key={n.id}
                  data-graph-node
                  role="button"
                  aria-label={`${n.kind}: ${n.label}`}
                  tabIndex={0}
                  transform={`translate(${p.x},${p.y})`}
                  className={
                    n.id === selected
                      ? "selected"
                      : n.attributes.status === "running"
                        ? "running"
                        : ""
                  }
                  onClick={() => setSelected(n.id)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      setSelected(n.id);
                    }
                  }}
                >
                  <rect width="210" height="60" rx="5" />
                  <text x="10" y="19">
                    {n.kind} · {n.graph}
                  </text>
                  <text x="10" y="42">
                    {Array.from(n.label).slice(0, 26).join("")}
                  </text>
                  <title>{n.label}</title>
                </g>
              );
            })}
          </svg>
          {!page.nodes.length && <p>No stored nodes match these filters.</p>}
          {page.edges_truncated && (
            <p>
              Connecting edges exceed this page's 1,000-edge display budget.
              Export includes the bounded complete graph.
            </p>
          )}
          <div className="graph-toolbar">
            <Button
              disabled={offset === 0 || loading}
              onClick={() => setOffset((v) => Math.max(0, v - 100))}
            >
              Previous graph page
            </Button>
            <Button
              disabled={page.next_offset === null || loading}
              onClick={() => setOffset(page.next_offset!)}
            >
              Next graph page
            </Button>
          </div>
          {active && (
            <details open className="graph-node-detail">
              <summary>Node details: {active.kind}</summary>
              <p>{active.label}</p>
              <p>
                Source:{" "}
                <code>
                  {active.source_table} / {active.source_id}
                </code>
              </p>
              <pre>{JSON.stringify(active.attributes, null, 2)}</pre>
              <p>Relationships in this loaded page (first 30):</p>
              <ul className="graph-relationships">
                {page.edges
                  .filter(
                    (e) => e.source === active.id || e.target === active.id,
                  )
                  .slice(0, 30)
                  .map((e) => {
                    const other = e.source === active.id ? e.target : e.source;
                    const n = page.nodes.find((n) => n.id === other);
                    return (
                      <li key={e.id}>
                        {e.source === active.id ? "Outgoing" : "Incoming"} ·{" "}
                        {e.relation} ·{" "}
                        <Button onClick={() => setSelected(other)}>
                          {n?.label ?? other}
                        </Button>
                      </li>
                    );
                  })}
              </ul>
              <Button onClick={() => void inspect()}>
                Inspect linked record
              </Button>
              <p>
                Integrity status is not inferred from graph edges; verify the
                original evidence separately.
              </p>
            </details>
          )}
        </>
      )}
      <Modal
        title="Linked provenance record"
        open={record !== null}
        onClose={() => setRecord(null)}
      >
        <pre className="graph-record-preview">
          {JSON.stringify(record, null, 2)}
        </pre>
      </Modal>
    </section>
  );
}
