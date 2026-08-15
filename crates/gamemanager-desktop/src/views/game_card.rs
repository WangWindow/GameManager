use gamemanager_core::GameSummary;
use iced::{
    Element, Length,
    widget::{button, column, container, row, text},
};

use crate::message::{LibraryMessage, Message};

pub fn view<'a>(game: &'a GameSummary, launching: bool) -> Element<'a, Message> {
    let title = column![
        text(game.title.as_str()).size(20),
        text(format!("{} · {}", game.engine_type, game.game_type)).size(14),
    ]
    .spacing(4)
    .width(Length::Fill);

    let launch = if launching {
        button(text("…")).padding([8, 14])
    } else {
        button(text("▶"))
            .padding([8, 14])
            .on_press(Message::Library(LibraryMessage::LaunchRequested(
                game.id.clone(),
            )))
    };

    container(row![title, launch].spacing(12))
        .width(Length::Fill)
        .padding(14)
        .into()
}
