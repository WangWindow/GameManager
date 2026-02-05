/**
 * 引擎类型常量
 */

import { EngineType, ENGINE_DISPLAY_NAMES } from '@/types/engine'

/**
 * 所有支持的引擎类型
 */
export const SUPPORTED_ENGINES = [
  EngineType.RpgMakerVX,
  EngineType.RpgMakerVXAce,
  EngineType.RpgMakerMV,
  EngineType.RpgMakerMZ,
  EngineType.RenPy,
  EngineType.Other,
]

/**
 * 获取引擎显示名称
 */
export function getEngineDisplayName(engineType: string): string {
  return ENGINE_DISPLAY_NAMES[engineType as EngineType] || engineType
}

/**
 * 引擎图标映射
 */
export const ENGINE_ICONS: Record<string, string> = {
  [EngineType.RpgMakerVX]: 'ri:gamepad-line',
  [EngineType.RpgMakerVXAce]: 'ri:gamepad-line',
  [EngineType.RpgMakerMV]: 'ri:gamepad-line',
  [EngineType.RpgMakerMZ]: 'ri:gamepad-line',
  [EngineType.RenPy]: 'ri:book-2-line',
  [EngineType.Other]: 'ri:question-line',
}

/**
 * 获取引擎图标
 */
export function getEngineIcon(engineType: string): string {
  return ENGINE_ICONS[engineType] || ENGINE_ICONS[EngineType.Other]
}
