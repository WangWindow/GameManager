/**
 * 本地存储持久化状态 hook
 *
 * 自动将状态同步到 localStorage，并在初始化时恢复
 */

import { useState, useEffect, useCallback } from 'react'

/**
 * 创建与 localStorage 同步的状态
 *
 * @param key localStorage 键名
 * @param defaultValue 默认值
 * @returns [value, setValue] 类似 useState 的返回值
 *
 * @example
 * const [showStatusBar, setShowStatusBar] = usePersistedState('gm_show_status_bar', true)
 * const [viewMode, setViewMode] = usePersistedState<'grid' | 'list'>('gm_view_mode', 'list')
 */
export function usePersistedState<T>(
  key: string,
  defaultValue: T
): [T, (value: T | ((prev: T) => T)) => void] {
  // 惰性初始化，从 localStorage 读取
  const [value, setValue] = useState<T>(() => {
    try {
      const saved = localStorage.getItem(key)
      if (saved !== null) {
        return JSON.parse(saved) as T
      }
    } catch {
      // JSON 解析失败时使用默认值
    }
    return defaultValue
  })

  // 值变化时同步到 localStorage
  useEffect(() => {
    try {
      localStorage.setItem(key, JSON.stringify(value))
    } catch {
      // 存储失败时静默处理
    }
  }, [key, value])

  return [value, setValue]
}

/**
 * 布尔值专用的持久化状态
 *
 * @param key localStorage 键名
 * @param defaultValue 默认值
 * @returns [value, setValue, toggle] 包含 toggle 函数
 */
export function usePersistedBoolean(
  key: string,
  defaultValue: boolean
): [boolean, (value: boolean) => void, () => void] {
  const [value, setValue] = usePersistedState(key, defaultValue)

  const toggle = useCallback(() => {
    setValue((prev) => !prev)
  }, [setValue])

  return [value, setValue, toggle]
}
