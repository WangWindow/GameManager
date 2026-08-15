use std::sync::Arc;

use iced::window::Direction;

use gamemanager_core::{BootstrapSnapshot, GameManagerCore, ThemeMode};

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
}
