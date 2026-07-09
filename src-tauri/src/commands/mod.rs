// 命令层模块
// 定义Tauri可调用的命令接口

pub mod engine;
pub mod game;
pub mod platform;
pub mod scan;
pub mod settings;
pub mod state;

// Re-export extension commands from new service location
pub mod extension {
    pub use crate::services::extension::integrations::*;
}

// 重新导出命令
pub use engine::*;
pub use extension::*;
pub use game::*;
pub use platform::*;
pub use scan::*;
pub use settings::*;
