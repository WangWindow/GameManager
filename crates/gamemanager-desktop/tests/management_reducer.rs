use gamemanager_core::{EngineSummary, ThemeMode};
use gamemanager_desktop::state::{
    AppearanceDialogState, DialogState, EngineListState, EngineRow, MaintenanceState,
    PreferencesState, UtilityDialog,
};

fn engine(id: &str, enabled: bool) -> EngineRow {
    EngineRow {
        id: id.to_owned(),
        name: id.to_owned(),
        enabled,
        valid: true,
        rule_count: 1,
        minimum_score: 0,
        rules: Vec::new(),
        strategy: "native".to_owned(),
        entry_patterns: Vec::new(),
        exclude_patterns: Vec::new(),
        errors: Vec::new(),
    }
}

#[test]
fn disabling_an_engine_updates_only_that_engine_row() {
    let mut state =
        EngineListState::with_entries(vec![engine("html", true), engine("godot", true)]);
    state.apply_enabled("html", false);
    assert!(!state.entry("html").expect("html").enabled);
    assert!(state.entry("godot").expect("godot").enabled);
}

#[test]
fn expanding_a_row_never_changes_its_enabled_switch() {
    let mut state = EngineListState::with_entries(vec![engine("html", true)]);
    state.toggle_expanded("html");
    assert!(state.is_expanded("html"));
    assert!(state.entry("html").expect("html").enabled);

    state.toggle_expanded("html");
    assert!(!state.is_expanded("html"));
}

#[test]
fn changing_theme_schedules_sqlite_preference_persistence() {
    let mut state = PreferencesState::default();
    state.set_theme_mode(ThemeMode::Dark);
    assert!(state.is_dirty());
    assert_eq!(state.value().theme_mode, ThemeMode::Dark);
    assert_eq!(
        state.take_dirty_value().expect("dirty").theme_mode,
        ThemeMode::Dark
    );
    assert!(!state.is_dirty());
}

#[test]
fn engine_rows_can_be_created_from_core_summaries() {
    let row = EngineListState::from_details(vec![gamemanager_core::EngineDetail {
        summary: EngineSummary {
            id: "html".to_owned(),
            name: "HTML".to_owned(),
            category: "web".to_owned(),
            priority: 1,
            description: String::new(),
            enabled: true,
            entry_patterns: vec!["*.html".to_owned()],
        },
        valid: true,
        rule_count: 1,
        minimum_score: 0,
        rules: Vec::new(),
        strategy: "nwjs".to_owned(),
        exclude_patterns: Vec::new(),
        errors: Vec::new(),
    }]);
    assert_eq!(row.entry("html").expect("html").strategy, "nwjs");
}

#[test]
fn opening_runtime_does_not_open_engine_or_appearance() {
    let mut dialogs = DialogState::default();
    dialogs.open_utility(UtilityDialog::Runtime);

    assert_eq!(dialogs.utility, Some(UtilityDialog::Runtime));
    assert!(!dialogs.app_menu_open);
}

#[test]
fn appearance_dialog_cannot_be_dismissed_while_a_maintenance_task_is_running() {
    let mut state = DialogState::default();
    state.open_utility(UtilityDialog::Appearance);
    state.appearance = AppearanceDialogState {
        cleaning_profiles: true,
        ..AppearanceDialogState::default()
    };

    assert!(!state.dismiss_non_busy());
    assert_eq!(state.utility, Some(UtilityDialog::Appearance));

    state.appearance.cleaning_profiles = false;
    assert!(state.dismiss_non_busy());
    assert_eq!(state.utility, None);
}

#[test]
fn appearance_preferences_remain_persistable() {
    let mut state = PreferencesState::default();
    state.set_theme_mode(ThemeMode::Dark);
    state.set_show_status_bar(false);

    assert_eq!(
        state.take_dirty_value().expect("preferences").theme_mode,
        ThemeMode::Dark
    );
}

#[test]
fn bottle_refresh_preserves_the_last_successful_list_on_error() {
    let mut state = MaintenanceState::with_runtime_snapshot(
        Vec::new(),
        true,
        true,
        vec!["Games".to_owned()],
        None,
        None,
    );

    assert!(state.can_select_bottles());
    state.begin_bottle_refresh();
    assert!(state.bottles_loading());
    assert!(!state.can_select_bottles());

    state.finish_bottle_refresh(Err("bottles-cli failed".to_owned()));
    assert_eq!(state.bottles(), ["Games"]);
    assert_eq!(state.bottles_error(), Some("bottles-cli failed"));
    assert!(state.can_select_bottles());
}
