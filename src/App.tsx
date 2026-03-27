/**
 * GameManager 主应用组件
 *
 * 游戏管理器的核心入口，负责：
 * - 游戏列表展示和操作（启动、编辑、删除）
 * - 游戏导入和目录扫描
 * - 设置和维护功能管理
 * - 拖拽导入支持
 * - 状态栏任务进度显示
 */

import { useEffect, useMemo, useCallback } from "react";
import TitleBar from "@/components/layouts/TitleBar";
import ImportDialog from "@/components/games/ImportDialog";
import ScanDialog from "@/components/games/ScanDialog";
import GameGrid from "@/components/games/GameGrid";
import GameSettingsDialog from "@/components/games/GameSettingsDialog";
import GameLibraryHeader from "@/components/games/GameLibraryHeader";
import ManagementDialog from "@/components/settings/ManagementDialog";
import SettingsDialog from "@/components/settings/SettingsDialog";
import ConfirmDeleteDialog from "@/components/common/ConfirmDeleteDialog";
import { Progress } from "@/components/ui/progress";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Toaster } from "@/components/ui/sonner";
import { toast } from "sonner";
import { useGames } from "@/hooks/useGames";
import type { GameDto } from "@/types";
import { useThemeMode } from "@/hooks/useThemeMode";
import { useTaskStatus } from "@/hooks/useTaskStatus";
import { useTauriEvents } from "@/hooks/useTauriEvents";
import { useGameLibraryActions } from "@/hooks/useGameLibraryActions";
import { useMaintenanceActions } from "@/hooks/useMaintenanceActions";
import { useDialogState, useDeleteConfirmState } from "@/hooks/useDialogState";
import { useDragDrop } from "@/hooks/useDragDrop";
import { usePersistedState, usePersistedBoolean } from "@/hooks/usePersistedState";
import {
  ENGINE_FILTER_OPTIONS,
  ENGINE_OPTION_NWJS,
} from "@/constants/engines";

/** 游戏列表视图模式 */
export type ViewMode = "grid" | "list";

