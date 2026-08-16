use gamemanager_core::{GameSummary, GameViewMode};
use gamemanager_desktop::{
    DesktopApp,
    message::{LibraryMessage, Message},
    state::{EngineListState, EngineRow, LibraryState},
};

fn game(id: &str, title: &str, engine: &str) -> GameSummary {
    game_at(id, title, engine, 0, None)
}

fn game_at(
    id: &str,
    title: &str,
    engine: &str,
    created_at: i64,
    last_played_at: Option<i64>,
) -> GameSummary {
    GameSummary {
        id: id.to_owned(),
        profile_key: format!("profile-{id}"),
        title: title.to_owned(),
        engine_type: engine.to_owned(),
        game_path: format!("/games/{id}"),
        game_type: "Other".to_owned(),
        detection_confidence: 100,
        runtime_version: None,
        cover_path: None,
        play_count: 0,
        created_at,
        last_played_at,
    }
}

#[test]
fn game_summaries_stay_sorted_after_insert_and_update() {
    let mut state = LibraryState::with_games(vec![game_at("older", "Older", "godot", 10, None)]);
    state.apply_game(game_at("newer", "Newer", "godot", 20, None));
    state.apply_game(game_at("older", "Older", "godot", 10, Some(30)));

    let ids = state
        .games()
        .iter()
        .map(|game| game.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids, ["older", "newer"]);
}

#[test]
fn relative_metadata_omits_unplayed_games() {
    assert_eq!(LibraryState::relative_played_time(None, 10_000), None);
    assert_eq!(
        LibraryState::relative_played_time(Some(9_880), 10_000),
        Some("2 分钟前".to_owned())
    );
}

#[test]
fn card_metadata_uses_the_engine_display_name() {
    let engines = EngineListState::with_entries(vec![EngineRow {
        id: "rpgmakervx".to_owned(),
        name: "RPG Maker VX / VX Ace".to_owned(),
        enabled: true,
        valid: true,
        rule_count: 1,
        minimum_score: 0,
        rules: Vec::new(),
        strategy: "mkxpz".to_owned(),
        entry_patterns: Vec::new(),
        exclude_patterns: Vec::new(),
        errors: Vec::new(),
    }]);
    assert_eq!(engines.display_name("rpgmakervx"), "RPG Maker VX / VX Ace");
    assert_eq!(engines.display_name("other"), "Other");
}

#[test]
fn direct_view_mode_selection_is_not_a_toggle() {
    let mut app = DesktopApp::for_test();
    app.update_for_test(Message::Library(LibraryMessage::ViewModeChanged(
        GameViewMode::Grid,
    )));
    app.update_for_test(Message::Library(LibraryMessage::ViewModeChanged(
        GameViewMode::Grid,
    )));

    assert_eq!(app.library.view_mode, GameViewMode::Grid);
}

#[test]
fn library_search_matches_title_and_engine_without_mutating_source_games() {
    let mut state = LibraryState::with_games(vec![
        game("one", "Moonlight", "nwjs"),
        game("two", "Forest", "godot"),
    ]);
    state.apply(LibraryMessage::SearchChanged("NWJS".to_owned()));

    let visible = state.filtered_games();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].id, "one");
    assert_eq!(state.games().len(), 2);
}

#[test]
fn launch_messages_only_mark_existing_games_busy_and_finish_cleanly() {
    let mut state = LibraryState::with_games(vec![game("one", "Moonlight", "nwjs")]);
    state.apply(LibraryMessage::LaunchRequested("missing".to_owned()));
    assert!(!state.is_launching("missing"));

    state.apply(LibraryMessage::LaunchRequested("one".to_owned()));
    assert!(state.is_launching("one"));
    state.apply(LibraryMessage::LaunchFinished {
        game_id: "one".to_owned(),
        success: true,
    });
    assert!(!state.is_launching("one"));
}

#[test]
fn completed_launch_refreshes_the_library_entry() {
    let mut state = LibraryState::with_games(vec![game_at("one", "Moonlight", "nwjs", 1, None)]);
    assert!(state.start_launch("one"));

    state.finish_launch("one");
    state.apply_game(game_at("one", "Moonlight", "nwjs", 1, Some(10)));

    assert!(!state.is_launching("one"));
    assert_eq!(state.games()[0].last_played_at, Some(10));
}

#[test]
fn desktop_app_reduces_library_messages() {
    let mut app = DesktopApp::for_test();
    app.library
        .replace_games(vec![game("one", "Moonlight", "nwjs")]);
    app.update_for_test(Message::Library(LibraryMessage::ViewModeChanged(
        GameViewMode::Grid,
    )));
    assert_eq!(app.library.view_mode, GameViewMode::Grid);
}
