import { useCallback, useEffect, useState, useMemo } from "react"
import { invoke } from "@tauri-apps/api/core"
import type { EngineProfile } from "@/types/engine"

interface EngineRegistry {
  engines: EngineProfile[]
  /** 仅已启用的引擎（导入选择器用） */
  enabledEngines: EngineProfile[]
  /** 按 category 分组的引擎列表 */
  categories: Map<string, EngineProfile[]>
  /** 按 ID 查找引擎 */
  getById: (id: string) => EngineProfile | undefined
  /** 获取引擎显示名称 */
  getName: (id: string) => string
  /** 获取引擎图标 */
  getIcon: (id: string) => string
  /** 获取引擎 category */
  getCategory: (id: string) => string
  /** 属于某个 category 的引擎 ID 列表 */
  getIdsByCategory: (category: string) => string[]
  /** 加载状态 */
  loading: boolean
  /** 清除模块级缓存并重新拉取引擎注册表 */
  refresh: () => void
}

let cachedEngines: EngineProfile[] | null = null
let pendingEngines: Promise<EngineProfile[]> | null = null
let pendingIsRefresh = false

function loadEngines(force = false): Promise<EngineProfile[]> {
  if (!force && cachedEngines) {
    return Promise.resolve(cachedEngines)
  }
  if (force && !pendingIsRefresh) {
    pendingEngines = null
    pendingIsRefresh = true
  }
  if (!pendingEngines) {
    pendingEngines = invoke<EngineProfile[]>("get_engine_registry")
      .then((data) => {
        cachedEngines = data
        return data
      })
      .finally(() => {
        pendingEngines = null
        pendingIsRefresh = false
      })
  }
  return pendingEngines
}

export function useEngineRegistry(): EngineRegistry {
  const [engines, setEngines] = useState<EngineProfile[]>(cachedEngines ?? [])
  const [loading, setLoading] = useState(!cachedEngines)

  const fetchEngines = useCallback((force = false) => {
    setLoading(true)
    loadEngines(force)
      .then((data) => {
        setEngines(data)
      })
      .catch(console.error)
      .finally(() => setLoading(false))
  }, [])

  const refresh = useCallback(() => {
    cachedEngines = null
    fetchEngines(true)
  }, [fetchEngines])

  useEffect(() => {
    if (cachedEngines) return
    fetchEngines()
  }, [fetchEngines])

  // gm:refresh-engines 的缓存失效入口：由 PluginsDialog/useMaintenanceActions 派发
  useEffect(() => {
    const handler = () => refresh()
    window.addEventListener("gm:refresh-engines", handler)
    return () => window.removeEventListener("gm:refresh-engines", handler)
  }, [refresh])

  const categories = useMemo(() => {
    const map = new Map<string, EngineProfile[]>()
    for (const e of engines) {
      const list = map.get(e.category) ?? []
      list.push(e)
      map.set(e.category, list)
    }
    return map
  }, [engines])

  const engineMap = useMemo(
    () => new Map(engines.map((e) => [e.id, e])),
    [engines],
  )

  return {
    engines,
    enabledEngines: engines.filter((e) => e.enabled),
    categories,
    loading,
    getById: (id) => engineMap.get(id),
    getName: (id) => engineMap.get(id)?.name ?? id,
    getIcon: (id) => engineMap.get(id)?.icon ?? "ri:question-line",
    getCategory: (id) => engineMap.get(id)?.category ?? "other",
    getIdsByCategory: (cat) =>
      engines.filter((e) => e.category === cat).map((e) => e.id),
    refresh,
  }
}
