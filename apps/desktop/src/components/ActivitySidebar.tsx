import { providerKinds, type Project, type Snapshot } from "../api";
import { ActivityItem, Icon, StatusIndicator } from "./ui";
export default function ActivitySidebar({
  data,
  project,
  navigate,
  error,
  notice,
}: {
  data: Snapshot;
  project?: Project;
  navigate: (page: string) => void;
  error: string;
  notice: string;
}) {
  const provider =
    data.providers.find((p) => p.id === data.settings.default_provider_id) ??
    data.providers.find((p) => p.enabled);
  const providerEvent = provider
    ? data.audit.find(
        (e) => e.entity_id === provider.id && e.action.startsWith("ai."),
      )
    : undefined;
  const aiEvents = data.audit
    .filter((e) => e.action.startsWith("ai."))
    .slice(0, 4);
  return (
    <aside className="activity-sidebar" aria-label="Workspace activity">
      <div className="activity-toolbar">
        <span>Workspace activity</span>
        <button
          className="icon-button"
          title="Open settings"
          aria-label="Activity settings"
          onClick={() => navigate("Settings")}
        >
          <Icon name="settings" size={18} />
        </button>
      </div>
      <div className="activity-scroll">
        <div className="workspace-profile">
          <div className="profile-hex">
            <span>
              R<span>×</span>
            </span>
          </div>
          <h2 aria-label={project ? "Workspace: " + project.name : undefined}>
            {project?.name ?? "Local workspace"}
          </h2>
          <p>
            {project
              ? "Selected research project"
              : "ReconX AI · personal workspace"}
          </p>
          <span className="workspace-location">
            <Icon name="folder" size={13} />
            SQLite workspace
          </span>
        </div>
        <section className="activity-section">
          <div className="activity-section-title">
            <h3>AI connection</h3>
            <button
              className="text-button"
              onClick={() => navigate("Settings")}
            >
              Manage
            </button>
          </div>
          <button
            className="provider-activity-card"
            onClick={() => navigate("Settings")}
          >
            <div className="provider-avatar">
              <Icon name="ai" size={23} />
            </div>
            <div>
              <b>
                {provider
                  ? providerKinds[provider.kind].label
                  : "Choose your model"}
              </b>
              <small>{provider?.model ?? "Cloud or local inference"}</small>
            </div>
          </button>
          <div className="provider-state">
            <StatusIndicator
              label={
                providerEvent
                  ? providerEvent.outcome === "completed"
                    ? "Last request succeeded"
                    : "Last request " + providerEvent.outcome
                  : provider
                    ? "Configured · untested"
                    : "Not configured"
              }
              tone={providerEvent?.outcome === "completed" ? "good" : "neutral"}
            />
          </div>
        </section>
        <section className="activity-section">
          <div className="activity-section-title">
            <h3>AI activity</h3>
            <button
              className="text-button"
              onClick={() => navigate("AI Workspace")}
            >
              Open
            </button>
          </div>
          {aiEvents.length ? (
            aiEvents.map((e) => <ActivityItem event={e} compact key={e.id} />)
          ) : (
            <div className="activity-empty">
              <Icon name="ai" />
              <p>
                No AI requests recorded.
                <br />
                Start a conversation to begin.
              </p>
            </div>
          )}
        </section>
        <section className="activity-section">
          <div className="activity-section-title">
            <h3>Research services</h3>
          </div>
          <button
            className="service-status"
            onClick={() => navigate("Agent Workflows")}
          >
            <Icon name="workflow" />
            <span>
              Agent workflows<small>Execution not implemented</small>
            </span>
            <span className="micro-badge">Planned</span>
          </button>
          <button
            className="service-status"
            onClick={() => navigate("MCP Hub")}
          >
            <Icon name="plug" />
            <span>
              MCP gateway<small>Connections not implemented</small>
            </span>
            <span className="micro-badge">Planned</span>
          </button>
          <button
            className="service-status"
            onClick={() => navigate("Findings")}
          >
            <Icon name="flag" />
            <span>
              Findings & jobs<small>Storage and scheduler planned</small>
            </span>
            <Icon name="chevron" size={12} />
          </button>
        </section>
        <section className="activity-section">
          <div className="activity-section-title">
            <h3>Recent changes</h3>
          </div>
          {data.audit.slice(0, 3).map((e) => (
            <ActivityItem event={e} compact key={e.id} />
          ))}
          {!data.audit.length && (
            <p className="activity-empty-copy">
              Your project and configuration changes will appear here.
            </p>
          )}
        </section>
        {(notice || error) && (
          <section className="activity-section">
            <div className="activity-section-title">
              <h3>Notification</h3>
            </div>
            <p className={error ? "activity-note error" : "activity-note"}>
              {error || notice}
            </p>
          </section>
        )}
      </div>
      <div className="activity-footer">
        <Icon name="shield" size={14} />
        <span>Local workspace · v{data.version}</span>
      </div>
    </aside>
  );
}
