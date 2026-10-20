import type { Project, Snapshot, ToolInventory } from "../api";
import { ActivityItem, Heading, Icon } from "../components/ui";
import { Button, EmptyState, PanelHeader } from "../components/controls";

export default function Dashboard({
  data,
  currentProject,
  navigate,
  selectProject,
  onAddTarget,
}: {
  data: Snapshot;
  currentProject?: Project;
  tools: ToolInventory[];
  navigate: (page: string) => void;
  selectProject: (id: string) => void;
  onAddTarget: () => void;
}) {
  const projects = data.projects.filter((project) => !project.archived);
  const jobs = (data.jobs ?? []).filter(
    (job) => !currentProject || job.project_id === currentProject.id,
  );
  return (
    <div className="dashboard">
      <Heading
        eyebrow="Workspace"
        title="Workspace overview"
        description={
          currentProject?.name ??
          "Your local research projects and recent work."
        }
        action={
          currentProject ? (
            <Button variant="primary" onClick={onAddTarget}>
              <Icon name="plus" />
              Add Target
            </Button>
          ) : undefined
        }
      />
      {!projects.length ? (
        <EmptyState
          title="Create your first project"
          description="Keep targets, evidence and research history together in a local workspace."
          action={
            <Button variant="primary" onClick={() => navigate("Projects")}>
              <Icon name="plus" />
              Create project
            </Button>
          }
        />
      ) : (
        <section className="panel overview-projects">
          <PanelHeader title="Projects" />
          {projects.slice(0, 6).map((project) => (
            <button
              key={project.id}
              className="overview-project"
              onClick={() => {
                selectProject(project.id);
                navigate("Projects");
              }}
            >
              <Icon name="folder" size={20} />
              <span>
                <b>{project.name}</b>
                <small>{project.description || "No project notes"}</small>
                <span className="overview-counts">
                  <span>
                    {data.targets?.filter(
                      (target) => target.project_id === project.id,
                    ).length ?? 0}{" "}
                    targets
                  </span>
                  <span>
                    Updated {new Date(project.updated_at).toLocaleDateString()}
                  </span>
                </span>
              </span>
              <Icon name="chevron" size={16} />
            </button>
          ))}
        </section>
      )}
      <div className="overview-grid">
        <section className="panel">
          <PanelHeader title="Recent jobs" />
          <div className="overview-list">
            {jobs.slice(0, 6).map((job) => (
              <div className="overview-job" key={job.id}>
                <span>
                  {job.kind.replaceAll("_", " ")}
                  <small>{job.id.slice(0, 8)}</small>
                </span>
                <span className="badge muted">{job.status}</span>
              </div>
            ))}
            {!jobs.length && (
              <p className="overview-help">
                No analysis jobs yet. Import a file or select a target to begin.
              </p>
            )}
            {!!jobs.length && (
              <Button variant="ghost" onClick={() => navigate("Analysis Jobs")}>
                View job history
                <Icon name="arrow" />
              </Button>
            )}
          </div>
        </section>
        <section className="panel">
          <PanelHeader title="Recent activity" />
          <div className="overview-list">
            {data.audit.slice(0, 5).map((event) => (
              <ActivityItem event={event} key={event.id} compact />
            ))}
            {!data.audit.length && (
              <p className="overview-help">
                Project and configuration changes will appear here.
              </p>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}
