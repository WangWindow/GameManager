use serde::{Deserialize, Serialize};

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
    /// RenPy视觉小说
    RenPy,
    /// Unity 游戏引擎
    Unity,
    /// Godot 游戏引擎
    Godot,
    /// HTML 游戏
    Html,
    /// 其他类型
    Other,
}

impl EngineType {
    /// 从字符串解析引擎类型
    //
    // NOTE: `strum` is intentionally not used here.
    // `EnumString` derive cannot express multi-alias support (e.g., both
    // "rpgmakervx" and "rpg_maker_vx" map to RpgMakerVX) without losing
    // one alias. The manual match gives full control over alternate names.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "rpgmakervx" | "rpg_maker_vx" => Self::RpgMakerVX,
            "rpgmakervxace" | "rpg_maker_vx_ace" => Self::RpgMakerVXAce,
            "rpgmakermv" | "rpg_maker_mv" | "rpgmakermz" | "rpg_maker_mz" => Self::RpgMakerMV,
            "renpy" => Self::RenPy,
            "unity" => Self::Unity,
            "godot" => Self::Godot,
            "html" => Self::Html,
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
            Self::RenPy => "renpy",
            Self::Unity => "unity",
            Self::Godot => "godot",
            Self::Html => "html",
            Self::Other => "other",
        }
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
    /// 启动运行器：auto | native | nwjs | mkxpz | bottles。
    #[serde(default = "default_runner")]
    pub runner: String,
    /// 启动参数
    #[serde(default)]
    pub args: Vec<String>,
    /// 是否启用沙盒主目录
    #[serde(default = "default_true")]
    pub sandbox_home: bool,
    /// 是否使用 Bottles
    #[serde(default)]
    pub use_bottles: bool,
    /// Bottles bottle 名称
    #[serde(default)]
    pub bottle_name: Option<String>,
    /// 封面图片文件名
    #[serde(default)]
    pub cover_file: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_runner() -> String {
    "auto".to_string()
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            engine_type: String::new(),
            entry_path: String::new(),
            runtime_version: None,
            runner: default_runner(),
            args: Vec::new(),
            sandbox_home: true,
            use_bottles: false,
            bottle_name: None,
            cover_file: None,
        }
    }
}
