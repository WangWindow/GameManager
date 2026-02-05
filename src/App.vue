<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import TitleBar from '@/components/layouts/TitleBar.vue'
import ImportDialog from '@/components/games/ImportDialog.vue'
import ScanDialog from '@/components/games/ScanDialog.vue'
import GameGrid from '@/components/games/GameGrid.vue'
import GameSettingsDialog from '@/components/games/GameSettingsDialog.vue'
import GameLibraryHeader from '@/components/games/GameLibraryHeader.vue'
import ManagementDialog from '@/components/settings/ManagementDialog.vue'
import SettingsDialog from '@/components/settings/SettingsDialog.vue'
import ConfirmDeleteDialog from '@/components/common/ConfirmDeleteDialog.vue'
import { Progress } from '@/components/ui/progress'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Toaster } from '@/components/ui/sonner'
import { toast } from 'vue-sonner'
import { useGames } from '@/hooks/useGames'
import type { GameDto } from '@/types'
import { useThemeMode } from '@/hooks/useThemeMode'
import { useTaskStatus } from '@/hooks/useTaskStatus'
import { useTauriEvents } from '@/hooks/useTauriEvents'
import { useGameLibraryActions } from '@/hooks/useGameLibraryActions'
import { useMaintenanceActions } from '@/hooks/useMaintenanceActions'

const manageOpen = ref(false)
const settingsOpen = ref(false)
const importOpen = ref(false)
const scanOpen = ref(false)
const showStatusBar = ref(true)
const searchQuery = ref('')
const gameSettingsOpen = ref(false)
const selectedGameId = ref<string | null>(null)
const deleteConfirmOpen = ref(false)
const pendingDeleteId = ref<string | null>(null)
const pendingDeleteTitle = ref('')

const { games, loading, fetchGames, handleLaunchGame, handleDeleteGame } = useGames()
const { themeMode } = useThemeMode()
const { currentTask, statusBarVisible: taskStatusVisible, updateTask } = useTaskStatus()
useTauriEvents(updateTask)
const {
  importLoading,
  scanLoading,
  handleImportSubmit,
  handleScanSubmit,
  handleGameSave,
} = useGameLibraryActions({
  refresh: fetchGames,
  updateTask,
  closeImport: () => (importOpen.value = false),
  closeScan: () => (scanOpen.value = false),
  closeGameSettings: () => (gameSettingsOpen.value = false),
})
const { handleDownloadNwjs, handleCleanupContainers, handleUpdateEngine, handleRemoveEngine } =
  useMaintenanceActions({ updateTask })

const filteredGames = computed(() => {
  if (!searchQuery.value) return games.value
  const query = searchQuery.value.toLowerCase()
  return games.value.filter(
    (game: GameDto) =>
      game.title.toLowerCase().includes(query) ||
      game.engineType.toLowerCase().includes(query)
  )
})

const statusBarVisible = computed(() => showStatusBar.value && taskStatusVisible.value)
const selectedGame = computed(() =>
  selectedGameId.value ? games.value.find((g) => g.id === selectedGameId.value) ?? null : null
)

watch(
  showStatusBar,
  (val) => {
    localStorage.setItem('gm_show_status_bar', String(val))
  },
  { immediate: true }
)

onMounted(() => {
  const saved = localStorage.getItem('gm_show_status_bar')
  if (saved !== null) showStatusBar.value = saved === 'true'
  fetchGames()
  window.addEventListener('gm:refresh-games', handleRefresh)
})

onUnmounted(() => {
  window.removeEventListener('gm:refresh-games', handleRefresh)
})

function handleRefresh() {
  fetchGames()
}

async function onLaunchGame(id: string) {
  const success = await handleLaunchGame(id)
  if (success) {
    toast.success('游戏启动成功')
  }
}

function onEditGame(id: string) {
  selectedGameId.value = id
  gameSettingsOpen.value = true
}

async function onDeleteGame(id: string) {
  const game = games.value.find((g) => g.id === id)
  pendingDeleteId.value = id
  pendingDeleteTitle.value = game?.title ?? ''
  deleteConfirmOpen.value = true
}

async function confirmDeleteGame() {
  if (!pendingDeleteId.value) return
  const id = pendingDeleteId.value
  deleteConfirmOpen.value = false
  pendingDeleteId.value = null
  const success = await handleDeleteGame(id)
  if (success) {
    toast.success('游戏删除成功')
  }
}
</script>

<template>
  <div class="flex h-screen flex-col overflow-hidden">
    <!-- 标题栏 - 固定在顶部 -->
    <TitleBar @manage="manageOpen = true" @settings="settingsOpen = true" @import="importOpen = true"
      @scan="scanOpen = true" />

    <!-- 主内容区 - 标题栏下方 -->
    <div class="flex flex-1 overflow-hidden pt-10">
      <!-- 页面内容 - 可滚动 -->
      <main class="flex-1">
        <ScrollArea class="h-full">
          <div class="container mx-auto py-6">
            <GameLibraryHeader :count="filteredGames.length" :search="searchQuery"
              @update:search="(v) => (searchQuery = v)" />

            <GameGrid :games="filteredGames" :loading="loading" @launch="onLaunchGame" @edit="onEditGame"
              @delete="onDeleteGame" />
          </div>
        </ScrollArea>
      </main>
    </div>

    <ImportDialog v-model:open="importOpen" :loading="importLoading" @submit="handleImportSubmit" />
    <ScanDialog v-model:open="scanOpen" :loading="scanLoading" @submit="handleScanSubmit" />

    <GameSettingsDialog v-model:open="gameSettingsOpen" :game="selectedGame" @save="handleGameSave" />

    <ManagementDialog v-model:open="manageOpen" :show-status-bar="showStatusBar"
      @update:showStatusBar="(v) => (showStatusBar = v)" @downloadNwjs="handleDownloadNwjs"
      @cleanupContainers="handleCleanupContainers" @updateEngine="handleUpdateEngine"
      @removeEngine="handleRemoveEngine" />

    <SettingsDialog v-model:open="settingsOpen" :theme-mode="themeMode" @update:themeMode="(v) => (themeMode = v)" />

    <ConfirmDeleteDialog :open="deleteConfirmOpen" :title="pendingDeleteTitle"
      @update:open="(v) => (deleteConfirmOpen = v)" @confirm="confirmDeleteGame" />

    <div v-if="statusBarVisible" class="border-t bg-background/70 px-4 py-2 text-xs backdrop-blur">
      <div class="flex items-center justify-between gap-3">
        <div class="min-w-0 truncate">
          <span class="font-medium">状态：</span>
          <span class="text-muted-foreground">{{ currentTask?.label }}</span>
        </div>
        <div class="flex items-center gap-2">
          <div class="w-40">
            <Progress :model-value="currentTask?.progress ?? 0" />
          </div>
          <span class="tabular-nums text-muted-foreground">{{ currentTask?.progress ?? 0 }}%</span>
        </div>
      </div>
    </div>

    <Toaster position="top-right" />
  </div>
</template>
