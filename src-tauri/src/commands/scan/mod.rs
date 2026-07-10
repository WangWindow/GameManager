//! 游戏扫描命令：遍历本地目录，通过引擎检测自动发现并导入游戏。
pub(crate) mod legacy_detection;
pub(crate) mod scan;

// Tauri commands — glob re-export carries generated __cmd__ / __tauri_command_name_ items
pub use scan::*;
