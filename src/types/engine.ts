/** 引擎插件描述（从后端 EngineRegistry 动态获取） */
export interface EngineProfile {
  id: string
  name: string
  category: string
  icon: string
  priority: number
  description: string
  enabled: boolean
  entryPatterns?: string[]
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

/** 插件管理面板用的引擎详情 */
export interface EngineDetail {
  id: string
  name: string
  category: string
  icon: string
  description: string
  enabled: boolean
  valid: boolean
  ruleCount: number
  strategy: string
  errors: string[]
}

/** 插件完整配置详情（查看内容用） */
export interface EngineProfileDetail {
  id: string
  name: string
  category: string
  icon: string
  description: string
  enabled: boolean
  valid: boolean
  detection: DetectionDetail
  launch: LaunchDetail
  errors: string[]
}

export interface DetectionDetail {
  minScore: number
  rules: RuleDetail[]
}

export interface RuleDetail {
  ruleType: string
  path: string
  pattern: string
  ext: string
  weight: number
}

export interface LaunchDetail {
  strategy: string
  entryPatterns: string[]
  excludePatterns: string[]
  args: string[]
  sandboxHome: boolean
  runtimeId: string
  program: string
}
