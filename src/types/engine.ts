/**
 * 引擎相关类型定义
 */

/**
 * 引擎数据传输对象
 */
export interface EngineDto {
  /** 引擎ID */
  id: string
  /** 引擎名称 */
  name: string
  /** 引擎版本 */
  version: string
  /** 引擎类型 */
  engineType: string
  /** 引擎路径 */
  path: string
  /** 安装时间（Unix毫秒时间戳） */
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

/**
 * 引擎类型枚举
 */
export enum EngineType {
  RpgMakerVX = 'rpgmakervx',
  RpgMakerVXAce = 'rpgmakervxace',
  RpgMakerMV = 'rpgmakermv',
  RpgMakerMZ = 'rpgmakermz',
  NWjs = 'nwjs',
  RenPy = 'renpy',
  Other = 'other',
}

/**
 * 引擎显示名称映射
 */
export const ENGINE_DISPLAY_NAMES: Record<EngineType, string> = {
  [EngineType.RpgMakerVX]: 'RPG Maker VX',
  [EngineType.RpgMakerVXAce]: 'RPG Maker VX Ace',
  [EngineType.RpgMakerMV]: 'RPG Maker MV (NW.js)',
  [EngineType.RpgMakerMZ]: 'RPG Maker MZ (NW.js)',
  [EngineType.NWjs]: 'NWjs',
  [EngineType.RenPy]: 'RenPy',
  [EngineType.Other]: 'Other',
}
