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

type ViewMode = 'grid' | 'list'

const manageOpen = ref(false)
const settingsOpen = ref(false)
const importOpen = ref(false)
const scanOpen = ref(false)
const showStatusBar = ref(true)
const searchQuery = ref('')
const viewMode = ref<ViewMode>('list')
const gameSettingsOpen = ref(false)
const selectedGameId = ref<string | null>(null)
const deleteConfirmOpen = ref(false)
const pendingDeleteId = ref<string | null>(null)
const pendingDeleteTitle = ref('')
const pendingImportPath = ref('')
const isDragging = ref(false)
const dragCounter = ref(0)
let unlistenDragDrop: (() => void) | null = null

const { games, loading, fetchGames, handleLaunchGame, handleDeleteGame } = useGames()
const { themeMode } = useThemeMode()
const { currentTask, statusBarVisible: taskStatusVisible, updateTask } = useTaskStatus()
useTauriEvents(updateTask)
const closeImportDialog = () => {
  importOpen.value = false
  pendingImportPath.value = ''
}

const {
  importLoading,
  scanLoading,
  handleImportSubmit,
  handleScanSubmit,
  handleGameSave,
  handleRefreshCover,
} = useGameLibraryActions({
  refresh: fetchGames,
  updateTask,
  closeImport: closeImportDialog,
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

watch(
  viewMode,
  (val) => {
    localStorage.setItem('gm_game_view_mode', val)
  },
  { immediate: true }
)

onMounted(() => {
  const saved = localStorage.getItem('gm_show_status_bar')
  if (saved !== null) showStatusBar.value = saved === 'true'
  const savedView = localStorage.getItem('gm_game_view_mode')
  if (savedView === 'grid' || savedView === 'list') {
    viewMode.value = savedView
  }
  fetchGames()
  window.addEventListener('gm:refresh-games', handleRefresh)
  void initDragDrop()
})

onUnmounted(() => {
  window.removeEventListener('gm:refresh-games', handleRefresh)
  teardownDomDragDrop()
  try {
    unlistenDragDrop?.()
  } catch {
    // ignore
  }
  unlistenDragDrop = null
})

function handleRefresh() {
  fetchGames()
}

async function initDragDrop() {
  try {
    const mod = await import('@tauri-apps/api/webviewWindow')
    const win = mod.getCurrentWebviewWindow()
    unlistenDragDrop = await win.onDragDropEvent((event) => {
      const payload = 'payload' in event ? event.payload : event
      const type = (payload as { type?: string }).type ?? ''
      const paths = (payload as { paths?: string[] }).paths ?? []

      if (type === 'enter' || type === 'over') {
        isDragging.value = true
        return
      }

      if (type === 'leave') {
        isDragging.value = false
        dragCounter.value = 0
        return
      }

      if (type === 'drop') {
        isDragging.value = false
        dragCounter.value = 0
        const path = paths[0]
        if (path) {
          pendingImportPath.value = path
          importOpen.value = true
          toast.info('已选择拖拽文件，请选择引擎类型')
        } else {
          toast.error('仅支持拖拽本地可执行文件')
        }
      }
    })
  } catch (e) {
    // Web 环境使用 DOM 事件兜底
    setupDomDragDrop()
  }
}

function setupDomDragDrop() {
  window.addEventListener('dragenter', handleDragEnter)
  window.addEventListener('dragover', handleDragOver)
  window.addEventListener('dragleave', handleDragLeave)
  window.addEventListener('drop', handleDrop)
}

function teardownDomDragDrop() {
  window.removeEventListener('dragenter', handleDragEnter)
  window.removeEventListener('dragover', handleDragOver)
  window.removeEventListener('dragleave', handleDragLeave)
  window.removeEventListener('drop', handleDrop)
}

function handleDragEnter(event: DragEvent) {
  if (!event.dataTransfer?.types?.includes('Files')) return
  dragCounter.value += 1
  isDragging.value = true
}

function handleDragOver(event: DragEvent) {
  if (!event.dataTransfer?.types?.includes('Files')) return
  event.preventDefault()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'copy'
  }
}

function handleDragLeave(event: DragEvent) {
  if (!event.dataTransfer?.types?.includes('Files')) return
  dragCounter.value -= 1
  if (dragCounter.value <= 0) {
    dragCounter.value = 0
    isDragging.value = false
  }
}

function handleDrop(event: DragEvent) {
  if (!event.dataTransfer?.files?.length) {
    dragCounter.value = 0
    isDragging.value = false
    return
  }
  event.preventDefault()
  const file = event.dataTransfer.files[0]
  const path = (file as File & { path?: string }).path
  dragCounter.value = 0
  isDragging.value = false
  if (!path) {
    toast.error('仅支持拖拽本地可执行文件')
    return
  }
  pendingImportPath.value = path
  importOpen.value = true
  toast.info('已选择拖拽文件，请选择引擎类型')
}

function openImportDialog() {
  pendingImportPath.value = ''
  importOpen.value = true
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
    <TitleBar @manage="manageOpen = true" @settings="settingsOpen = true" @import="openImportDialog"
      @scan="scanOpen = true" />

    <!-- 主内容区 - 标题栏下方 -->
    <div class="flex flex-1 overflow-hidden pt-10">
      <!-- 页面内容 - 可滚动 -->
      <main class="flex-1">
        <ScrollArea class="h-full">
          <div class="container mx-auto py-6">
            <GameLibraryHeader :count="filteredGames.length" :search="searchQuery" :view-mode="viewMode"
              @update:search="(v) => (searchQuery = v)" @update:view-mode="(v) => (viewMode = v)" />

            <GameGrid :games="filteredGames" :loading="loading" :view-mode="viewMode" @launch="onLaunchGame"
              @edit="onEditGame" @delete="onDeleteGame" />
          </div>
        </ScrollArea>
      </main>
    </div>

    <ImportDialog v-model:open="importOpen" :loading="importLoading" :initial-executable-path="pendingImportPath"
      @submit="handleImportSubmit" />
    <ScanDialog v-model:open="scanOpen" :loading="scanLoading" @submit="handleScanSubmit" />

    <GameSettingsDialog v-model:open="gameSettingsOpen" :game="selectedGame" @save="handleGameSave"
      @refreshCover="handleRefreshCover" />

    <ManagementDialog v-model:open="manageOpen" :show-status-bar="showStatusBar"
      @update:showStatusBar="(v) => (showStatusBar = v)" @downloadNwjs="handleDownloadNwjs"
      @cleanupContainers="handleCleanupContainers" @updateEngine="handleUpdateEngine"
      @removeEngine="handleRemoveEngine" />

    <SettingsDialog v-model:open="settingsOpen" :theme-mode="themeMode" @update:themeMode="(v) => (themeMode = v)" />

    <ConfirmDeleteDialog :open="deleteConfirmOpen" :title="pendingDeleteTitle"
      @update:open="(v) => (deleteConfirmOpen = v)" @confirm="confirmDeleteGame" />

    <Transition enter-active-class="duration-200 ease-out" enter-from-class="opacity-0" enter-to-class="opacity-100"
      leave-active-class="duration-150 ease-in" leave-from-class="opacity-100" leave-to-class="opacity-0">
      <div v-if="isDragging"
        class="pointer-events-none fixed inset-0 z-50 flex items-center justify-center bg-background/70 backdrop-blur">
        <div class="rounded-2xl border-2 border-dashed border-primary/50 bg-background/80 px-8 py-6 text-center">
          <div class="mb-2 text-sm font-semibold">拖拽文件以导入游戏</div>
          <div class="text-xs text-muted-foreground">仅支持可执行文件</div>
        </div>
      </div>
    </Transition>

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
