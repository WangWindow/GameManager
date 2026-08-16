use iced::{
    Element, Event, Length,
    alignment::{Horizontal, Vertical},
    mouse,
    widget::{Space, container, mouse_area, stack},
    window,
};

use crate::{
    message::{Message, WindowAction, WindowMessage},
    ui::UiTokens,
};

pub fn route_event(event: &Event) -> Option<Message> {
    match event {
        Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            ..
        }) => Some(Message::DismissOverlay),
        Event::Window(window::Event::FileHovered(path)) => {
            Some(Message::Window(WindowMessage::FileHovered(path.clone())))
        }
        Event::Window(window::Event::FileDropped(path)) => {
            Some(Message::Window(WindowMessage::FileDropped(path.clone())))
        }
        Event::Window(window::Event::FilesHoveredLeft) => {
            Some(Message::Window(WindowMessage::FilesHoveredLeft))
        }
        Event::Window(window::Event::Focused) => {
            Some(Message::Window(WindowMessage::Focused(true)))
        }
        Event::Window(window::Event::Unfocused) => {
            Some(Message::Window(WindowMessage::Focused(false)))
        }
        _ => None,
    }
}

pub fn resize_handles<'a>(base: Element<'a, Message>, enabled: bool) -> Element<'a, Message> {
    stack![
        base,
        aligned_handle(
            window::Direction::North,
            mouse::Interaction::ResizingVertically,
            Length::Fill,
            Length::Fixed(UiTokens::RESIZE_EDGE_SIZE),
            Horizontal::Left,
            Vertical::Top,
            enabled,
        ),
        aligned_handle(
            window::Direction::South,
            mouse::Interaction::ResizingVertically,
            Length::Fill,
            Length::Fixed(UiTokens::RESIZE_EDGE_SIZE),
            Horizontal::Left,
            Vertical::Bottom,
            enabled,
        ),
        aligned_handle(
            window::Direction::West,
            mouse::Interaction::ResizingHorizontally,
            Length::Fixed(UiTokens::RESIZE_EDGE_SIZE),
            Length::Fill,
            Horizontal::Left,
            Vertical::Top,
            enabled,
        ),
        aligned_handle(
            window::Direction::East,
            mouse::Interaction::ResizingHorizontally,
            Length::Fixed(UiTokens::RESIZE_EDGE_SIZE),
            Length::Fill,
            Horizontal::Right,
            Vertical::Top,
            enabled,
        ),
        aligned_handle(
            window::Direction::NorthWest,
            mouse::Interaction::ResizingDiagonallyDown,
            Length::Fixed(UiTokens::RESIZE_CORNER_SIZE),
            Length::Fixed(UiTokens::RESIZE_CORNER_SIZE),
            Horizontal::Left,
            Vertical::Top,
            enabled,
        ),
        aligned_handle(
            window::Direction::NorthEast,
            mouse::Interaction::ResizingDiagonallyUp,
            Length::Fixed(UiTokens::RESIZE_CORNER_SIZE),
            Length::Fixed(UiTokens::RESIZE_CORNER_SIZE),
            Horizontal::Right,
            Vertical::Top,
            enabled,
        ),
        aligned_handle(
            window::Direction::SouthWest,
            mouse::Interaction::ResizingDiagonallyUp,
            Length::Fixed(UiTokens::RESIZE_CORNER_SIZE),
            Length::Fixed(UiTokens::RESIZE_CORNER_SIZE),
            Horizontal::Left,
            Vertical::Bottom,
            enabled,
        ),
        aligned_handle(
            window::Direction::SouthEast,
            mouse::Interaction::ResizingDiagonallyDown,
            Length::Fixed(UiTokens::RESIZE_CORNER_SIZE),
            Length::Fixed(UiTokens::RESIZE_CORNER_SIZE),
            Horizontal::Right,
            Vertical::Bottom,
            enabled,
        ),
    ]
    .into()
}

fn aligned_handle<'a>(
    direction: window::Direction,
    interaction: mouse::Interaction,
    width: Length,
    height: Length,
    horizontal: Horizontal,
    vertical: Vertical,
    enabled: bool,
) -> Element<'a, Message> {
    let mut handle = mouse_area(Space::new().width(width).height(height)).interaction(interaction);
    if enabled {
        handle = handle.on_press(Message::Window(WindowMessage::Action(
            WindowAction::Resize(direction),
        )));
    }
    container(handle)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(horizontal)
        .align_y(vertical)
        .into()
}
