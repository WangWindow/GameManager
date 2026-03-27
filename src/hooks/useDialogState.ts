/**
 * 对话框状态管理 hook
 *
 * 提供统一的对话框开关状态管理，减少重复的 useState 代码
 */

import { useState, useCallback, useMemo } from 'react'

export interface DialogState<T = undefined> {
  /** 对话框是否打开 */
  isOpen: boolean
  /** 对话框附带的数据 */
  data: T | undefined
  /** 打开对话框 */
  open: (data?: T) => void
  /** 关闭对话框 */
  close: () => void
  /** 设置对话框状态（兼容 onOpenChange） */
  setOpen: (open: boolean) => void
}

/**
 * 创建单个对话框的状态管理
 *
 * @param initialData 初始数据
 * @returns 对话框状态和控制函数
 *
 * @example
 * const importDialog = useDialogState<string>()
 * // 打开对话框
 * importDialog.open('/path/to/file')
 * // 关闭对话框
 * importDialog.close()
 * // 在组件中使用
 * <Dialog open={importDialog.isOpen} onOpenChange={importDialog.setOpen} />
 */
export function useDialogState<T = undefined>(initialData?: T): DialogState<T> {
  const [isOpen, setIsOpen] = useState(false)
  const [data, setData] = useState<T | undefined>(initialData)

  const open = useCallback((newData?: T) => {
    setData(newData)
    setIsOpen(true)
  }, [])

  const close = useCallback(() => {
    setIsOpen(false)
    setData(undefined)
  }, [])

  const setOpen = useCallback((open: boolean) => {
    setIsOpen(open)
    if (!open) {
      setData(undefined)
    }
  }, [])

  return useMemo(
    () => ({ isOpen, data, open, close, setOpen }),
    [isOpen, data, open, close, setOpen]
  )
}

/**
 * 删除确认对话框专用状态
 */
export interface DeleteConfirmState {
  isOpen: boolean
  id: string | null
  title: string
  open: (id: string, title: string) => void
  close: () => void
  setOpen: (open: boolean) => void
}

/**
 * 创建删除确认对话框状态
 *
 * @example
 * const deleteConfirm = useDeleteConfirmState()
 * // 打开确认对话框
 * deleteConfirm.open(game.id, game.title)
 * // 确认删除
 * if (deleteConfirm.id) await deleteGame(deleteConfirm.id)
 */
export function useDeleteConfirmState(): DeleteConfirmState {
  const [isOpen, setIsOpen] = useState(false)
  const [id, setId] = useState<string | null>(null)
  const [title, setTitle] = useState('')

  const open = useCallback((newId: string, newTitle: string) => {
    setId(newId)
    setTitle(newTitle)
    setIsOpen(true)
  }, [])

  const close = useCallback(() => {
    setIsOpen(false)
    setId(null)
    setTitle('')
  }, [])

  const setOpen = useCallback((open: boolean) => {
    setIsOpen(open)
    if (!open) {
      setId(null)
      setTitle('')
    }
  }, [])

  return useMemo(
    () => ({ isOpen, id, title, open, close, setOpen }),
    [isOpen, id, title, open, close, setOpen]
  )
}
