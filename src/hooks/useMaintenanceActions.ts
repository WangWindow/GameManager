import { useState } from "react";
import { toast } from "sonner";
import {
  cleanupUnusedContainers,
  cleanupOldNwjsVersions,
  deleteEngine,
  downloadNwjsStable,
  getEngineUpdateInfo,
  getNwjsStableInfo,
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

  async function handleCleanupContainers() {
    if (maintenanceLoading) return;
    setMaintenanceLoading(true);
    try {
      const res = await cleanupUnusedContainers();
      toast.success(translate("maintenance.toastCleanupDone", { count: res.deleted }));
    } catch (e) {
      const msg = e instanceof Error ? e.message : translate("maintenance.toastCleanupFailed");
      toast.error(msg);
    } finally {
      setMaintenanceLoading(false);
    }
  }

  async function handleCleanupOldNwjs() {
    if (maintenanceLoading) return;
    setMaintenanceLoading(true);
    try {
      const res = await cleanupOldNwjsVersions();
      toast.success(translate("maintenance.toastCleanupOldNwjsDone", { count: res.deleted }));
      window.dispatchEvent(new CustomEvent("gm:refresh-engines"));
    } catch (e) {
      const msg = e instanceof Error ? e.message : translate("maintenance.toastCleanupOldNwjsFailed");
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

  return {
    maintenanceLoading,
    handleDownloadNwjs,
    handleCleanupContainers,
    handleCleanupOldNwjs,
    handleUpdateEngine,
    handleRemoveEngine,
  };
}
