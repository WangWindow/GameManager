use gamemanager_core::{Runner, RuntimeStatus};
use gamemanager_desktop::{state::GameSettingsState, views::game_settings_body_height};

#[test]
fn settings_update_preserves_engine_runtime_arguments_and_cover() {
    let mut state = GameSettingsState::with_runner(Runner::Nwjs);
    state.title = "Moonlight".into();
    state.engine_type = "rpgmakermz".into();
    state.entry_path = "/games/www/index.html".into();
    state.runtime_version = Some("0.84.0".into());
    state.args = vec!["--debug".into(), "--disable-gpu".into()];
    state.cover_file = Some("/games/cover.webp".into());

    let update = state.into_update_request();
    assert_eq!(update.game.engine_id.as_deref(), Some("rpgmakermz"));
    assert_eq!(update.config.runtime_version.as_deref(), Some("0.84.0"));
    assert_eq!(update.config.args, ["--debug", "--disable-gpu"]);
    assert_eq!(
        update.config.cover_file.as_deref(),
        Some("/games/cover.webp")
    );
}

#[test]
fn settings_reject_empty_title_or_entry_before_save() {
    let mut state = GameSettingsState::with_runner(Runner::Native);
    assert!(state.validate().is_err());
    state.title = "Native game".into();
    state.entry_path = "/games/run".into();

    assert!(state.validate().is_ok());
}

#[test]
fn mkxpz_runner_exposes_the_home_sandbox_toggle() {
    let state = GameSettingsState::with_runner(Runner::Mkxpz);
    assert!(state.shows_sandbox_home());
}

#[test]
fn bottles_runner_retains_the_selected_bottle_when_the_form_is_saved() {
    let mut state = GameSettingsState::with_runner(Runner::Bottles);
    state.bottle_name = Some("Games".to_owned());
    assert_eq!(
        state.into_update_request().config.bottle_name.as_deref(),
        Some("Games")
    );
}

#[test]
fn selecting_the_default_bottle_clears_the_game_override() {
    let mut state = GameSettingsState::with_runner(Runner::Bottles);
    state.select_bottle(Some("Games".to_owned()));
    state.select_bottle(None);

    assert!(state.into_update_request().config.bottle_name.is_none());
}

#[test]
fn mkxpz_is_only_offered_for_vx_and_vx_ace_profiles() {
    let mut state = GameSettingsState::with_runner(Runner::Native);
    state.engine_type = "godot".to_owned();
    assert!(
        !state
            .runner_choices(true, true, true)
            .contains(&Runner::Mkxpz)
    );

    state.engine_type = "rpgmakervxace".to_owned();
    assert!(
        state
            .runner_choices(true, true, true)
            .contains(&Runner::Mkxpz)
    );
}

#[test]
fn compact_form_uses_natural_height_until_it_reaches_the_viewport_cap() {
    let native = GameSettingsState::with_runner(Runner::Native);
    let nwjs = GameSettingsState::with_runner(Runner::Nwjs);

    assert!(native.natural_body_height() < nwjs.natural_body_height());
    assert!(native.natural_body_height() > 0.0);
}

#[test]
fn nwjs_version_options_are_discovered_not_typed() {
    let versions = GameSettingsState::nwjs_versions(&[
        RuntimeStatus {
            id: "new".into(),
            name: "NW.js".into(),
            version: "0.84.0".into(),
            engine_type: "nwjs".into(),
            path: "/tmp/new".into(),
        },
        RuntimeStatus {
            id: "other".into(),
            name: "mkxp-z".into(),
            version: "1".into(),
            engine_type: "mkxpz".into(),
            path: "/tmp/mkxpz".into(),
        },
    ]);

    assert_eq!(versions, ["0.84.0"]);
}

#[test]
fn settings_body_uses_its_natural_height_when_the_form_is_short() {
    let state = GameSettingsState::with_runner(Runner::Native);
    assert_eq!(
        game_settings_body_height(1_000.0, state.natural_body_height()),
        state.natural_body_height()
    );
}

#[test]
fn settings_body_scrolls_only_after_reaching_sixty_percent_of_viewport() {
    assert_eq!(game_settings_body_height(600.0, 900.0), 360.0);
    assert_eq!(game_settings_body_height(300.0, 20.0), 20.0);
}
