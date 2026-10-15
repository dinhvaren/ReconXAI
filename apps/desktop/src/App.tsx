import { useCallback, useEffect, useRef, useState } from "react";
import {
  request,
  errorText,
  type Snapshot,
  type ToolInventory,
  type AppTheme,
} from "./api";
import { ThemeProvider, useTheme } from "./theme";
import ThemeMenu from "./components/ThemeMenu";
import { Icon, Modal, type Perform } from "./components/ui";
import { navigation, routes } from "./components/navigation";
import ActivitySidebar from "./components/ActivitySidebar";
import Dashboard from "./pages/Dashboard";
import TargetsPage from "./pages/TargetsPage";
import ReversePage from "./pages/ReversePage";
import JobsPage from "./pages/JobsPage";
import TargetDialog from "./components/TargetDialog";
import QuickActions from "./components/QuickActions";
import { useJobEvents, type Target, type TargetKind } from "./research";
import type { AnalysisJob } from "./research";
import {
  Projects,
  SettingsPage,
  AiWorkspace,
  ToolManagerPage,
  PlannedModule,
} from "./pages/WorkspacePages";
const initials = (name: string) =>
  name
    .trim()
    .split(/\s+/)
    .slice(0, 2)
    .map((s) => s[0])
    .join("")
    .toUpperCase();
