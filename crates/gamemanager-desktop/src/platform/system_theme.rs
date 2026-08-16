use crate::{message::SystemTheme, state::AppTheme};

#[derive(Clone, Copy, Debug, Default)]
pub struct DesktopTheme;

impl DesktopTheme {
    pub fn detect(self) -> SystemTheme {
        match dark_light::detect().unwrap_or(dark_light::Mode::Dark) {
            dark_light::Mode::Light => SystemTheme::Light,
            _ => SystemTheme::Dark,
        }
    }

    pub fn as_app_theme(self) -> AppTheme {
        match self.detect() {
            SystemTheme::Light => AppTheme::Light,
            SystemTheme::Dark => AppTheme::Dark,
        }
    }
}
