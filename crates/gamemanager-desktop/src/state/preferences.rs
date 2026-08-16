use gamemanager_core::{GameViewMode, ThemeMode, UiPreferences, WindowBackend};

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
    window_maximized: bool,
}

#[derive(Clone, Debug, Default)]
pub struct PreferencesState {
    value: UiPreferences,
    dirty: bool,
    revision: u64,
}

impl PreferencesState {
    pub fn from_value(value: UiPreferences) -> Self {
        Self {
            value,
            dirty: false,
            revision: 0,
        }
    }

    pub fn value(&self) -> &UiPreferences {
        &self.value
    }

    pub fn set_theme_mode(&mut self, mode: ThemeMode) {
        if self.value.theme_mode != mode {
            self.value.theme_mode = mode;
            self.mark_dirty();
        }
    }

    pub fn set_show_status_bar(&mut self, show: bool) {
        if self.value.show_status_bar != show {
            self.value.show_status_bar = show;
            self.mark_dirty();
        }
    }

    pub fn set_search_query(&mut self, query: String) {
        if self.value.search_query != query {
            self.value.search_query = query;
            self.mark_dirty();
        }
    }

    pub fn set_view_mode(&mut self, view_mode: GameViewMode) {
        if self.value.view_mode != view_mode {
            self.value.view_mode = view_mode;
            self.mark_dirty();
        }
    }

    pub fn set_window_backend(&mut self, backend: WindowBackend) {
        if self.value.window_backend != backend {
            self.value.window_backend = backend;
            self.mark_dirty();
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn dirty_snapshot(&self) -> Option<(u64, UiPreferences)> {
        self.dirty.then(|| (self.revision, self.value.clone()))
    }

    pub fn is_current_revision(&self, revision: u64) -> bool {
        self.revision == revision
    }

    pub fn mark_saved(&mut self, revision: u64) {
        if self.revision == revision {
            self.dirty = false;
        }
    }

    pub fn take_dirty_value(&mut self) -> Option<UiPreferences> {
        self.dirty.then(|| {
            self.dirty = false;
            self.value.clone()
        })
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.revision = self.revision.wrapping_add(1);
    }
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
            window_maximized: false,
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

    pub fn toggle_window_maximized(&mut self) {
        self.window_maximized = !self.window_maximized;
    }

    pub fn is_window_maximized(&self) -> bool {
        self.window_maximized
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

    pub fn shadcn_theme(&self) -> iced_shadcn_v2::Theme {
        let mode = match self.resolved_theme() {
            AppTheme::Light => iced_shadcn_v2::ThemeMode::Light,
            AppTheme::Dark => iced_shadcn_v2::ThemeMode::Dark,
        };
        iced_shadcn_v2::Theme::light()
            .with_style(iced_shadcn_v2::StyleId::Vega)
            .with_mode(mode)
    }
}
