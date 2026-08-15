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

#[derive(Clone, Copy, Debug)]
pub enum Message {
    ThemeModeChanged(ThemeMode),
    SystemThemeChanged(SystemTheme),
    Window(WindowAction),
}
