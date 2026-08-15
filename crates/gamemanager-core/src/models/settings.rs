use serde::{Deserialize, Serialize};

pub const SETTING_CONTAINER_ROOT: &str = "container_root";
pub const SETTING_BOTTLES_DEFAULT: &str = "bottles_default";
pub const SETTING_BOTTLES_ENABLED: &str = "bottles_enabled";
pub const SETTING_UI_PREFERENCES: &str = "ui_preferences";
pub const SETTING_ENGINE_ENABLED: &str = "engine_enabled";

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub container_root: String,
}

/// Appearance mode persisted in SQLite instead of WebView local storage.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GameViewMode {
    Grid,
    #[default]
    List,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiPreferences {
    #[serde(default)]
    pub theme_mode: ThemeMode,
    #[serde(default = "default_true")]
    pub show_status_bar: bool,
    #[serde(default)]
    pub view_mode: GameViewMode,
    #[serde(default)]
    pub search_query: String,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::System,
            show_status_bar: true,
            view_mode: GameViewMode::List,
            search_query: String::new(),
        }
    }
}

fn default_true() -> bool {
    true
}
