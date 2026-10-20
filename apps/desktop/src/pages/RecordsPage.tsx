import { Button } from "../components/controls";
import { useWorkspaceProject, useWorkspace } from "../workspace";
import { useCallback, useEffect, useState, useRef } from "react";
import { request, errorText, type Snapshot } from "../api";
import { selectPaths } from "../research";
import { Heading, Modal, type Perform } from "../components/ui";
const evidenceKinds = [
  "http_request",
  "http_response",
  "tool_output",
  "structured_json",
  "terminal_log",
  "screenshot",
  "binary_metadata",
  "static_analysis",
  "yara_matches",
  "source_reference",
  "mcp_invocation",
  "ai_summary",
  "user_note",
  "attachment",
];
const statuses = [
  "hypothesis",
  "needs_verification",
  "verified",
  "refuted",
  "false_positive",
  "duplicate",
  "accepted_risk",
  "remediated",
];
const severities = ["informational", "low", "medium", "high", "critical"];
const label = (s: string) => s.replaceAll("_", " ");
interface Evidence {
  id: string;
  project_id: string;
  target_id: string | null;
  job_id: string | null;
  title: string;
  kind: string;
  sha256: string;
  size: number;
  source_tool: string;
  verification_state: string;
  sensitivity: string;
  created_at: string;
  review_note: string;
  media_type: string;
}
interface EvidenceView {
  evidence: Evidence;
  text: string | null;
  image_data: string | null;
  preview_omitted: boolean;
}
interface FindingInput {
  project_id: string;
  target_id: string | null;
  title: string;
  description: string;
  affected_asset: string;
  category: string;
  cwe: string | null;
  cve: string | null;
  cvss_v3: string | null;
  cvss_v4: string | null;
  severity: string;
  confidence: string;
  status: string;
  technical_details: string;
  reproduction_steps: string;
  impact: string;
  remediation: string;
  references: string[];
  notes: string;
  evidence_ids: string[];
  verification_note: string;
  ai_generated: boolean;
}
interface Finding {
  id: string;
  input: FindingInput;
  revision: number;
  created_at: string;
  updated_at: string;
  duplicate_of: string | null;
  cvss: { version: string; score: number }[];
}
interface Report {
  id: string;
  title: string;
  sha256: string;
  template: string;
  created_at: string;
}
interface ReportView {
  report: Report;
  document: { text: string; redactions: number; warnings: string[] };
}
const initialFinding = (project: string): FindingInput => ({
  project_id: project,
  target_id: null,
  title: "",
  description: "",
  affected_asset: "",
  category: "",
  cwe: null,
  cve: null,
  cvss_v3: null,
  cvss_v4: null,
  severity: "informational",
  confidence: "low",
  status: "hypothesis",
  technical_details: "",
  reproduction_steps: "",
  impact: "",
  remediation: "",
  references: [],
  notes: "",
  evidence_ids: [],
  verification_note: "",
  ai_generated: false,
});

