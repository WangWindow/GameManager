#![forbid(unsafe_code)]

mod core;
mod db;
mod engines;
mod error;
mod game_library;
mod integrations;
mod launcher;
mod models;
mod operation;
mod paths;
mod profiles;
mod runtime;

/// Stable application identifier used to locate v0.9 user data.
pub const APP_ID: &str = "io.choco.gamemanager";

pub use core::{BootstrapSnapshot, GameManagerCore, IntegrationStatus, RuntimeStatus};
pub use db::Database;
pub use engines::{
    DetectionConfig, DetectionContext, DetectionMatch, DetectionRuleDefinition, EngineDetail,
    EngineMeta, EngineProfile, EngineRegistry, EngineRuleRequirement, EngineRuleSummary,
    EngineSummary, FsDetectionContext, LaunchConfig, RegistryReport, RegistryWarning,
};
pub use error::{CoreError, Result};
pub use game_library::{
    EntryPoint, GameLibraryService, ImportRequest, ScanCandidate, ScanPlan, ScanPlanner,
    ScanRequest, ScanResult, UpdateGameRequest,
};
pub use integrations::{
    BottlesCli, BottlesCliLocator, BottlesCommandOutput, BottlesCommandRunner,
    SystemBottlesCliLocator,
};
pub use launcher::{LaunchPlan, LaunchResult, Launcher};
pub use models::{
    AppSettings, EngineRecord, GameConfig, GameRecord, GameSummary, GameViewMode, Runner,
    SETTING_BOTTLES_DEFAULT, SETTING_BOTTLES_ENABLED, SETTING_CONTAINER_ROOT,
    SETTING_ENGINE_ENABLED, SETTING_UI_PREFERENCES, ThemeMode, UiPreferences, WindowBackend,
};
pub use operation::{
    Operation, OperationId, OperationOutcome, OperationProgress, OperationReporter, OperationStage,
};
pub use paths::AppPaths;
pub use profiles::{CoverResolver, IconAsset, IconSource, PeIconSource, ProfileStore};
pub use runtime::{
    DownloadProgressCallback, HttpClient, MkxpzInstallResult, NwjsFlavor, NwjsInstallResult,
    NwjsStableInfo, RuntimeManager, build_nwjs_download_url, current_nwjs_target,
    ensure_compatibility_patch,
};
