/** 引擎插件描述（从后端 EngineRegistry 动态获取） */
export interface EngineProfile {
  id: string
  name: string
  category: string
  icon: string
  priority: number
  description: string
}

/** 引擎运行时 DTO（已安装的 NW.js 等运行时） */
export interface EngineDto {
  id: string
  name: string
  version: string
  engineType: string
  path: string
  installedAt: number
}

export interface EngineUpdateInfo {
  engineId: string
  currentVersion: string
  latestVersion: string
  updateAvailable: boolean
}

export interface EngineUpdateResult {
  engineId: string
  updated: boolean
  fromVersion: string
  toVersion: string
  installDir?: string
}
