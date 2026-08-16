use std::path::PathBuf;

use gamemanager_core::{AppPaths, GameViewMode, ThemeMode, UiPreferences, WindowBackend};

#[test]
fn v09_layout_uses_the_existing_file_names() {
    let paths = AppPaths::from_data_dir(PathBuf::from("/tmp/io.choco.gamemanager"));

    assert_eq!(
        paths.database(),
        PathBuf::from("/tmp/io.choco.gamemanager/db/app.sqlite")
    );
    assert_eq!(
        paths.container_root(),
        PathBuf::from("/tmp/io.choco.gamemanager/containers")
    );
    assert_eq!(
        paths.engine_dir(),
        PathBuf::from("/tmp/io.choco.gamemanager/engines")
    );
    assert_eq!(
        paths.nwjs_runtime_root(),
        PathBuf::from("/tmp/io.choco.gamemanager/runtimes/nwjs")
    );
    assert_eq!(
        paths.mkxpz_runtime_root(),
        PathBuf::from("/tmp/io.choco.gamemanager/runtimes/mkxpz")
    );
}

#[test]
fn ui_preferences_do_not_require_browser_local_storage() {
    let preferences = UiPreferences::default();

    assert_eq!(preferences.theme_mode, ThemeMode::System);
    assert_eq!(preferences.view_mode, GameViewMode::List);
    assert!(preferences.show_status_bar);
    assert!(preferences.search_query.is_empty());
    assert_eq!(preferences.window_backend, WindowBackend::Auto);
    assert!(preferences.remember_window_size);
    assert_eq!(preferences.window_size, None);
}
