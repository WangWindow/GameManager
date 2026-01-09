import { onMounted, onUnmounted, ref } from "vue";

export function useWindowControls() {
  const isTauri = ref(false);
  const isMaximized = ref(false);
  let win: any | null = null;
  let unlistenResize: (() => void) | null = null;

  async function syncMaximized() {
    if (!win || typeof win.isMaximized !== "function") return;
    try {
      isMaximized.value = Boolean(await win.isMaximized());
    } catch {
      // ignore
    }
  }

  async function init() {
    try {
      // 中文说明：优先使用 Window API，失败再退回 WebviewWindow。
      try {
        const mod = await import("@tauri-apps/api/window");
        win = mod.getCurrentWindow();
      } catch {
        const mod = await import("@tauri-apps/api/webviewWindow");
        win = mod.getCurrentWebviewWindow();
      }

      isTauri.value = true;
      await syncMaximized();

      // 中文说明：尽量保持最大化状态同步（用于切换图标）。
      if (win && typeof win.onResized === "function") {
        unlistenResize = await win.onResized(() => {
          void syncMaximized();
        });
      }
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error("[window] init failed", err);
      isTauri.value = false;
      win = null;
    }
  }

  async function minimize() {
    if (!win) return;
    try {
      await win.minimize();
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error("[window] minimize failed", err);
    }
  }

  async function toggleMaximize() {
    if (!win) return;
    try {
      // 中文说明：不要调用 toggleMaximize（它需要额外 capability: core:window:allow-toggle-maximize）。
      // 我们用 maximize/unmaximize 组合即可（已经在 capabilities 里允许）。
      const max = await win.isMaximized();
      await (max ? win.unmaximize() : win.maximize());
      await syncMaximized();
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error("[window] toggleMaximize failed", err);
    }
  }

  async function close() {
    if (!win) return;
    try {
      await win.close();
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error("[window] close failed", err);
    }
  }

  onMounted(() => {
    void init();
  });

  onUnmounted(() => {
    try {
      unlistenResize?.();
    } catch {
      // ignore
    }
    unlistenResize = null;
  });

  return {
    isTauri,
    isMaximized,
    minimize,
    toggleMaximize,
    close,
  };
}
