import { useState } from "react";
import { toast } from "sonner";
import { importGameDir, scanGames, saveGameSettings, updateGame, refreshGameCover } from "@/lib/api";
import { useI18n } from "@/i18n";
import type { GameConfig } from "@/types";

interface Options {
  refresh: (force?: boolean) => void | Promise<void>;
  updateTask: (label: string, progress: number) => void;
  closeImport: () => void;
  closeScan: () => void;
  closeGameSettings: () => void;
}

export function useGameLibraryActions(options: Options) {
  const { t } = useI18n();
  const [importLoading, setImportLoading] = useState(false);
  const [scanLoading, setScanLoading] = useState(false);
  const [saveLoading, setSaveLoading] = useState(false);
  const [coverRefreshing, setCoverRefreshing] = useState(false);

  async function handleImportSubmit(payload: { executablePath: string; engineType: string }) {
    if (importLoading) return;
    setImportLoading(true);
    try {
      await importGameDir(payload.executablePath, payload.engineType);
      toast.success(t("toast.importSuccess"));
      options.closeImport();
      await options.refresh(true);
    } catch (e) {
      const msg = e instanceof Error ? e.message : t("toast.importFailed");
      if (typeof msg === "string" && msg.includes("已存在")) {
        toast.error(t("toast.gameExists"));
      } else {
        toast.error(msg);
      }
    } finally {
      setImportLoading(false);
    }
  }

  async function handleScanSubmit(payload: { root: string; maxDepth: number }) {
    if (scanLoading) return;
    setScanLoading(true);
    try {
      const res = await scanGames(payload);
      toast.success(t("toast.scanComplete", { imported: res.imported, skipped: res.skippedExisting }));
      options.updateTask(t("task.scanComplete"), 100);
      options.closeScan();
      await options.refresh(true);
    } catch (e) {
      const msg = e instanceof Error ? e.message : t("toast.scanFailed");
      toast.error(msg);
    } finally {
      setScanLoading(false);
    }
  }

  async function handleGameSave(payload: {
    id: string;
    title: string;
    engineType: string;
    path: string;
    runtimeVersion?: string;
    settings: GameConfig;
  }) {
    if (saveLoading) return;
    setSaveLoading(true);
    try {
      await updateGame(payload.id, {
        title: payload.title,
        engineType: payload.engineType,
        path: payload.path,
        runtimeVersion: payload.runtimeVersion,
      });
      await saveGameSettings(payload.id, payload.settings);
      toast.success(t("toast.settingsSaved"));
      options.closeGameSettings();
      await options.refresh(true);
    } catch (e) {
      const msg = e instanceof Error ? e.message : t("toast.saveFailed");
      toast.error(msg);
    } finally {
      setSaveLoading(false);
    }
  }

  async function handleRefreshCover(id: string) {
    if (coverRefreshing) return;
    setCoverRefreshing(true);
    try {
      await refreshGameCover(id);
      toast.success(t("toast.coverUpdated"));
      await options.refresh(true);
    } catch (e) {
      const msg = e instanceof Error ? e.message : t("toast.coverUpdateFailed");
      toast.error(msg);
    } finally {
      setCoverRefreshing(false);
    }
  }

  return {
    importLoading,
    scanLoading,
    saveLoading,
    coverRefreshing,
    handleImportSubmit,
    handleScanSubmit,
    handleGameSave,
    handleRefreshCover,
  };
}
