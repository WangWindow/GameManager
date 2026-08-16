use gamemanager_core::{
    AppPaths, BootstrapSnapshot, GameManagerCore, ImportRequest, OperationProgress, ScanRequest,
};
use iced::{
    Element, Length, Subscription, Task, Theme, event,
    widget::{column, container, stack, text},
    window,
};
use std::{sync::Arc, time::Duration};

use crate::{
    components::Modal,
    message::{Message, WindowAction, WindowMessage},
    platform::{DesktopDialog, DesktopOpener},
    shell::{Toast, resize_handles, route_event},
    state::{
        DialogState, EngineListState, GameSettingsState, LibraryState, MaintenanceState,
        OperationState, PreferencesState, ShellState, UtilityDialog,
    },
    ui::{UiTokens, dialog_surface, icons},
    views::{
        appearance_view, delete_game_view, engines_view, game_settings_view, import_view,
        library_view, runtime_view, scan_view, status_bar_view, title_bar_view,
    },
};

pub struct DesktopApp {
    pub shell: ShellState,
    pub dialogs: DialogState,
    pub library: LibraryState,
    pub operations: OperationState,
    pub engines: EngineListState,
    pub maintenance: MaintenanceState,
    pub preferences: PreferencesState,
    pub ui_theme: iced_shadcn_v2::Theme,
    pub core: Option<Arc<GameManagerCore>>,
    pub bootstrap_error: Option<String>,
    pub toast: Option<Toast>,
}

impl DesktopApp {
    pub fn boot() -> Self {
        let shell = ShellState::default();
        Self {
            ui_theme: shell.shadcn_theme(),
            shell,
            dialogs: DialogState::default(),
            library: LibraryState::default(),
            operations: OperationState::default(),
            engines: EngineListState::default(),
            maintenance: MaintenanceState::default(),
            preferences: PreferencesState::default(),
            core: None,
            bootstrap_error: None,
            toast: None,
        }
    }

    fn boot_with_task() -> (Self, Task<Message>) {
        (Self::boot(), bootstrap_task())
    }

