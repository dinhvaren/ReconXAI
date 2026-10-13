import { useEffect, useRef, type ReactNode } from "react";
import type { Audit, Snapshot } from "../api";
export type Perform = (
  action: () => Promise<unknown>,
  message?: string,
) => Promise<void>;
export const iconPaths: Record<string, string> = {
  grid: "M3 3h7v7H3z M14 3h7v7h-7z M3 14h7v7H3z M14 14h7v7h-7z",
  folder: "M3 7V4h6l3 3h9v13H3z",
  ai: "M12 3l2.4 6.6L21 12l-6.6 2.4L12 21l-2.4-6.6L3 12l6.6-2.4z",
  settings:
    "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8 M12 2v3 M12 19v3 M2 12h3 M19 12h3 M5 5l2 2 M17 17l2 2 M5 19l2-2 M17 7l2-2",
  shield: "M12 2L3 6v6c0 5 9 10 9 10s9-5 9-10V6z M8 12l3 3 5-6",
  sun: "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8 M12 2v2 M12 20v2 M2 12h2 M20 12h2 M5 5l1.5 1.5 M17.5 17.5L19 19 M5 19l1.5-1.5 M17.5 6.5L19 5",
  moon: "M20.7 13.2A9 9 0 0 1 10.8 3.3 9 9 0 1 0 20.7 13.2z",
  search: "M10.5 3a7.5 7.5 0 1 0 0 15 7.5 7.5 0 0 0 0-15 M16 16l5 5",
  bell: "M6 9a6 6 0 0 1 12 0v7l2 2H4l2-2z M10 21h4",
  close: "M5 5l14 14 M19 5L5 19",
  plus: "M12 4v16 M4 12h16",
  arrow: "M5 12h14 M14 7l5 5-5 5",
  chevron: "M8 4l8 8-8 8",
  down: "M5 9l7 7 7-7",
  collapse: "M14 5l-7 7 7 7",
  panel: "M3 4h18v16H3z M15 4v16",
  globe:
    "M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20 M2 12h20 M12 2c-6 5-6 15 0 20 6-5 6-15 0-20",
  code: "M8 6l-6 6 6 6 M16 6l6 6-6 6 M14 3l-4 18",
  api: "M4 4h6v6H4z M14 14h6v6h-6z M14 4h6 M17 1v6 M1 17h6 M4 14v6 M10 7h7v7 M7 10v7h7",
  network: "M9 3h6v6H9z M3 16h6v6H3z M15 16h6v6h-6z M12 9v4 M6 16v-3h12v3",
  binary: "M4 3h16v18H4z M8 7h2v4H8z M14 7h2 M8 15h2 M14 13h2v4h-2z",
  phone: "M7 2h10v20H7z M10 18h4",
  cloud: "M7 18a5 5 0 0 1-1-10 7 7 0 0 1 13 1 4.5 4.5 0 0 1-1 9z",
  pipeline: "M3 4h6v6H3z M15 4h6v6h-6z M9 7h6 M18 10v6H9 M3 13h6v8H3z",
  workflow: "M4 3h6v6H4z M14 15h6v6h-6z M7 9v9h7 M10 6h7v9",
  plug: "M8 2v5 M16 2v5 M5 7h14v5a7 7 0 0 1-14 0z M12 19v3",
  terminal: "M3 4h18v16H3z M6 8l4 4-4 4 M12 16h5",
  flag: "M5 22V3h13l-3 4 3 4H5",
  file: "M5 2h9l5 5v15H5z M14 2v6h5 M8 12h8 M8 16h8",
  clock: "M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20 M12 6v6l4 2",
  check: "M4 12l5 5L20 6",
  archive: "M3 3h18v5H3z M5 8v13h14V8 M9 12h6",
  layers: "M12 2L2 7l10 5 10-5z M2 12l10 5 10-5 M2 17l10 5 10-5",
};
export function Icon({
  name = "grid",
  size = 18,
}: {
  name?: string;
  size?: number;
}) {
  return (
    <svg
      aria-hidden="true"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      strokeLinejoin="round"
      strokeLinecap="round"
    >
      <path d={iconPaths[name] ?? iconPaths.shield} />
    </svg>
  );
}
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
        <p className="eyebrow">{eyebrow}</p>
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
    "ai.models": "Model discovery",
    "ai.chat": "Assistant response",
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
}: {
  title: string;
  open: boolean;
  onClose: () => void;
  children: ReactNode;
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
      onCancel={(e) => {
        e.preventDefault();
        onClose();
      }}
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div className="modal-heading">
        <h2>{title}</h2>
        <button
          className="icon-button"
          aria-label="Close dialog"
          onClick={onClose}
        >
          <Icon name="close" />
        </button>
      </div>
      {children}
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
