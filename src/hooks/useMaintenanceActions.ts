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
import { translate } from "@/i18n";
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
      options.updateTask(translate("maintenance.taskInstallNwjs", { version: info.version }), 0);
      await downloadNwjsStable("normal");
      toast.success(translate("maintenance.toastInstallDone"));
      options.updateTask(translate("maintenance.taskInstallDone"), 100);
      window.dispatchEvent(new CustomEvent("gm:refresh-engines"));
    } catch (e) {
      const msg = e instanceof Error ? e.message : translate("maintenance.toastDownloadFailed");
      toast.error(msg);
    } finally {
      setMaintenanceLoading(false);
    }
  }

  async function handleUpdateEngine(engine: EngineDto) {
    const info = await getEngineUpdateInfo(engine.id);
    if (!info.updateAvailable) {
      toast.info(translate("maintenance.toastAlreadyLatest"));
      return;
    }
    const result = await updateEngine(engine.id);
    if (result.updated) {
      toast.success(translate("maintenance.toastUpdatedTo", { version: result.toVersion }));
      window.dispatchEvent(new CustomEvent("gm:refresh-engines"));
    }
  }

  async function handleRemoveEngine(engine: EngineDto) {
    if (maintenanceLoading) return;
    setMaintenanceLoading(true);
    try {
      await deleteEngine(engine.id);
      toast.success(translate("maintenance.toastUninstalled", { name: engine.name }));
      window.dispatchEvent(new CustomEvent("gm:refresh-engines"));
    } catch (e) {
      const msg = e instanceof Error ? e.message : translate("maintenance.toastUninstallFailed");
      toast.error(msg);
    } finally {
      setMaintenanceLoading(false);
    }
  }

  async function handleImportMkxpz() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: false,
        title: translate("maintenance.importMkxpzTitle"),
        filters: [{ name: "ZIP Archive", extensions: ["zip"] }],
      });
      if (!selected) return;
      const path = Array.isArray(selected) ? selected[0] ?? "" : selected;
      if (!path) return;

      setMaintenanceLoading(true);
      const result = await importMkxpzArchive(path);
      toast.success(translate("maintenance.toastMkxpzImportDone", { version: result.version }));
      window.dispatchEvent(new CustomEvent("gm:refresh-engines"));
    } catch (e) {
      const msg = e instanceof Error ? e.message : translate("maintenance.toastMkxpzImportFailed");
      toast.error(msg);
    } finally {
      setMaintenanceLoading(false);
    }
  }

  return {
    maintenanceLoading,
    handleDownloadNwjs,
    handleImportMkxpz,
    handleUpdateEngine,
    handleRemoveEngine,
  };
}
