use gamemanager_core::GameViewMode;
use iced::{
    Alignment, Background, Element, Length,
    widget::{Space, column, container, responsive, row, scrollable, text},
};
use iced_shadcn_v2::{
    Button, ButtonSize, ButtonVariant, Input, InputGroup, InputGroupAddon, InputGroupAddonAlign,
    InputSize, Theme,
};

use crate::{
    message::{LibraryMessage, Message},
    state::{EngineListState, LibraryState},
    ui::{UiTokens, icons},
};

use super::game_card;

pub fn view<'a>(
    state: &'a LibraryState,
    engines: &'a EngineListState,
    theme: &'a Theme,
) -> Element<'a, Message> {
    let visible_games = state.filtered_games();

    let search = InputGroup::new(theme)
        .push_input(
            Input::new(theme)
                .value(&state.search_query)
                .placeholder("搜索游戏…")
                .size(InputSize::Sm)
                .on_input(|query| Message::Library(LibraryMessage::SearchChanged(query))),
        )
        .push_addon(
            InputGroupAddon::new(
                icons::search()
                    .size(17)
                    .color(theme.palette.muted_foreground),
                theme,
            )
            .align(InputGroupAddonAlign::InlineStart),
        )
        .width(Length::Fill)
        .height(Length::Fixed(UiTokens::CONTROL_HEIGHT));

    let grid_mode = Button::icon(icons::layout_grid().size(17), theme)
        .variant(if state.view_mode == GameViewMode::Grid {
            ButtonVariant::Secondary
        } else {
            ButtonVariant::Ghost
        })
        .size(ButtonSize::IconSm)
        .on_press(Message::Library(LibraryMessage::ViewModeChanged(
            GameViewMode::Grid,
        )));

    let list_mode = Button::icon(icons::list().size(17), theme)
        .variant(if state.view_mode == GameViewMode::List {
            ButtonVariant::Secondary
        } else {
            ButtonVariant::Ghost
        })
        .size(ButtonSize::IconSm)
        .on_press(Message::Library(LibraryMessage::ViewModeChanged(
            GameViewMode::List,
        )));

    let header = row![
        text("游戏库")
            .size(UiTokens::LIBRARY_HEADING_SIZE)
            .color(theme.palette.foreground),
        container(search)
            .max_width(UiTokens::SEARCH_MAX_WIDTH)
            .width(Length::Fill),
        Space::new().width(Length::Fill),
        row![grid_mode, list_mode].spacing(2),
        text(format!("{} 个游戏", visible_games.len()))
            .size(13)
            .color(theme.palette.muted_foreground),
    ]
    .align_y(Alignment::Center)
    .spacing(12)
    .width(Length::Fill);

    let content: Element<'_, Message> = if state.games().is_empty() {
        empty_state("还没有游戏", "从入口文件导入一个游戏开始。", theme)
    } else if visible_games.is_empty() {
        empty_state("没有匹配的游戏", "尝试更换搜索关键词。", theme)
    } else {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs() as i64);
        let view_mode = state.view_mode;
        responsive(move |size| {
            let columns = match view_mode {
                GameViewMode::List => 1,
                GameViewMode::Grid if size.width >= 1_280.0 => 3,
                GameViewMode::Grid if size.width >= 640.0 => 2,
                GameViewMode::Grid => 1,
            };
            game_grid(&visible_games, columns, state, engines, now, theme)
        })
        .height(Length::Fill)
        .into()
    };

    let content = container(column![header, content].spacing(16))
        .width(Length::Fill)
        .max_width(UiTokens::CONTENT_MAX_WIDTH)
        .height(Length::Fill);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([UiTokens::CONTENT_PADDING_Y, 24.0])
        .center_x(Length::Fill)
        .style(background_style(theme))
        .into()
}

fn game_grid<'a>(
    games: &[&'a gamemanager_core::GameSummary],
    columns: usize,
    state: &'a LibraryState,
    engines: &'a EngineListState,
    now: i64,
    theme: &'a Theme,
) -> Element<'a, Message> {
    let mut grid = column![].spacing(12).width(Length::Fill);
    for chunk in games.chunks(columns) {
        let mut line = row![].spacing(12).width(Length::Fill);
        for game in chunk {
            line = line.push(
                container(game_card::view(
                    game,
                    engines.display_name(&game.engine_type),
                    LibraryState::relative_played_time(game.last_played_at, now),
                    state.is_launching(&game.id),
                    theme,
                ))
                .width(Length::Fill),
            );
        }
        for _ in chunk.len()..columns {
            line = line.push(Space::new().width(Length::Fill));
        }
        grid = grid.push(line);
    }
    scrollable(grid).height(Length::Fill).into()
}

fn empty_state<'a>(title: &'a str, description: &'a str, theme: &'a Theme) -> Element<'a, Message> {
    container(
        column![
            icons::gamepad_2()
                .size(UiTokens::EMPTY_STATE_ICON_SIZE)
                .color(theme.palette.muted_foreground),
            text(title).size(18).color(theme.palette.foreground),
            text(description)
                .size(13)
                .color(theme.palette.muted_foreground),
        ]
        .align_x(Alignment::Center)
        .spacing(8),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

fn background_style(theme: &Theme) -> impl Fn(&iced::Theme) -> iced::widget::container::Style + '_ {
    let background = theme.palette.background;
    move |_| iced::widget::container::Style {
        background: Some(Background::Color(background)),
        text_color: Some(theme.palette.foreground),
        ..Default::default()
    }
}
