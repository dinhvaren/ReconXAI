import { Button } from "./components/controls";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  request,
  errorText,
  type Snapshot,
  type ToolInventory,
  type AppTheme,
} from "./api";
import { ThemeProvider, useTheme } from "./theme";
import { WorkspaceProvider, useWorkspace } from "./workspace";
import ResearchPage from "./pages/ResearchPage";
import ThemeMenu from "./components/ThemeMenu";
import AiProviderManager from "./components/AiProviderManager";
import { Heading } from "./components/ui";
import { Icon, type Perform } from "./components/ui";
import { navigation, routes } from "./components/navigation";
import ActivitySidebar from "./components/ActivitySidebar";
import Dashboard from "./pages/Dashboard";
import TargetsPage from "./pages/TargetsPage";
import ReversePage from "./pages/ReversePage";
import { McpPage } from "./pages/McpPage";
import { OrchestrationPage } from "./pages/OrchestrationPage";
import JobsPage from "./pages/JobsPage";
import RecordsPage from "./pages/RecordsPage";
import ReconPage from "./pages/ReconPage";
import ExtensionsPage from "./pages/ExtensionsPage";
import HttpPage from "./pages/HttpPage";
import TargetDialog from "./components/TargetDialog";
import QuickActions from "./components/QuickActions";
import { useJobEvents, type Target, type TargetKind } from "./research";
import type { ImportedFile } from "./research";
import {
  Projects,
  SettingsPage,
  AiWorkspace,
  ToolManagerPage,
  PlannedModule,
} from "./pages/WorkspacePages";
export default function App({ initialTheme }: { initialTheme?: AppTheme }) {
  return (
    <ThemeProvider initialTheme={initialTheme}>
      <WorkspaceProvider>
        <Workbench />
      </WorkspaceProvider>
    </ThemeProvider>
  );
}
function Workbench() {
  const theme = useTheme();
  const workspace = useWorkspace()!;
  const [researchProvider, setResearchProvider] = useState<string>();
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
  const [findingCreateRequest, setFindingCreateRequest] = useState(0);
  const [mcpAddRequested, setMcpAddRequested] = useState(false);
  const [editingTarget, setEditingTarget] = useState<Target>();
  const [newTargetKind, setNewTargetKind] = useState<TargetKind>("domain");
  const [reconTarget, setReconTarget] = useState<string>();
  const [reverseFile, setReverseFile] = useState<string>();
  const [importRequest, setImportRequest] = useState(0);
  const [navCollapsed, setNavCollapsed] = useState(
    () => localStorage.getItem("reconx.navCollapsed") === "true",
  );
  const [groups, setGroups] = useState<string[]>([]);
  const [showActivity, setShowActivity] = useState(
    () => localStorage.getItem("reconx.activityOpen") === "true",
  );
  const selected = workspace.selection.project_id;
  useEffect(() => {
    localStorage.setItem("reconx.activityOpen", String(showActivity));
  }, [showActivity]);
  useEffect(() => {
    localStorage.setItem("reconx.navCollapsed", String(navCollapsed));
  }, [navCollapsed]);
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
      void request<Snapshot>("snapshot")
        .then((snapshot) => {
          if (!closed) setData(snapshot);
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
    void workspace
      .select(
        id || null,
        id === selected ? workspace.selection.target_id : null,
      )
      .catch(() => {});
  };
  const openImportedFile = (id: string, projectId: string) =>
    void perform(async () => {
      const file = (
        await request<ImportedFile[]>("files_list", { project: projectId })
      ).find((f) => f.id === id);
      if (!file)
        throw new Error("Imported file is no longer available in this project");
      await workspace.select(file.project_id, file.target_id);
      setReverseFile(id);
      navigate("Reverse Engineering");
    });
  const projects = data?.projects.filter((p) => !p.archived) ?? [];
  useEffect(() => {
    if (
      !workspace.ready ||
      workspace.saving ||
      workspace.selection.revision !== 0 ||
      selected ||
      !data
    )
      return;
    const active = data.projects.filter((p) => !p.archived);
    const previous = localStorage.getItem("reconx.selectedProject");
    const initial = active.find((p) => p.id === previous) ?? active[0];
    if (initial) void workspace.select(initial.id).catch(() => {});
  }, [
    workspace.ready,
    workspace.saving,
    workspace.selection.revision,
    selected,
    data,
  ]);
  useEffect(() => {
    const open = (event: Event) => {
      setResearchProvider(
        (event as CustomEvent<{ providerId?: string }>).detail?.providerId,
      );
      setPage("AI Research");
      if (workspaceRef.current) workspaceRef.current.scrollTop = 0;
    };
    window.addEventListener("reconx-start-research", open);
    return () => window.removeEventListener("reconx-start-research", open);
  }, []);
  const addTarget = (kind: TargetKind = "domain") => {
    setEditingTarget(undefined);
    setNewTargetKind(kind);
    setTargetOpen(true);
  };
  const project = projects.find((p) => p.id === selected);
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
  return (
    <div
      className={`app-shell ${navCollapsed ? "nav-collapsed" : ""} ${showActivity ? "activity-visible" : "activity-hidden"}`}
    >
      <aside className="icon-rail" aria-label="Workspace rail">
        <div className="rail-brand" aria-label="ReconX AI">
          <span>
            R<span>×</span>
          </span>
        </div>
        <div className="rail-bottom">
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
            aria-label="Settings"
            onClick={() => navigate("Settings")}
          >
            <Icon name="settings" size={21} />
          </button>
          <ThemeMenu />
        </div>
      </aside>
      <aside className="navigation-sidebar" aria-label="Module navigation">
        <div className="navigation-heading">
          <span>ReconX AI</span>
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
      </aside>
      <div className="main-workspace">
        <header className="workspace-toolbar">
          <div className="toolbar-location">
            <Icon name="grid" size={15} />
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
          <select
            className="project-selector"
            aria-label="Current target"
            value={workspace.selection.target_id ?? ""}
            disabled={!project || workspace.saving}
            onChange={(e) =>
              void workspace
                .select(selected, e.target.value || null)
                .catch(() => {})
            }
          >
            <option value="">Select target</option>
            {(data?.targets ?? [])
              .filter((t) => t.project_id === selected)
              .map((t) => (
                <option key={t.id} value={t.id}>
                  {t.name}
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
                if (action === "Create Finding") {
                  if (data?.projects.some((p) => !p.archived))
                    setFindingCreateRequest((n) => n + 1);
                  else setNotice("Create a project before recording findings");
                  navigate("Findings");
                }
                if (action === "Start Recon") {
                  setReconTarget(undefined);
                  navigate("Reconnaissance");
                }
                if (action === "Start API Testing") navigate("API Security");
                if (action === "Start AI Research") navigate("AI Research");
                if (action === "Export Report") navigate("Reports");
                if (action === "View Jobs") navigate("Analysis Jobs");
                if (action === "Open AI Workspace") navigate("AI Workspace");
                if (action === "Install Tools") navigate("Tool Manager");
                if (action === "Add MCP Server") {
                  setMcpAddRequested(true);
                  navigate("MCP Hub");
                }
                if (action === "Plan local analysis")
                  navigate("Agent Workflows");
              }}
            />
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
          {(error || theme.error || workspace.error) && (
            <div role="alert" className="banner error">
              <Icon name="flag" size={16} />
              <span>{error || theme.error || workspace.error}</span>
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
              <Button
                className="button secondary"
                onClick={() => void perform(refresh)}
              >
                Retry connection
              </Button>
            </div>
          ) : (
            <div
              className="page-view"
              key={`${page}:${project?.id ?? "workspace"}`}
            >
              {(["Evidence", "Findings", "Reports"] as const).map((name) =>
                page === name ? (
                  <RecordsPage
                    key={name}
                    page={name}
                    data={data}
                    projectId={project?.id}
                    perform={perform}
                    busy={busy}
                    newFindingRequest={findingCreateRequest}
                    onFindingRequestConsumed={() => setFindingCreateRequest(0)}
                  />
                ) : null,
              )}
              {(page === "Web Security" || page === "API Security") && (
                <HttpPage
                  page={page}
                  data={data}
                  projectId={project?.id}
                  perform={perform}
                  busy={busy}
                  navigate={navigate}
                />
              )}
              {page === "Reconnaissance" && (
                <ReconPage
                  data={data}
                  projectId={project?.id}
                  initialTarget={reconTarget}
                  perform={perform}
                  busy={busy}
                  onAddTarget={() => addTarget("url")}
                  navigate={(page, project) => {
                    if (project) selectProject(project);
                    navigate(page);
                  }}
                />
              )}
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
              {page === "AI Providers" && (
                <>
                  <Heading
                    eyebrow="AI"
                    title="AI Providers"
                    description="Configure cloud providers and local inference. Saving a configuration does not test a connection."
                  />
                  <AiProviderManager
                    data={data}
                    busy={busy}
                    perform={perform}
                  />
                </>
              )}
              {page === "Settings" && (
                <SettingsPage data={data} busy={busy} perform={perform} />
              )}
              {page === "AI Workspace" && (
                <AiWorkspace
                  data={data}
                  busy={busy}
                  perform={perform}
                  onTools={() => navigate("Agent Workflows")}
                />
              )}
              {page === "MCP Hub" && (
                <McpPage
                  addRequested={mcpAddRequested}
                  onAddHandled={() => setMcpAddRequested(false)}
                  data={data}
                  projectId={project?.id}
                  busy={busy}
                  perform={perform}
                />
              )}
              {page === "Agent Workflows" && (
                <OrchestrationPage
                  data={data}
                  projectId={project?.id}
                  busy={busy}
                  perform={perform}
                />
              )}
              {page === "AI Research" && (
                <ResearchPage
                  key={
                    workspace.selection.target_id ??
                    workspace.selection.project_id ??
                    "none"
                  }
                  data={data}
                  providerId={researchProvider}
                  navigate={navigate}
                />
              )}
              {page === "Tool Manager" && (
                <ToolManagerPage busy={busy} perform={perform} />
              )}
              {page === "Targets & Scope" && (
                <TargetsPage
                  onRecon={(id, project) => {
                    setReconTarget(id);
                    void workspace.select(project, id).catch(() => {});
                    navigate("Reconnaissance");
                  }}
                  data={data}
                  projectId={project?.id}
                  busy={busy}
                  perform={perform}
                  onAdd={() => addTarget()}
                  onEdit={(target) => {
                    setEditingTarget(target);
                    setTargetOpen(true);
                  }}
                  onReverse={openImportedFile}
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
              {page === "Extensions" && (
                <ExtensionsPage
                  onOpenRecon={() => navigate("Reconnaissance")}
                  onOpenMcp={() => navigate("MCP Hub")}
                  projectId={project?.id}
                  onOpenFiles={() => {
                    setImportRequest((n) => n + 1);
                    navigate("Reverse Engineering");
                  }}
                  onOpenJobs={() => navigate("Analysis Jobs")}
                />
              )}
              {page === "Analysis Jobs" && (
                <JobsPage
                  data={data}
                  busy={busy}
                  perform={perform}
                  onOpenFile={openImportedFile}
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
          <Button
            className="text-button"
            disabled={busy}
            onClick={() => void perform(refresh, "Workspace refreshed")}
            disabledReason={"An operation is in progress"}
          >
            <Icon name="clock" size={12} />
            Refresh
          </Button>
          <span>
            ReconX AI <b>{data?.version ?? "—"}</b>
          </span>
        </footer>
      </div>
      {data && showActivity && (
        <ActivitySidebar
          data={data}
          project={project}
          onClose={() => setShowActivity(false)}
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
          onSaved={async (saved) => {
            await workspace.select(saved.project_id, saved.id);
            await refresh();
            setNotice(editingTarget ? "Target updated" : "Target created");
          }}
        />
      )}
    </div>
  );
}
