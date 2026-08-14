#![forbid(unsafe_code)]

mod error;
mod models;
mod paths;

/// Stable application identifier used to locate v0.9 user data.
pub const APP_ID: &str = "io.choco.gamemanager";

pub use error::{CoreError, Result};
pub use models::{
    AppSettings, EngineRecord, GameConfig, GameRecord, GameSummary, GameViewMode, Runner,
    SETTING_BOTTLES_DEFAULT, SETTING_BOTTLES_ENABLED, SETTING_CONTAINER_ROOT,
    SETTING_UI_PREFERENCES, ThemeMode, UiPreferences,
};
pub use paths::AppPaths;
