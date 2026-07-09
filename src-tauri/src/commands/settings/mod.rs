//! 应用设置命令：容器根目录配置、NW.js 下载与版本管理和无用容器清理。
pub(crate) mod settings;

// Tauri commands — glob re-export carries generated __cmd__ / __tauri_command_name_ items
pub use settings::*;

// Types
pub(crate) use settings::SettingsState;
