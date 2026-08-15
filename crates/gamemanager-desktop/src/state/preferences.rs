use gamemanager_core::ThemeMode;

use crate::message::SystemTheme;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppTheme {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShellState {
    theme_mode: ThemeMode,
    system_theme: SystemTheme,
}

impl Default for ShellState {
    fn default() -> Self {
        Self::with_theme_mode(ThemeMode::System)
    }
}

impl ShellState {
    pub fn with_theme_mode(theme_mode: ThemeMode) -> Self {
        Self {
            theme_mode,
            system_theme: SystemTheme::Dark,
        }
    }

    pub fn theme_mode(&self) -> ThemeMode {
        self.theme_mode
    }

    pub fn set_theme_mode(&mut self, theme_mode: ThemeMode) {
        self.theme_mode = theme_mode;
    }

    pub fn apply_system_theme(&mut self, theme: SystemTheme) {
        self.system_theme = theme;
    }

    pub fn resolved_theme(&self) -> AppTheme {
        match self.theme_mode {
            ThemeMode::Light => AppTheme::Light,
            ThemeMode::Dark => AppTheme::Dark,
            ThemeMode::System => match self.system_theme {
                SystemTheme::Light => AppTheme::Light,
                SystemTheme::Dark => AppTheme::Dark,
            },
        }
    }

    pub fn iced_theme(&self) -> iced::Theme {
        match self.resolved_theme() {
            AppTheme::Light => iced::Theme::Light,
            AppTheme::Dark => iced::Theme::Dark,
        }
    }
}
