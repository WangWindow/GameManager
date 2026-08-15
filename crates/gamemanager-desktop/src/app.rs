use gamemanager_core::{AppPaths, GameManagerCore, ImportRequest, OperationProgress, ScanRequest};
use iced::{
    Element, Length, Task, Theme,
    widget::{button, column, container, row, text},
    window,
};
use std::sync::Arc;

use crate::{
    components::Modal,
    components::action_button,
    message::{Message, WindowAction, WindowMessage},
    platform::{DesktopDialog, DesktopOpener},
    state::{
        DialogState, EngineListState, GameSettingsState, LibraryState, MaintenanceState,
        OperationState, PreferencesState, ShellState,
    },
    views::{game_settings_view, import_view, library_view, scan_view, settings_view},
};

pub struct DesktopApp {
    pub shell: ShellState,
    pub dialogs: DialogState,
    pub library: LibraryState,
    pub operations: OperationState,
    pub engines: EngineListState,
    pub maintenance: MaintenanceState,
    pub preferences: PreferencesState,
    pub settings_open: bool,
    pub core: Option<Arc<GameManagerCore>>,
    pub bootstrap_error: Option<String>,
}

impl DesktopApp {
    pub fn boot() -> Self {
        Self {
            shell: ShellState::default(),
            dialogs: DialogState::default(),
            library: LibraryState::default(),
            operations: OperationState::default(),
            engines: EngineListState::default(),
            maintenance: MaintenanceState::default(),
            preferences: PreferencesState::default(),
            settings_open: false,
            core: None,
            bootstrap_error: None,
        }
    }

    fn boot_with_task() -> (Self, Task<Message>) {
        (Self::boot(), bootstrap_task())
    }

