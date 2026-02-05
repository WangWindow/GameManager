use serde::{Deserialize, Serialize};

/// 游戏数据传输对象（用于前端展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDto {
    /// 游戏ID
    pub id: String,
    /// 游戏标题
    pub title: String,
    /// 引擎类型
    pub engine_type: String,
    /// 游戏路径
    pub path: String,
    /// 路径是否有效
    pub path_valid: bool,
    /// 运行时版本
    pub runtime_version: Option<String>,
    /// 封面路径
    pub cover_path: Option<String>,
    /// 创建时间
    pub created_at: i64,
    /// 最后游玩时间
    pub last_played_at: Option<i64>,
}

/// 添加游戏输入
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddGameInput {
    /// 游戏标题（可选，自动检测）
    pub title: Option<String>,
    /// 引擎类型
    pub engine_type: String,
    /// 游戏路径
    pub path: String,
    /// 运行时版本
    pub runtime_version: Option<String>,
}

/// 导入游戏目录输入
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportGameInput {
    /// 可执行文件路径
    pub executable_path: String,
    /// 引擎类型
    pub engine_type: String,
}

/// 更新游戏输入
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGameInput {
    /// 游戏标题
    pub title: Option<String>,
    /// 引擎类型
    pub engine_type: Option<String>,
    /// 游戏路径
    pub path: Option<String>,
    /// 运行时版本
    pub runtime_version: Option<String>,
}

/// 引擎数据传输对象
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineDto {
    /// 引擎ID
    pub id: String,
    /// 引擎名称
    pub name: String,
    /// 引擎版本
    pub version: String,
    /// 引擎类型
    pub engine_type: String,
    /// 引擎路径
    pub path: String,
    /// 安装时间
    pub installed_at: i64,
}

/// 引擎更新检测信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineUpdateInfo {
    pub engine_id: String,
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
}

/// 引擎更新结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineUpdateResult {
    pub engine_id: String,
    pub updated: bool,
    pub from_version: String,
    pub to_version: String,
    pub install_dir: Option<String>,
}

/// 游戏启动结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchResult {
    /// 进程ID
    pub pid: u32,
}

/// 设置容器根目录输入
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetContainerRootInput {
    /// 容器根目录路径
    pub container_root: String,
}

/// 扫描游戏输入
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanGamesInput {
    /// 扫描根目录
    pub root: String,
    /// 最大扫描深度
    pub max_depth: u32,
}

/// 扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanGamesResult {
    /// 扫描目录数
    pub scanned_dirs: u32,
    /// 发现游戏数
    pub found_games: u32,
    /// 导入数量
    pub imported: u32,
    /// 已存在数量
    pub skipped_existing: u32,
}

/// 清理容器结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
    /// 删除数量
    pub deleted: u32,
}

/// 设置默认 Bottles bottle 输入
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetDefaultBottleInput {
    pub default_bottle: Option<String>,
}

/// 启用/禁用 Bottles
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBottlesEnabledInput {
    pub enabled: bool,
}
