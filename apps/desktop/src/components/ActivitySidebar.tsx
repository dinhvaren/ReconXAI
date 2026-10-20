import type { Project, Snapshot } from "../api";
import { ActivityItem } from "./ui";
import { Drawer, PanelHeader } from "./controls";

export default function ActivitySidebar({
  data,
  project,
  error,
  notice,
  onClose,
}: {
  data: Snapshot;
  project?: Project;
  error: string;
  notice: string;
  onClose: () => void;
}) {
  const jobs = (data.jobs ?? []).filter(
    (job) => !project || job.project_id === project.id,
  );
  return (
    <Drawer
      className="activity-sidebar"
      title="Workspace activity"
      closeLabel="Hide activity panel"
      onClose={onClose}
    >
      <div className="activity-scroll">
        {(notice || error) && (
          <section className="activity-section">
            <h3>{error ? "Operation failed" : "Notification"}</h3>
            <p>{error || notice}</p>
          </section>
        )}
        <section className="activity-section">
          <PanelHeader title="Jobs" />
          {jobs.slice(0, 5).map((job) => (
            <div className="overview-job" key={job.id}>
              <span>
                {job.kind.replaceAll("_", " ")}
                <small>{job.id.slice(0, 8)}</small>
              </span>
              <span className="badge muted">{job.status}</span>
            </div>
          ))}
          {!jobs.length && (
            <p className="helper">
              No jobs in {project?.name ?? "this workspace"}.
            </p>
          )}
        </section>
        <section className="activity-section">
          <PanelHeader title="Recent changes" />
          {data.audit.slice(0, 15).map((event) => (
            <ActivityItem key={event.id} event={event} compact />
          ))}
          {!data.audit.length && <p className="helper">No changes recorded.</p>}
        </section>
      </div>
    </Drawer>
  );
}
