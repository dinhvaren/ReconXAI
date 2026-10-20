import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
  type SetStateAction,
} from "react";
import { request, errorText } from "./api";

export interface WorkspaceSelection {
  revision: number;
  project_id: string | null;
  target_id: string | null;
}
interface ActiveWorkspace {
  selection: WorkspaceSelection;
  ready: boolean;
  saving: boolean;
  error: string;
  select: (
    project: string | null,
    target?: string | null,
  ) => Promise<WorkspaceSelection>;
}
const Workspace = createContext<ActiveWorkspace | null>(null);
export function WorkspaceProvider({ children }: { children: ReactNode }) {
  const [selection, setSelection] = useState<WorkspaceSelection>({
    revision: 0,
    project_id: null,
    target_id: null,
  });
  const [ready, setReady] = useState(false),
    [saving, setSaving] = useState(false),
    [error, setError] = useState("");
  const value = useRef(selection),
    generation = useRef(0),
    pending = useRef(0),
    alive = useRef(true);
  const queue = useRef<Promise<unknown>>(Promise.resolve());
  value.current = selection;
  const select = useCallback(
    (project: string | null, target: string | null = null) => {
      project ||= null;
      target ||= null;
      if (
        value.current.project_id === project &&
        value.current.target_id === target &&
        pending.current === 0
      )
        return Promise.resolve(value.current);
      const ticket = ++generation.current;
      pending.current += 1;
      setSaving(true);
      setError("");
      setSelection((previous) => ({
        ...previous,
        project_id: project,
        target_id: target,
      }));
      const task = queue.current
        .catch(() => {})
        .then(async () => {
          const current = await request<WorkspaceSelection>(
            "workspace_selection",
          );
          const saved = await request<WorkspaceSelection>("workspace_select", {
            project,
            target,
            revision: current.revision,
          });
          if (alive.current && generation.current === ticket)
            setSelection(saved);
          return saved;
        })
        .catch(async (e) => {
          if (alive.current && generation.current === ticket) {
            setError(errorText(e));
            try {
              setSelection(
                await request<WorkspaceSelection>("workspace_selection"),
              );
            } catch {
              /* Keep the visible error; never invent persisted IDs. */
            }
          }
          throw e;
        })
        .finally(() => {
          pending.current -= 1;
          if (alive.current && pending.current === 0) setSaving(false);
        });
      queue.current = task;
      return task;
    },
    [],
  );
  useEffect(() => {
    alive.current = true;
    const load = async () => {
      if (pending.current) return;
      const ticket = generation.current;
      try {
        const saved = await request<WorkspaceSelection>("workspace_selection");
        if (
          alive.current &&
          ticket === generation.current &&
          !pending.current
        ) {
          setSelection(saved);
          setReady(true);
        }
      } catch (e) {
        if (alive.current) {
          setError(errorText(e));
          setReady(true);
        }
      }
    };
    void load();
    const timer = setInterval(() => void load(), 1500);
    return () => {
      alive.current = false;
      clearInterval(timer);
    };
  }, []);
  return (
    <Workspace.Provider value={{ selection, ready, saving, error, select }}>
      {children}
    </Workspace.Provider>
  );
}
export const useWorkspace = () => useContext(Workspace);
export function useWorkspaceProject(initial = "") {
  const workspace = useWorkspace();
  const [local, setLocal] = useState(initial);
  const current = workspace ? (workspace.selection.project_id ?? "") : local;
  const update = (next: SetStateAction<string>) => {
    const id = typeof next === "function" ? next(current) : next;
    if (workspace)
      void workspace
        .select(
          id || null,
          id === current ? workspace.selection.target_id : null,
        )
        .catch(() => {});
    else setLocal(id);
  };
  return [current, update] as const;
}
export function useWorkspaceTarget(initial = "") {
  const workspace = useWorkspace();
  const [local, setLocal] = useState(initial);
  const current = workspace ? (workspace.selection.target_id ?? "") : local;
  const update = (next: SetStateAction<string>) => {
    const id = typeof next === "function" ? next(current) : next;
    if (workspace)
      void workspace
        .select(workspace.selection.project_id, id || null)
        .catch(() => {});
    else setLocal(id);
  };
  return [current, update] as const;
}
