<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import TitleBar from '@/components/layout/TitleBar.vue'
import ImportDialog from '@/components/games/ImportDialog.vue'
import ScanDialog from '@/components/games/ScanDialog.vue'
import GameGrid from '@/components/games/GameGrid.vue'
import GameSettingsDialog from '@/components/games/GameSettingsDialog.vue'
import ManagementDialog from '@/components/settings/ManagementDialog.vue'
import SettingsDialog from '@/components/settings/SettingsDialog.vue'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Toaster } from '@/components/ui/sonner'
import { toast } from 'vue-sonner'
import { Input } from '@/components/ui/input'
import { Icon } from '@iconify/vue'
import { useGames } from '@/hooks/useGames'
import type { GameConfig, GameDto } from '@/types'
import {
  cleanupUnusedContainers,
  downloadNwjsStable,
  getNwjsStableInfo,
  importGameDir,
  scanGames,
  saveGameSettings,
  updateGame,
} from '@/lib/api'

type TaskStatus = { label: string; progress: number }

const manageOpen = ref(false)
const settingsOpen = ref(false)
const importOpen = ref(false)
const scanOpen = ref(false)
const importLoading = ref(false)
const scanLoading = ref(false)
const maintenanceLoading = ref(false)
const showStatusBar = ref(true)
const currentTask = ref<TaskStatus | null>(null)
const themeMode = ref<'system' | 'light' | 'dark'>('system')
const systemDark = ref(false)
const searchQuery = ref('')
const gameSettingsOpen = ref(false)
const selectedGameId = ref<string | null>(null)
const deleteConfirmOpen = ref(false)
const pendingDeleteId = ref<string | null>(null)
const pendingDeleteTitle = ref('')
let taskClearTimer: number | null = null

const { games, loading, fetchGames, handleLaunchGame, handleDeleteGame } = useGames()

const filteredGames = computed(() => {
  if (!searchQuery.value) return games.value
  const query = searchQuery.value.toLowerCase()
  return games.value.filter(
    (game: GameDto) =>
      game.title.toLowerCase().includes(query) ||
      game.engineType.toLowerCase().includes(query)
  )
})

const statusBarVisible = computed(() => showStatusBar.value && currentTask.value !== null)
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
  themeMode,
  (val) => {
    localStorage.setItem('gm_theme', val)
    applyTheme(val)
  },
  { immediate: true }
)

onMounted(async () => {
  const saved = localStorage.getItem('gm_show_status_bar')
  if (saved !== null) showStatusBar.value = saved === 'true'

  const theme = localStorage.getItem('gm_theme') as 'system' | 'light' | 'dark' | null
  if (theme) themeMode.value = theme

  try {
    const media = window.matchMedia('(prefers-color-scheme: dark)')
    systemDark.value = media.matches
    applyTheme(themeMode.value)
    const handler = (e: MediaQueryListEvent) => {
      systemDark.value = e.matches
      if (themeMode.value === 'system') applyTheme('system')
    }
    media.addEventListener('change', handler)
    onUnmounted(() => media.removeEventListener('change', handler))
  } catch {
    // ignore
  }

  try {
    const { listen } = await import('@tauri-apps/api/event')
    await listen<{
      taskId: string
      version: string
      flavor: 'normal' | 'sdk'
      target: string
      downloaded: number
      total?: number | null
      percent?: number | null
    }>('nwjs_download_progress', (event) => {
      const p = event.payload?.percent ?? 0
      updateTask(`下载 NW.js ${event.payload.version}（${event.payload.flavor}）`, p)
    })

    await listen<{ taskId: string; label: string; progress: number }>('scan_progress', (event) => {
      updateTask(event.payload?.label ?? '扫描中…', Number(event.payload?.progress ?? 0))
    })
  } catch {
    // ignore when not in tauri
  }

  fetchGames()
  window.addEventListener('gm:refresh-games', handleRefresh)
})

onUnmounted(() => {
  window.removeEventListener('gm:refresh-games', handleRefresh)
})

function handleRefresh() {
  fetchGames()
}

function applyTheme(mode: 'system' | 'light' | 'dark') {
  const root = document.documentElement
  const isDark = mode === 'dark' || (mode === 'system' && systemDark.value)
  if (isDark) {
    root.classList.add('dark')
  } else {
    root.classList.remove('dark')
  }
}

