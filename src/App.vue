<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watchEffect } from "vue";

import TitleBar from "./components/layout/TitleBar.vue";
import GameList from "./features/games/GameList.vue";
import GameSettingsModal from "./features/games/GameSettingsModal.vue";
import SettingsModal from "./features/settings/SettingsModal.vue";
import { invokeResult, safeInvoke } from "./lib/tauri";

type EngineType = "rmmz" | "rmmv" | "unknown";

type GameEntry = {
  id: string;
  title: string;
  engineType: EngineType;
  version?: string;
  path: string;
  pathValid: boolean;
};

const games = ref<GameEntry[]>([]);

const selectedGameId = ref("");

const showStatusBar = ref(true);
watchEffect(() => {
  const saved = localStorage.getItem("gm_show_status_bar");
  if (saved !== null) showStatusBar.value = saved === "true";
});
watchEffect(() => {
  localStorage.setItem("gm_show_status_bar", String(showStatusBar.value));
});

type TaskStatus = { label: string; progress: number };

const currentTask = ref<TaskStatus | null>(null);
const statusBarVisible = computed(() => showStatusBar.value || currentTask.value !== null);

const settingsOpen = ref(false);

const gameSettingsOpen = ref(false);
const gameSettingsGameId = ref<string | null>(null);
const gameSettingsGame = computed(() => {
  if (!gameSettingsGameId.value) return null;
  return games.value.find((g) => g.id === gameSettingsGameId.value) ?? null;
});

function openSettings() {
  settingsOpen.value = true;
}

function openGameSettings(id: string) {
  gameSettingsGameId.value = id;
  gameSettingsOpen.value = true;
}

function setStatus(label: string, progress: number = 0) {
  currentTask.value = { label, progress: Math.max(0, Math.min(100, Number(progress) || 0)) };
}

async function refreshGames(selectId?: string) {
  const list = await safeInvoke<
    Array<{
      id: string;
      title: string;
      engineType: string;
      path: string;
      pathValid: boolean;
      runtimeVersion?: string | null;
    }>
  >("list_games");

  if (!list) return;
  games.value = list.map((g) => ({
    id: g.id,
    title: g.title,
    engineType: (g.engineType as EngineType) ?? "unknown",
    path: g.path,
    pathValid: g.pathValid,
  }));

  selectedGameId.value = selectId ?? games.value[0]?.id ?? "";
}

onMounted(async () => {
  // Listen to Tauri events when running inside the desktop app.
  try {
    const { listen } = await import("@tauri-apps/api/event");
    await listen<{
      taskId: string;
      version: string;
      flavor: "normal" | "sdk";
      target: string;
      downloaded: number;
      total?: number | null;
      percent?: number | null;
    }>("nwjs_download_progress", (event) => {
      const p = event.payload?.percent ?? 0;
      currentTask.value = {
        label: `下载 NW.js ${event.payload.version}（${event.payload.flavor}）`,
        progress: Math.max(0, Math.min(100, Number(p) || 0)),
      };
    });

    await listen<{ taskId: string; label: string; progress: number }>("scan_progress", (event) => {
      currentTask.value = {
        label: event.payload?.label ?? "扫描中…",
        progress: Math.max(0, Math.min(100, Number(event.payload?.progress ?? 0) || 0)),
      };
    });
  } catch {
    // ignore when not in tauri
  }

  await refreshGames();
});

function onKeyDown(e: KeyboardEvent) {
  if (e.key !== "Escape") return;
  if (gameSettingsOpen.value) gameSettingsOpen.value = false;
  if (settingsOpen.value) settingsOpen.value = false;
}

onMounted(() => {
  window.addEventListener("keydown", onKeyDown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeyDown);
});

async function pickDirectory(title: string): Promise<string | null> {
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const res = await open({ directory: true, multiple: false, title });
    if (!res) return null;
    if (Array.isArray(res)) return res[0] ?? null;
    return res;
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error("[dialog] open failed", err);
    return null;
  }
}

async function importGame() {
  const dir = await pickDirectory("选择游戏目录");
  if (!dir) return;

  setStatus("导入中…", 0);
  const res = await invokeResult<{
    id: string;
    title: string;
    engineType: string;
    path: string;
    pathValid: boolean;
  }>("import_game_dir", { path: dir });

  if (!res.ok) {
    setStatus(`导入失败：${res.error}`, 0);
    return;
  }

  await refreshGames(res.data.id);
  openGameSettings(res.data.id);
  setStatus(`已导入：${res.data.title}`, 100);
}