export default function RecordsPage({
  page,
  data,
  projectId,
  perform,
  busy,
  newFindingRequest = 0,
  onFindingRequestConsumed,
}: {
  page: "Evidence" | "Findings" | "Reports";
  data: Snapshot;
  projectId?: string;
  perform: Perform;
  busy: boolean;
  newFindingRequest?: number;
  onFindingRequestConsumed?: () => void;
}) {
  const workspace = useWorkspace();
  const [project, setProject] = useWorkspaceProject(
    projectId ?? data.projects[0]?.id ?? "",
  );
  const [evidence, setEvidence] = useState<Evidence[]>([]),
    [findings, setFindings] = useState<Finding[]>([]),
    [reports, setReports] = useState<Report[]>([]);
  const [query, setQuery] = useState(""),
    [filter, setFilter] = useState(""),
    [error, setError] = useState("");
  const [evOpen, setEvOpen] = useState(false),
    [evTitle, setEvTitle] = useState(""),
    [evKind, setEvKind] = useState("user_note"),
    [evText, setEvText] = useState(""),
    [evPath, setEvPath] = useState(""),
    [evTarget, setEvTarget] = useState(""),
    [sensitivity, setSensitivity] = useState("confidential");
  const [view, setView] = useState<EvidenceView | null>(null),
    [review, setReview] = useState(""),
    [raw, setRaw] = useState(false),
    [job, setJob] = useState("");
  const [findingOpen, setFindingOpen] = useState(false),
    [editing, setEditing] = useState<Finding | null>(null),
    [draft, setDraft] = useState<FindingInput>(initialFinding(project)),
    [confirm, setConfirm] = useState(false),
    [cvss, setCvss] = useState("");
  const [remove, setRemove] = useState<Finding | null>(null),
    [compareA, setCompareA] = useState(""),
    [compareB, setCompareB] = useState(""),
    [comparison, setComparison] = useState("");
  const [title, setTitle] = useState(""),
    [template, setTemplate] = useState("research"),
    [summary, setSummary] = useState(""),
    [methodology, setMethodology] = useState("");
  const [selectedFindings, setSelectedFindings] = useState<string[]>([]),
    [selectedEvidence, setSelectedEvidence] = useState<string[]>([]),
    [report, setReport] = useState<ReportView | null>(null),
    [approved, setApproved] = useState(false),
    [format, setFormat] = useState("markdown");
  const loadVersion = useRef(0);
  const previousProject = useRef(project);
  const load = useCallback(async () => {
    const version = ++loadVersion.current;
    if (!project) {
      setEvidence([]);
      setFindings([]);
      setReports([]);
      return;
    }
    const [e, f, r] = await Promise.all([
      request<Evidence[]>("evidence_list", { project }),
      request<Finding[]>("findings_list", { project }),
      request<Report[]>("reports_list", { project }),
    ]);
    if (version !== loadVersion.current) return;
    setEvidence(e);
    setFindings(f);
    setReports(r);
  }, [project]);
  useEffect(() => {
    void load().catch((e) => setError(errorText(e)));
  }, [load, data.audit[0]?.id]);
  useEffect(() => {
    if (projectId) setProject(projectId);
  }, [projectId]);
  const openFinding = (f: Finding | null) => {
    setEditing(f);
    setDraft(
      f
        ? structuredClone(f.input)
        : {
            ...initialFinding(project),
            target_id: workspace?.selection.target_id ?? null,
          },
    );
    setConfirm(false);
    setCvss("");
    setError("");
    setFindingOpen(true);
  };
  useEffect(() => {
    if (newFindingRequest && page === "Findings") {
      openFinding(null);
      onFindingRequestConsumed?.();
    }
  }, [newFindingRequest]);
  useEffect(() => {
    setSelectedFindings([]);
    setSelectedEvidence([]);
    setCompareA("");
    setCompareB("");
    setJob("");
    setEvTarget(workspace?.selection.target_id ?? "");
    if (previousProject.current !== project) {
      setView(null);
      setReport(null);
      setFindingOpen(false);
      setEvOpen(false);
    }
    previousProject.current = project;
  }, [project]);
  const act = (fn: () => Promise<unknown>, message: string) =>
    perform(async () => {
      try {
        await fn();
        await load();
        setError("");
      } catch (e) {
        setError(errorText(e));
        throw e;
      }
    }, message);
  const pick = async (
    save: boolean,
    action: (path: string) => Promise<unknown>,
    message: string,
    fileName?: string,
  ) => {
    try {
      const paths = await selectPaths(false, save, fileName);
      if (paths[0]) await act(() => action(paths[0]), message);
    } catch (e) {
      setError(errorText(e));
    }
  };
  const change = <K extends keyof FindingInput>(
    key: K,
    value: FindingInput[K],
  ) => setDraft((d) => ({ ...d, [key]: value }));
  const toggle = (items: string[], id: string) =>
    items.includes(id) ? items.filter((i) => i !== id) : [...items, id];
  const projectTargets = (data.targets ?? []).filter(
    (t) => t.project_id === project,
  );
  const selectEvidence = (items: string[], set: (s: string[]) => void) => (
    <div className="record-checklist">
      {evidence.map((e) => (
        <label key={e.id}>
          <input
            type="checkbox"
            checked={items.includes(e.id)}
            onChange={() => set(toggle(items, e.id))}
          />
          <span>
            {e.title}
            <small>
              {label(e.kind)} · {e.verification_state}
            </small>
          </span>
        </label>
      ))}
      {!evidence.length && (
        <p>
          No evidence in this project. Complete a local job or add a record.
        </p>
      )}
    </div>
  );
  const targets = (value: string, set: (v: string) => void) => (
    <label>
      Related target
      <select
        aria-label="Related target"
        value={value}
        onChange={(e) => set(e.target.value)}
      >
        <option value="">No target</option>
        {projectTargets.map((t) => (
          <option value={t.id} key={t.id}>
            {t.name}
          </option>
        ))}
      </select>
    </label>
  );
  const saveEvidence = () =>
    act(async () => {
      const input = {
        project_id: project,
        target_id: evTarget || null,
        title: evTitle,
        kind: evKind,
        source_tool: "manual evidence",
        sensitivity,
      };
      if (evPath) await request("evidence_import", { input, path: evPath });
      else await request("evidence_add", { input, text: evText });
      setEvOpen(false);
    }, "Evidence saved locally");
  const openReport = (id: string) =>
    act(async () => {
      setReport(await request<ReportView>("report_show", { id }));
      setApproved(false);
    }, "Report preview opened");
  return (
    <>
      <Heading
        eyebrow="Research records"
        title={page}
        description={
          page === "Evidence"
            ? "Immutable local artifacts, source links and integrity. Hash matching does not verify a vulnerability."
            : page === "Findings"
              ? "Manually reviewed observations and hypotheses linked to their supporting evidence."
              : "Review an immutable redacted snapshot before exporting. Raw attachments stay separate."
        }
        action={
          page === "Evidence" ? (
            <Button
              className="button primary"
              disabled={!project || busy}
              onClick={() => {
                setEvOpen(true);
                setError("");
                setEvPath("");
                setEvTitle("");
                setEvText("");
                setEvKind("user_note");
                setEvTarget("");
              }}
              disabledReason={
                !project
                  ? "Select a project first"
                  : "An operation is in progress"
              }
            >
              Add Evidence
            </Button>
          ) : page === "Findings" ? (
            <Button
              className="button primary"
              disabled={!project || busy}
              onClick={() => openFinding(null)}
              disabledReason={
                !project
                  ? "Select a project first"
                  : "An operation is in progress"
              }
            >
              Create Finding
            </Button>
          ) : undefined
        }
      />
      {error && <p role="alert">{error}</p>}
      <section className="panel records-toolbar">
        <label>
          Records project
          <select
            aria-label="Records project"
            value={project}
            onChange={(e) => {
              setProject(e.target.value);
              setSelectedEvidence([]);
              setSelectedFindings([]);
            }}
          >
            <option value="">Choose project</option>
            {data.projects
              .filter((p) => !p.archived)
              .map((p) => (
                <option value={p.id} key={p.id}>
                  {p.name}
                </option>
              ))}
          </select>
        </label>
        <label>
          Search {page.toLowerCase()}
          <input value={query} onChange={(e) => setQuery(e.target.value)} />
        </label>
        {page !== "Reports" && (
          <label>
            Filter
            <select
              aria-label="Record filter"
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
            >
              <option value="">All</option>
              {(page === "Evidence" ? evidenceKinds : statuses).map((k) => (
                <option key={k} value={k}>
                  {label(k)}
                </option>
              ))}
            </select>
          </label>
        )}
        <Button
          className="button secondary"
          disabled={busy}
          onClick={() => void act(load, "Records refreshed")}
          disabledReason={"An operation is in progress"}
        >
          Refresh records
        </Button>
      </section>
      {!project && (
        <p className="empty">
          Create or select a project to manage research records.
        </p>
      )}
      {page === "Evidence" && project && (
        <>
          <section className="panel inline-actions records-panel">
            <label>
              Completed job
              <select value={job} onChange={(e) => setJob(e.target.value)}>
                <option value="">Select existing result</option>
                {(data.jobs ?? [])
                  .filter(
                    (j) => j.project_id === project && j.status === "completed",
                  )
                  .map((j) => (
                    <option key={j.id} value={j.id}>
                      {label(j.kind)} · {j.id.slice(0, 8)}
                    </option>
                  ))}
              </select>
            </label>
            <Button
              className="button secondary"
              disabled={!job || busy}
              onClick={() =>
                void act(
                  () => request("evidence_from_job", { id: job }),
                  "Job evidence collected",
                )
              }
              disabledReason={
                !job ? "Select a completed job" : "An operation is in progress"
              }
            >
              Collect job evidence
            </Button>
            <p>
              New completed jobs create evidence automatically. Older results
              can be collected here.
            </p>
          </section>
          <div className="table-scroll">
            <table>
              <thead>
                <tr>
                  <th>Evidence</th>
                  <th>Type / source</th>
                  <th>Review / sensitivity</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {evidence
                  .filter(
                    (e) =>
                      (!filter || e.kind === filter) &&
                      `${e.title} ${e.id} ${e.source_tool}`
                        .toLowerCase()
                        .includes(query.toLowerCase()),
                  )
                  .map((e) => (
                    <tr key={e.id}>
                      <td>
                        {e.title}
                        <small className="record-meta">
                          {e.id} · {e.size.toLocaleString()} bytes
                        </small>
                      </td>
                      <td>
                        {label(e.kind)}
                        <small className="record-meta">{e.source_tool}</small>
                      </td>
                      <td>
                        {label(e.verification_state)}
                        <small className="record-meta">{e.sensitivity}</small>
                      </td>
                      <td className="record-actions">
                        <Button
                          className="text-button"
                          onClick={() =>
                            void act(async () => {
                              setView(
                                await request<EvidenceView>("evidence_show", {
                                  id: e.id,
                                }),
                              );
                              setReview("");
                              setRaw(false);
                            }, "Evidence opened")
                          }
                        >
                          Open evidence
                        </Button>
                        <Button
                          className="text-button"
                          onClick={() =>
                            void act(
                              () => request("evidence_verify", { id: e.id }),
                              "Evidence integrity matched",
                            )
                          }
                        >
                          Verify integrity
                        </Button>
                      </td>
                    </tr>
                  ))}
              </tbody>
            </table>
            {!evidence.length && <p className="empty">No evidence recorded.</p>}
          </div>
        </>
      )}
      {page === "Findings" && project && (
        <>
          <div className="table-scroll">
            <table>
              <thead>
                <tr>
                  <th>Finding</th>
                  <th>Severity</th>
                  <th>Status</th>
                  <th>Evidence / score</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {findings
                  .filter(
                    (f) =>
                      (!filter || f.input.status === filter) &&
                      `${f.input.title} ${f.input.category} ${f.input.cwe ?? ""}`
                        .toLowerCase()
                        .includes(query.toLowerCase()),
                  )
                  .sort(
                    (a, b) =>
                      severities.indexOf(b.input.severity) -
                      severities.indexOf(a.input.severity),
                  )
                  .map((f) => (
                    <tr key={f.id}>
                      <td>
                        {f.input.title}
                        <small className="record-meta">
                          {f.id}
                          {f.input.ai_generated ? " · AI-generated" : ""}
                        </small>
                      </td>
                      <td>{f.input.severity}</td>
                      <td>{label(f.input.status)}</td>
                      <td>
                        {f.input.evidence_ids.length} linked
                        {f.cvss.map((s) => (
                          <small className="record-meta" key={s.version}>
                            CVSS {s.version}: {s.score}
                          </small>
                        ))}
                      </td>
                      <td className="record-actions">
                        <Button
                          className="text-button"
                          onClick={() => openFinding(f)}
                        >
                          Open / edit
                        </Button>
                        <Button
                          className="text-button"
                          onClick={() => setRemove(f)}
                        >
                          Delete finding
                        </Button>
                      </td>
                    </tr>
                  ))}
              </tbody>
            </table>
            {!findings.length && (
              <p className="empty">
                No findings. Create a hypothesis or a manually reviewed
                observation.
              </p>
            )}
          </div>
          <section className="panel records-panel">
            <h2>Compare and merge duplicates</h2>
            <div className="inline-actions">
              {[compareA, compareB].map((v, i) => (
                <label key={i}>
                  {i === 0 ? "Primary finding" : "Comparison finding"}
                  <select
                    value={v}
                    onChange={(e) =>
                      (i === 0 ? setCompareA : setCompareB)(e.target.value)
                    }
                  >
                    <option value="">Choose finding</option>
                    {findings.map((f) => (
                      <option key={f.id} value={f.id}>
                        {f.input.title}
                      </option>
                    ))}
                  </select>
                </label>
              ))}
              <Button
                className="button secondary"
                disabled={
                  !compareA || !compareB || compareA === compareB || busy
                }
                onClick={() =>
                  void act(
                    async () =>
                      setComparison(
                        JSON.stringify(
                          await request("finding_compare", {
                            a: compareA,
                            b: compareB,
                          }),
                          null,
                          2,
                        ),
                      ),
                    "Comparison opened",
                  )
                }
                disabledReason={
                  !compareA
                    ? "Select the first finding to compare"
                    : !compareB
                      ? "Select the second finding to compare"
                      : compareA === compareB
                        ? "Select two different findings"
                        : "An operation is in progress"
                }
              >
                Compare findings
              </Button>
            </div>
          </section>
        </>
      )}
      {page === "Reports" && project && (
        <>
          <section className="panel records-panel">
            <h2>Prepare report preview</h2>
            <form
              className="form records-form"
              onSubmit={(e) => {
                e.preventDefault();
                void act(async () => {
                  setReport(
                    await request<ReportView>("report_prepare", {
                      input: {
                        project_id: project,
                        title,
                        template,
                        summary,
                        methodology,
                        finding_ids: selectedFindings,
                        evidence_ids: selectedEvidence,
                      },
                    }),
                  );
                  setApproved(false);
                }, "Redacted report preview saved");
              }}
            >
              <label>
                Report title
                <input
                  required
                  maxLength={200}
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                />
              </label>
              <label>
                Report template
                <select
                  value={template}
                  onChange={(e) => setTemplate(e.target.value)}
                  aria-label="Report template"
                >
                  {[
                    "pentest",
                    "bug_bounty",
                    "hacker_one",
                    "executive",
                    "technical",
                    "reverse_engineering",
                    "research",
                  ].map((t) => (
                    <option key={t} value={t}>
                      {label(t)}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                Executive summary
                <textarea
                  value={summary}
                  onChange={(e) => setSummary(e.target.value)}
                />
              </label>
              <label>
                Methodology
                <textarea
                  value={methodology}
                  onChange={(e) => setMethodology(e.target.value)}
                />
              </label>
              <fieldset>
                <legend>Findings to include</legend>
                <div className="record-checklist">
                  {findings.map((f) => (
                    <label key={f.id}>
                      <input
                        type="checkbox"
                        checked={selectedFindings.includes(f.id)}
                        onChange={() =>
                          setSelectedFindings(toggle(selectedFindings, f.id))
                        }
                      />
                      <span>
                        {f.input.title}
                        <small>
                          {label(f.input.status)} · {f.input.severity}
                        </small>
                      </span>
                    </label>
                  ))}
                </div>
              </fieldset>
              <fieldset>
                <legend>Additional evidence</legend>
                {selectEvidence(selectedEvidence, setSelectedEvidence)}
              </fieldset>
              <p>
                Source evidence linked to selected findings is included
                automatically. Binary/image content is excluded; its hash and
                metadata are retained.
              </p>
              <Button
                className="button primary"
                disabled={busy}
                disabledReason={"An operation is in progress"}
              >
                Prepare preview
              </Button>
            </form>
          </section>
          <section className="panel records-panel">
            <h2>Saved report snapshots</h2>
            {reports
              .filter((r) =>
                r.title.toLowerCase().includes(query.toLowerCase()),
              )
              .map((r) => (
                <div key={r.id} className="record-row">
                  <span>
                    {r.title}
                    <small className="record-meta">
                      {label(r.template)} ·{" "}
                      {new Date(r.created_at).toLocaleString()}
                    </small>
                  </span>
                  <Button
                    className="button secondary"
                    onClick={() => void openReport(r.id)}
                  >
                    Open report preview
                  </Button>
                </div>
              ))}
            {!reports.length && <p>No saved report snapshots.</p>}
          </section>
        </>
      )}
      <Modal
        dismissDisabled={busy}
        title="Add Evidence"
        open={evOpen}
        onClose={() => {
          if (!busy) setEvOpen(false);
        }}
      >
        <form
          className="form records-form"
          onSubmit={(e) => {
            e.preventDefault();
            void saveEvidence();
          }}
        >
          {error && <p role="alert">{error}</p>}
          <label>
            Evidence title
            <input
              required
              value={evTitle}
              maxLength={180}
              onChange={(e) => setEvTitle(e.target.value)}
            />
          </label>
          <label>
            Evidence type
            <select
              aria-label="Evidence type"
              value={evKind}
              onChange={(e) => {
                setEvKind(e.target.value);
                setEvPath("");
              }}
            >
              {evidenceKinds.map((k) => (
                <option key={k} value={k}>
                  {label(k)}
                </option>
              ))}
            </select>
          </label>
          {targets(evTarget, setEvTarget)}
          <label>
            Sensitivity
            <select
              value={sensitivity}
              onChange={(e) => setSensitivity(e.target.value)}
              aria-label="Sensitivity"
            >
              {["public", "internal", "confidential", "restricted"].map((s) => (
                <option key={s}>{s}</option>
              ))}
            </select>
          </label>
          <Button
            type="button"
            className="button secondary"
            onClick={() =>
              void pick(
                false,
                async (path) => setEvPath(path),
                "Local file selected",
              )
            }
          >
            {evKind === "screenshot" ? "Import Screenshot" : "Attach File"}
          </Button>
          {evPath ? (
            <p className="technical break-value">{evPath}</p>
          ) : !["screenshot", "attachment"].includes(evKind) ? (
            <label>
              Evidence content
              <textarea
                required
                rows={9}
                value={evText}
                onChange={(e) => setEvText(e.target.value)}
              />
            </label>
          ) : (
            <p>Select a local file. No data is uploaded.</p>
          )}
          <Button
            className="button primary"
            disabled={
              busy || (["screenshot", "attachment"].includes(evKind) && !evPath)
            }
            disabledReason={
              busy
                ? "An operation is in progress"
                : "Choose the attachment file first"
            }
          >
            Save evidence
          </Button>
        </form>
      </Modal>
      <Modal
        dismissDisabled={busy}
        title="Evidence details"
        open={!!view}
        onClose={() => {
          if (!busy) setView(null);
        }}
      >
        {view && (
          <div className="records-form">
            {error && <p role="alert">{error}</p>}
            <h3>{view.evidence.title}</h3>
            <p>
              {label(view.evidence.kind)} · {view.evidence.sensitivity} ·{" "}
              {view.evidence.verification_state}
            </p>
            <p className="technical break-value">
              SHA-256: {view.evidence.sha256}
            </p>
            <p className="record-meta">
              Target: {view.evidence.target_id ?? "No target"}
              <br />
              Job: {view.evidence.job_id ?? "No job"}
            </p>
            {view.text !== null && (
              <pre className="hex-preview record-preview">{view.text}</pre>
            )}
            {view.image_data && (
              <img
                className="evidence-image"
                src={view.image_data}
                alt="Imported research evidence"
              />
            )}
            {view.preview_omitted && (
              <p>
                Preview unavailable or exceeds 1 MiB. Original bytes are
                retained; CLI export requires explicit approval for binary/image
                data.
              </p>
            )}
            <label>
              Manual review note
              <textarea
                value={review}
                onChange={(e) => setReview(e.target.value)}
              />
            </label>
            <Button
              className="button secondary"
              disabled={
                busy || !review.trim() || view.evidence.kind === "ai_summary"
              }
              title={
                view.evidence.kind === "ai_summary"
                  ? "AI summaries remain unverified"
                  : "Review supporting content, not only its hash"
              }
              onClick={() =>
                void act(async () => {
                  await request("evidence_review", {
                    id: view.evidence.id,
                    note: review,
                  });
                  setView(
                    await request<EvidenceView>("evidence_show", {
                      id: view.evidence.id,
                    }),
                  );
                }, "Manual evidence review saved")
              }
            >
              Mark evidence reviewed
            </Button>
            <label className="check">
              <input
                type="checkbox"
                checked={raw}
                onChange={(e) => setRaw(e.target.checked)}
              />
              I approve exporting original sensitive bytes without redaction.
            </label>
            <Button
              className="button secondary"
              disabled={busy}
              onClick={() =>
                void pick(
                  true,
                  (path) =>
                    request("evidence_export", {
                      id: view.evidence.id,
                      path,
                      raw_approved: raw,
                    }),
                  "Evidence export completed",
                  `evidence-${view.evidence.id}.${view.evidence.media_type === "application/json" || view.evidence.kind === "structured_json" ? "json" : view.evidence.media_type === "text/plain" ? "txt" : view.evidence.media_type === "image/png" ? "png" : view.evidence.media_type === "image/jpeg" ? "jpg" : "bin"}`,
                )
              }
              disabledReason={"An operation is in progress"}
            >
              Export evidence
            </Button>
          </div>
        )}
      </Modal>
      <Modal
        dismissDisabled={busy}
        title={editing ? "Edit finding" : "Create Finding"}
        open={findingOpen}
        onClose={() => {
          if (!busy) setFindingOpen(false);
        }}
      >
        <form
          className="form records-form"
          onSubmit={(e) => {
            e.preventDefault();
            void act(async () => {
              await request("finding_save", {
                id: editing?.id ?? null,
                input: draft,
                revision: editing?.revision ?? null,
                confirm_verified: confirm,
              });
              setFindingOpen(false);
            }, "Finding saved");
          }}
        >
          {error && <p role="alert">{error}</p>}
          <label>
            Finding title
            <input
              required
              value={draft.title}
              maxLength={200}
              onChange={(e) => change("title", e.target.value)}
            />
          </label>
          {targets(draft.target_id ?? "", (v) =>
            change("target_id", v || null),
          )}
          <div className="records-fields">
            {(["severity", "confidence", "status"] as const).map((k) => (
              <label key={k}>
                {label(k)}
                <select
                  aria-label={`Finding ${k}`}
                  value={draft[k]}
                  onChange={(e) => change(k, e.target.value)}
                >
                  {(k === "severity"
                    ? severities
                    : k === "status"
                      ? statuses
                      : ["low", "medium", "high"]
                  ).map((v) => (
                    <option key={v} value={v}>
                      {label(v)}
                    </option>
                  ))}
                </select>
              </label>
            ))}
          </div>
          {(
            [
              "description",
              "affected_asset",
              "category",
              "technical_details",
              "reproduction_steps",
              "impact",
              "remediation",
              "notes",
              "verification_note",
            ] as const
          ).map((k) => (
            <label key={k}>
              {label(k)}
              <textarea
                aria-label={`Finding ${label(k)}`}
                value={draft[k]}
                onChange={(e) => change(k, e.target.value)}
              />
            </label>
          ))}
          <div className="records-fields">
            {(["cwe", "cve", "cvss_v3", "cvss_v4"] as const).map((k) => (
              <label key={k}>
                {k.replaceAll("_", " ").toUpperCase()}
                <input
                  aria-label={k}
                  value={draft[k] ?? ""}
                  onChange={(e) => change(k, e.target.value || null)}
                />
              </label>
            ))}
          </div>
          <Button
            type="button"
            className="button secondary"
            disabled={busy || (!draft.cvss_v3 && !draft.cvss_v4)}
            onClick={() =>
              void act(async () => {
                const scores = await Promise.all(
                  [draft.cvss_v3, draft.cvss_v4]
                    .filter(Boolean)
                    .map((vector) => request("cvss_calculate", { vector })),
                );
                setCvss(JSON.stringify(scores, null, 2));
              }, "CVSS calculated from supplied metrics")
            }
            disabledReason={
              busy
                ? "An operation is in progress"
                : "Enter a CVSS 3.1 or 4.0 vector"
            }
          >
            Calculate CVSS
          </Button>
          {cvss && <pre className="hex-preview">{cvss}</pre>}
          <label>
            References (one per line)
            <textarea
              value={draft.references.join("\n")}
              onChange={(e) =>
                change("references", e.target.value.split("\n").filter(Boolean))
              }
            />
          </label>
          <fieldset>
            <legend>Supporting evidence</legend>
            {selectEvidence(draft.evidence_ids, (v) =>
              change("evidence_ids", v),
            )}
          </fieldset>
          <label className="check">
            <input
              type="checkbox"
              checked={draft.ai_generated}
              onChange={(e) => change("ai_generated", e.target.checked)}
            />
            Contains AI-generated conclusions
          </label>
          {draft.status === "verified" && (
            <label className="check">
              <input
                type="checkbox"
                required
                checked={confirm}
                onChange={(e) => setConfirm(e.target.checked)}
              />
              I independently reviewed the supporting non-AI evidence and
              confirm this finding.
            </label>
          )}
          <p>
            Verified requires reviewed source evidence and a verification note.
            CVSS is calculated only from metrics you provide.
          </p>
          <Button
            className="button primary"
            disabled={busy}
            disabledReason={"An operation is in progress"}
          >
            Save finding
          </Button>
        </form>
      </Modal>
      <Modal
        dismissDisabled={busy}
        title="Delete finding"
        open={!!remove}
        onClose={() => setRemove(null)}
      >
        {remove && (
          <div className="records-form">
            {error && <p role="alert">{error}</p>}
            <p>
              Delete {remove.input.title}? Original evidence and saved report
              snapshots are retained.
            </p>
            <Button
              className="button primary"
              disabled={busy}
              onClick={() =>
                void act(async () => {
                  await request("finding_remove", {
                    id: remove.id,
                    revision: remove.revision,
                  });
                  setRemove(null);
                }, "Finding deleted")
              }
              disabledReason={"An operation is in progress"}
            >
              Confirm deletion
            </Button>
          </div>
        )}
      </Modal>
      <Modal
        dismissDisabled={busy}
        title="Compare findings"
        open={!!comparison}
        onClose={() => setComparison("")}
      >
        {error && <p role="alert">{error}</p>}
        <pre className="hex-preview record-preview">{comparison}</pre>
        <p>
          Merging retains the source finding as Duplicate, preserves its content
          and links its evidence to the primary finding.
        </p>
        <Button
          className="button secondary"
          disabled={busy}
          onClick={() =>
            void act(async () => {
              await request("finding_merge", {
                primary: compareA,
                duplicate: compareB,
              });
              setComparison("");
            }, "Duplicate merged with provenance retained")
          }
          disabledReason={"An operation is in progress"}
        >
          Approve duplicate merge
        </Button>
      </Modal>
      <Modal
        dismissDisabled={busy}
        title="Report preview"
        open={!!report}
        onClose={() => {
          if (!busy) setReport(null);
        }}
      >
        {report && (
          <div className="records-form">
            {error && <p role="alert">{error}</p>}
            <h3>{report.report.title}</h3>
            <p>{report.document.redactions} redactions · immutable snapshot</p>
            {report.document.warnings.map((w, i) => (
              <p className="record-warning" key={i}>
                {w}
              </p>
            ))}
            <pre className="hex-preview record-preview">
              {format === "json"
                ? JSON.stringify(report.document, null, 2)
                : report.document.text}
            </pre>
            <p className="technical break-value">
              Preview SHA-256: {report.report.sha256}
            </p>
            <label>
              Export format
              <select
                value={format}
                onChange={(e) => {
                  setFormat(e.target.value);
                  setApproved(false);
                }}
                aria-label="Export format"
              >
                {["markdown", "json", "html", "pdf"].map((f) => (
                  <option value={f} key={f}>
                    {f.toUpperCase()}
                  </option>
                ))}
              </select>
            </label>
            <label className="check">
              <input
                type="checkbox"
                checked={approved}
                onChange={(e) => setApproved(e.target.checked)}
              />
              I reviewed this redacted snapshot and approve exporting it.
            </label>
            <Button
              className="button primary"
              disabled={busy || !approved}
              onClick={() =>
                void pick(
                  true,
                  (path) =>
                    request("report_export", {
                      id: report.report.id,
                      format,
                      path,
                      reviewed_sha256: report.report.sha256,
                    }),
                  "Reviewed report exported",
                  `report-${report.report.id}.${format === "markdown" ? "md" : format}`,
                )
              }
              disabledReason={
                busy
                  ? "An operation is in progress"
                  : "Review and approve the current plan or report"
              }
            >
              Export reviewed report
            </Button>
          </div>
        )}
      </Modal>
    </>
  );
}
