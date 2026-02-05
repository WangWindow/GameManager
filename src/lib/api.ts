/**
 * Tauri API 封装
 */

import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import type {
  GameDto,
  AddGameInput,
  UpdateGameInput,
  LaunchResult,
  ScanGamesInput,
  ScanGamesResult,
  GameConfig,
  EngineDto,
  AppSettings,
  SetContainerRootInput,
  NwjsStableInfo,
  NwjsInstallResult,
  CleanupResult,
} from '@/types'

/**
 * 调用Tauri命令的通用封装
 */
async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await tauriInvoke<T>(command, args)
  } catch (error) {
    console.error(`Tauri命令调用失败 [${command}]:`, error)
    throw error
  }
}

// ============ 游戏相关API ============

/**
 * 获取所有游戏列表
 */
export async function getGames(): Promise<GameDto[]> {
  return invoke<GameDto[]>('get_games')
}

/**
 * 获取单个游戏
 */
export async function getGame(id: string): Promise<GameDto | null> {
  return invoke<GameDto | null>('get_game', { id })
}

/**
 * 添加游戏
 */
export async function addGame(input: AddGameInput): Promise<GameDto> {
  return invoke<GameDto>('add_game', { input })
}

/**
 * 更新游戏
 */
export async function updateGame(id: string, input: UpdateGameInput): Promise<GameDto> {
  return invoke<GameDto>('update_game', { id, input })
}

/**
 * 删除游戏
 */
export async function deleteGame(id: string): Promise<void> {
  return invoke<void>('delete_game', { id })
}

/**
 * 启动游戏
 */
export async function launchGame(id: string): Promise<LaunchResult> {
  return invoke<LaunchResult>('launch_game', { id })
}

/**
 * 导入游戏目录
 */
export async function importGameDir(path: string, engineType: string): Promise<GameDto> {
  return invoke<GameDto>('import_game_dir', { input: { path, engineType } })
}

/**
 * 扫描游戏
 */
export async function scanGames(input: ScanGamesInput): Promise<ScanGamesResult> {
  return invoke<ScanGamesResult>('scan_games', { input })
}

/**
 * 获取游戏设置
 */
export async function getGameSettings(id: string): Promise<GameConfig> {
  return invoke<GameConfig>('get_game_settings', { id })
}

/**
 * 保存游戏设置
 */
export async function saveGameSettings(id: string, input: GameConfig): Promise<void> {
  return invoke<void>('save_game_settings', { id, input })
}

/**
 * 获取游戏 profile 目录
 */
export async function getGameProfileDir(id: string): Promise<string> {
  return invoke<string>('get_game_profile_dir', { id })
}

/**
 * 打开本地路径（文件或目录）
 */
export async function openPath(path: string): Promise<void> {
  return invoke<void>('open_path', { path })
}

// ============ 引擎相关API ============

/**
 * 获取所有引擎列表
 */
export async function getEngines(): Promise<EngineDto[]> {
  return invoke<EngineDto[]>('get_engines')
}

/**
 * 查找引擎
 */
export async function findEngine(
  engineType: string,
  version?: string
): Promise<EngineDto | null> {
  return invoke<EngineDto | null>('find_engine', { engineType, version })
}

/**
 * 添加引擎
 */
export async function addEngine(
  name: string,
  version: string,
  engineType: string,
  path: string
): Promise<EngineDto> {
  return invoke<EngineDto>('add_engine', { name, version, engineType, path })
}

/**
 * 删除引擎
 */
export async function deleteEngine(id: string): Promise<void> {
  return invoke<void>('delete_engine', { id })
}

// ============ 设置相关API ============

/**
 * 获取应用设置
 */
export async function getAppSettings(): Promise<AppSettings> {
  return invoke<AppSettings>('get_app_settings')
}

/**
 * 设置容器根目录
 */
export async function setContainerRoot(input: SetContainerRootInput): Promise<void> {
  return invoke<void>('set_container_root', { input })
}

/**
 * 获取 NW.js 稳定版信息
 */
export async function getNwjsStableInfo(): Promise<NwjsStableInfo> {
  return invoke<NwjsStableInfo>('get_nwjs_stable_info')
}

/**
 * 下载 NW.js 稳定版
 */
export async function downloadNwjsStable(flavor: 'normal' | 'sdk'): Promise<NwjsInstallResult> {
  return invoke<NwjsInstallResult>('download_nwjs_stable', { flavor })
}

/**
 * 清理无用容器
 */
export async function cleanupUnusedContainers(): Promise<CleanupResult> {
  return invoke<CleanupResult>('cleanup_unused_containers')
}
