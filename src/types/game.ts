/**
 * 游戏相关类型定义
 */

/**
 * 游戏数据传输对象
 */
export interface GameDto {
  /** 游戏ID */
  id: string
  /** 游戏标题 */
  title: string
  /** 引擎类型 */
  engineType: string
  /** 游戏路径 */
  path: string
  /** 路径是否有效 */
  pathValid: boolean
  /** 运行时版本 */
  runtimeVersion?: string
  /** 封面路径 */
  coverPath?: string
  /** 创建时间（Unix毫秒时间戳） */
  createdAt: number
  /** 最后游玩时间（Unix毫秒时间戳） */
  lastPlayedAt?: number
}

/**
 * 添加游戏输入
 */
export interface AddGameInput {
  /** 游戏标题（可选） */
  title?: string
  /** 引擎类型 */
  engineType: string
  /** 游戏路径 */
  path: string
  /** 运行时版本 */
  runtimeVersion?: string
}

/**
 * 更新游戏输入
 */
export interface UpdateGameInput {
  /** 游戏标题 */
  title?: string
  /** 引擎类型 */
  engineType?: string
  /** 游戏路径 */
  path?: string
  /** 运行时版本 */
  runtimeVersion?: string
}

/**
 * 游戏启动结果
 */
export interface LaunchResult {
  /** 进程ID */
  pid: number
}

/**
 * 扫描游戏输入
 */
export interface ScanGamesInput {
  /** 扫描根目录 */
  root: string
  /** 最大扫描深度 */
  maxDepth: number
}

/**
 * 扫描结果
 */
export interface ScanGamesResult {
  /** 扫描目录数 */
  scannedDirs: number
  /** 发现游戏数 */
  foundGames: number
  /** 导入数量 */
  imported: number
  /** 已存在数量 */
  skippedExisting: number
}

/**
 * 游戏配置（settings.toml）
 */
export interface GameConfig {
  /** 引擎类型 */
  engineType: string
  /** 入口路径 */
  entryPath: string
  /** 运行时版本 */
  runtimeVersion?: string
  /** 启动参数 */
  args: string[]
  /** 沙盒主目录 */
  sandboxHome: boolean
  /** 封面文件名 */
  coverFile?: string
}