    pub fn run() -> iced::Result {
        let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new(
                "gamemanager_core=info,gamemanager_desktop=info,warn",
            )
        });
        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .compact()
            .try_init();
        let display_backends = crate::platform::DisplayBackendAvailability::detect();
        tracing::info!(
            wayland = display_backends.wayland,
            x11 = display_backends.x11,
            "starting desktop application"
        );
        let mut application = iced::application(Self::boot_with_task, Self::update, Self::view)
            .title("GameManager")
            .theme(Self::theme)
            .subscription(Self::subscription)
            .font(lucide_icons::LUCIDE_FONT_BYTES)
            .default_font(iced_shadcn_v2::iced_font(iced_shadcn_v2::FontId::Geist))
            .window(window::Settings {
                decorations: false,
                resizable: true,
                size: load_initial_window_size(),
                ..window::Settings::default()
            });
        for font in iced_shadcn_v2::fonts::ALL_FACES {
            application = application.font(*font);
        }
        application.run()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ThemeModeChanged(mode) => {
                self.shell.set_theme_mode(mode);
                self.ui_theme = self.shell.shadcn_theme();
                self.preferences.set_theme_mode(mode);
                if self.core.is_some() {
                    return self.save_preferences_task();
                }
            }
            Message::WindowBackendChanged(backend) => {
                if !crate::platform::DisplayBackendAvailability::detect().supports(backend) {
                    return Task::none();
                }
                self.preferences.set_window_backend(backend);
                let save = if self.core.is_some() {
                    self.save_preferences_task()
                } else {
                    Task::none()
                };
                let toast = self.show_toast("窗口后端将在重启后生效");
                return Task::batch([save, toast]);
            }
            Message::SystemThemeChanged(theme) => {
                self.shell.apply_system_theme(theme);
                self.ui_theme = self.shell.shadcn_theme();
            }
            Message::Window(WindowMessage::Action(WindowAction::ToggleMaximize)) => {
                self.shell.toggle_window_maximized();
                return window_task(WindowAction::ToggleMaximize);
            }
            Message::Window(WindowMessage::Action(action)) => return window_task(action),
            Message::Window(WindowMessage::FileDropped(path)) => {
                self.dialogs.import.accept_dropped_entry(path)
            }
            Message::Window(WindowMessage::FileHovered(_)) => self.dialogs.import.begin_drop(),
            Message::Window(WindowMessage::FilesHoveredLeft) => self.dialogs.import.end_drop(),
            Message::Window(WindowMessage::Focused(_)) => {}
            Message::Window(WindowMessage::Resized(size)) => {
                if self.preferences.value().remember_window_size {
                    let size = [size.width as u32, size.height as u32];
                    return Task::perform(
                        async move {
                            tokio::time::sleep(Duration::from_millis(300)).await;
                            size
                        },
                        Message::WindowSizeSettled,
                    );
                }
                return Task::none();
            }
            Message::WindowSizeSettled(size) => {
                return window::latest().then(move |id| {
                    let Some(id) = id else {
                        return Task::none();
                    };
                    window::is_maximized(id).then(move |maximized| {
                        if maximized {
                            Task::none()
                        } else {
                            Task::done(Message::WindowSizeSave(size))
                        }
                    })
                });
            }
            Message::WindowSizeSave(size) => {
                self.preferences.set_window_size(size);
                return self.save_preferences_task();
            }
            Message::Library(message) => match message {
                crate::state::LibraryMessage::DeleteRequested(game_id) => {
                    if let Some(game) = self.library.games().iter().find(|game| game.id == game_id)
                    {
                        self.dialogs.delete = Some(crate::state::DeleteDialogState::new(
                            game.id.clone(),
                            game.title.clone(),
                        ));
                    }
                }
                crate::state::LibraryMessage::SearchChanged(query) => {
                    self.library
                        .apply(crate::state::LibraryMessage::SearchChanged(query.clone()));
                    self.preferences.set_search_query(query);
                    let revision = self
                        .preferences
                        .dirty_snapshot()
                        .map(|(revision, _)| revision);
                    return revision.map_or_else(Task::none, |revision| {
                        Task::perform(
                            async move {
                                tokio::time::sleep(Duration::from_millis(300)).await;
                                revision
                            },
                            Message::PreferencesPersistDue,
                        )
                    });
                }
                crate::state::LibraryMessage::ViewModeChanged(mode) => {
                    self.library
                        .apply(crate::state::LibraryMessage::ViewModeChanged(mode));
                    self.preferences.set_view_mode(mode);
                    return self.save_preferences_task();
                }
                crate::state::LibraryMessage::LaunchRequested(game_id) => {
                    if !self.library.start_launch(&game_id) {
                        return Task::none();
                    }
                    let Some(core) = self.core.clone() else {
                        self.library.finish_launch(&game_id);
                        return self.show_toast("应用尚未完成初始化");
                    };
                    let launch_id = game_id.clone();
                    return Task::perform(
                        async move {
                            core.launch_game(&launch_id)
                                .await
                                .map_err(|error| error.to_string())
                        },
                        move |result| Message::LaunchFinished { game_id, result },
                    );
                }
                message => self.library.apply(message),
            },
            Message::LaunchFinished { game_id, result } => {
                self.library.finish_launch(&game_id);
                match result {
                    Ok(game) => {
                        let title = game.title.clone();
                        self.library.apply_game(game);
                        return self.show_toast(format!("已启动 {title}"));
                    }
                    Err(error) => return self.show_toast(error),
                }
            }
            Message::ToastDismissed => self.toast = None,
            Message::BootstrapFinished(result) => match result {
                Ok((core, snapshot)) => self.apply_bootstrap_snapshot(core, snapshot),
                Err(error) => {
                    self.maintenance
                        .finish_runtime_operation(Err(error.clone()));
                    self.bootstrap_error = Some(error);
                }
            },
            Message::OpenImport => {
                self.dialogs.import.open = true;
                self.dialogs.import.error = None;
            }
            Message::OpenAppMenu => self.dialogs.app_menu_open = true,
            Message::DismissAppMenu => self.dialogs.app_menu_open = false,
            Message::OpenUtilityDialog(utility) => self.dialogs.open_utility(utility),
            Message::CloseUtilityDialog => self.dialogs.close_utility(),
            Message::DismissOverlay => {
                self.dialogs.dismiss_non_busy();
            }
            Message::CloseImport => self.dialogs.import.open = false,
            Message::PickImportEntry => {
                return Task::perform(DesktopDialog.pick_file(), Message::ImportEntryPicked);
            }
            Message::ImportEntryPicked(path) => {
                if let Some(path) = path {
                    self.dialogs.import.set_entry_path(path);
                }
            }
            Message::SubmitImport => {
                let Some(core) = self.core.clone() else {
                    self.dialogs.import.error = Some("应用尚未完成初始化".to_owned());
                    return Task::none();
                };
                let Some(entry) = self.dialogs.import.entry_path.clone() else {
                    return Task::none();
                };
                self.dialogs.import.submitting = true;
                return Task::perform(
                    async move {
                        core.import_game(ImportRequest::from_entry(entry))
                            .await
                            .map_err(|error| error.to_string())
                    },
                    Message::ImportFinished,
                );
            }
            Message::ImportFinished(result) => {
                self.dialogs.import.submitting = false;
                match result {
                    Ok(game) => {
                        self.library.apply_game(game);
                        self.dialogs.import.open = false;
                    }
                    Err(error) => self.dialogs.import.error = Some(error),
                }
            }
            Message::OpenScan => {
                self.dialogs.scan = Some(crate::state::ScanDialogState::open(
                    std::path::PathBuf::new(),
                    3,
                ));
            }
            Message::CloseScan => self.dialogs.scan = None,
            Message::PickScanRoot => {
                return Task::perform(DesktopDialog.pick_directory(), Message::ScanRootPicked);
            }
            Message::ScanRootPicked(path) => {
                if let Some(path) = path
                    && let Some(scan) = self.dialogs.scan.as_mut()
                {
                    scan.root = path;
                    scan.error = None;
                }
            }
            Message::ScanDepthChanged(value) => {
                if let Some(scan) = self.dialogs.scan.as_mut() {
                    scan.set_max_depth_text(&value);
                }
            }
            Message::ScanDepthAdjusted(delta) => {
                if let Some(scan) = self.dialogs.scan.as_mut() {
                    scan.adjust_max_depth(delta);
                }
            }
            Message::SubmitScan => {
                let Some(core) = self.core.clone() else {
                    if let Some(scan) = self.dialogs.scan.as_mut() {
                        scan.error = Some("应用尚未完成初始化".to_owned());
                    }
                    return Task::none();
                };
                let Some(scan) = self.dialogs.scan.as_mut() else {
                    return Task::none();
                };
                if !scan.can_submit() {
                    scan.error = Some("请选择有效的扫描目录".to_owned());
                    return Task::none();
                }
                let operation = core.scan(ScanRequest::new(scan.root.clone(), scan.max_depth));
                let operation_id = operation.id();
                scan.set_operation(operation_id);
                let progress = operation.progress();
                let future = operation.into_future();
                return Task::batch([
                    Task::run(progress, Message::ScanProgress),
                    Task::perform(future, |result| {
                        Message::ScanFinished(result.map_err(|error| error.to_string()))
                    }),
                ]);
            }
            Message::ScanProgress(progress) => {
                self.apply_scan_progress(progress);
            }
            Message::ScanFinished(result) => match result {
                Ok(_) => {
                    let operation_id = self
                        .dialogs
                        .scan
                        .as_ref()
                        .and_then(|scan| scan.operation_id);
                    self.dialogs.scan = None;
                    let dismiss = operation_id.map_or_else(Task::none, operation_dismiss_task);
                    if let Some(core) = self.core.clone() {
                        let bootstrap = Task::perform(
                            async move {
                                let snapshot =
                                    core.bootstrap().await.map_err(|error| error.to_string())?;
                                Ok::<_, String>((core, snapshot))
                            },
                            Message::BootstrapFinished,
                        );
                        return Task::batch([bootstrap, dismiss]);
                    }
                    return dismiss;
                }
                Err(error) => {
                    let operation_id = self
                        .dialogs
                        .scan
                        .as_ref()
                        .and_then(|scan| scan.operation_id);
                    if let Some(scan) = self.dialogs.scan.as_mut() {
                        scan.error = Some(error);
                    }
                    return operation_id.map_or_else(Task::none, operation_dismiss_task);
                }
            },
            Message::OpenGameSettings(game_id) => {
                let Some(core) = self.core.clone() else {
                    return Task::none();
                };
                let Some(game) = self
                    .library
                    .games()
                    .iter()
                    .find(|game| game.id == game_id)
                    .cloned()
                else {
                    return Task::none();
                };
                return Task::perform(
                    async move {
                        let config = core
                            .game_config(&game.profile_key)
                            .await
                            .map_err(|error| error.to_string())?;
                        Ok::<_, String>((game, config))
                    },
                    Message::GameSettingsLoaded,
                );
            }
            Message::CloseGameSettings => self.dialogs.settings = None,
            Message::CloseDeleteGame => self.dialogs.delete = None,
            Message::ConfirmDeleteGame => {
                let Some(core) = self.core.clone() else {
                    return Task::none();
                };
                let Some(delete) = self.dialogs.delete.as_mut() else {
                    return Task::none();
                };
                delete.deleting = true;
                let game_id = delete.game_id.clone();
                return Task::perform(
                    async move {
                        core.remove_game(&game_id)
                            .await
                            .map(|()| game_id)
                            .map_err(|error| error.to_string())
                    },
                    Message::DeleteGameFinished,
                );
            }
            Message::DeleteGameFinished(result) => match result {
                Ok(game_id) => {
                    self.library.remove_game(&game_id);
                    self.dialogs.delete = None;
                }
                Err(error) => {
                    if let Some(delete) = self.dialogs.delete.as_mut() {
                        delete.deleting = false;
                        delete.error = Some(error);
                    }
                }
            },
            Message::GameSettingsLoaded(result) => match result {
                Ok((game, config)) => {
                    self.dialogs.settings =
                        Some(GameSettingsState::from_game_and_config(&game, &config));
                }
                Err(error) => {
                    if let Some(settings) = self.dialogs.settings.as_mut() {
                        settings.error = Some(error);
                    }
                }
            },
            Message::GameSettingsTitleChanged(title) => {
                if let Some(settings) = self.dialogs.settings.as_mut() {
                    settings.title = title;
                }
            }
            Message::GameSettingsEngineChanged(engine_type) => {
                if let Some(settings) = self.dialogs.settings.as_mut() {
                    settings.set_engine_type(engine_type);
                }
            }
            Message::GameSettingsEntryChanged(entry) => {
                if let Some(settings) = self.dialogs.settings.as_mut() {
                    settings.entry_path = entry;
                }
            }
            Message::GameSettingsRuntimeVersionSelected(version) => {
                if let Some(settings) = self.dialogs.settings.as_mut() {
                    settings.runtime_version = version;
                }
            }
            Message::GameSettingsRunnerChanged(runner) => {
                if let Some(settings) = self.dialogs.settings.as_mut() {
                    settings.runner = runner;
                }
            }
            Message::GameSettingsArgumentsChanged(args) => {
                if let Some(settings) = self.dialogs.settings.as_mut() {
                    settings.set_arguments_text(args);
                }
            }
            Message::GameSettingsSandboxChanged(enabled) => {
                if let Some(settings) = self.dialogs.settings.as_mut() {
                    settings.sandbox_home = enabled;
                }
            }
            Message::GameSettingsBottleSelected(bottle) => {
                if let Some(settings) = self.dialogs.settings.as_mut() {
                    settings.select_bottle(bottle);
                }
            }
            Message::GameSettingsCoverChanged(cover_file) => {
                if let Some(settings) = self.dialogs.settings.as_mut() {
                    settings.set_cover_file(cover_file);
                }
            }
            Message::PickGameSettingsEntryFile => {
                return Task::perform(DesktopDialog.pick_file(), Message::GameSettingsEntryPicked);
            }
            Message::PickGameSettingsEntryDirectory => {
                return Task::perform(
                    DesktopDialog.pick_directory(),
                    Message::GameSettingsEntryPicked,
                );
            }
            Message::PickGameSettingsCover => {
                return Task::perform(
                    DesktopDialog.pick_cover_file(),
                    Message::GameSettingsCoverPicked,
                );
            }
            Message::GameSettingsEntryPicked(path) => {
                if let Some(path) = path
                    && let Some(settings) = self.dialogs.settings.as_mut()
                {
                    settings.entry_path = path.to_string_lossy().into_owned();
                }
            }
            Message::GameSettingsCoverPicked(path) => {
                if let Some(path) = path
                    && let Some(settings) = self.dialogs.settings.as_mut()
                {
                    settings.set_cover_file(path.to_string_lossy().into_owned());
                }
            }
            Message::OpenGameSettingsDirectory => {
                let Some(settings) = self.dialogs.settings.as_mut() else {
                    return Task::none();
                };
                if let Err(error) =
                    DesktopOpener.open_path(std::path::Path::new(&settings.game_path))
                {
                    settings.error = Some(error.to_string());
                }
            }
            Message::OpenGameProfileDirectory => {
                let Some(core) = self.core.clone() else {
                    return Task::none();
                };
                let Some(settings) = self.dialogs.settings.as_mut() else {
                    return Task::none();
                };
                let profile_dir = core.profiles().profile_dir(&settings.profile_key);
                if let Err(error) = DesktopOpener.open_path(&profile_dir) {
                    settings.error = Some(error.to_string());
                }
            }
            Message::RefreshGameCover => {
                let Some(core) = self.core.clone() else {
                    return Task::none();
                };
                let Some(settings) = self.dialogs.settings.as_mut() else {
                    return Task::none();
                };
                settings.refreshing_cover = true;
                let game_id = settings.game_id.clone();
                return Task::perform(
                    async move {
                        core.refresh_cover(&game_id)
                            .await
                            .map_err(|error| error.to_string())
                    },
                    Message::GameCoverRefreshed,
                );
            }
            Message::GameCoverRefreshed(result) => {
                if let Some(settings) = self.dialogs.settings.as_mut() {
                    settings.refreshing_cover = false;
                    match result {
                        Ok(game) => self.library.apply_game(game),
                        Err(error) => settings.error = Some(error),
                    }
                }
            }
            Message::SaveGameSettings => {
                let Some(core) = self.core.clone() else {
                    return Task::none();
                };
                let Some(settings) = self.dialogs.settings.as_mut() else {
                    return Task::none();
                };
                if let Err(error) = settings.validate() {
                    settings.error = Some(error);
                    return Task::none();
                }
                let game_id = settings.game_id.clone();
                let cover_source = settings.changed_cover_source();
                let update = settings.into_update_request();
                settings.saving = true;
                return Task::perform(
                    async move {
                        let game = core
                            .save_game_settings(&game_id, update.game, &update.config)
                            .await
                            .map_err(|error| error.to_string())?;
                        if let Some(source) = cover_source {
                            core.set_custom_cover(&game.id, source)
                                .await
                                .map_err(|error| error.to_string())
                        } else {
                            Ok(game)
                        }
                    },
                    Message::GameSettingsFinished,
                );
            }
            Message::GameSettingsFinished(result) => {
                if let Some(settings) = self.dialogs.settings.as_mut() {
                    settings.saving = false;
                    match result {
                        Ok(game) => {
                            self.library.apply_game(game);
                            self.dialogs.settings = None;
                        }
                        Err(error) => settings.error = Some(error),
                    }
                }
            }
            Message::EngineEnabledChanged { id, enabled } => {
                self.engines.apply_enabled(&id, enabled);
                if let Some(core) = self.core.clone() {
                    return Task::perform(
                        async move {
                            core.set_engine_enabled(&id, enabled)
                                .await
                                .map_err(|error| error.to_string())
                        },
                        Message::EngineEnabledSaved,
                    );
                }
            }
            Message::ToggleEngineExpanded(id) => self.engines.toggle_expanded(&id),
            Message::StatusBarChanged(show) => {
                self.preferences.set_show_status_bar(show);
                if self.core.is_some() {
                    return self.save_preferences_task();
                }
            }
            Message::RememberWindowSizeChanged(remember) => {
                self.preferences.set_remember_window_size(remember);
                if self.core.is_some() {
                    return self.save_preferences_task();
                }
            }
            Message::PreferencesPersistDue(revision) => {
                if self.preferences.is_current_revision(revision) {
                    return self.save_preferences_task();
                }
            }
            Message::PreferencesSaved { revision, result } => {
                if let Err(error) = result {
                    self.bootstrap_error = Some(error);
                } else {
                    self.preferences.mark_saved(revision);
                }
            }
            Message::EngineEnabledSaved(result) => {
                if let Err(error) = result {
                    self.bootstrap_error = Some(error);
                }
            }
            Message::RefreshRuntimes => {
                let Some(core) = self.core.clone() else {
                    return Task::none();
                };
                self.maintenance.begin_runtime_operation();
                return Task::perform(
                    async move {
                        let snapshot = core.bootstrap().await.map_err(|error| error.to_string())?;
                        Ok::<_, String>((core, snapshot))
                    },
                    Message::BootstrapFinished,
                );
            }
            Message::BottlesEnabledChanged(enabled) => {
                self.maintenance.set_bottles_enabled(enabled);
                if let Some(core) = self.core.clone() {
                    let save = Task::perform(
                        async move {
                            core.set_bottles_enabled(enabled)
                                .await
                                .map_err(|error| error.to_string())
                        },
                        Message::BottlesIntegrationSaved,
                    );
                    if enabled {
                        return Task::batch([save, self.refresh_bottles_task()]);
                    }
                    return save;
                }
            }
            Message::RefreshBottles => return self.refresh_bottles_task(),
            Message::BottlesRefreshed(result) => self.maintenance.finish_bottle_refresh(result),
            Message::BottlesIntegrationSaved(result) => {
                if let Err(error) = result {
                    self.bootstrap_error = Some(error);
                }
            }
            Message::BottlesDefaultSelected(bottle) => {
                self.maintenance.set_bottles_default(bottle.clone());
                let Some(core) = self.core.clone() else {
                    return Task::none();
                };
                return Task::perform(
                    async move {
                        core.set_default_bottle(bottle.as_deref())
                            .await
                            .map_err(|error| error.to_string())
                    },
                    Message::BottlesDefaultSaved,
                );
            }
            Message::BottlesDefaultSaved(result) => {
                if let Err(error) = result {
                    self.bootstrap_error = Some(error);
                }
            }
            Message::AppearanceContainerRootChanged(path) => {
                self.dialogs.appearance.set_container_root(path);
            }
            Message::PickContainerRoot => {
                return Task::perform(DesktopDialog.pick_directory(), Message::ContainerRootPicked);
            }
            Message::ContainerRootPicked(path) => {
                if let Some(path) = path {
                    self.dialogs
                        .appearance
                        .set_container_root(path.to_string_lossy().into_owned());
                }
            }
            Message::SaveContainerRoot => {
                let root = self.dialogs.appearance.container_root.trim().to_owned();
                if root.is_empty() {
                    self.dialogs.appearance.error = Some("容器根目录不能为空".to_owned());
                    return Task::none();
                }
                let Some(core) = self.core.clone() else {
                    self.dialogs.appearance.error = Some("应用尚未完成初始化".to_owned());
                    return Task::none();
                };
                self.dialogs.appearance.saving_root = true;
                return Task::perform(
                    async move {
                        let core = Arc::new(
                            core.replace_container_root(root)
                                .await
                                .map_err(|error| error.to_string())?,
                        );
                        let snapshot = core.bootstrap().await.map_err(|error| error.to_string())?;
                        Ok::<_, String>((core, snapshot))
                    },
                    Message::ContainerRootReplaced,
                );
            }
            Message::ContainerRootReplaced(result) => {
                self.dialogs.appearance.saving_root = false;
                match result {
                    Ok((core, snapshot)) => {
                        self.apply_bootstrap_snapshot(core, snapshot);
                        return self.show_toast("容器根目录已更新");
                    }
                    Err(error) => self.dialogs.appearance.error = Some(error),
                }
            }
            Message::CleanupUnusedProfiles => {
                let Some(core) = self.core.clone() else {
                    self.dialogs.appearance.error = Some("应用尚未完成初始化".to_owned());
                    return Task::none();
                };
                self.dialogs.appearance.cleaning_profiles = true;
                self.dialogs.appearance.error = None;
                return Task::perform(
                    async move {
                        core.cleanup_unused_profiles()
                            .await
                            .map_err(|error| error.to_string())
                    },
                    Message::UnusedProfilesCleaned,
                );
            }
            Message::UnusedProfilesCleaned(result) => {
                self.dialogs.appearance.cleaning_profiles = false;
                match result {
                    Ok(count) => return self.show_toast(format!("已清理 {count} 个容器")),
                    Err(error) => self.dialogs.appearance.error = Some(error),
                }
            }
            Message::RequestRemoveAllGames => {
                self.dialogs.appearance.confirm_remove_all = true;
                self.dialogs.appearance.error = None;
            }
            Message::CancelRemoveAllGames => self.dialogs.appearance.confirm_remove_all = false,
            Message::ConfirmRemoveAllGames => {
                if !self.dialogs.appearance.confirm_remove_all {
                    return Task::none();
                }
                let Some(core) = self.core.clone() else {
                    self.dialogs.appearance.error = Some("应用尚未完成初始化".to_owned());
                    return Task::none();
                };
                self.dialogs.appearance.removing_games = true;
                return Task::perform(
                    async move {
                        core.remove_all_games()
                            .await
                            .map_err(|error| error.to_string())
                    },
                    Message::AllGamesRemoved,
                );
            }
            Message::AllGamesRemoved(result) => {
                self.dialogs.appearance.removing_games = false;
                self.dialogs.appearance.confirm_remove_all = false;
                match result {
                    Ok(count) => {
                        self.library.replace_games(Vec::new());
                        return self.show_toast(format!("已移除 {count} 个游戏"));
                    }
                    Err(error) => self.dialogs.appearance.error = Some(error),
                }
            }
            Message::DownloadNwjs => {
                let Some(core) = self.core.clone() else {
                    return Task::none();
                };
                let operation = core
                    .runtime_manager()
                    .download_latest_nwjs(gamemanager_core::NwjsFlavor::Normal);
                let operation_id = operation.id();
                self.maintenance
                    .begin_runtime_operation_with_id(Some(operation_id));
                let progress = operation.progress();
                let future = operation.into_future();
                return Task::batch([
                    Task::run(progress, Message::RuntimeProgress),
                    Task::perform(
                        async move {
                            let install = future.await.map_err(|error| error.to_string())?;
                            core.register_nwjs_runtime(&install)
                                .await
                                .map_err(|error| error.to_string())?;
                            Ok::<_, String>(install)
                        },
                        Message::NwjsDownloadFinished,
                    ),
                ]);
            }
            Message::NwjsDownloadFinished(result) => match result {
                Ok(_) => {
                    let operation_id = self.maintenance.runtime_operation_id();
                    self.maintenance.finish_runtime_operation(Ok(()));
                    let dismiss = operation_id.map_or_else(Task::none, operation_dismiss_task);
                    if let Some(core) = self.core.clone() {
                        let bootstrap = Task::perform(
                            async move {
                                let snapshot =
                                    core.bootstrap().await.map_err(|error| error.to_string())?;
                                Ok::<_, String>((core, snapshot))
                            },
                            Message::BootstrapFinished,
                        );
                        return Task::batch([bootstrap, dismiss]);
                    }
                    return dismiss;
                }
                Err(error) => {
                    let operation_id = self.maintenance.runtime_operation_id();
                    self.maintenance.finish_runtime_operation(Err(error));
                    return operation_id.map_or_else(Task::none, operation_dismiss_task);
                }
            },
            Message::PickMkxpzArchive => {
                return Task::perform(DesktopDialog.pick_file(), Message::MkxpzArchivePicked);
            }
            Message::MkxpzArchivePicked(path) => {
                let Some(path) = path else {
                    return Task::none();
                };
                let Some(core) = self.core.clone() else {
                    self.bootstrap_error = Some("应用尚未完成初始化".to_owned());
                    return Task::none();
                };
                let operation =
                    gamemanager_core::Operation::from_future("导入 mkxp-z", async move {
                        let install = core.runtime_manager().import_mkxpz_archive(&path)?;
                        core.register_mkxpz_runtime(&install).await?;
                        Ok::<_, gamemanager_core::CoreError>(install)
                    });
                let operation_id = operation.id();
                self.maintenance
                    .begin_runtime_operation_with_id(Some(operation_id));
                let progress = operation.progress();
                let future = operation.into_future();
                return Task::batch([
                    Task::run(progress, Message::RuntimeProgress),
                    Task::perform(
                        async move { future.await.map_err(|error| error.to_string()) },
                        Message::MkxpzImportFinished,
                    ),
                ]);
            }
            Message::MkxpzImportFinished(result) => match result {
                Ok(_) => {
                    let operation_id = self.maintenance.runtime_operation_id();
                    self.maintenance.finish_runtime_operation(Ok(()));
                    let dismiss = operation_id.map_or_else(Task::none, operation_dismiss_task);
                    if let Some(core) = self.core.clone() {
                        let bootstrap = Task::perform(
                            async move {
                                let snapshot =
                                    core.bootstrap().await.map_err(|error| error.to_string())?;
                                Ok::<_, String>((core, snapshot))
                            },
                            Message::BootstrapFinished,
                        );
                        return Task::batch([bootstrap, dismiss]);
                    }
                    return dismiss;
                }
                Err(error) => {
                    let operation_id = self.maintenance.runtime_operation_id();
                    self.maintenance.finish_runtime_operation(Err(error));
                    return operation_id.map_or_else(Task::none, operation_dismiss_task);
                }
            },
            Message::RuntimeProgress(progress) => self.operations.apply(progress),
            Message::OperationDismissed(operation_id) => {
                self.operations.clear(operation_id);
                self.maintenance.clear_runtime_operation(operation_id);
            }
            Message::OpenMkxpzBuilds => {
                if let Err(error) = DesktopOpener
                    .open_url("https://github.com/mkxp-z/mkxp-z/actions/workflows/autobuild.yml?query=event%3Apush")
                {
                    self.bootstrap_error = Some(error.to_string());
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let theme = &self.ui_theme;
        let header = title_bar_view(self.dialogs.app_menu_open, theme);
        let mut content = column![header];
        if let Some(error) = self.bootstrap_error.as_deref() {
            content = content.push(
                container(text(error).size(13).color(theme.palette.destructive)).padding([8, 12]),
            );
        }
        let library = library_view(&self.library, &self.engines, theme);
        let body: Element<'_, Message> = if self.preferences.value().show_status_bar {
            if let Some(operation) = self.operations.current() {
                column![library, status_bar_view(operation, theme)]
                    .height(Length::Fill)
                    .into()
            } else {
                column![library].height(Length::Fill).into()
            }
        } else {
            column![library].height(Length::Fill).into()
        };
        let base: Element<'_, Message> = container(content.push(body))
            .width(Length::Fill)
            .height(Length::Fill)
            .clip(true)
            .style(move |_| iced::widget::container::Style {
                background: Some(iced::Background::Color(theme.palette.background)),
                text_color: Some(theme.palette.foreground),
                border: iced::Border {
                    color: theme.palette.border,
                    width: UiTokens::WINDOW_BORDER,
                    radius: UiTokens::WINDOW_RADIUS.into(),
                },
                ..Default::default()
            })
            .into();
        let base = resize_handles(base, !self.shell.is_window_maximized());
        let base = if self.dialogs.import.drop_active {
            stack![
                base,
                container(
                    column![
                        icons::file().size(28).color(theme.palette.foreground),
                        text("松开以导入游戏")
                            .size(14)
                            .color(theme.palette.foreground),
                    ]
                    .spacing(8)
                    .align_x(iced::alignment::Horizontal::Center),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(move |_| iced::widget::container::Style {
                    background: Some(iced::Background::Color(iced::Color {
                        a: 0.72,
                        ..theme.palette.background
                    })),
                    ..Default::default()
                }),
            ]
            .into()
        } else {
            base
        };
        let base = if let Some(toast) = self.toast.as_ref() {
            stack![
                base,
                container(
                    container(
                        text(&toast.message)
                            .size(13)
                            .color(theme.palette.foreground)
                    )
                    .padding([8, 12])
                    .style(move |_| dialog_surface(theme)),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(16)
                .align_x(iced::alignment::Horizontal::Right)
                .align_y(iced::alignment::Vertical::Bottom),
            ]
            .into()
        } else {
            base
        };
        let with_import = if self.dialogs.import.open {
            Modal::new(import_view(&self.dialogs.import, theme)).overlay(
                base,
                (!self.dialogs.import.submitting).then_some(Message::CloseImport),
            )
        } else {
            base
        };
        let with_scan = if let Some(scan) = self.dialogs.scan.as_ref() {
            Modal::new(scan_view(scan, theme)).overlay(
                with_import,
                scan.operation_id.is_none().then_some(Message::CloseScan),
            )
        } else {
            with_import
        };
        let with_settings = if let Some(settings) = self.dialogs.settings.as_ref() {
            Modal::new(game_settings_view(
                settings,
                &self.engines,
                &self.maintenance,
                theme,
            ))
            .overlay(
                with_scan,
                (!settings.saving).then_some(Message::CloseGameSettings),
            )
        } else {
            with_scan
        };
        let with_delete = if let Some(delete) = self.dialogs.delete.as_ref() {
            Modal::new(delete_game_view(delete, theme)).overlay(
                with_settings,
                (!delete.deleting).then_some(Message::CloseDeleteGame),
            )
        } else {
            with_settings
        };
        if let Some(utility) = self.dialogs.utility {
            let utility_view = match utility {
                UtilityDialog::Runtime => runtime_view(&self.maintenance, theme),
                UtilityDialog::Engines => engines_view(&self.engines, theme),
                UtilityDialog::Appearance => {
                    appearance_view(&self.preferences, &self.dialogs.appearance, theme)
                }
            };
            Modal::new(utility_view).overlay(with_delete, Some(Message::CloseUtilityDialog))
        } else {
            with_delete
        }
    }

    fn theme(&self) -> Theme {
        self.shell.iced_theme()
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _, _| route_event(&event))
    }

    pub fn for_test() -> Self {
        Self::boot()
    }

    pub fn update_for_test(&mut self, message: Message) {
        let _ = self.update(message);
    }

    fn apply_scan_progress(&mut self, progress: OperationProgress) {
        self.operations.apply(progress.clone());
        if let Some(scan) = self.dialogs.scan.as_mut()
            && scan.operation_id == Some(progress.id)
        {
            scan.apply_progress(progress.stage, progress.percent);
        }
    }

    fn apply_bootstrap_snapshot(
        &mut self,
        core: Arc<GameManagerCore>,
        snapshot: BootstrapSnapshot,
    ) {
        self.dialogs.appearance =
            crate::state::AppearanceDialogState::from_settings(snapshot.app_settings.clone());
        self.core = Some(core);
        self.shell
            .set_theme_mode(snapshot.ui_preferences.theme_mode);
        self.ui_theme = self.shell.shadcn_theme();
        self.preferences = PreferencesState::from_value(snapshot.ui_preferences);
        self.library.search_query = self.preferences.value().search_query.clone();
        self.library.view_mode = self.preferences.value().view_mode;
        self.library.replace_games(snapshot.games);
        self.engines = EngineListState::from_details(snapshot.engine_details);
        let bottles = snapshot
            .integrations
            .iter()
            .find(|integration| integration.id == "bottles");
        self.maintenance = MaintenanceState::with_runtime_snapshot(
            snapshot.runtimes,
            bottles.is_some_and(|integration| integration.enabled),
            bottles.is_some_and(|integration| integration.available),
            bottles.map_or_else(Vec::new, |integration| integration.bottles.clone()),
            bottles.and_then(|integration| integration.default_bottle.clone()),
            bottles.and_then(|integration| integration.bottles_error.clone()),
        );
        self.bootstrap_error = None;
    }

    fn save_preferences_task(&self) -> Task<Message> {
        let Some(core) = self.core.clone() else {
            return Task::none();
        };
        let Some((revision, preferences)) = self.preferences.dirty_snapshot() else {
            return Task::none();
        };
        Task::perform(
            async move {
                core.save_ui_preferences(&preferences)
                    .await
                    .map_err(|error| error.to_string())
            },
            move |result| Message::PreferencesSaved { revision, result },
        )
    }

    fn refresh_bottles_task(&mut self) -> Task<Message> {
        let Some(core) = self.core.clone() else {
            self.maintenance
                .finish_bottle_refresh(Err("应用尚未完成初始化".to_owned()));
            return Task::none();
        };
        self.maintenance.begin_bottle_refresh();
        Task::perform(
            async move { core.list_bottles().await.map_err(|error| error.to_string()) },
            Message::BottlesRefreshed,
        )
    }

    fn show_toast(&mut self, message: impl Into<String>) -> Task<Message> {
        self.toast = Some(Toast::new(message));
        Task::perform(
            async {
                tokio::time::sleep(Duration::from_secs(5)).await;
            },
            |_| Message::ToastDismissed,
        )
    }
}

