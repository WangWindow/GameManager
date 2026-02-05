use serde::{Deserialize, Serialize};

/// 应用设置常量
pub const SETTING_CONTAINER_ROOT: &str = "container_root";
pub const SETTING_BOTTLES_DEFAULT: &str = "bottles_default";
pub const SETTING_BOTTLES_ENABLED: &str = "bottles_enabled";

/// 应用全局设置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// 容器根目录
    pub container_root: String,
}

/// Bottles 状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BottlesStatus {
    pub installed: bool,
    pub enabled: bool,
    pub bottles: Vec<String>,
    pub default_bottle: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            container_root: String::new(),
        }
    }
}

// GameSettings 已迁移至 game::GameConfig（settings.toml）。
