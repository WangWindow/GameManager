import { useEffect } from "react";

export function useTauriEvents(updateTask: (label: string, progress: number) => void) {
  useEffect(() => {
    let unlisteners: Array<() => void> = [];
    (async () => {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        const u1 = await listen<{
          taskId: string;
          version: string;
          flavor: "normal" | "sdk";
          target: string;
          downloaded: number;
          total?: number | null;
          percent?: number | null;
        }>("nwjs_download_progress", (event) => {
          const p = event.payload?.percent ?? 0;
          updateTask(`下载 NW.js ${event.payload.version}（${event.payload.flavor}）`, p);
        });
        unlisteners.push(u1);

        const u2 = await listen<{
          taskId: string;
          version: string;
          flavor: "normal" | "sdk";
          target: string;
          stage: "downloaded" | "installed";
          label: string;
        }>("nwjs_install_stage", (event) => {
          const label = event.payload?.label ?? "处理中…";
          updateTask(label, 100);
        });
        unlisteners.push(u2);

        const u3 = await listen<{ taskId: string; label: string; progress: number }>(
          "scan_progress",
          (event) => {
            updateTask(event.payload?.label ?? "扫描中…", Number(event.payload?.progress ?? 0));
          },
        );
        unlisteners.push(u3);
      } catch {
        // ignore when not in tauri
      }
    })();

    return () => {
      unlisteners.forEach((u) => {
        try {
          u();
        } catch { }
      });
      unlisteners = [];
    };
  }, [updateTask]);
}
