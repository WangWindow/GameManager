use std::sync::Arc;

use iced::window::Direction;

use gamemanager_core::{
    BootstrapSnapshot, GameConfig, GameManagerCore, GameSummary, OperationProgress, Runner,
    ScanResult, ThemeMode, WindowBackend,
};

use crate::state::UtilityDialog;

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
    FileHovered(std::path::PathBuf),
    FileDropped(std::path::PathBuf),
    FilesHoveredLeft,
    Focused(bool),
}

#[derive(Clone)]
pub enum Message {
    ThemeModeChanged(ThemeMode),
    WindowBackendChanged(WindowBackend),
    SystemThemeChanged(SystemTheme),
    Window(WindowMessage),
    Library(LibraryMessage),
    LaunchFinished {
        game_id: String,
        result: Result<GameSummary, String>,
    },
    ToastDismissed,
    BootstrapFinished(Result<(Arc<GameManagerCore>, BootstrapSnapshot), String>),
    OpenImport,
    OpenAppMenu,
    DismissAppMenu,
    OpenUtilityDialog(UtilityDialog),
    CloseUtilityDialog,
    DismissOverlay,
    CloseImport,
    PickImportEntry,
    ImportEntryPicked(Option<std::path::PathBuf>),
    SubmitImport,
    ImportFinished(Result<GameSummary, String>),
    OpenScan,
    CloseScan,
    PickScanRoot,
    ScanRootPicked(Option<std::path::PathBuf>),
    ScanDepthChanged(String),
    ScanDepthAdjusted(i8),
    SubmitScan,
    ScanProgress(OperationProgress),
    ScanFinished(Result<ScanResult, String>),
    RuntimeProgress(OperationProgress),
    OperationDismissed(gamemanager_core::OperationId),
    OpenGameSettings(String),
    CloseGameSettings,
    CloseDeleteGame,
    ConfirmDeleteGame,
    DeleteGameFinished(Result<String, String>),
    GameSettingsLoaded(Result<(GameSummary, GameConfig), String>),
    GameSettingsTitleChanged(String),
    GameSettingsEngineChanged(String),
    GameSettingsEntryChanged(String),
    GameSettingsRuntimeVersionSelected(Option<String>),
    GameSettingsRunnerChanged(Runner),
    GameSettingsArgumentsChanged(String),
    GameSettingsSandboxChanged(bool),
    GameSettingsBottleSelected(Option<String>),
    GameSettingsCoverChanged(String),
    PickGameSettingsEntryFile,
    PickGameSettingsEntryDirectory,
    PickGameSettingsCover,
    GameSettingsEntryPicked(Option<std::path::PathBuf>),
    GameSettingsCoverPicked(Option<std::path::PathBuf>),
    OpenGameSettingsDirectory,
    OpenGameProfileDirectory,
    RefreshGameCover,
    GameCoverRefreshed(Result<GameSummary, String>),
    SaveGameSettings,
    GameSettingsFinished(Result<GameSummary, String>),
    EngineEnabledChanged {
        id: String,
        enabled: bool,
    },
    ToggleEngineExpanded(String),
    RefreshRuntimes,
    RefreshBottles,
    BottlesRefreshed(Result<Vec<String>, String>),
    BottlesEnabledChanged(bool),
    BottlesIntegrationSaved(Result<(), String>),
    BottlesDefaultSelected(Option<String>),
    BottlesDefaultSaved(Result<(), String>),
    AppearanceContainerRootChanged(String),
    PickContainerRoot,
    ContainerRootPicked(Option<std::path::PathBuf>),
    SaveContainerRoot,
    ContainerRootReplaced(Result<(Arc<GameManagerCore>, BootstrapSnapshot), String>),
    CleanupUnusedProfiles,
    UnusedProfilesCleaned(Result<usize, String>),
    RequestRemoveAllGames,
    CancelRemoveAllGames,
    ConfirmRemoveAllGames,
    AllGamesRemoved(Result<usize, String>),
    StatusBarChanged(bool),
    PreferencesPersistDue(u64),
    PreferencesSaved {
        revision: u64,
        result: Result<(), String>,
    },
    EngineEnabledSaved(Result<(), String>),
    DownloadNwjs,
    NwjsDownloadFinished(Result<gamemanager_core::NwjsInstallResult, String>),
    PickMkxpzArchive,
    MkxpzArchivePicked(Option<std::path::PathBuf>),
    MkxpzImportFinished(Result<gamemanager_core::MkxpzInstallResult, String>),
    OpenMkxpzBuilds,
}
