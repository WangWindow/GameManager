use std::path::PathBuf;

use gamemanager_core::OperationId;
use gamemanager_desktop::state::{DialogState, ImportDialogState, ScanDialogState, UtilityDialog};

#[test]
fn selecting_an_entry_enables_import_submission() {
    let mut state = ImportDialogState::default();
    state.set_entry_path(PathBuf::from("/games/Game.exe"));
    assert!(state.can_submit());
}

#[test]
fn import_cannot_submit_twice() {
    let mut state = ImportDialogState::default();
    state.set_entry_path(PathBuf::from("/tmp/Game.exe"));
    state.submitting = true;

    assert!(!state.can_submit());
}

#[test]
fn import_drop_is_visible_then_prefills_the_same_import_flow() {
    let mut state = ImportDialogState::default();
    state.begin_drop();
    assert!(state.drop_active);

    state.accept_dropped_entry(PathBuf::from("/tmp/game.exe"));
    assert!(state.open);
    assert!(!state.drop_active);
    assert_eq!(state.entry_path, Some(PathBuf::from("/tmp/game.exe")));
    assert!(state.error.is_none());
}

#[test]
fn scan_requires_a_directory_and_an_idle_operation_before_starting() {
    let mut state = ScanDialogState::open(PathBuf::new(), 3);
    assert!(!state.can_submit());
    state.root = PathBuf::from("/tmp");
    assert!(state.can_submit());
    state.operation_id = Some(OperationId::new(7));

    assert!(!state.can_submit());
}

#[test]
fn scan_progress_keeps_the_dialog_open_until_completion() {
    let mut state = ScanDialogState::open(PathBuf::from("/games"), 3);
    state.apply_progress("Scanning", Some(60));
    assert!(state.open);
    assert_eq!(state.progress, Some(60));
}

#[test]
fn scan_depth_accepts_numeric_text_and_clamps_button_changes() {
    let mut scan = ScanDialogState::open(PathBuf::from("/games"), 3);
    scan.set_max_depth_text("99");
    assert_eq!(scan.max_depth, 10);
    scan.adjust_max_depth(-99);
    assert_eq!(scan.max_depth, 1);
    scan.set_max_depth_text("not-a-number");
    assert_eq!(scan.max_depth, 1);
}

#[test]
fn escape_dismisses_an_idle_modal_but_not_an_active_scan() {
    let mut dialogs = DialogState::default();
    dialogs.import.open = true;
    assert!(dialogs.dismiss_non_busy());
    dialogs.scan = Some(ScanDialogState::open(PathBuf::from("/games"), 3));
    dialogs.scan.as_mut().expect("scan").operation_id = Some(OperationId::new(1));
    assert!(!dialogs.dismiss_non_busy());
}

#[test]
fn opening_a_utility_dialog_closes_the_title_bar_menu() {
    let mut dialogs = DialogState {
        app_menu_open: true,
        ..DialogState::default()
    };
    dialogs.open_utility(UtilityDialog::Runtime);
    assert_eq!(dialogs.utility, Some(UtilityDialog::Runtime));
    assert!(!dialogs.app_menu_open);
}
