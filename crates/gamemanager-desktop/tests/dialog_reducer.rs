use std::path::PathBuf;

use gamemanager_desktop::state::{ImportDialogState, ScanDialogState};

#[test]
fn selecting_an_entry_enables_import_submission() {
    let mut state = ImportDialogState::default();
    state.set_entry_path(PathBuf::from("/games/Game.exe"));
    assert!(state.can_submit());
}

#[test]
fn scan_progress_keeps_the_dialog_open_until_completion() {
    let mut state = ScanDialogState::open(PathBuf::from("/games"), 3);
    state.apply_progress("Scanning", Some(60));
    assert!(state.open);
    assert_eq!(state.progress, Some(60));
}
