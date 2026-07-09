//! 游戏管理命令：CRUD、导入、启动、封面解析和游戏设置。
pub(crate) mod cover;
pub(crate) mod cover_resolver;
pub(crate) mod game;
pub(crate) mod game_executable;
pub(crate) mod game_settings;
pub(crate) mod import;
pub(crate) mod launch;

// Tauri commands — glob re-exports carry generated __cmd__ / __tauri_command_name_ items
pub use cover::*;
pub use game::*;
pub use game_settings::*;
pub use import::*;
pub use launch::*;
