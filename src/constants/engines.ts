/**
 * 引擎类型常量 - 简化版本
 */

import { EngineType, ENGINE_DISPLAY_NAMES } from '@/types/engine'

export const ENGINE_OPTION_NWJS = 'nwjs'

/**
 * 导入时可选的引擎类型 (简化为3种)
 */
export const ENGINE_PICKER_OPTIONS = [
  { value: EngineType.RenPy, label: 'RenPy' },
  { value: ENGINE_OPTION_NWJS, label: 'NW.js' },
  { value: EngineType.Other, label: 'Other' },
]

/**
 * 筛选器选项
 */
export const ENGINE_FILTER_OPTIONS = [
  { value: 'all', label: '全部' },
  { value: EngineType.RenPy, label: 'RenPy' },
  { value: ENGINE_OPTION_NWJS, label: 'NW.js' },
  { value: EngineType.Other, label: 'Other' },
]

/**
 * 将选择的引擎类型转换为实际存储类型
 */
export function resolveSelectedEngineType(selected: string): string {
  if (selected === ENGINE_OPTION_NWJS) {
    return EngineType.RpgMakerMV
  }
  return selected
}

/**
 * 将存储的引擎类型转换为选择器显示类型
 */
export function normalizeEngineTypeForSelect(engineType: string): string {
  // NW.js 类型游戏统一显示为 nwjs
  if (
    engineType === EngineType.RpgMakerMV ||
    engineType === EngineType.RpgMakerMZ ||
    engineType === EngineType.RpgMakerVX ||
    engineType === EngineType.RpgMakerVXAce
  ) {
    return ENGINE_OPTION_NWJS
  }
  // Unity/Godot 归类为其他
  if (engineType === EngineType.Unity || engineType === EngineType.Godot) {
    return EngineType.Other
  }
  return engineType
}

/**
 * 获取引擎显示名称
 */
export function getEngineDisplayName(engineType: string): string {
  // NW.js 类引擎
  if (
    engineType === EngineType.RpgMakerMV ||
    engineType === EngineType.RpgMakerMZ ||
    engineType === EngineType.RpgMakerVX ||
    engineType === EngineType.RpgMakerVXAce ||
    engineType === ENGINE_OPTION_NWJS
  ) {
    return 'NW.js'
  }
  if (engineType === EngineType.RenPy) return 'RenPy'
  if (engineType === EngineType.Unity) return 'Unity'
  if (engineType === EngineType.Godot) return 'Godot'
  return ENGINE_DISPLAY_NAMES[engineType as EngineType] || 'Other'
}

/**
 * 引擎图标映射
 */
export const ENGINE_ICONS: Record<string, string> = {
  [EngineType.RpgMakerVX]: 'ri:window-line',
  [EngineType.RpgMakerVXAce]: 'ri:window-line',
  [EngineType.RpgMakerMV]: 'ri:window-line',
  [EngineType.RpgMakerMZ]: 'ri:window-line',
  [ENGINE_OPTION_NWJS]: 'ri:window-line',
  [EngineType.RenPy]: 'ri:slideshow-line',
  [EngineType.Unity]: 'ri:gamepad-line',
  [EngineType.Godot]: 'ri:gamepad-line',
  [EngineType.Other]: 'ri:question-line',
}

/**
 * 获取引擎图标
 */
export function getEngineIcon(engineType: string): string {
  return ENGINE_ICONS[engineType] || ENGINE_ICONS[EngineType.Other]
}
