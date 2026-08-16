use gamemanager_core::GameSummary;
use iced::{
    Alignment, Background, Border, Element, Length,
    widget::{column, container, image, row, text},
};
use iced_shadcn_v2::{Button, ButtonSize, ButtonVariant, Theme};

use crate::{
    message::{LibraryMessage, Message},
    ui::{UiTokens, icons},
};

pub fn view<'a>(
    game: &'a GameSummary,
    engine_name: String,
    played_time: Option<String>,
    launching: bool,
    theme: &'a Theme,
) -> Element<'a, Message> {
    let media: Element<'a, Message> = game
        .cover_path
        .as_deref()
        .map(|path| {
            image(image::Handle::from_path(path))
                .width(Length::Fixed(UiTokens::CARD_COVER_SIZE))
                .height(Length::Fixed(UiTokens::CARD_COVER_SIZE))
                .content_fit(iced::ContentFit::Cover)
                .border_radius(8.0)
                .into()
        })
        .unwrap_or_else(|| {
            container(
                icons::engine(&game.engine_type)
                    .size(UiTokens::CARD_EMPTY_ICON_SIZE)
                    .color(theme.palette.muted_foreground),
            )
            .width(Length::Fixed(UiTokens::CARD_COVER_SIZE))
            .height(Length::Fixed(UiTokens::CARD_COVER_SIZE))
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .style(move |_| iced::widget::container::Style {
                background: Some(Background::Color(theme.palette.muted)),
                border: Border {
                    radius: 8.0.into(),
                    ..Border::default()
                },
                ..Default::default()
            })
            .into()
        });

    let actions =
        row![
            Button::icon(
                if launching {
                    icons::loader_circle()
                } else {
                    icons::play()
                },
                theme
            )
            .variant(ButtonVariant::Default)
            .size(ButtonSize::IconSm)
            .width(Length::Fixed(UiTokens::CARD_ACTION_SIZE))
            .height(Length::Fixed(UiTokens::CARD_ACTION_SIZE))
            .loading(launching)
            .on_press_maybe((!launching).then_some(Message::Library(
                LibraryMessage::LaunchRequested(game.id.clone()),
            ))),
            Button::icon(icons::settings(), theme)
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::IconSm)
                .width(Length::Fixed(UiTokens::CARD_ACTION_SIZE))
                .height(Length::Fixed(UiTokens::CARD_ACTION_SIZE))
                .on_press(Message::OpenGameSettings(game.id.clone())),
            Button::icon(icons::trash_2(), theme)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .width(Length::Fixed(UiTokens::CARD_ACTION_SIZE))
                .height(Length::Fixed(UiTokens::CARD_ACTION_SIZE))
                .on_press(Message::Library(LibraryMessage::DeleteRequested(
                    game.id.clone(),
                ))),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

    let title = text(game.title.as_str())
        .size(UiTokens::CARD_TITLE_SIZE)
        .color(theme.palette.foreground)
        .wrapping(iced::widget::text::Wrapping::None);
    let mut metadata = row![
        icons::engine(&game.engine_type)
            .size(14)
            .color(theme.palette.muted_foreground),
        text(engine_name)
            .size(UiTokens::CARD_METADATA_SIZE)
            .color(theme.palette.muted_foreground)
            .wrapping(iced::widget::text::Wrapping::None),
    ]
    .spacing(6)
    .align_y(Alignment::Center);
    if let Some(played_time) = played_time {
        metadata = metadata.push(
            text("·")
                .size(UiTokens::CARD_METADATA_SIZE)
                .color(theme.palette.muted_foreground),
        );
        metadata = metadata.push(
            text(played_time)
                .size(UiTokens::CARD_METADATA_SIZE)
                .color(theme.palette.muted_foreground)
                .wrapping(iced::widget::text::Wrapping::None),
        );
    }

    let information = container(
        column![title, container(metadata).width(Length::Fill).clip(true),]
            .spacing(5)
            .width(Length::Fill)
            .align_x(Alignment::Start),
    )
    .width(Length::Fill)
    .clip(true);

    container(
        row![
            container(media).width(Length::Shrink),
            information,
            container(actions).width(Length::Shrink),
        ]
        .align_y(Alignment::Center)
        .spacing(UiTokens::CARD_GAP)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .padding([UiTokens::CARD_PADDING_Y, UiTokens::CARD_PADDING_X])
    .style(move |_| iced::widget::container::Style {
        background: Some(Background::Color(theme.palette.card)),
        text_color: Some(theme.palette.card_foreground),
        border: Border {
            color: theme.palette.border,
            width: UiTokens::WINDOW_BORDER,
            radius: UiTokens::CARD_RADIUS.into(),
        },
        ..Default::default()
    })
    .into()
}
