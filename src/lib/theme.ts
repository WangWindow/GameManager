export type ThemeMode = "system" | "light" | "dark";
export type SystemTheme = "light" | "dark";

export function parseThemeMode(value: string | null | undefined): ThemeMode {
  return value === "light" || value === "dark" ? value : "system";
}

export function parseSystemTheme(value: string | null | undefined): SystemTheme | null {
  return value === "light" || value === "dark" ? value : null;
}

export function resolveInitialSystemTheme(
  cached: string | null | undefined,
  mediaPrefersDark: boolean,
): SystemTheme {
  return parseSystemTheme(cached) ?? (mediaPrefersDark ? "dark" : "light");
}

export function resolveIsDark(mode: ThemeMode, systemTheme: SystemTheme): boolean {
  return mode === "dark" || (mode === "system" && systemTheme === "dark");
}

export function resolveTheme(mode: ThemeMode, systemTheme: SystemTheme): SystemTheme {
  return resolveIsDark(mode, systemTheme) ? "dark" : "light";
}

export function isThemeRequestCurrent(requestVersion: number, currentVersion: number): boolean {
  return requestVersion === currentVersion;
}
