#![forbid(unsafe_code)]

mod db;
mod engines;
mod error;
mod game_library;
mod models;
mod paths;
mod profiles;

/// Stable application identifier used to locate v0.9 user data.
pub const APP_ID: &str = "io.choco.gamemanager";

pub use db::Database;
pub use engines::{
    DetectionConfig, DetectionContext, DetectionMatch, DetectionRuleDefinition, EngineDetail,
    EngineMeta, EngineProfile, EngineRegistry, EngineSummary, FsDetectionContext, LaunchConfig,
    RegistryReport, RegistryWarning,
};
pub use error::{CoreError, Result};
pub use game_library::{
    EntryPoint, GameLibraryService, ImportRequest, ScanPlan, ScanPlanner, ScanRequest, ScanResult,
    UpdateGameRequest,
};
pub use models::{
    AppSettings, EngineRecord, GameConfig, GameRecord, GameSummary, GameViewMode, Runner,
    SETTING_BOTTLES_DEFAULT, SETTING_BOTTLES_ENABLED, SETTING_CONTAINER_ROOT,
    SETTING_UI_PREFERENCES, ThemeMode, UiPreferences,
};
pub use paths::AppPaths;
pub use profiles::{CoverResolver, IconAsset, IconSource, PeIconSource, ProfileStore};
