import { useEffect, useState, useCallback } from "react";

export type ThemeMode = "system" | "light" | "dark";

async function fetchSystemTheme(): Promise<string> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<string>("get_system_theme");
  } catch {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
}

export function useThemeMode() {
  const [themeMode, setThemeMode] = useState<ThemeMode>("system");
  const [systemDark, setSystemDark] = useState(false);
  const [initialized, setInitialized] = useState(false);

  const refreshSystemTheme = useCallback(async () => {
    const sys = await fetchSystemTheme();
    setSystemDark(sys === "dark");
  }, []);

  function applyTheme(mode: ThemeMode) {
    const root = document.documentElement;
    const isDark = mode === "dark" || (mode === "system" && systemDark);
    if (isDark) root.classList.add("dark");
    else root.classList.remove("dark");
  }

  // 启动时获取系统主题
  useEffect(() => {
    async function init() {
      const saved = localStorage.getItem("gm_theme") as ThemeMode | null;
      if (saved) setThemeMode(saved);
      await refreshSystemTheme();
      setInitialized(true);
    }
    init();
  }, []);

  // 持久化 + 应用
  useEffect(() => {
    if (!initialized) return;
    localStorage.setItem("gm_theme", themeMode);
    applyTheme(themeMode);
  }, [themeMode, systemDark, initialized]);

  // 监听系统主题变化（matchMedia + 窗口焦点）
  useEffect(() => {
    // matchMedia 监听（Windows/macOS 可用）
    const onMatchMedia = (e: MediaQueryListEvent) => setSystemDark(e.matches);
    let media: MediaQueryList | null = null;
    try {
      media = window.matchMedia("(prefers-color-scheme: dark)");
      media.addEventListener("change", onMatchMedia);
    } catch { }

    // 窗口焦点监听（Linux 下 matchMedia 不触发，用焦点做补偿）
    let unlistenFocus: (() => void) | null = null;
    (async () => {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        unlistenFocus = await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
          if (focused) refreshSystemTheme();
        });
      } catch { }
    })();

    return () => {
      if (media) media.removeEventListener("change", onMatchMedia);
      if (unlistenFocus) unlistenFocus();
    };
  }, []);

  return { themeMode, setThemeMode };
}