fn operation_dismiss_task(operation_id: gamemanager_core::OperationId) -> Task<Message> {
    Task::perform(
        async {
            tokio::time::sleep(Duration::from_millis(1_200)).await;
        },
        move |_| Message::OperationDismissed(operation_id),
    )
}

fn bootstrap_task() -> Task<Message> {
    let paths = match AppPaths::discover() {
        Ok(paths) => paths,
        Err(error) => return Task::done(Message::BootstrapFinished(Err(error.to_string()))),
    };
    Task::perform(
        async move {
            let core = Arc::new(
                GameManagerCore::open(paths)
                    .await
                    .map_err(|error| error.to_string())?,
            );
            let snapshot = core.bootstrap().await.map_err(|error| error.to_string())?;
            Ok::<_, String>((core, snapshot))
        },
        Message::BootstrapFinished,
    )
}

fn load_initial_window_size() -> iced::Size {
    let fallback = window::Settings::default().size;
    let Ok(paths) = AppPaths::discover() else {
        return fallback;
    };
    let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    else {
        return fallback;
    };
    match runtime.block_on(GameManagerCore::read_ui_preferences(&paths)) {
        Ok(preferences) if preferences.remember_window_size => preferences
            .window_size
            .map_or(fallback, |[width, height]| {
                iced::Size::new(width as f32, height as f32)
            }),
        _ => fallback,
    }
}

fn window_task(action: WindowAction) -> Task<Message> {
    window::latest().then(move |id| {
        let Some(id) = id else {
            return Task::none();
        };
        match action {
            WindowAction::Drag => window::drag(id),
            WindowAction::Resize(direction) => window::drag_resize(id, direction),
            WindowAction::Minimize => window::minimize(id, true),
            WindowAction::ToggleMaximize => window::toggle_maximize(id),
            WindowAction::Close => window::close(id),
        }
    })
}