    pub fn run() -> iced::Result {
        iced::application(Self::boot_with_task, Self::update, Self::view)
            .title("GameManager")
            .theme(Self::theme)
            .window(window::Settings {
                decorations: false,
                transparent: true,
                ..window::Settings::default()
            })
            .run()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ThemeModeChanged(mode) => {
                self.shell.set_theme_mode(mode);
                self.preferences.set_theme_mode(mode);
                if let Some(core) = self.core.clone() {
                    let preferences = self.preferences.value().clone();
                    return Task::perform(
                        async move {
                            core.save_ui_preferences(&preferences)
                                .await
                                .map_err(|error| error.to_string())
                        },
                        Message::PreferencesSaved,
                    );
                }
            }
            Message::SystemThemeChanged(theme) => self.shell.apply_system_theme(theme),
            Message::Window(WindowMessage::Action(action)) => return window_task(action),
            Message::Window(WindowMessage::FileDropped(path)) => {
                self.dialogs.import.set_entry_path(path)
            }
            Message::Window(WindowMessage::FileHovered(_))
            | Message::Window(WindowMessage::FilesHoveredLeft)
            | Message::Window(WindowMessage::Focused(_)) => {}
            Message::Library(message) => self.library.apply(message),
            Message::BootstrapFinished(result) => match result {
                Ok((core, snapshot)) => {
                    self.core = Some(core);
                    self.shell
                        .set_theme_mode(snapshot.ui_preferences.theme_mode);
                    self.preferences = PreferencesState::from_value(snapshot.ui_preferences);
                    self.library.replace_games(snapshot.games);
                    self.engines = EngineListState::from_details(snapshot.engine_details);
                    self.maintenance = MaintenanceState::with_runtimes(snapshot.runtimes);
                    self.bootstrap_error = None;
                }
                Err(error) => self.bootstrap_error = Some(error),
            },
            Message::OpenImport => {
                self.dialogs.import.open = true;
                self.dialogs.import.error = None;
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
                if !scan.root.is_dir() {
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
                    self.dialogs.scan = None;
                    if let Some(core) = self.core.clone() {
                        return Task::perform(
                            async move {
                                let snapshot =
                                    core.bootstrap().await.map_err(|error| error.to_string())?;
                                Ok::<_, String>((core, snapshot))
                            },
                            Message::BootstrapFinished,
                        );
                    }
                }
                Err(error) => {
                    if let Some(scan) = self.dialogs.scan.as_mut() {
                        scan.error = Some(error);
                    }
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
            Message::GameSettingsEntryChanged(entry) => {
                if let Some(settings) = self.dialogs.settings.as_mut() {
                    settings.entry_path = entry;
                }
            }
            Message::GameSettingsRunnerChanged(runner) => {
                if let Some(settings) = self.dialogs.settings.as_mut() {
                    settings.runner = runner;
                }
            }
            Message::GameSettingsSandboxChanged(enabled) => {
                if let Some(settings) = self.dialogs.settings.as_mut() {
                    settings.sandbox_home = enabled;
                }
            }
            Message::GameSettingsBottleChanged(name) => {
                if let Some(settings) = self.dialogs.settings.as_mut() {
                    settings.bottle_name = (!name.trim().is_empty()).then_some(name);
                }
            }
            Message::SaveGameSettings => {
                let Some(core) = self.core.clone() else {
                    return Task::none();
                };
                let Some(settings) = self.dialogs.settings.as_mut() else {
                    return Task::none();
                };
                let game_id = settings.game_id.clone();
                let update = settings.into_update_request();
                settings.saving = true;
                return Task::perform(
                    async move {
                        core.save_game_settings(&game_id, update.game, &update.config)
                            .await
                            .map_err(|error| error.to_string())
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
            Message::OpenSettings => self.settings_open = true,
            Message::CloseSettings => self.settings_open = false,
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
            Message::StatusBarChanged(show) => {
                self.preferences.set_show_status_bar(show);
                if let Some(core) = self.core.clone() {
                    let preferences = self.preferences.value().clone();
                    return Task::perform(
                        async move {
                            core.save_ui_preferences(&preferences)
                                .await
                                .map_err(|error| error.to_string())
                        },
                        Message::PreferencesSaved,
                    );
                }
            }
            Message::PreferencesSaved(result) => {
                if let Err(error) = result {
                    self.bootstrap_error = Some(error);
                } else {
                    let _ = self.preferences.take_dirty_value();
                }
            }
            Message::EngineEnabledSaved(result) => {
                if let Err(error) = result {
                    self.bootstrap_error = Some(error);
                }
            }
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
                return Task::perform(
                    async move {
                        let install = core
                            .runtime_manager()
                            .import_mkxpz_archive(&path)
                            .map_err(|error| error.to_string())?;
                        core.register_mkxpz_runtime(&install)
                            .await
                            .map_err(|error| error.to_string())?;
                        Ok::<_, String>(install)
                    },
                    Message::MkxpzImportFinished,
                );
            }
            Message::MkxpzImportFinished(result) => match result {
                Ok(_) => {
                    if let Some(core) = self.core.clone() {
                        return Task::perform(
                            async move {
                                let snapshot =
                                    core.bootstrap().await.map_err(|error| error.to_string())?;
                                Ok::<_, String>((core, snapshot))
                            },
                            Message::BootstrapFinished,
                        );
                    }
                }
                Err(error) => self.bootstrap_error = Some(error),
            },
            Message::OpenMkxpzBuilds => {
                if let Err(error) = DesktopOpener
                    .open_url("https://github.com/mkxp-z/mkxp-z/actions/workflows/autobuild.yml")
                {
                    self.bootstrap_error = Some(error.to_string());
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let controls = row![
            action_button("＋", Message::OpenImport),
            action_button("⌕", Message::OpenScan),
            action_button("⚙", Message::OpenSettings),
            action_button(
                "—",
                Message::Window(WindowMessage::Action(WindowAction::Minimize))
            ),
            action_button(
                "□",
                Message::Window(WindowMessage::Action(WindowAction::ToggleMaximize))
            ),
            action_button(
                "×",
                Message::Window(WindowMessage::Action(WindowAction::Close))
            ),
        ]
        .spacing(4);
        let title = button(text("GameManager").size(22))
            .width(Length::Fill)
            .padding(16)
            .on_press(Message::Window(WindowMessage::Action(WindowAction::Drag)));
        let base: Element<'_, Message> = container(column![
            row![title, controls].height(Length::Shrink),
            text("游戏库").size(30),
            self.bootstrap_error
                .as_deref()
                .map(|error| text(error).size(14))
                .unwrap_or_else(|| text("")),
            library_view(&self.library)
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(16)
        .into();
        let with_import = if self.dialogs.import.open {
            Modal::new(import_view(&self.dialogs.import)).overlay(base)
        } else {
            base
        };
        let with_scan = if let Some(scan) = self.dialogs.scan.as_ref() {
            Modal::new(scan_view(scan)).overlay(with_import)
        } else {
            with_import
        };
        let with_settings = if let Some(settings) = self.dialogs.settings.as_ref() {
            Modal::new(game_settings_view(settings)).overlay(with_scan)
        } else {
            with_scan
        };
        if self.settings_open {
            Modal::new(settings_view(
                &self.preferences,
                &self.engines,
                &self.maintenance,
            ))
            .overlay(with_settings)
        } else {
            with_settings
        }
    }

    fn theme(&self) -> Theme {
        self.shell.iced_theme()
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
