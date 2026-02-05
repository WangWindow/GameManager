use serde::{Deserialize, Serialize};

/// 应用设置常量
pub const SETTING_CONTAINER_ROOT: &str = "container_root";

/// 应用全局设置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// 容器根目录
    pub container_root: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            container_root: String::new(),
        }
    }
}

// GameSettings 已迁移至 game::GameConfig（settings.toml）。
