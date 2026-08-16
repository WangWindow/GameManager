use gamemanager_core::ThemeMode;
use gamemanager_desktop::{
    message::SystemTheme,
    state::{AppTheme, ShellState},
};

#[test]
fn system_theme_changes_only_affect_system_mode() {
    let mut state = ShellState::with_theme_mode(ThemeMode::System);
    state.apply_system_theme(SystemTheme::Dark);
    assert_eq!(state.resolved_theme(), AppTheme::Dark);
    state.set_theme_mode(ThemeMode::Light);
    state.apply_system_theme(SystemTheme::Dark);
    assert_eq!(state.resolved_theme(), AppTheme::Light);
}
