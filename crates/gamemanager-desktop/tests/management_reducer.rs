use gamemanager_core::{EngineSummary, ThemeMode};
use gamemanager_desktop::state::{EngineListState, EngineRow, PreferencesState};

fn engine(id: &str, enabled: bool) -> EngineRow {
    EngineRow {
        id: id.to_owned(),
        name: id.to_owned(),
        enabled,
        valid: true,
        rule_count: 1,
        strategy: "native".to_owned(),
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
            icon: "ri:html5-line".to_owned(),
            priority: 1,
            description: String::new(),
            enabled: true,
            entry_patterns: vec!["*.html".to_owned()],
        },
        valid: true,
        rule_count: 1,
        strategy: "nwjs".to_owned(),
        errors: Vec::new(),
    }]);
    assert_eq!(row.entry("html").expect("html").strategy, "nwjs");
}
