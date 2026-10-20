import { Button } from "./controls";
import { useEffect, useRef, type ReactNode } from "react";
import type { Audit, Snapshot } from "../api";
export type Perform = (
  action: () => Promise<unknown>,
  message?: string,
) => Promise<void>;
export { Modal as Dialog };
export { Icon } from "./icons";
import { Icon } from "./icons";
export function Heading({
  eyebrow,
  title,
  description,
  action,
}: {
  eyebrow: string;
  title: string;
  description: string;
  action?: ReactNode;
}) {
  return (
    <div className="page-heading">
      <div>
        <p className="eyebrow">{eyebrow[0] + eyebrow.slice(1).toLowerCase()}</p>
        <h1>{title}</h1>
        <p className="subtitle">{description}</p>
      </div>
      {action}
    </div>
  );
}
export function SectionTitle({
  title,
  action,
}: {
  title: string;
  action?: ReactNode;
}) {
  return (
    <div className="section-title">
      <h2>{title}</h2>
      {action}
    </div>
  );
}
export function StatusIndicator({
  label,
  tone = "neutral",
}: {
  label: string;
  tone?: "neutral" | "good" | "warning" | "error";
}) {
  return (
    <span className={"status-indicator " + tone}>
      <i />
      {label}
    </span>
  );
}
export function ActivityItem({
  event,
  compact = false,
}: {
  event: Audit;
  compact?: boolean;
}) {
  const labels: Record<string, string> = {
    "project.created": "Project created",
    "project.updated": "Project updated",
    "project.archived": "Project archived",
    "project.restored": "Project restored",
    "provider.saved": "Model configured",
    "provider.deleted": "Provider removed",
    "settings.updated": "Preferences updated",
    "settings.migrated": "Appearance migrated",
    "ai.models": "Model discovery",
    "ai.chat": "Assistant response",
    "ai.artifact_review": "Artifact review",
    "ai.stream": "Streamed analysis",
    "ai.fallback": "Fallback response",
    "ai.stream_fallback": "Fallback analysis",
    "tool.install": "Tool installation",
    "tool.doctor": "Tool health check",
    "tool.uninstall": "Tool removed",
    "credential.updated": "Credential updated",
    "credential.removed": "Credential removed",
  };
  return (
    <div className={"activity-item " + (compact ? "compact" : "")}>
      <div
        className={
          "activity-avatar " +
          (event.action.startsWith("ai.") ? "pink" : "gold")
        }
      >
        <Icon
          name={
            event.action.startsWith("ai.")
              ? "ai"
              : event.action.startsWith("tool.")
                ? "terminal"
                : event.action.startsWith("project.")
                  ? "folder"
                  : "settings"
          }
          size={16}
        />
      </div>
      <div className="activity-description">
        <b>{labels[event.action] ?? event.action}</b>
        <small>
          {compact
            ? new Date(event.timestamp).toLocaleTimeString([], {
                hour: "2-digit",
                minute: "2-digit",
              })
            : new Date(event.timestamp).toLocaleString()}{" "}
          · {event.outcome}
        </small>
      </div>
      {!compact && (
        <span
          className={"badge " + (event.outcome === "completed" ? "" : "muted")}
        >
          {event.outcome}
        </span>
      )}
    </div>
  );
}
export function Modal({
  title,
  open,
  onClose,
  children,
  dismissDisabled = false,
}: {
  title: string;
  open: boolean;
  onClose: () => void;
  children: ReactNode;
  dismissDisabled?: boolean;
}) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    if (open && !ref.current?.open) ref.current?.showModal();
    if (!open && ref.current?.open) ref.current?.close();
  }, [open]);
  return (
    <dialog
      ref={ref}
      className="rx-modal"
      aria-label={title}
      aria-busy={dismissDisabled}
      onCancel={(e) => {
        e.preventDefault();
        if (!dismissDisabled) onClose();
      }}
      onClick={(e) => {
        if (!dismissDisabled && e.target === e.currentTarget) onClose();
      }}
    >
      <div className="modal-heading">
        <h2>{title}</h2>
        <Button
          className="icon-button"
          aria-label="Close dialog"
          disabled={dismissDisabled}
          title={
            dismissDisabled
              ? "Operation in progress; wait to close"
              : "Close dialog"
          }
          onClick={onClose}
        >
          <Icon name="close" />
        </Button>
      </div>
      {open ? children : null}
    </dialog>
  );
}
export function AuditTable({ data }: { data: Snapshot }) {
  return data.audit.length ? (
    <div className="table-scroll">
      <table>
        <thead>
          <tr>
            <th>Action</th>
            <th>Record</th>
            <th>Outcome</th>
            <th>Time</th>
          </tr>
        </thead>
        <tbody>
          {data.audit.slice(0, 30).map((e) => (
            <tr key={e.id}>
              <td className="mono">{e.action}</td>
              <td className="mono muted-text">{e.entity_id.slice(0, 12)}</td>
              <td>
                <span
                  className={
                    e.outcome === "completed" ? "badge" : "badge muted"
                  }
                >
                  {e.outcome}
                </span>
              </td>
              <td className="muted-text">
                {new Date(e.timestamp).toLocaleString()}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  ) : (
    <div className="empty compact">
      <p>No actions recorded yet.</p>
    </div>
  );
}
