/**
 * 设置相关类型定义
 */

/**
 * 应用设置
 */
export interface AppSettings {
  /** 容器根目录 */
  containerRoot: string
}

/**
 * 设置容器根目录输入
 */
export interface SetContainerRootInput {
  /** 容器根目录路径 */
  containerRoot: string
}

/**
 * NW.js 稳定版信息
 */
export interface NwjsStableInfo {
  version: string
  target: string
  normalUrl: string
  sdkUrl: string
}

/**
 * NW.js 下载结果
 */
export interface NwjsInstallResult {
  taskId: string
  version: string
  flavor: 'normal' | 'sdk'
  target: string
  installDir: string
}

/**
 * 清理容器结果
 */
export interface CleanupResult {
  deleted: number
}

/**
 * 集成选项（可扩展）
 */
export interface IntegrationOptions {
  installed?: boolean
  bottles?: string[]
  defaultBottle?: string
}

/**
 * 集成状态
 */
export interface IntegrationStatus {
  key: string
  available: boolean
  enabled: boolean
  options?: IntegrationOptions
}

/**
 * 能力列表
 */
export interface Capabilities {
  integrations: IntegrationStatus[]
}

/**
 * 集成设置输入
 */
export interface IntegrationSettingsInput {
  key: string
  enabled?: boolean
  options?: IntegrationOptions
}
