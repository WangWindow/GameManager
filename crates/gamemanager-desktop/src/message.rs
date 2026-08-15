use iced::window::Direction;

use gamemanager_core::ThemeMode;

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

#[derive(Clone, Debug)]
pub enum Message {
    ThemeModeChanged(ThemeMode),
    SystemThemeChanged(SystemTheme),
    Window(WindowMessage),
}
