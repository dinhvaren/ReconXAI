import type { Project, Snapshot, ToolInventory } from "../api";
import {
  ActivityItem,
  Icon,
  SectionTitle,
  StatusIndicator,
} from "../components/ui";
export default function Dashboard({
  data,
  currentProject,
  tools,
  navigate,
  selectProject,
}: {
  data: Snapshot;
  currentProject?: Project;
  tools: ToolInventory[];
  navigate: (page: string) => void;
  selectProject: (id: string) => void;
}) {
  const projects = data.projects.filter((p) => !p.archived);
  const featured = currentProject
    ? [
        currentProject,
        ...projects.filter((p) => p.id !== currentProject.id),
      ].slice(0, 2)
    : projects.slice(0, 2);
  const cards = featured.map((project, index) => ({
    title: project.name,
    description: project.description || "Add research notes to this project.",
    art: index ? "prism" : "orbital",
    project,
  }));
  const configured = data.providers.filter((p) => p.enabled);
  return (
    <div className="dashboard">
      <div className="dashboard-intro">
        <div>
          <p className="eyebrow">ReconX / Your workspace</p>
          <h1>Workspace overview</h1>
        </div>
        <span className="dashboard-summary">
          {projects.length} projects <i /> {configured.length} enabled models
        </span>
      </div>
      <section className="workspace-feature" aria-label="Featured workspace">
        <div className="feature-content">
          <span className="feature-kicker">Research, in one place</span>
          <h2>
            Your security
            <br />
            research workspace
          </h2>
          <p>
            {currentProject
              ? currentProject.name
              : "A place for the questions worth investigating."}
          </p>
          <div className="feature-actions">
            <button
              className="button feature-button"
              onClick={() => navigate("Projects")}
            >
              {currentProject ? "Open project" : "Create a project"}
              <Icon name="arrow" size={16} />
            </button>
            <button
              className="feature-link"
              onClick={() => navigate("AI Workspace")}
            >
              Open AI workspace
            </button>
          </div>
        </div>
        <span className="feature-tag">
          <Icon name="shield" size={13} />
          {data.settings.security_mode.replaceAll("_", " ")}
        </span>
      </section>
      <SectionTitle
        title="Featured projects"
        action={
          <button className="text-button" onClick={() => navigate("Projects")}>
            See all
            <Icon name="arrow" size={13} />
          </button>
        }
      />
      <div className="featured-projects">
        {cards.map(({ title, description, art, project }) => (
          <button
            className="feature-card"
            key={project.id}
            onClick={() => {
              selectProject(project.id);
              navigate("Projects");
            }}
          >
            <div className={`feature-card-image cover-${art}`}>
              <span className="cover-label">Research project</span>
            </div>
            <div className="project-card-emblem">
              <Icon name="folder" size={22} />
            </div>
            <div className="feature-card-body">
              <h3>{title}</h3>
              <p>{description}</p>
              <div className="project-card-metadata">
                <span>
                  <Icon name="shield" size={12} />
                  Scope not configured
                </span>
                <span>{new Date(project.updated_at).toLocaleDateString()}</span>
              </div>
              <div className="profile-progress">
                <span>Project profile</span>
                <span>{project.description ? 2 : 1}/2</span>
                <progress
                  aria-label={`${title} project profile completeness`}
                  value={project.description ? 2 : 1}
                  max={2}
                />
              </div>
            </div>
          </button>
        ))}
        {cards.length === 0 && (
          <button className="feature-card" onClick={() => navigate("Projects")}>
            <div className="feature-card-image cover-orbital">
              <span className="cover-label">Start a project</span>
            </div>
            <div className="project-card-emblem">
              <Icon name="folder" size={22} />
            </div>
            <div className="feature-card-body">
              <h3>Your next investigation</h3>
              <p>
                Keep research notes and project history together in a persistent
                local workspace.
              </p>
              <span className="card-cta">
                Create your first project
                <Icon name="arrow" size={14} />
              </span>
            </div>
          </button>
        )}
        {cards.length < 2 && (
          <button
            className="feature-card model-card"
            onClick={() => navigate("Settings")}
          >
            <div className="feature-card-image cover-prism">
              <span className="cover-label">Your models, your choice</span>
            </div>
            <div className="project-card-emblem">
              <Icon name="ai" size={22} />
            </div>
            <div className="feature-card-body">
              <h3>
                {configured.length
                  ? "Your analysis companions"
                  : "Connect your preferred model"}
              </h3>
              <p>
                {configured.length
                  ? `${configured.length} enabled model configurations. Manage providers, gateways and local inference.`
                  : "Bring a cloud gateway, Ollama or LM Studio to your research."}
              </p>
              <span className="card-cta">
                Configure AI
                <Icon name="arrow" size={14} />
              </span>
            </div>
          </button>
        )}
      </div>
      <SectionTitle
        title="Active research"
        action={<span className="section-caption">Pick up a thread</span>}
      />
      <div className="research-shortcuts">
        {[
          {
            title: "Reconnaissance",
            icon: "globe",
            page: "Reconnaissance",
            detail: "Target intelligence",
            status: "Planned",
            tone: "gold",
          },
          {
            title: "API Security",
            icon: "api",
            page: "API Security",
            detail: "Traffic & endpoint review",
            status: "Planned",
            tone: "pink",
          },
          {
            title: "Reverse Engineering",
            icon: "binary",
            page: "Reverse Engineering",
            detail: "Local binary workspace",
            status: "Planned",
            tone: "violet",
          },
          {
            title: "AI Analysis",
            icon: "ai",
            page: "AI Workspace",
            detail: configured.length
              ? "Use a configured model"
              : "Select a model to begin",
            status: configured.length ? "Configured" : "Not configured",
            tone: "rose",
          },
        ].map((item) => (
          <button
            key={item.page}
            className={`research-card ${item.tone}`}
            onClick={() => navigate(item.page)}
          >
            <div className="research-art">
              <Icon name={item.icon} size={28} />
              <i />
              <i />
            </div>
            <div>
              <h3>{item.title}</h3>
              <p>{item.detail}</p>
              <span>{item.status}</span>
            </div>
            <Icon name="chevron" size={14} />
          </button>
        ))}
      </div>
      <div className="dashboard-lower">
        <section className="activity-feed">
          <SectionTitle
            title="Recent activity"
            action={
              <span className="section-caption">Persisted audit history</span>
            }
          />
          {data.audit.length ? (
            data.audit
              .slice(0, 5)
              .map((event) => <ActivityItem event={event} key={event.id} />)
          ) : (
            <div className="inline-empty">
              <Icon name="clock" />
              <div>
                <b>A fresh research workspace</b>
                <p>Project, model and tool actions appear here as you work.</p>
              </div>
            </div>
          )}
        </section>
        <section className="workspace-tools">
          <SectionTitle
            title="Local toolkit"
            action={
              <button
                className="text-button"
                onClick={() => navigate("Tool Manager")}
              >
                Manage
                <Icon name="arrow" size={13} />
              </button>
            }
          />
          {tools.length ? (
            tools.map((tool) => (
              <button
                key={tool.manifest.id}
                className="tool-overview"
                onClick={() => navigate("Tool Manager")}
              >
                <div className="tool-emblem">
                  <Icon name="terminal" size={23} />
                </div>
                <div>
                  <h3>{tool.manifest.name}</h3>
                  <p>
                    {tool.manifest.version} · {tool.manifest.platform}
                  </p>
                  <StatusIndicator
                    label={tool.status.replaceAll("_", " ")}
                    tone={tool.status === "tested" ? "good" : "neutral"}
                  />
                </div>
              </button>
            ))
          ) : (
            <div className="inline-empty">
              <Icon name="terminal" />
              <p>Open Tool Manager to inspect supported local utilities.</p>
            </div>
          )}
          <div className="jobs-empty">
            <Icon name="workflow" size={15} />
            <div>
              <b>Recent jobs</b>
              <p>A persistent job scheduler is not available in this build.</p>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}
