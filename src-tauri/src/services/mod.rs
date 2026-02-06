// 服务层模块
// 实现应用的业务逻辑

pub mod db;
pub mod download;
pub mod engine_manager;
pub mod game_launcher;
pub mod game_manager;

// 扩展目录（插件化）
pub mod extension;

pub mod path;
pub mod utils;

// 重新导出服务
pub use engine_manager::EngineService;
pub use game_launcher::LauncherService;
pub use game_manager::GameService;
// Bottles 由 extension 模块导出（在非 Linux 上 extension 提供 stub）
pub use extension::BottlesService;
pub use path::FileService;
pub use utils::{ArchiveService, now_unix_ms};
