//! 平台检测命令：操作系统类型识别与系统主题获取。
pub(crate) mod platform;

// Tauri commands — glob re-export carries generated __cmd__ / __tauri_command_name_ items
pub use platform::*;
