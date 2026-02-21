import { useEffect, useState } from "react";

export type ThemeMode = "system" | "light" | "dark";

export function useThemeMode() {
  const [themeMode, setThemeMode] = useState<ThemeMode>("system");
  const [systemDark, setSystemDark] = useState(false);

  function applyTheme(mode: ThemeMode) {
    const root = document.documentElement;
    const isDark = mode === "dark" || (mode === "system" && systemDark);
    if (isDark) {
      root.classList.add("dark");
    } else {
      root.classList.remove("dark");
    }
  }

  useEffect(() => {
    localStorage.setItem("gm_theme", themeMode);
    applyTheme(themeMode);
  }, [themeMode, systemDark]);

  useEffect(() => {
    const theme = localStorage.getItem("gm_theme") as ThemeMode | null;
    if (theme) setThemeMode(theme);

    try {
      const media = window.matchMedia("(prefers-color-scheme: dark)");
      setSystemDark(media.matches);
      applyTheme(themeMode);
      const handler = (e: MediaQueryListEvent) => {
        setSystemDark(e.matches);
        if (themeMode === "system") applyTheme("system");
      };
      media.addEventListener("change", handler);
      return () => media.removeEventListener("change", handler);
    } catch {
      // ignore
    }
  }, []);

  return { themeMode, setThemeMode };
}