function updateTask(label: string, progress: number) {
  if (taskClearTimer) {
    window.clearTimeout(taskClearTimer)
    taskClearTimer = null
  }
  const safeProgress = Math.max(0, Math.min(100, Number(progress) || 0))
  currentTask.value = { label, progress: safeProgress }
  if (safeProgress >= 100) {
    taskClearTimer = window.setTimeout(() => {
      currentTask.value = null
    }, 1200)
  }
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

async function handleDownloadNwjs() {
  if (maintenanceLoading.value) return
  maintenanceLoading.value = true
  try {
    const info = await getNwjsStableInfo()
    updateTask(`下载 NW.js ${info.version}`, 0)
    await downloadNwjsStable('normal')
    toast.success('NW.js 下载完成')
    updateTask('下载完成', 100)
  } catch (e) {
    const msg = e instanceof Error ? e.message : '下载失败'
    toast.error(msg)
  } finally {
    maintenanceLoading.value = false
  }
}

async function handleCleanupContainers() {
  if (maintenanceLoading.value) return
  maintenanceLoading.value = true
  try {
    const res = await cleanupUnusedContainers()
    toast.success(`清理完成：${res.deleted} 个`)
  } catch (e) {
    const msg = e instanceof Error ? e.message : '清理失败'
    toast.error(msg)
  } finally {
    maintenanceLoading.value = false
  }
}

async function handleImportSubmit(payload: { path: string; engineType: string }) {
  if (importLoading.value) return
  importLoading.value = true
  try {
    await importGameDir(payload.path, payload.engineType)
    toast.success('导入成功')
    importOpen.value = false
    window.dispatchEvent(new CustomEvent('gm:refresh-games'))
  } catch (e) {
    const msg = e instanceof Error ? e.message : '导入失败'
    toast.error(msg)
  } finally {
    importLoading.value = false
  }
}

async function handleScanSubmit(payload: { root: string; maxDepth: number }) {
  if (scanLoading.value) return
  scanLoading.value = true
  try {
    const res = await scanGames(payload)
    toast.success(`扫描完成：新增 ${res.imported}，已存在 ${res.skippedExisting}`)
    updateTask('扫描完成', 100)
    scanOpen.value = false
    window.dispatchEvent(new CustomEvent('gm:refresh-games'))
  } catch (e) {
    const msg = e instanceof Error ? e.message : '扫描失败'
    toast.error(msg)
  } finally {
    scanLoading.value = false
  }
}

async function handleGameSave(payload: {
  id: string
  title: string
  engineType: string
  path: string
  runtimeVersion?: string
  settings: GameConfig
}) {
  try {
    await updateGame(payload.id, {
      title: payload.title,
      engineType: payload.engineType,
      path: payload.path,
      runtimeVersion: payload.runtimeVersion,
    })
    await saveGameSettings(payload.id, payload.settings)
    toast.success('已保存游戏设置')
    gameSettingsOpen.value = false
    window.dispatchEvent(new CustomEvent('gm:refresh-games'))
  } catch (e) {
    const msg = e instanceof Error ? e.message : '保存失败'
    toast.error(msg)
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
            <div class="mb-5 flex items-center justify-between">
              <h1 class="text-2xl font-semibold">游戏库</h1>

              <div class="mx-4 flex flex-1 items-center">
                <div class="relative w-full max-w-lg">
                  <Icon icon="ri:search-line"
                    class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                  <Input v-model="searchQuery" placeholder="搜索游戏..." class="h-8 pl-10 rounded-md border" />
                </div>
              </div>

              <span class="text-xs text-muted-foreground">{{ filteredGames.length }} 项</span>
            </div>

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
      @cleanupContainers="handleCleanupContainers" />

    <SettingsDialog v-model:open="settingsOpen" :theme-mode="themeMode" @update:themeMode="(v) => (themeMode = v)" />

    <Dialog :open="deleteConfirmOpen" @update:open="(v) => (deleteConfirmOpen = v)">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>确认删除</DialogTitle>
          <DialogDescription>
            确定要删除游戏{{ pendingDeleteTitle ? `「${pendingDeleteTitle}」` : '' }}吗？
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="ghost" @click="deleteConfirmOpen = false">取消</Button>
          <Button variant="destructive" @click="confirmDeleteGame">删除</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

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