export default function App() {
  // ============ 对话框状态 ============
  const manageDialog = useDialogState();
  const settingsDialog = useDialogState();
  const importDialog = useDialogState<string>(); // data: 待导入的文件路径
  const scanDialog = useDialogState();
  const gameSettingsDialog = useDialogState<string>(); // data: 游戏 ID
  const deleteConfirm = useDeleteConfirmState();

  // 持久化状态
  const [showStatusBar, setShowStatusBar] = usePersistedBoolean("gm_show_status_bar", true);
  const [viewMode, setViewMode] = usePersistedState<ViewMode>("gm_game_view_mode", "list");

  // UI 状态
  const [searchQuery, setSearchQuery] = usePersistedState("gm_search_query", "");
  const [engineFilter, setEngineFilter] = usePersistedState("gm_engine_filter", "all");

  // 拖拽导入
  const { isDragging, droppedPath, clearDroppedPath } = useDragDrop();

  // 处理拖拽导入
  useEffect(() => {
    if (droppedPath) {
      importDialog.open(droppedPath);
      clearDroppedPath();
    }
  }, [droppedPath, importDialog, clearDroppedPath]);

  // 游戏数据
  const { games, loading, fetchGames, handleLaunchGame, handleDeleteGame } = useGames();
  const { themeMode, setThemeMode } = useThemeMode();
  const { currentTask, statusBarVisible: taskStatusVisible, updateTask } = useTaskStatus();
  useTauriEvents(updateTask);

  const {
    importLoading,
    scanLoading,
    saveLoading,
    handleImportSubmit,
    handleScanSubmit,
    handleGameSave,
    handleRefreshCover,
  } = useGameLibraryActions({
    refresh: fetchGames,
    updateTask,
    closeImport: importDialog.close,
    closeScan: scanDialog.close,
    closeGameSettings: gameSettingsDialog.close,
  });

  const {
    handleDownloadNwjs,
    handleCleanupContainers,
    handleCleanupOldNwjs,
    handleUpdateEngine,
    handleRemoveEngine,
  } = useMaintenanceActions({ updateTask });

  // 过滤后的游戏列表
  const filteredGames = useMemo(() => {
    const query = searchQuery.trim().toLowerCase();
    return games.filter((game: GameDto) => {
      const matchesKeyword =
        query.length === 0 ||
        game.title.toLowerCase().includes(query) ||
        game.engineType.toLowerCase().includes(query);
      const matchesType =
        engineFilter === "all" ||
        (engineFilter === ENGINE_OPTION_NWJS
          ? ["rpgmakermv", "rpgmakermz", "rpgmakervx", "rpgmakervxace"].includes(game.engineType)
          : game.engineType === engineFilter);
      return matchesKeyword && matchesType;
    });
  }, [games, searchQuery, engineFilter]);

  // 状态栏可见性
  const statusBarVisible = useMemo(
    () => showStatusBar && taskStatusVisible,
    [showStatusBar, taskStatusVisible],
  );

  // 选中的游戏
  const selectedGame = useMemo(
    () => (gameSettingsDialog.data ? games.find((game) => game.id === gameSettingsDialog.data) ?? null : null),
    [games, gameSettingsDialog.data],
  );

  // 刷新游戏列表事件
  const handleRefreshGamesEvent = useCallback(() => {
    void fetchGames(true);
  }, [fetchGames]);

  useEffect(() => {
    fetchGames();
    window.addEventListener("gm:refresh-games", handleRefreshGamesEvent);
    return () => {
      window.removeEventListener("gm:refresh-games", handleRefreshGamesEvent);
    };
  }, [fetchGames, handleRefreshGamesEvent]);

  // 游戏操作回调
  const onLaunchGame = useCallback(async (id: string) => {
    const success = await handleLaunchGame(id);
    if (success) {
      toast.success("游戏启动成功");
    }
  }, [handleLaunchGame]);

  const onEditGame = useCallback((id: string) => {
    gameSettingsDialog.open(id);
  }, [gameSettingsDialog]);

  const onDeleteGame = useCallback((id: string) => {
    const game = games.find((g) => g.id === id);
    deleteConfirm.open(id, game?.title ?? "");
  }, [games, deleteConfirm]);

  const confirmDeleteGame = useCallback(async () => {
    if (!deleteConfirm.id) return;
    const id = deleteConfirm.id;
    deleteConfirm.close();
    const success = await handleDeleteGame(id);
    if (success) {
      toast.success("游戏删除成功");
    }
  }, [deleteConfirm, handleDeleteGame]);

  const openImportDialog = useCallback(() => {
    importDialog.open("");
  }, [importDialog]);

  return (
    <div className="flex h-screen flex-col overflow-hidden">
      <TitleBar
        onManage={manageDialog.open}
        onSettings={settingsDialog.open}
        onImport={openImportDialog}
        onScan={scanDialog.open}
      />

      <div className="flex flex-1 overflow-hidden pt-10">
        <main className="flex-1">
          <ScrollArea className="h-full">
            <div className="container mx-auto py-6">
              <GameLibraryHeader
                count={filteredGames.length}
                search={searchQuery}
                selectedEngine={engineFilter}
                engineOptions={ENGINE_FILTER_OPTIONS.map((item) => ({
                  value: item.value,
                  label: item.label,
                }))}
                viewMode={viewMode}
                onSearchChange={setSearchQuery}
                onEngineChange={setEngineFilter}
                onViewModeChange={setViewMode}
              />

              <GameGrid
                games={filteredGames}
                loading={loading}
                viewMode={viewMode}
                onLaunch={onLaunchGame}
                onEdit={onEditGame}
                onDelete={onDeleteGame}
              />
            </div>
          </ScrollArea>
        </main>
      </div>

      <ImportDialog
        open={importDialog.isOpen}
        loading={importLoading}
        initialExecutablePath={importDialog.data ?? ""}
        onOpenChange={importDialog.setOpen}
        onSubmit={handleImportSubmit}
      />
      <ScanDialog
        open={scanDialog.isOpen}
        loading={scanLoading}
        onOpenChange={scanDialog.setOpen}
        onSubmit={handleScanSubmit}
      />

      <GameSettingsDialog
        open={gameSettingsDialog.isOpen}
        game={selectedGame}
        loading={saveLoading}
        onOpenChange={gameSettingsDialog.setOpen}
        onSave={handleGameSave}
        onRefreshCover={handleRefreshCover}
      />

      <ManagementDialog
        open={manageDialog.isOpen}
        onOpenChange={manageDialog.setOpen}
        onDownloadNwjs={handleDownloadNwjs}
        onCleanupOldNwjs={handleCleanupOldNwjs}
        onCleanupContainers={handleCleanupContainers}
        onUpdateEngine={handleUpdateEngine}
        onRemoveEngine={handleRemoveEngine}
      />

      <SettingsDialog
        open={settingsDialog.isOpen}
        themeMode={themeMode}
        showStatusBar={showStatusBar}
        onOpenChange={settingsDialog.setOpen}
        onThemeModeChange={setThemeMode}
        onShowStatusBarChange={setShowStatusBar}
      />

      <ConfirmDeleteDialog
        open={deleteConfirm.isOpen}
        title={deleteConfirm.title}
        onOpenChange={deleteConfirm.setOpen}
        onConfirm={confirmDeleteGame}
      />

      {isDragging ? (
        <div className="pointer-events-none fixed inset-0 z-50 flex items-center justify-center bg-background/70 backdrop-blur">
          <div className="rounded-2xl border-2 border-dashed border-primary/50 bg-background/80 px-8 py-6 text-center">
            <div className="mb-2 text-sm font-semibold">拖拽文件以导入游戏</div>
            <div className="text-xs text-muted-foreground">仅支持可执行文件</div>
          </div>
        </div>
      ) : null}

      {statusBarVisible ? (
        <div className="border-t bg-background/70 px-4 py-2 text-xs backdrop-blur">
          <div className="flex items-center justify-between gap-3">
            <div className="min-w-0 truncate">
              <span className="font-medium">状态：</span>
              <span className="text-muted-foreground">{currentTask?.label}</span>
            </div>
            <div className="flex items-center gap-2">
              <div className="w-40">
                <Progress value={currentTask?.progress ?? 0} />
              </div>
              <span className="tabular-nums text-muted-foreground">{currentTask?.progress ?? 0}%</span>
            </div>
          </div>
        </div>
      ) : null}

      <Toaster position="bottom-right" />
    </div>
  );
}
