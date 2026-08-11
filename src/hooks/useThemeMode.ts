import { useCallback, useEffect, useRef, useState } from "react";
import {
  parseSystemTheme,
  parseThemeMode,
  resolveInitialSystemTheme,
  resolveTheme,
  isThemeRequestCurrent,
  type SystemTheme,
  type ThemeMode,
} from "@/lib/theme";

export type { ThemeMode } from "@/lib/theme";

const THEME_STORAGE_KEY = "gm_theme";
const SYSTEM_THEME_STORAGE_KEY = "gm_system_theme";

function readStorage(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

function writeStorage(key: string, value: string): void {
  try {
    localStorage.setItem(key, value);
  } catch {
    // Storage can be unavailable in restricted webviews; the in-memory state
    // remains the source of truth for the current session.
  }
}

function readMediaPrefersDark(): boolean {
  try {
    return window.matchMedia("(prefers-color-scheme: dark)").matches;
  } catch {
    return false;
  }
}

async function fetchSystemTheme(): Promise<SystemTheme | null> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const detected = await invoke<string>("get_system_theme");
    return parseSystemTheme(detected);
  } catch {
    return null;
  }
}

export function useThemeMode() {
  const [themeMode, setThemeMode] = useState<ThemeMode>(() =>
    parseThemeMode(readStorage(THEME_STORAGE_KEY)),
  );
  const [systemDark, setSystemDark] = useState(
    () =>
      resolveInitialSystemTheme(
        readStorage(SYSTEM_THEME_STORAGE_KEY),
        readMediaPrefersDark(),
      ) === "dark",
  );
  const mountedRef = useRef(true);
  const systemRevisionRef = useRef(0);

  const updateSystemTheme = useCallback((theme: SystemTheme) => {
    if (!mountedRef.current) return;
    systemRevisionRef.current += 1;
    setSystemDark(theme === "dark");
    writeStorage(SYSTEM_THEME_STORAGE_KEY, theme);
  }, []);

  const refreshSystemTheme = useCallback(async () => {
    const requestRevision = systemRevisionRef.current + 1;
    systemRevisionRef.current = requestRevision;
    const detected = await fetchSystemTheme();
    if (!isThemeRequestCurrent(requestRevision, systemRevisionRef.current)) return;
    if (detected) updateSystemTheme(detected);
  }, [updateSystemTheme]);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  useEffect(() => {
    writeStorage(THEME_STORAGE_KEY, themeMode);
    document.documentElement.classList.toggle(
      "dark",
      resolveTheme(themeMode, systemDark ? "dark" : "light") === "dark",
    );
  }, [themeMode, systemDark]);

  // Let the backend correct browser/cache guesses after the first render.
  useEffect(() => {
    void refreshSystemTheme();
  }, [refreshSystemTheme]);

  // Listen for browser theme changes and compensate for Linux desktops where
  // matchMedia may not emit an event by refreshing whenever the window focuses.
  useEffect(() => {
    const onMatchMedia = (event: MediaQueryListEvent) => {
      updateSystemTheme(event.matches ? "dark" : "light");
    };
    let media: MediaQueryList | null = null;
    try {
      media = window.matchMedia("(prefers-color-scheme: dark)");
      media.addEventListener("change", onMatchMedia);
    } catch {
      // Some webviews do not expose matchMedia listeners.
      media = null;
    }

    let active = true;
    let unlistenFocus: (() => void) | null = null;
    (async () => {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        const cleanup = await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
          if (focused) void refreshSystemTheme();
        });
        if (active) unlistenFocus = cleanup;
        else cleanup();
      } catch {
        // Browser tests and non-Tauri previews do not expose a Tauri window.
      }
    })();

    return () => {
      active = false;
      if (media) media.removeEventListener("change", onMatchMedia);
      if (unlistenFocus) unlistenFocus();
    };
  }, [refreshSystemTheme, updateSystemTheme]);

  const resolvedTheme = resolveTheme(themeMode, systemDark ? "dark" : "light");

  return { themeMode, resolvedTheme, setThemeMode };
}
