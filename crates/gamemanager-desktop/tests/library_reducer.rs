use gamemanager_core::{GameSummary, GameViewMode};
use gamemanager_desktop::{
    DesktopApp,
    message::{LibraryMessage, Message},
    state::LibraryState,
};

fn game(id: &str, title: &str, engine: &str) -> GameSummary {
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
        last_played_at: None,
    }
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
fn desktop_app_reduces_library_messages() {
    let mut app = DesktopApp::for_test();
    app.library
        .replace_games(vec![game("one", "Moonlight", "nwjs")]);
    app.update_for_test(Message::Library(LibraryMessage::ViewModeChanged(
        GameViewMode::Grid,
    )));
    assert_eq!(app.library.view_mode, GameViewMode::Grid);
}
