use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 游戏引擎类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EngineType {
    /// RPG Maker VX
    RpgMakerVX,
    /// RPG Maker VX Ace
    RpgMakerVXAce,
    /// RPG Maker MV
    RpgMakerMV,
    /// RPG Maker MZ
    RpgMakerMZ,
    /// NWjs游戏
    NWjs,
    /// RenPy视觉小说
    RenPy,
    /// 其他类型
    Other,
}

impl EngineType {
    /// 从字符串解析引擎类型
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "rpgmakervx" | "rpg_maker_vx" => Self::RpgMakerVX,
            "rpgmakervxace" | "rpg_maker_vx_ace" => Self::RpgMakerVXAce,
            "rpgmakermv" | "rpg_maker_mv" => Self::RpgMakerMV,
            "rpgmakermz" | "rpg_maker_mz" => Self::RpgMakerMZ,
            "nwjs" => Self::NWjs,
            "renpy" => Self::RenPy,
            _ => Self::Other,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RpgMakerVX => "rpgmakervx",
            Self::RpgMakerVXAce => "rpgmakervxace",
            Self::RpgMakerMV => "rpgmakermv",
            Self::RpgMakerMZ => "rpgmakermz",
            Self::NWjs => "nwjs",
            Self::RenPy => "renpy",
            Self::Other => "other",
        }
    }
}

/// 游戏数据库记录
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Game {
    /// 游戏唯一标识
    pub id: String,
    /// Profile目录名称（可读）
    pub profile_key: String,
    /// 游戏标题
    pub title: String,
    /// 引擎类型
    pub engine_type: String,
    /// 游戏路径
    pub path: String,
    /// 运行时版本（可选）
    pub runtime_version: Option<String>,
    /// 封面图片路径（可选）
    pub cover_path: Option<String>,
    /// 创建时间（Unix毫秒时间戳）
    pub created_at: i64,
    /// 最后游玩时间（Unix毫秒时间戳，可选）
    pub last_played_at: Option<i64>,
}

impl Game {
    /// 创建新游戏记录
    pub fn new(
        id: String,
        profile_key: String,
        title: String,
        engine_type: EngineType,
        path: String,
        runtime_version: Option<String>,
    ) -> Self {
        let now = crate::services::now_unix_ms();
        Self {
            id,
            profile_key,
            title,
            engine_type: engine_type.as_str().to_string(),
            path,
            runtime_version,
            cover_path: None,
            created_at: now,
            last_played_at: None,
        }
    }

    /// 获取引擎类型枚举
    pub fn get_engine_type(&self) -> EngineType {
        EngineType::from_str(&self.engine_type)
    }
}

/// 游戏配置文件（TOML格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameConfig {
    /// 引擎类型
    pub engine_type: String,
    /// 入口文件路径
    pub entry_path: String,
    /// 运行时版本
    pub runtime_version: Option<String>,
    /// 启动参数
    #[serde(default)]
    pub args: Vec<String>,
    /// 是否启用沙盒主目录
    #[serde(default = "default_true")]
    pub sandbox_home: bool,
    /// 封面图片文件名
    #[serde(default)]
    pub cover_file: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            engine_type: String::new(),
            entry_path: String::new(),
            runtime_version: None,
            args: Vec::new(),
            sandbox_home: true,
            cover_file: None,
        }
    }
}
