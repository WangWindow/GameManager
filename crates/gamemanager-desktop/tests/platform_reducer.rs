use std::path::PathBuf;

use gamemanager_core::{OperationId, OperationProgress};
use gamemanager_desktop::{
    app::DesktopApp,
    message::{Message, WindowMessage},
    shell::route_event,
    state::OperationState,
};
use iced::{Event, window};

#[test]
fn dropped_file_opens_the_import_dialog_with_its_path() {
    let mut app = DesktopApp::for_test();
    let path = PathBuf::from("/games/Game.exe");
    app.update_for_test(Message::Window(WindowMessage::FileDropped(path.clone())));
    assert!(app.dialogs.import.open);
    assert_eq!(
        app.dialogs.import.entry_path.as_deref(),
        Some(path.as_path())
    );
}

#[test]
fn native_file_events_reach_window_messages() {
    let path = PathBuf::from("/tmp/drop/Game.exe");
    assert!(matches!(
        route_event(&Event::Window(window::Event::FileHovered(path.clone()))),
        Some(Message::Window(WindowMessage::FileHovered(found))) if found == path
    ));
    assert!(matches!(
        route_event(&Event::Window(window::Event::FileDropped(path.clone()))),
        Some(Message::Window(WindowMessage::FileDropped(found))) if found == path
    ));
    assert!(matches!(
        route_event(&Event::Window(window::Event::FilesHoveredLeft)),
        Some(Message::Window(WindowMessage::FilesHoveredLeft))
    ));
    assert!(matches!(
        route_event(&Event::Window(window::Event::Focused)),
        Some(Message::Window(WindowMessage::Focused(true)))
    ));
    assert!(matches!(
        route_event(&Event::Window(window::Event::Resized(iced::Size::new(1280.0, 720.0)))),
        Some(Message::Window(WindowMessage::Resized(size)))
            if size == iced::Size::new(1280.0, 720.0)
    ));
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

#[test]
fn resized_events_are_ignored_when_remember_window_size_is_off() {
    let mut app = DesktopApp::for_test();
    app.update_for_test(Message::RememberWindowSizeChanged(false));
    app.update_for_test(Message::Window(WindowMessage::Resized(iced::Size::new(
        1280.0, 720.0,
    ))));
    assert_eq!(app.preferences.value().window_size, None);
}

#[test]
fn settled_window_size_updates_preferences() {
    let mut app = DesktopApp::for_test();
    app.update_for_test(Message::WindowSizeSave([1280, 720]));
    assert_eq!(app.preferences.value().window_size, Some([1280, 720]));
}
