import { useEffect, useState, useRef } from "react";

export function useWindowControls() {
  const [isTauri, setIsTauri] = useState(false);
  const [isMaximized, setIsMaximized] = useState(false);
  const winRef = useRef<any | null>(null);
  const unlistenResizeRef = useRef<(() => void) | null>(null);

  async function syncMaximized() {
    const win = winRef.current;
    if (!win || typeof win.isMaximized !== "function") return;
    try {
      setIsMaximized(Boolean(await win.isMaximized()));
    } catch {
      // ignore
    }
  }

  async function init() {
    try {
      try {
        const mod = await import("@tauri-apps/api/window");
        winRef.current = mod.getCurrentWindow();
      } catch {
        const mod = await import("@tauri-apps/api/webviewWindow");
        winRef.current = mod.getCurrentWebviewWindow();
      }

      setIsTauri(true);
      await syncMaximized();

      const win = winRef.current;
      if (win && typeof win.onResized === "function") {
        unlistenResizeRef.current = await win.onResized(() => {
          void syncMaximized();
        });
      }
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error("[window] init failed", err);
      setIsTauri(false);
      winRef.current = null;
    }
  }

  async function minimize() {
    const win = winRef.current;
    if (!win) return;
    try {
      await win.minimize();
    } catch (err) {
      console.error("[window] minimize failed", err);
    }
  }

  async function toggleMaximize() {
    const win = winRef.current;
    if (!win) return;
    try {
      const max = await win.isMaximized();
      await (max ? win.unmaximize() : win.maximize());
      await syncMaximized();
    } catch (err) {
      console.error("[window] toggleMaximize failed", err);
    }
  }

  async function close() {
    const win = winRef.current;
    if (!win) return;
    try {
      await win.close();
    } catch (err) {
      console.error("[window] close failed", err);
    }
  }

  useEffect(() => {
    void init();
    return () => {
      try {
        unlistenResizeRef.current?.();
      } catch {
        // ignore
      }
      unlistenResizeRef.current = null;
    };
  }, []);

  return {
    isTauri,
    isMaximized,
    minimize,
    toggleMaximize,
    close,
  };
}
