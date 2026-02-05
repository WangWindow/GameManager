import { computed, ref } from 'vue'
import type { TaskStatus } from '@/types'

export function useTaskStatus() {
  const currentTask = ref<TaskStatus | null>(null)
  let taskClearTimer: number | null = null

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

  const statusBarVisible = computed(() => currentTask.value !== null)

  return { currentTask, statusBarVisible, updateTask }
}
