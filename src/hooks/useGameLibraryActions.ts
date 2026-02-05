import { ref } from 'vue'
import { toast } from 'vue-sonner'
import { importGameDir, scanGames, saveGameSettings, updateGame, refreshGameCover } from '@/lib/api'
import type { GameConfig } from '@/types'

interface Options {
  refresh: () => void
  updateTask: (label: string, progress: number) => void
  closeImport: () => void
  closeScan: () => void
  closeGameSettings: () => void
}

export function useGameLibraryActions(options: Options) {
  const importLoading = ref(false)
  const scanLoading = ref(false)

  async function handleImportSubmit(payload: { executablePath: string; engineType: string }) {
    if (importLoading.value) return
    importLoading.value = true
    try {
      await importGameDir(payload.executablePath, payload.engineType)
      toast.success('导入成功')
      options.closeImport()
      options.refresh()
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
      options.updateTask('扫描完成', 100)
      options.closeScan()
      options.refresh()
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
      options.closeGameSettings()
      options.refresh()
    } catch (e) {
      const msg = e instanceof Error ? e.message : '保存失败'
      toast.error(msg)
    }
  }

  async function handleRefreshCover(id: string) {
    try {
      await refreshGameCover(id)
      toast.success('图标已更新')
      options.refresh()
    } catch (e) {
      const msg = e instanceof Error ? e.message : '图标更新失败'
      toast.error(msg)
    }
  }

  return {
    importLoading,
    scanLoading,
    handleImportSubmit,
    handleScanSubmit,
    handleGameSave,
    handleRefreshCover,
  }
}
