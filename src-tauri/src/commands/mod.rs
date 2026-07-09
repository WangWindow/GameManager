// 命令层模块
// 定义Tauri可调用的命令接口

pub mod cover;
pub mod cover_resolver;
pub mod engine;
pub mod game;
pub mod game_executable;
pub mod game_settings;
pub mod import;
pub mod launch;
pub mod legacy_detection;
pub mod platform;
pub mod scan;
pub mod scan_walk;
pub mod settings;
pub mod state;

// Re-export extension commands from new service location
pub mod extension {
    pub use crate::service::extension::integrations::*;
}

// 重新导出命令
pub use cover::*;
pub use engine::*;
pub use extension::*;
pub use game::*;
pub use game_settings::*;
pub use import::*;
pub use launch::*;
pub use platform::*;
pub use scan::*;
pub use settings::*;
