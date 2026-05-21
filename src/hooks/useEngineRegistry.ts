import { useEffect, useState, useMemo } from "react"
import { invoke } from "@tauri-apps/api/core"
import type { EngineProfile } from "@/types/engine"

interface EngineRegistry {
  engines: EngineProfile[]
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
}

let cachedEngines: EngineProfile[] | null = null

export function useEngineRegistry(): EngineRegistry {
  const [engines, setEngines] = useState<EngineProfile[]>(cachedEngines ?? [])
  const [loading, setLoading] = useState(!cachedEngines)

  useEffect(() => {
    if (cachedEngines) return

    invoke<EngineProfile[]>("get_engine_registry")
      .then((data) => {
        cachedEngines = data
        setEngines(data)
      })
      .catch(console.error)
      .finally(() => setLoading(false))
  }, [])

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
    categories,
    loading,
    getById: (id) => engineMap.get(id),
    getName: (id) => engineMap.get(id)?.name ?? id,
    getIcon: (id) => engineMap.get(id)?.icon ?? "ri:question-line",
    getCategory: (id) => engineMap.get(id)?.category ?? "other",
    getIdsByCategory: (cat) =>
      engines.filter((e) => e.category === cat).map((e) => e.id),
  }
}
