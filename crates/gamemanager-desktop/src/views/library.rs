use gamemanager_core::GameViewMode;
use iced::{
    Element, Length,
    widget::{button, column, container, row, text, text_input},
};

use crate::{
    message::{LibraryMessage, Message},
    state::LibraryState,
};

use super::game_card;

pub fn view(state: &LibraryState) -> Element<'_, Message> {
    let mode = match state.view_mode {
        GameViewMode::List => GameViewMode::Grid,
        GameViewMode::Grid => GameViewMode::List,
    };
    let controls = row![
        text_input("搜索", &state.search_query)
            .on_input(|query| Message::Library(LibraryMessage::SearchChanged(query)))
            .padding([8, 12])
            .width(Length::Fill),
        button(text(match state.view_mode {
            GameViewMode::List => "▦",
            GameViewMode::Grid => "☷",
        }))
        .padding([8, 12])
        .on_press(Message::Library(LibraryMessage::ViewModeChanged(mode))),
    ]
    .spacing(8);

    let visible_games = state.filtered_games();
    let content = match state.view_mode {
        GameViewMode::List => {
            let mut list = column![].spacing(8);
            for game in &visible_games {
                list = list.push(game_card::view(game, state.is_launching(&game.id)));
            }
            list
        }
        GameViewMode::Grid => {
            let mut grid = column![].spacing(8);
            for chunk in visible_games.chunks(3) {
                let mut line = row![].spacing(8).width(Length::Fill);
                for game in chunk {
                    line = line.push(game_card::view(game, state.is_launching(&game.id)));
                }
                grid = grid.push(line);
            }
            grid
        }
    };

    let content = if state.games().is_empty() {
        column![text("暂无游戏").size(18)]
    } else if visible_games.is_empty() {
        column![text("没有匹配的游戏").size(18)]
    } else {
        content
    };

    container(column![controls, content].spacing(12))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
