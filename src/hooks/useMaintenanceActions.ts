import { ref } from 'vue'
import { toast } from 'vue-sonner'
import {
  cleanupUnusedContainers,
  deleteEngine,
  downloadNwjsStable,
  getEngineUpdateInfo,
  getNwjsStableInfo,
  updateEngine,
} from '@/lib/api'
import type { EngineDto } from '@/types'

interface Options {
  updateTask: (label: string, progress: number) => void
}

export function useMaintenanceActions(options: Options) {
  const maintenanceLoading = ref(false)

  async function handleDownloadNwjs() {
    if (maintenanceLoading.value) return
    maintenanceLoading.value = true
    try {
      const info = await getNwjsStableInfo()
      options.updateTask(`下载 NW.js ${info.version}`, 0)
      await downloadNwjsStable('normal')
      toast.success('NW.js 下载完成')
      options.updateTask('下载完成', 100)
      window.dispatchEvent(new CustomEvent('gm:refresh-engines'))
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

  async function handleUpdateEngine(engine: EngineDto) {
    const info = await getEngineUpdateInfo(engine.id)
    if (!info.updateAvailable) {
      toast.info('已是最新版本')
      return
    }
    const result = await updateEngine(engine.id)
    if (result.updated) {
      toast.success(`已更新到 ${result.toVersion}`)
      window.dispatchEvent(new CustomEvent('gm:refresh-engines'))
    }
  }

  async function handleRemoveEngine(engine: EngineDto) {
    if (maintenanceLoading.value) return
    maintenanceLoading.value = true
    try {
      await deleteEngine(engine.id)
      toast.success(`已卸载 ${engine.name}`)
      window.dispatchEvent(new CustomEvent('gm:refresh-engines'))
    } catch (e) {
      const msg = e instanceof Error ? e.message : '卸载失败'
      toast.error(msg)
    } finally {
      maintenanceLoading.value = false
    }
  }

  return {
    maintenanceLoading,
    handleDownloadNwjs,
    handleCleanupContainers,
    handleUpdateEngine,
    handleRemoveEngine,
  }
}
