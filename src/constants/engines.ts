/**
 * 引擎类型常量
 */

import { EngineType, ENGINE_DISPLAY_NAMES } from '@/types/engine'

export const ENGINE_OPTION_RPGMAKER_NWJS = 'rpgmaker_nwjs'

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

export const ENGINE_PICKER_OPTIONS = [
  { value: EngineType.RpgMakerVX, label: getEngineDisplayName(EngineType.RpgMakerVX) },
  { value: EngineType.RpgMakerVXAce, label: getEngineDisplayName(EngineType.RpgMakerVXAce) },
  { value: ENGINE_OPTION_RPGMAKER_NWJS, label: 'RPG Maker MV/MZ (NW.js)' },
  { value: EngineType.RenPy, label: getEngineDisplayName(EngineType.RenPy) },
  { value: EngineType.Other, label: getEngineDisplayName(EngineType.Other) },
]

export const ENGINE_FILTER_OPTIONS = [
  { value: 'all', label: '全部引擎' },
  ...ENGINE_PICKER_OPTIONS,
]

export function normalizeEngineTypeForSelect(engineType: string): string {
  if (engineType === EngineType.RpgMakerMV || engineType === EngineType.RpgMakerMZ) {
    return ENGINE_OPTION_RPGMAKER_NWJS
  }
  return engineType
}

export function resolveSelectedEngineType(selected: string, previous?: string): string {
  if (selected !== ENGINE_OPTION_RPGMAKER_NWJS) {
    return selected
  }
  if (previous === EngineType.RpgMakerMZ) {
    return EngineType.RpgMakerMZ
  }
  return EngineType.RpgMakerMV
}

/**
 * 获取引擎显示名称
 */
export function getEngineDisplayName(engineType: string): string {
  if (engineType === EngineType.RpgMakerMV || engineType === EngineType.RpgMakerMZ) {
    return 'RPG Maker MV/MZ (NW.js)'
  }
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
