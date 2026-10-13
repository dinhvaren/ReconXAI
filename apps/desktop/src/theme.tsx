import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { errorText, request, type Settings, type ThemePreference } from "./api";

export function resolveTheme(preference: ThemePreference): "light" | "dark" {
  return preference === "system"
    ? window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light"
    : preference;
}
export function applyTheme(preference: ThemePreference) {
  const resolved = resolveTheme(preference);
  document.documentElement.dataset.theme = resolved;
  document.documentElement.dataset.themePreference = preference;
  document.documentElement.style.colorScheme = resolved;
  return resolved;
}
interface ThemeState {
  preference: ThemePreference;
  resolved: "light" | "dark";
  saving: boolean;
  error: string;
  activeMenu: string | null;
  setActiveMenu: (menu: string | null) => void;
  setTheme: (preference: ThemePreference) => Promise<void>;
  savePreferences: (settings: Omit<Settings, "theme">) => Promise<void>;
  syncFromStorage: (preference: ThemePreference, revision: number) => void;
  getRevision: () => number;
  clearError: () => void;
}
const ThemeContext = createContext<ThemeState | null>(null);

export function ThemeProvider({
  initialTheme = "dark",
  children,
}: {
  initialTheme?: ThemePreference;
  children: ReactNode;
}) {
  const [preference, setPreference] = useState(initialTheme);
  const [resolved, setResolved] = useState(() => resolveTheme(initialTheme));
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const [activeMenu, setActiveMenu] = useState<string | null>(null);
  const desired = useRef(initialTheme);
  const committed = useRef(initialTheme);
  const revision = useRef(0);
  const pending = useRef(0);
  const queue = useRef<Promise<void>>(Promise.resolve());
  const getRevision = useCallback(() => revision.current, []);
  const syncFromStorage = useCallback(
    (theme: ThemePreference, readRevision: number) => {
      if (pending.current || readRevision !== revision.current) return;
      desired.current = committed.current = theme;
      setPreference(theme);
    },
    [],
  );

  useLayoutEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () => setResolved(applyTheme(preference));
    apply();
    if (preference === "system") media.addEventListener("change", apply);
    return () => media.removeEventListener("change", apply);
  }, [preference]);

  useEffect(() => {
    let cancelled = false;
    // The native window starts hidden, so saved appearance and fonts precede its first visible frame.
    void (
      document.fonts
        ? Promise.all([
            document.fonts.load('500 14px "DM Sans"'),
            document.fonts.load('400 13px "IBM Plex Mono"'),
          ]).then(() => document.fonts.ready)
        : Promise.resolve()
    )
      .catch((e) =>
        setError("Bundled typography could not load. " + errorText(e)),
      )
      .then(() => {
        if (!cancelled)
          void invoke("apply_window_theme", { theme: resolved }).catch((e) =>
            setError(errorText(e)),
          );
      });
    return () => {
      cancelled = true;
    };
  }, [resolved]);

  function enqueue(write: () => Promise<void>): Promise<void> {
    pending.current += 1;
    setSaving(true);
    const task = queue.current.then(write).finally(() => {
      pending.current -= 1;
      setSaving(pending.current > 0);
    });
    queue.current = task.catch(() => {});
    return task;
  }
  function setTheme(theme: ThemePreference): Promise<void> {
    const change = ++revision.current;
    desired.current = theme;
    setPreference(theme);
    applyTheme(theme);
    setError("");
    return enqueue(async () => {
      try {
        const settings = await request<Settings>("set_theme", { theme });
        committed.current = settings.theme;
      } catch (e) {
        if (change === revision.current) {
          desired.current = committed.current;
          setPreference(committed.current);
          applyTheme(committed.current);
          setError("Appearance was not saved. " + errorText(e));
        }
        throw e;
      }
    });
  }
  function savePreferences(settings: Omit<Settings, "theme">): Promise<void> {
    const selectedTheme = desired.current;
    return enqueue(async () => {
      await request("save_settings", {
        settings: { ...settings, theme: selectedTheme },
      });
      committed.current = selectedTheme;
    });
  }
  return (
    <ThemeContext.Provider
      value={{
        preference,
        resolved,
        saving,
        error,
        activeMenu,
        setActiveMenu,
        setTheme,
        savePreferences,
        syncFromStorage,
        getRevision,
        clearError: () => setError(""),
      }}
    >
      {children}
    </ThemeContext.Provider>
  );
}
export function useTheme() {
  const theme = useContext(ThemeContext);
  if (!theme) throw new Error("Theme controls require ThemeProvider");
  return theme;
}