export default function App({ initialTheme }: { initialTheme?: AppTheme }) {
  return (
    <ThemeProvider initialTheme={initialTheme}>
      <Workbench />
    </ThemeProvider>
  );
}
function Workbench() {
  const theme = useTheme();
  const [page, setPage] = useState("Dashboard");
  const [data, setData] = useState<Snapshot | null>(null);
  useJobEvents((job) =>
    setData((current) =>
      current
        ? {
            ...current,
            jobs: [job, ...(current.jobs ?? []).filter((j) => j.id !== job.id)],
          }
        : current,
    ),
  );
  const [tools, setTools] = useState<ToolInventory[]>([]);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [busy, setBusy] = useState(false);
  const [targetOpen, setTargetOpen] = useState(false);
  const [editingTarget, setEditingTarget] = useState<Target>();
  const [newTargetKind, setNewTargetKind] = useState<TargetKind>("domain");
  const [reverseFile, setReverseFile] = useState<string>();
  const [importRequest, setImportRequest] = useState(0);
  const [navCollapsed, setNavCollapsed] = useState(false);
  const [groups, setGroups] = useState<string[]>([]);
  const [showActivity, setShowActivity] = useState(
    () => window.innerWidth >= 1340,
  );
  const [selected, setSelected] = useState<string | null>(() =>
    localStorage.getItem("reconx.selectedProject"),
  );
  const [notifications, setNotifications] = useState(false);
  const [query, setQuery] = useState("");
  const [searchOpen, setSearchOpen] = useState(false);
  const [searchIndex, setSearchIndex] = useState(0);
  const searchRef = useRef<HTMLInputElement>(null);
  const workspaceRef = useRef<HTMLElement>(null);
  const refresh = useCallback(async () => {
    const revision = theme.getRevision();
    const snapshot = await request<Snapshot>("snapshot");
    setData(snapshot);
    theme.syncFromStorage(snapshot.settings.theme, revision);
    try {
      setTools(await request<ToolInventory[]>("tools_list"));
    } catch {
      setTools([]);
    }
  }, [theme.getRevision, theme.syncFromStorage]);
  useEffect(() => {
    refresh().catch((e) => setError(errorText(e)));
  }, [refresh]);
  useEffect(() => {
    let closed = false;
    // Reconciliation and changes from another CLI process also update Dashboard.
    const timer = setInterval(() => {
      void request<AnalysisJob[]>("jobs_list", { project: null })
        .then((jobs) => {
          if (!closed)
            setData((current) => (current ? { ...current, jobs } : current));
        })
        .catch((e) => {
          if (!closed) setError(errorText(e));
        });
    }, 2000);
    return () => {
      closed = true;
      clearInterval(timer);
    };
  }, []);
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        searchRef.current?.focus();
        setSearchOpen(true);
      }
      if (e.key === "Escape") setSearchOpen(false);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
  const perform: Perform = async (action, message) => {
    setBusy(true);
    setError("");
    setNotice("");
    try {
      await action();
      await refresh();
      if (message) setNotice(message);
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };
  const navigate = (name: string) => {
    if (workspaceRef.current) workspaceRef.current.scrollTop = 0;
    setPage(name);
    setError("");
    setNotice("");
    setSearchOpen(false);
    setQuery("");
    setSearchIndex(0);
    const group = navigation.find((g) => g.items.some((i) => i.name === name));
    if (group) setGroups((current) => current.filter((g) => g !== group.group));
  };
  const selectProject = (id: string) => {
    setSelected(id);
    localStorage.setItem("reconx.selectedProject", id);
  };
  const projects = data?.projects.filter((p) => !p.archived) ?? [];
  const addTarget = (kind: TargetKind = "domain") => {
    setEditingTarget(undefined);
    setNewTargetKind(kind);
    setTargetOpen(true);
  };
  const project =
    selected === null ? projects[0] : projects.find((p) => p.id === selected);
  const provider =
    data?.providers.find((p) => p.id === data.settings.default_provider_id) ??
    data?.providers.find((p) => p.enabled);
  const needle = query.trim().toLowerCase();
  const searchResults = [
    ...projects
      .filter(
        (p) =>
          !needle ||
          `${p.name} ${p.description}`.toLowerCase().includes(needle),
      )
      .slice(0, 4)
      .map((p) => ({
        key: p.id,
        label: p.name,
        detail: "Project",
        icon: "folder",
        run: () => {
          selectProject(p.id);
          navigate("Projects");
        },
      })),
    ...(data?.providers ?? [])
      .filter(
        (p) => needle && `${p.name} ${p.model}`.toLowerCase().includes(needle),
      )
      .slice(0, 3)
      .map((p) => ({
        key: p.id,
        label: p.name,
        detail: p.model,
        icon: "ai",
        run: () => navigate("Settings"),
      })),
    ...routes
      .filter((r) => !needle || r.name.toLowerCase().includes(needle))
      .slice(0, needle ? 6 : 4)
      .map((r) => ({
        key: r.name,
        label: r.name,
        detail: r.implemented ? "Workspace" : "Planned module",
        icon: r.icon,
        run: () => navigate(r.name),
      })),
  ].slice(0, 9);
  const failed =
    data?.audit.filter((e) => e.outcome === "failed").slice(0, 5) ?? [];
  return (
    <div
      className={`app-shell ${navCollapsed ? "nav-collapsed" : ""} ${showActivity ? "activity-visible" : "activity-hidden"}`}
    >
      <aside className="icon-rail" aria-label="Workspace rail">
        <button
          className="rail-brand"
          title="ReconX AI home"
          aria-label="Open Dashboard"
          onClick={() => navigate("Dashboard")}
        >
          <span>
            R<span>×</span>
          </span>
        </button>
        <span className="rail-divider" />
        <button
          className={"rail-item " + (page === "Dashboard" ? "active" : "")}
          title="Local workspace"
          aria-label="Open workspace overview"
          onClick={() => navigate("Dashboard")}
        >
          <Icon name="layers" size={23} />
        </button>
        <div className="rail-projects">
          {projects.slice(0, 3).map((p, index) => (
            <button
              key={p.id}
              className={`rail-project tone-${index} ${project?.id === p.id ? "selected" : ""}`}
              title={p.name}
              aria-label={`Select project ${p.name}`}
              onClick={() => {
                selectProject(p.id);
                navigate("Projects");
              }}
            >
              <span>{initials(p.name)}</span>
            </button>
          ))}
        </div>
        <button
          className={"rail-item " + (page === "AI Workspace" ? "active" : "")}
          aria-label="Open AI Workspace"
          title="AI Workspace"
          onClick={() => navigate("AI Workspace")}
        >
          <Icon name="ai" size={23} />
        </button>
        <button
          className="rail-add"
          aria-label="Add project"
          title="Add a research project"
          onClick={() => navigate("Projects")}
        >
          <Icon name="plus" size={23} />
        </button>
        <div className="rail-bottom">
          <ThemeMenu location="Sidebar" rail />
          <button
            className="rail-item"
            title={navCollapsed ? "Expand navigation" : "Collapse navigation"}
            aria-label={
              navCollapsed ? "Expand navigation" : "Collapse navigation"
            }
            onClick={() => setNavCollapsed(!navCollapsed)}
          >
            <Icon name={navCollapsed ? "chevron" : "collapse"} />
          </button>
          <button
            className={"rail-item " + (page === "Settings" ? "active" : "")}
            title="Settings"
            aria-label="Open Settings"
            onClick={() => navigate("Settings")}
          >
            <Icon name="settings" size={21} />
          </button>
        </div>
      </aside>
      <aside className="navigation-sidebar" aria-label="Module navigation">
        <div className="navigation-heading">
          <span>Research workspace</span>
          <span className="navigation-edition">AI</span>
        </div>
        <nav aria-label="Primary navigation">
          {navigation.map((group) => (
            <section className="navigation-group" key={group.group}>
              <button
                className="navigation-group-label"
                aria-expanded={!groups.includes(group.group)}
                onClick={() =>
                  setGroups((current) =>
                    current.includes(group.group)
                      ? current.filter((g) => g !== group.group)
                      : [...current, group.group],
                  )
                }
              >
                <span>{group.group}</span>
                <Icon name="down" size={12} />
              </button>
              {!groups.includes(group.group) &&
                group.items.map((item) => (
                  <button
                    title={
                      item.implemented ? item.name : item.name + " · planned"
                    }
                    aria-label={
                      item.name === "Agent Workflows" ? "Workflows" : item.name
                    }
                    key={item.name}
                    aria-current={page === item.name ? "page" : undefined}
                    className={`nav-item ${page === item.name ? "active" : ""}`}
                    onClick={() => navigate(item.name)}
                  >
                    <Icon name={item.icon} size={17} />
                    <span>
                      {item.name === "Agent Workflows"
                        ? "Workflows"
                        : item.name}
                    </span>
                    {!item.implemented && (
                      <i title="Planned" aria-hidden="true" />
                    )}
                  </button>
                ))}
            </section>
          ))}
        </nav>
        <button
          className="navigation-status"
          aria-label="Configure AI provider"
          onClick={() => navigate("Settings")}
        >
          <span className="nav-provider-avatar">
            <Icon name="ai" size={19} />
          </span>
          <span>
            <b>
              {provider
                ? `${provider.name} · ${provider.model}`
                : "No model selected"}
            </b>
            <small>
              {provider
                ? "Configured · test in Settings"
                : "Choose a cloud or local model"}
            </small>
          </span>
          <Icon name="settings" size={15} />
        </button>
      </aside>
      <div className="main-workspace">
        <header className="workspace-toolbar">
          <div className="toolbar-location">
            <button
              className="breadcrumb-home"
              aria-label="Go to Dashboard"
              onClick={() => navigate("Dashboard")}
            >
              <Icon name="grid" size={15} />
            </button>
            <span>{page}</span>
          </div>
          <div className="workspace-search">
            <Icon name="search" size={15} />
            <input
              ref={searchRef}
              role="combobox"
              aria-label="Search workspace"
              aria-expanded={searchOpen}
              aria-controls="workspace-search-results"
              aria-activedescendant={
                searchOpen && searchResults[searchIndex]
                  ? `search-option-${searchIndex}`
                  : undefined
              }
              value={query}
              placeholder="Search your workspace"
              onFocus={() => setSearchOpen(true)}
              onChange={(e) => {
                setQuery(e.target.value);
                setSearchOpen(true);
                setSearchIndex(0);
              }}
              onKeyDown={(e) => {
                if (e.key === "ArrowDown") {
                  e.preventDefault();
                  setSearchIndex(
                    (i) => (i + 1) % Math.max(searchResults.length, 1),
                  );
                }
                if (e.key === "ArrowUp") {
                  e.preventDefault();
                  setSearchIndex(
                    (i) =>
                      (i + searchResults.length - 1) %
                      Math.max(searchResults.length, 1),
                  );
                }
                if (e.key === "Enter" && searchOpen) {
                  e.preventDefault();
                  searchResults[searchIndex]?.run();
                }
                if (e.key === "Escape") setSearchOpen(false);
              }}
            />
            <kbd>Ctrl K</kbd>
            {searchOpen && (
              <>
                <button
                  className="search-dismiss"
                  aria-label="Close search"
                  onClick={() => setSearchOpen(false)}
                />
                <div
                  id="workspace-search-results"
                  className="search-results"
                  role="listbox"
                  aria-label="Search results"
                >
                  {searchResults.length ? (
                    searchResults.map((r, i) => (
                      <button
                        id={`search-option-${i}`}
                        role="option"
                        aria-selected={i === searchIndex}
                        key={r.key}
                        onMouseEnter={() => setSearchIndex(i)}
                        onClick={r.run}
                      >
                        <Icon name={r.icon} />
                        <span>
                          <b>{r.label}</b>
                          <small>{r.detail}</small>
                        </span>
                        <Icon name="arrow" size={13} />
                      </button>
                    ))
                  ) : (
                    <p>No matching projects, models or modules.</p>
                  )}
                </div>
              </>
            )}
          </div>
          <select
            className="project-selector"
            aria-label="Current project"
            value={project?.id ?? ""}
            onChange={(e) => selectProject(e.target.value)}
          >
            <option value="">Local workspace</option>
            {projects.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name}
              </option>
            ))}
          </select>
          <div className="toolbar-actions">
            <QuickActions
              run={(action) => {
                if (action === "New Project") navigate("Projects");
                if (action === "Add Target") addTarget();
                if (action === "Import Repository") addTarget("git_repository");
                if (action === "Import File") {
                  setReverseFile(undefined);
                  setImportRequest((n) => n + 1);
                  navigate("Reverse Engineering");
                }
                if (action === "View Jobs") navigate("Analysis Jobs");
                if (action === "Open AI Workspace") navigate("AI Workspace");
                if (action === "Install Tools") navigate("Tool Manager");
              }}
            />
            <ThemeMenu location="Topbar" />
            <button
              className="icon-button notification-button"
              aria-label="Notifications"
              title="Notifications"
              onClick={() => setNotifications(true)}
            >
              <Icon name="bell" />
              {(notice || error || failed.length > 0) && <i />}
            </button>
            <button
              className={"icon-button " + (showActivity ? "selected" : "")}
              aria-label={
                showActivity ? "Hide activity panel" : "Show activity panel"
              }
              title={
                showActivity ? "Hide activity panel" : "Show activity panel"
              }
              onClick={() => setShowActivity(!showActivity)}
            >
              <Icon name="panel" />
            </button>
          </div>
        </header>
        <main ref={workspaceRef} className="workspace-content">
          {(error || theme.error) && (
            <div role="alert" className="banner error">
              <Icon name="flag" size={16} />
              <span>{error || theme.error}</span>
              <button
                aria-label="Dismiss error"
                onClick={() => {
                  setError("");
                  theme.clearError();
                }}
              >
                <Icon name="close" size={16} />
              </button>
            </div>
          )}
          {notice && (
            <div role="status" className="banner success">
              <Icon name="check" size={16} />
              <span>{notice}</span>
              <button
                aria-label="Dismiss notification"
                onClick={() => setNotice("")}
              >
                <Icon name="close" size={16} />
              </button>
            </div>
          )}
          {!data ? (
            <div className="empty loading-workspace">
              <div className="loading-spinner" />
              <h1>Opening your workspace</h1>
              <p>Connecting to the native Rust service.</p>
              <button
                className="button secondary"
                onClick={() => void perform(refresh)}
              >
                Retry connection
              </button>
            </div>
          ) : (
            <div className="page-view" key={page}>
              {page === "Dashboard" && (
                <Dashboard
                  data={data}
                  currentProject={project}
                  tools={tools}
                  navigate={navigate}
                  selectProject={selectProject}
                  onAddTarget={() => addTarget()}
                />
              )}
              {page === "Projects" && (
                <Projects
                  data={data}
                  busy={busy}
                  perform={perform}
                  onAddTarget={() => addTarget()}
                />
              )}
              {page === "Settings" && (
                <SettingsPage data={data} busy={busy} perform={perform} />
              )}
              {page === "AI Workspace" && (
                <AiWorkspace data={data} busy={busy} perform={perform} />
              )}
              {page === "Tool Manager" && (
                <ToolManagerPage busy={busy} perform={perform} />
              )}
              {page === "Targets & Scope" && (
                <TargetsPage
                  data={data}
                  projectId={project?.id}
                  busy={busy}
                  perform={perform}
                  onAdd={() => addTarget()}
                  onEdit={(target) => {
                    setEditingTarget(target);
                    setTargetOpen(true);
                  }}
                  onReverse={(id, projectId) => {
                    selectProject(projectId);
                    setReverseFile(id);
                    navigate("Reverse Engineering");
                  }}
                />
              )}
              {page === "Reverse Engineering" && (
                <ReversePage
                  data={data}
                  projectId={project?.id}
                  initialFileId={reverseFile}
                  busy={busy}
                  perform={perform}
                  requestImport={importRequest}
                />
              )}
              {page === "Analysis Jobs" && (
                <JobsPage
                  data={data}
                  busy={busy}
                  perform={perform}
                  onOpenFile={(id, projectId) => {
                    selectProject(projectId);
                    setReverseFile(id);
                    navigate("Reverse Engineering");
                  }}
                />
              )}
              {page !== "Analysis Jobs" &&
                !routes.find((r) => r.name === page)?.implemented && (
                  <PlannedModule name={page} />
                )}
            </div>
          )}
        </main>
        <footer className="workspace-footer">
          {theme.saving && <span aria-live="polite">Saving appearance…</span>}
          <span>
            <i className={data ? "connection-dot" : "connection-dot waiting"} />
            {data ? "Local workspace connected" : "Connecting to workspace"}
          </span>
          <button
            className="text-button"
            disabled={busy}
            onClick={() => void perform(refresh, "Workspace refreshed")}
          >
            <Icon name="clock" size={12} />
            Refresh
          </button>
          <span>
            ReconX AI <b>0.1</b>
          </span>
        </footer>
      </div>
      {data && showActivity && (
        <ActivitySidebar
          data={data}
          project={project}
          navigate={navigate}
          notice={notice}
          error={error}
        />
      )}
      {targetOpen && (
        <TargetDialog
          open={targetOpen}
          target={editingTarget}
          initialKind={newTargetKind}
          projects={projects}
          projectId={project?.id}
          onClose={() => setTargetOpen(false)}
          onSaved={() =>
            perform(
              refresh,
              editingTarget ? "Target updated" : "Target created",
            )
          }
        />
      )}
      <Modal
        title="Notifications"
        open={notifications}
        onClose={() => setNotifications(false)}
      >
        <div className="notification-content">
          {notice && (
            <p className="notification-entry">
              <Icon name="check" />
              {notice}
            </p>
          )}
          {error && (
            <p className="notification-entry error">
              <Icon name="flag" />
              {error}
            </p>
          )}
          {failed.map((e) => (
            <div className="notification-entry" key={e.id}>
              <Icon name="clock" />
              <div>
                <b>{e.action} failed</b>
                <small>{new Date(e.timestamp).toLocaleString()}</small>
              </div>
            </div>
          ))}
          {!notice && !error && !failed.length && (
            <div className="empty compact">
              <Icon name="bell" />
              <h3>You're up to date</h3>
              <p>No application notifications.</p>
            </div>
          )}
        </div>
      </Modal>
    </div>
  );
}
