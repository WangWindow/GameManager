import { useState } from "react";
import { toast } from "sonner";
import {
  deleteEngine,
  downloadNwjsStable,
  getEngineUpdateInfo,
  getNwjsStableInfo,
  importMkxpzArchive,
  updateEngine,
} from "@/lib/api";
import { text } from "@/lib/text";
import type { EngineDto } from "@/types";

interface Options {
  updateTask: (label: string, progress: number) => void;
}

export function useMaintenanceActions(options: Options) {
  const [maintenanceLoading, setMaintenanceLoading] = useState(false);

  async function handleDownloadNwjs() {
    if (maintenanceLoading) return;
    setMaintenanceLoading(true);
    try {
      const info = await getNwjsStableInfo();
      options.updateTask(text("maintenance.taskInstallNwjs", { version: info.version }), 0);
      await downloadNwjsStable("normal");
      toast.success(text("maintenance.toastInstallDone"));
      options.updateTask(text("maintenance.taskInstallDone"), 100);
      window.dispatchEvent(new CustomEvent("gm:refresh-engines"));
    } catch (e) {
      const msg = e instanceof Error ? e.message : text("maintenance.toastDownloadFailed");
      toast.error(msg);
    } finally {
      setMaintenanceLoading(false);
    }
  }

  async function handleUpdateEngine(engine: EngineDto) {
    if (engine.engineType === "mkxpz") {
      await handleImportMkxpz();
      return;
    }
    const info = await getEngineUpdateInfo(engine.id);
    if (!info.updateAvailable) {
      toast.info(text("maintenance.toastAlreadyLatest"));
      return;
    }
    const result = await updateEngine(engine.id);
    if (result.updated) {
      toast.success(text("maintenance.toastUpdatedTo", { version: result.toVersion }));
      window.dispatchEvent(new CustomEvent("gm:refresh-engines"));
    }
  }

  async function handleImportMkxpz() {
    if (maintenanceLoading) return;
    setMaintenanceLoading(true);
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const archive = await open({
        title: "选择 mkxp-z GitHub Actions ZIP",
        multiple: false,
        filters: [{ name: "ZIP", extensions: ["zip"] }],
      });
      const path = Array.isArray(archive) ? archive[0] : archive;
      if (!path) return;

      options.updateTask("正在安装 mkxp-z…", 0);
      const result = await importMkxpzArchive(path);
      options.updateTask("mkxp-z 已安装", 100);
      toast.success(`mkxp-z 已安装：${result.version}`);
      window.dispatchEvent(new CustomEvent("gm:refresh-engines"));
    } catch (e) {
      const message = e instanceof Error ? e.message : "mkxp-z 导入失败";
      toast.error(message);
    } finally {
      setMaintenanceLoading(false);
    }
  }

  async function handleOpenMkxpzBuilds() {
    try {
      const { openUrl } = await import("@tauri-apps/plugin-opener");
      await openUrl("https://github.com/mkxp-z/mkxp-z/actions/workflows/autobuild.yml?query=event%3Apush");
    } catch (e) {
      const message = e instanceof Error ? e.message : "无法打开 mkxp-z 构建页面";
      toast.error(message);
    }
  }

  async function handleRemoveEngine(engine: EngineDto) {
    if (maintenanceLoading) return;
    setMaintenanceLoading(true);
    try {
      await deleteEngine(engine.id);
      toast.success(text("maintenance.toastUninstalled", { name: engine.name }));
      window.dispatchEvent(new CustomEvent("gm:refresh-engines"));
    } catch (e) {
      const msg = e instanceof Error ? e.message : text("maintenance.toastUninstallFailed");
      toast.error(msg);
    } finally {
      setMaintenanceLoading(false);
    }
  }

  return {
    maintenanceLoading,
    handleDownloadNwjs,
    handleUpdateEngine,
    handleRemoveEngine,
    handleImportMkxpz,
    handleOpenMkxpzBuilds,
  };
}
