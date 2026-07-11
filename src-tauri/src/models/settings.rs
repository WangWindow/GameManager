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

/// 集成选项（可扩展）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationOptions {
    pub installed: Option<bool>,
    pub bottles: Option<Vec<String>>,
    pub default_bottle: Option<String>,
}

/// 集成状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationStatus {
    pub key: String,
    pub available: bool,
    pub enabled: bool,
    pub options: Option<IntegrationOptions>,
}

/// 能力列表
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub integrations: Vec<IntegrationStatus>,
}

/// 集成设置输入
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationSettingsInput {
    pub key: String,
    pub enabled: Option<bool>,
    pub options: Option<IntegrationOptions>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            container_root: String::new(),
        }
    }
}

// GameSettings 已迁移至 game::GameConfig（settings.toml）。
