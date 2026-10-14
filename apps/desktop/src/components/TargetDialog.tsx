import { useEffect, useState } from "react";
import { request, errorText, type Project } from "../api";
import {
  targetKinds,
  selectPaths,
  type Target,
  type TargetInput,
  type TargetKind,
} from "../research";
import { Modal } from "./ui";
const lines = (text: string) =>
  text
    .split(/\r?\n/)
    .map((s) => s.trim())
    .filter(Boolean);
export default function TargetDialog({
  open,
  target,
  projects,
  projectId,
  initialKind,
  onClose,
  onSaved,
}: {
  open: boolean;
  target?: Target;
  projects: Project[];
  projectId?: string;
  initialKind?: TargetKind;
  onClose: () => void;
  onSaved: () => Promise<void>;
}) {
  const [draft, setDraft] = useState<TargetInput>({
    project_id: target?.project_id ?? projectId ?? projects[0]?.id ?? "",
    name: target?.name ?? "",
    kind: target?.kind ?? initialKind ?? "domain",
    value: target?.value ?? "",
    description: target?.description ?? "",
    tags: target?.tags ?? [],
    allowed_scope: target?.allowed_scope ?? [],
    excluded_scope: target?.excluded_scope ?? [],
    authorized: target?.authorized ?? false,
    mode: target?.mode ?? "passive_auto",
  });
  const [tags, setTags] = useState(target?.tags.join(", ") ?? "");
  const [allow, setAllow] = useState(target?.allowed_scope.join("\n") ?? "");
  const [exclude, setExclude] = useState(
    target?.excluded_scope.join("\n") ?? "",
  );
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);
  useEffect(() => {
    if (!open) return;
    const next: TargetInput = target
      ? {
          project_id: target.project_id,
          name: target.name,
          kind: target.kind,
          value: target.value,
          description: target.description,
          tags: target.tags,
          allowed_scope: target.allowed_scope,
          excluded_scope: target.excluded_scope,
          authorized: target.authorized,
          mode: target.mode,
        }
      : {
          project_id: projectId || projects[0]?.id || "",
          name: "",
          kind: initialKind ?? "domain",
          value: "",
          description: "",
          tags: [],
          allowed_scope: [],
          excluded_scope: [],
          authorized: false,
          mode: "passive_auto",
        };
    setDraft(next);
    setTags(next.tags.join(", "));
    setAllow(next.allowed_scope.join("\n"));
    setExclude(next.excluded_scope.join("\n"));
    setError("");
  }, [open, target, projectId, initialKind]);
  const change = <K extends keyof TargetInput>(key: K, value: TargetInput[K]) =>
    setDraft((d) => ({ ...d, [key]: value }));
  const browse = async () => {
    try {
      const paths = await selectPaths(
        ["source_directory", "iac_directory", "git_repository"].includes(
          draft.kind,
        ),
      );
      if (paths[0]) change("value", paths[0]);
    } catch (e) {
      setError(errorText(e));
    }
  };
  return (
    <Modal
      title={target ? "Edit target" : "Add Target"}
      open={open}
      onClose={() => {
        if (!saving) onClose();
      }}
    >
      <form
        className="form target-form"
        onSubmit={async (e) => {
          e.preventDefault();
          setSaving(true);
          setError("");
          try {
            const input = {
              ...draft,
              tags: tags
                .split(",")
                .map((s) => s.trim())
                .filter(Boolean),
              allowed_scope: lines(allow),
              excluded_scope: lines(exclude),
            };
            await request(target ? "target_update" : "target_create", {
              ...(target ? { id: target.id } : {}),
              input,
            });
            await onSaved();
            onClose();
          } catch (e) {
            setError(errorText(e));
          } finally {
            setSaving(false);
          }
        }}
      >
        {error && (
          <p role="alert" className="banner error">
            {error}
          </p>
        )}
        {!projects.length && (
          <p className="banner">Create a project before adding a target.</p>
        )}
        <div className="form-columns">
          <label>
            Target name
            <input
              required
              maxLength={120}
              value={draft.name}
              onChange={(e) => change("name", e.target.value)}
            />
          </label>
          <label>
            Associated project
            <select
              aria-label="Associated project"
              required
              disabled={!!target}
              value={draft.project_id}
              onChange={(e) => change("project_id", e.target.value)}
            >
              <option value="">Choose a project</option>
              {projects.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name}
                </option>
              ))}
            </select>
          </label>
        </div>
        <label>
          Target type
          <select
            aria-label="Target type"
            value={draft.kind}
            onChange={(e) => change("kind", e.target.value as TargetKind)}
          >
            {targetKinds.map(([id, label]) => (
              <option key={id} value={id}>
                {label}
              </option>
            ))}
          </select>
        </label>
        <label>
          Target value
          <input
            required
            maxLength={4096}
            value={draft.value}
            onChange={(e) => change("value", e.target.value)}
            placeholder="example.test, https://localhost:8080, or an absolute local path"
          />
        </label>
        {[
          "source_directory",
          "iac_directory",
          "git_repository",
          "binary",
          "shared_library",
          "firmware",
          "apk",
        ].includes(draft.kind) && (
          <button
            type="button"
            className="button secondary"
            disabled={saving}
            onClick={() => void browse()}
          >
            Browse local path
          </button>
        )}
        <label>
          Description
          <textarea
            maxLength={8192}
            value={draft.description}
            onChange={(e) => change("description", e.target.value)}
          />
        </label>
        <label>
          Tags / groups
          <input
            value={tags}
            onChange={(e) => setTags(e.target.value)}
            placeholder="production, internal — comma separated"
          />
        </label>
        <div className="form-columns">
          <label>
            Allowed scope
            <textarea
              value={allow}
              onChange={(e) => setAllow(e.target.value)}
              placeholder="One rule per line. Empty uses the exact target value."
            />
          </label>
          <label>
            Excluded scope
            <textarea
              value={exclude}
              onChange={(e) => setExclude(e.target.value)}
              placeholder="Exact host, *.domain, URL prefix, IP, CIDR or local path"
            />
          </label>
        </div>
        <label>
          Preferred analysis mode
          <select
            value={draft.mode}
            onChange={(e) =>
              change("mode", e.target.value as TargetInput["mode"])
            }
          >
            <option value="passive_auto">Passive Auto</option>
            <option value="authorized_auto">Authorized Auto</option>
            <option value="lab_research">Lab Research</option>
          </select>
        </label>
        <label className="checkbox-row">
          <input
            type="checkbox"
            checked={draft.authorized}
            onChange={(e) => change("authorized", e.target.checked)}
          />
          I confirm I have permission to analyze this target within the declared
          scope.
        </label>
        <p className="muted">
          Registration does not install tools or execute network operations.
          Exclusions take precedence.
        </p>
        <div className="inline-actions">
          <button className="button" disabled={saving || !draft.project_id}>
            {saving ? "Saving target…" : "Save target"}
          </button>
          <button
            type="button"
            className="button secondary"
            disabled={saving}
            onClick={onClose}
          >
            Cancel
          </button>
        </div>
      </form>
    </Modal>
  );
}
