/**
 * 拖拽导入 hook
 *
 * 封装游戏文件拖拽导入的逻辑，支持 Tauri 和 Web 两种环境
 */

import { useState, useRef, useEffect, useCallback } from 'react'
import { toast } from 'sonner'

export interface DragDropState {
  /** 是否正在拖拽 */
  isDragging: boolean
  /** 拖拽导入的文件路径 */
  droppedPath: string
  /** 清除拖拽路径 */
  clearDroppedPath: () => void
}

/**
 * 拖拽导入状态管理
 *
 * @param onDrop 可选的拖拽完成回调
 * @returns 拖拽状态和控制函数
 *
 * @example
 * const { isDragging, droppedPath, clearDroppedPath } = useDragDrop()
 *
 * useEffect(() => {
 *   if (droppedPath) {
 *     openImportDialog(droppedPath)
 *     clearDroppedPath()
 *   }
 * }, [droppedPath])
 */
export function useDragDrop(onDrop?: (path: string) => void): DragDropState {
  const [isDragging, setIsDragging] = useState(false)
  const [droppedPath, setDroppedPath] = useState('')
  const unlistenRef = useRef<(() => void) | null>(null)
  const onDropRef = useRef(onDrop)

  // 保持回调引用最新
  useEffect(() => {
    onDropRef.current = onDrop
  }, [onDrop])

  const clearDroppedPath = useCallback(() => {
    setDroppedPath('')
  }, [])

  const handleFileDrop = useCallback((path: string) => {
    setDroppedPath(path)
    if (onDropRef.current) {
      onDropRef.current(path)
    }
    toast.info('已选择拖拽文件，请选择引擎类型')
  }, [])

  // Tauri 拖拽事件处理
  useEffect(() => {
    let mounted = true

    async function initTauriDragDrop() {
      try {
        const mod = await import('@tauri-apps/api/webviewWindow')
        const win = mod.getCurrentWebviewWindow()

        unlistenRef.current = await win.onDragDropEvent((event) => {
          if (!mounted) return

          const payload = 'payload' in event ? event.payload : event
          const type = (payload as { type?: string }).type ?? ''
          const paths = (payload as { paths?: string[] }).paths ?? []

          if (type === 'enter' || type === 'over') {
            setIsDragging(true)
            return
          }

          if (type === 'leave') {
            setIsDragging(false)
            return
          }

          if (type === 'drop') {
            setIsDragging(false)
            const path = paths[0]
            if (path) {
              handleFileDrop(path)
            } else {
              toast.error('仅支持拖拽本地可执行文件')
            }
          }
        })
      } catch {
        // Web 环境使用 DOM 事件兜底
        setupDomDragDrop()
      }
    }

    // DOM 拖拽事件处理函数
    function handleDragEnter(event: DragEvent) {
      if (!event.dataTransfer?.types?.includes('Files')) return
      setIsDragging(true)
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
      setIsDragging(false)
    }

    function handleDrop(event: DragEvent) {
      if (!event.dataTransfer?.files?.length) {
        setIsDragging(false)
        return
      }
      event.preventDefault()
      const file = event.dataTransfer.files[0]
      const path = (file as File & { path?: string }).path
      setIsDragging(false)
      if (!path) {
        toast.error('仅支持拖拽本地可执行文件')
        return
      }
      handleFileDrop(path)
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

    void initTauriDragDrop()

    return () => {
      mounted = false
      teardownDomDragDrop()
      if (typeof unlistenRef.current === 'function') {
        try {
          unlistenRef.current()
        } catch {
          // ignore
        }
      }
      unlistenRef.current = null
    }
  }, [handleFileDrop])

  return { isDragging, droppedPath, clearDroppedPath }
}
