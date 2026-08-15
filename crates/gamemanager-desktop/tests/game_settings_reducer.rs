use gamemanager_core::Runner;
use gamemanager_desktop::state::GameSettingsState;

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
