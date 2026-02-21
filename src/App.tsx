import { useEffect, useMemo, useRef, useState } from "react";
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

export type ViewMode = "grid" | "list";

export default function App() {
  const [manageOpen, setManageOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const [scanOpen, setScanOpen] = useState(false);
  const [showStatusBar, setShowStatusBar] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");
  const [viewMode, setViewMode] = useState<ViewMode>("list");
  const [gameSettingsOpen, setGameSettingsOpen] = useState(false);
  const [selectedGameId, setSelectedGameId] = useState<string | null>(null);
  const [deleteConfirmOpen, setDeleteConfirmOpen] = useState(false);
  const [pendingDeleteId, setPendingDeleteId] = useState<string | null>(null);
  const [pendingDeleteTitle, setPendingDeleteTitle] = useState("");
  const [pendingImportPath, setPendingImportPath] = useState("");
  const [isDragging, setIsDragging] = useState(false);

  const { games, loading, fetchGames, handleLaunchGame, handleDeleteGame } = useGames();
  const { themeMode, setThemeMode } = useThemeMode();
  const { currentTask, statusBarVisible: taskStatusVisible, updateTask } = useTaskStatus();
  useTauriEvents(updateTask);

  const closeImportDialog = () => {
    setImportOpen(false);
    setPendingImportPath("");
  };

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
    closeImport: closeImportDialog,
    closeScan: () => setScanOpen(false),
    closeGameSettings: () => setGameSettingsOpen(false),
  });

  const { handleDownloadNwjs, handleCleanupContainers, handleUpdateEngine, handleRemoveEngine } =
    useMaintenanceActions({ updateTask });

  const filteredGames = useMemo(() => {
    if (!searchQuery) return games;
    const query = searchQuery.toLowerCase();
    return games.filter(
      (game: GameDto) =>
        game.title.toLowerCase().includes(query) ||
        game.engineType.toLowerCase().includes(query),
    );
  }, [games, searchQuery]);

  const statusBarVisible = useMemo(
    () => showStatusBar && taskStatusVisible,
    [showStatusBar, taskStatusVisible],
  );

  const selectedGame = useMemo(
    () => (selectedGameId ? games.find((game) => game.id === selectedGameId) ?? null : null),
    [games, selectedGameId],
  );

  const unlistenDragDropRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    localStorage.setItem("gm_show_status_bar", String(showStatusBar));
  }, [showStatusBar]);

  useEffect(() => {
    localStorage.setItem("gm_game_view_mode", viewMode);
  }, [viewMode]);

  useEffect(() => {
    const saved = localStorage.getItem("gm_show_status_bar");
    if (saved !== null) setShowStatusBar(saved === "true");

    const savedView = localStorage.getItem("gm_game_view_mode");
    if (savedView === "grid" || savedView === "list") {
      setViewMode(savedView);
    }

    fetchGames();
    window.addEventListener("gm:refresh-games", fetchGames);
    void initDragDrop();

    return () => {
      window.removeEventListener("gm:refresh-games", fetchGames);
      teardownDomDragDrop();
      if (typeof unlistenDragDropRef.current === "function") {
        try {
          unlistenDragDropRef.current();
        } catch {
          // ignore
        }
      }
      unlistenDragDropRef.current = null;
    };
  }, [fetchGames]);

  async function initDragDrop() {
    try {
      const mod = await import("@tauri-apps/api/webviewWindow");
      const win = mod.getCurrentWebviewWindow();
      unlistenDragDropRef.current = await win.onDragDropEvent((event) => {
        const payload = "payload" in event ? event.payload : event;
        const type = (payload as { type?: string }).type ?? "";
        const paths = (payload as { paths?: string[] }).paths ?? [];

        if (type === "enter" || type === "over") {
          setIsDragging(true);
          return;
        }

        if (type === "leave") {
          setIsDragging(false);
          return;
        }

        if (type === "drop") {
          setIsDragging(false);
          const path = paths[0];
          if (path) {
            setPendingImportPath(path);
            setImportOpen(true);
            toast.info("已选择拖拽文件，请选择引擎类型");
          } else {
            toast.error("仅支持拖拽本地可执行文件");
          }
        }
      });
    } catch {
      // Web 环境使用 DOM 事件兜底
      setupDomDragDrop();
    }
  }

  function setupDomDragDrop() {
    window.addEventListener("dragenter", handleDragEnter);
    window.addEventListener("dragover", handleDragOver);
    window.addEventListener("dragleave", handleDragLeave);
    window.addEventListener("drop", handleDrop);
  }

  function teardownDomDragDrop() {
    window.removeEventListener("dragenter", handleDragEnter);
    window.removeEventListener("dragover", handleDragOver);
    window.removeEventListener("dragleave", handleDragLeave);
    window.removeEventListener("drop", handleDrop);
  }

  function handleDragEnter(event: DragEvent) {
    if (!event.dataTransfer?.types?.includes("Files")) return;
    setIsDragging(true);
  }

  function handleDragOver(event: DragEvent) {
    if (!event.dataTransfer?.types?.includes("Files")) return;
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = "copy";
    }
  }

  function handleDragLeave(event: DragEvent) {
    if (!event.dataTransfer?.types?.includes("Files")) return;
    setIsDragging(false);
  }

  function handleDrop(event: DragEvent) {
    if (!event.dataTransfer?.files?.length) {
      setIsDragging(false);
      return;
    }
    event.preventDefault();
    const file = event.dataTransfer.files[0];
    const path = (file as File & { path?: string }).path;
    setIsDragging(false);
    if (!path) {
      toast.error("仅支持拖拽本地可执行文件");
      return;
    }
    setPendingImportPath(path);
    setImportOpen(true);
    toast.info("已选择拖拽文件，请选择引擎类型");
  }

  function openImportDialog() {
    setPendingImportPath("");
    setImportOpen(true);
  }

  async function onLaunchGame(id: string) {
    const success = await handleLaunchGame(id);
    if (success) {
      toast.success("游戏启动成功");
    }
  }

  function onEditGame(id: string) {
    setSelectedGameId(id);
    setGameSettingsOpen(true);
  }

  function onDeleteGame(id: string) {
    const game = games.find((g) => g.id === id);
    setPendingDeleteId(id);
    setPendingDeleteTitle(game?.title ?? "");
    setDeleteConfirmOpen(true);
  }

  async function confirmDeleteGame() {
    if (!pendingDeleteId) return;
    const id = pendingDeleteId;
    setDeleteConfirmOpen(false);
    setPendingDeleteId(null);
    const success = await handleDeleteGame(id);
    if (success) {
      toast.success("游戏删除成功");
    }
  }

  return (
    <div className="flex h-screen flex-col overflow-hidden">
      <TitleBar
        onManage={() => setManageOpen(true)}
        onSettings={() => setSettingsOpen(true)}
        onImport={openImportDialog}
        onScan={() => setScanOpen(true)}
      />

      <div className="flex flex-1 overflow-hidden pt-10">
        <main className="flex-1">
          <ScrollArea className="h-full">
            <div className="container mx-auto py-6">
              <GameLibraryHeader
                count={filteredGames.length}
                search={searchQuery}
                viewMode={viewMode}
                onSearchChange={setSearchQuery}
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
        open={importOpen}
        loading={importLoading}
        initialExecutablePath={pendingImportPath}
        onOpenChange={setImportOpen}
        onSubmit={handleImportSubmit}
      />
      <ScanDialog open={scanOpen} loading={scanLoading} onOpenChange={setScanOpen} onSubmit={handleScanSubmit} />

      <GameSettingsDialog
        open={gameSettingsOpen}
        game={selectedGame}
        loading={saveLoading}
        onOpenChange={setGameSettingsOpen}
        onSave={handleGameSave}
        onRefreshCover={handleRefreshCover}
      />

      <ManagementDialog
        open={manageOpen}
        showStatusBar={showStatusBar}
        onOpenChange={setManageOpen}
        onShowStatusBarChange={setShowStatusBar}
        onDownloadNwjs={handleDownloadNwjs}
        onCleanupContainers={handleCleanupContainers}
        onUpdateEngine={handleUpdateEngine}
        onRemoveEngine={handleRemoveEngine}
      />

      <SettingsDialog
        open={settingsOpen}
        themeMode={themeMode}
        onOpenChange={setSettingsOpen}
        onThemeModeChange={setThemeMode}
      />

      <ConfirmDeleteDialog
        open={deleteConfirmOpen}
        title={pendingDeleteTitle}
        onOpenChange={setDeleteConfirmOpen}
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
