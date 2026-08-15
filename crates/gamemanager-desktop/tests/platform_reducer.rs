use std::path::PathBuf;

use gamemanager_core::{OperationId, OperationProgress};
use gamemanager_desktop::{
    app::DesktopApp,
    message::{Message, WindowMessage},
    state::OperationState,
};

#[test]
fn dropped_file_opens_the_import_dialog_with_its_path() {
    let mut app = DesktopApp::for_test();
    let path = PathBuf::from("/games/Game.exe");
    app.update_for_test(Message::Window(WindowMessage::FileDropped(path.clone())));
    assert_eq!(
        app.dialogs.import.entry_path.as_deref(),
        Some(path.as_path())
    );
}

#[test]
fn operation_progress_is_replaced_by_the_latest_event() {
    let mut state = OperationState::default();
    state.apply(OperationProgress::new(
        OperationId::new(1),
        "download",
        Some(40),
    ));
    state.apply(OperationProgress::new(
        OperationId::new(1),
        "install",
        Some(100),
    ));
    assert_eq!(
        state.get(OperationId::new(1)).expect("operation").label,
        "install"
    );
}