async function scanGames() {
  const root = await pickDirectory("选择扫描根目录");
  if (!root) return;

  setStatus("开始扫描…", 0);
  const res = await invokeResult<{
    taskId: string;
    scannedDirs: number;
    foundGames: number;
    imported: number;
    skippedExisting: number;
  }>("scan_games", { root, maxDepth: 6 });

  if (!res.ok) {
    setStatus(`扫描失败：${res.error}`, 0);
    return;
  }

  await refreshGames();
  setStatus(`扫描完成：新增 ${res.data.imported}（已存在 ${res.data.skippedExisting}）`, 100);
}

async function downloadNwjsStable() {
  const infoRes = await invokeResult<{ version: string; target: string }>("get_nwjs_stable_info");
  if (infoRes.ok) {
    setStatus(`准备下载 NW.js ${infoRes.data.version}…`, 0);
  } else {
    setStatus(`准备下载 NW.js…（${infoRes.error}）`, 0);
  }

  const dlRes = await invokeResult<{
    version: string;
    target: string;
    installDir: string;
  }>("download_nwjs_stable", { flavor: "normal" });

  if (dlRes.ok) {
    setStatus(`已安装 NW.js ${dlRes.data.version}`, 100);
    return;
  }

  setStatus(`下载失败：${dlRes.error}`, 0);
}

async function cleanupContainers() {
  setStatus("清理无用容器…", 0);
  // 中文说明：后端当前返回 { deleted }（表示删除了多少个 profile 目录）。
  const res = await invokeResult<{ deleted: number }>("cleanup_unused_containers");
  if (!res.ok) {
    setStatus(`清理失败：${res.error}`, 0);
    return;
  }
  setStatus(`已清理：无用容器 ${res.data.deleted} 个`, 100);
}

async function launchGame(gameId: string) {
  const game = games.value.find((g) => g.id === gameId);
  if (!game) return;
  if (!game.pathValid) {
    setStatus("启动失败：游戏路径无效", 0);
    return;
  }

  setStatus(`启动：${game.title}…`, 0);
  const res = await invokeResult<{ pid: number }>("launch_game", { gameId });
  if (!res.ok) {
    setStatus(`启动失败：${res.error}`, 0);
    return;
  }
  setStatus(`已启动（PID ${res.data.pid}）`, 100);
}

async function openPath(path: string) {
  try {
    const mod = await import("@tauri-apps/plugin-opener");
    await mod.openPath(path);
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error("[opener] open failed", err);
    setStatus("打开失败：缺少 opener 权限或插件不可用", 0);
  }
}

function engineLabel(engineType: EngineType) {
  switch (engineType) {
    case "rmmz":
      return "RPG Maker MZ";
    case "rmmv":
      return "RPG Maker MV";
    default:
      return "未知类型";
  }
}
</script>

<template>
  <div class="flex h-full flex-col">
    <TitleBar title="GameManager" subtitle="" @import="importGame" @scan="scanGames" @settings="openSettings" />

    <main class="flex min-h-0 flex-1 gap-4 p-4">
      <GameList :games="games" :selected-game-id="selectedGameId" :engine-label="engineLabel"
        @select="(id) => (selectedGameId = id)" @start="launchGame" @settings="openGameSettings" />
    </main>

    <footer v-if="statusBarVisible"
      class="flex items-center justify-between gap-3 border-t border-zinc-200/70 bg-white/70 px-4 py-2 text-xs backdrop-blur dark:border-zinc-800/80 dark:bg-zinc-950/50">
      <div class="min-w-0 truncate">
        <span class="font-medium">状态：</span>
        <span class="text-zinc-500 dark:text-zinc-400">{{ currentTask?.label ?? "空闲" }}</span>
      </div>
      <div class="flex items-center gap-2">
        <div class="h-1.5 w-28 overflow-hidden rounded-full bg-zinc-200 dark:bg-zinc-800" aria-hidden="true">
          <div class="h-full bg-zinc-900 dark:bg-zinc-50" :style="{ width: `${currentTask?.progress ?? 0}%` }" />
        </div>
        <span class="tabular-nums text-zinc-500 dark:text-zinc-400">{{ currentTask?.progress ?? 0 }}%</span>
      </div>
    </footer>

    <SettingsModal v-model:open="settingsOpen" :show-status-bar="showStatusBar"
      @update:showStatusBar="(v) => (showStatusBar = v)" @downloadNwjs="downloadNwjsStable"
      @cleanupContainers="cleanupContainers" />

    <GameSettingsModal v-model:open="gameSettingsOpen" :game="gameSettingsGame" :engine-label="engineLabel"
      @openPath="openPath"
      @deleted="async (id) => { await refreshGames(); if (selectedGameId === id) selectedGameId = games[0]?.id ?? ''; }"
      @titleUpdated="async (id) => { await refreshGames(id); selectedGameId = id; }" />
  </div>
</template>
