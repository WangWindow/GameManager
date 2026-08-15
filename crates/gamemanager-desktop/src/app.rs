use iced::{
    Element, Length, Task, Theme,
    widget::{button, column, container, row, text},
    window,
};

use crate::{
    components::action_button,
    message::{Message, WindowAction},
    state::ShellState,
};

pub struct DesktopApp {
    pub shell: ShellState,
}

impl DesktopApp {
    pub fn boot() -> Self {
        Self {
            shell: ShellState::default(),
        }
    }

    pub fn run() -> iced::Result {
        iced::application(Self::boot, Self::update, Self::view)
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
            Message::Window(action) => return window_task(action),
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let controls = row![
            action_button("—", Message::Window(WindowAction::Minimize)),
            action_button("□", Message::Window(WindowAction::ToggleMaximize)),
            action_button("×", Message::Window(WindowAction::Close)),
        ]
        .spacing(4);
        let title = button(text("GameManager").size(22))
            .width(Length::Fill)
            .padding(16)
            .on_press(Message::Window(WindowAction::Drag));
        container(column![
            row![title, controls].height(Length::Shrink),
            text("游戏库").size(30)
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(16)
        .into()
    }

    fn theme(&self) -> Theme {
        self.shell.iced_theme()
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
