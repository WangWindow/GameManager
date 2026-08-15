use gamemanager_core::{AppPaths, GameManagerCore};
use iced::{
    Element, Length, Task, Theme,
    widget::{button, column, container, row, text},
    window,
};
use std::sync::Arc;

use crate::{
    components::action_button,
    message::{Message, WindowAction, WindowMessage},
    state::{DialogState, LibraryState, ShellState},
    views::library_view,
};

pub struct DesktopApp {
    pub shell: ShellState,
    pub dialogs: DialogState,
    pub library: LibraryState,
    pub core: Option<Arc<GameManagerCore>>,
    pub bootstrap_error: Option<String>,
}

impl DesktopApp {
    pub fn boot() -> Self {
        Self {
            shell: ShellState::default(),
            dialogs: DialogState::default(),
            library: LibraryState::default(),
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
            Message::ThemeModeChanged(mode) => self.shell.set_theme_mode(mode),
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
                    self.library.replace_games(snapshot.games);
                    self.bootstrap_error = None;
                }
                Err(error) => self.bootstrap_error = Some(error),
            },
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let controls = row![
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
        container(column![
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
        .into()
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
