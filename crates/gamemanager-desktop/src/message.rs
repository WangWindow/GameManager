use std::sync::Arc;

use iced::window::Direction;

use gamemanager_core::{
    BootstrapSnapshot, GameConfig, GameManagerCore, GameSummary, OperationProgress, Runner,
    ScanResult, ThemeMode,
};

pub use crate::state::LibraryMessage;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemTheme {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug)]
pub enum WindowAction {
    Drag,
    Resize(Direction),
    Minimize,
    ToggleMaximize,
    Close,
}

#[derive(Clone, Debug)]
pub enum WindowMessage {
    Action(WindowAction),
    FileHovered(Vec<std::path::PathBuf>),
    FileDropped(std::path::PathBuf),
    FilesHoveredLeft,
    Focused(bool),
}

#[derive(Clone)]
pub enum Message {
    ThemeModeChanged(ThemeMode),
    SystemThemeChanged(SystemTheme),
    Window(WindowMessage),
    Library(LibraryMessage),
    BootstrapFinished(Result<(Arc<GameManagerCore>, BootstrapSnapshot), String>),
    OpenImport,
    CloseImport,
    PickImportEntry,
    ImportEntryPicked(Option<std::path::PathBuf>),
    SubmitImport,
    ImportFinished(Result<GameSummary, String>),
    OpenScan,
    CloseScan,
    PickScanRoot,
    ScanRootPicked(Option<std::path::PathBuf>),
    SubmitScan,
    ScanProgress(OperationProgress),
    ScanFinished(Result<ScanResult, String>),
    OpenGameSettings(String),
    CloseGameSettings,
    GameSettingsLoaded(Result<(GameSummary, GameConfig), String>),
    GameSettingsTitleChanged(String),
    GameSettingsEntryChanged(String),
    GameSettingsRunnerChanged(Runner),
    GameSettingsSandboxChanged(bool),
    GameSettingsBottleChanged(String),
    SaveGameSettings,
    GameSettingsFinished(Result<GameSummary, String>),
    OpenSettings,
    CloseSettings,
    EngineEnabledChanged { id: String, enabled: bool },
    StatusBarChanged(bool),
    PreferencesSaved(Result<(), String>),
    EngineEnabledSaved(Result<(), String>),
    PickMkxpzArchive,
    MkxpzArchivePicked(Option<std::path::PathBuf>),
    MkxpzImportFinished(Result<gamemanager_core::MkxpzInstallResult, String>),
    OpenMkxpzBuilds,
}
