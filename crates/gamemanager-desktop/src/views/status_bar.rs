use iced::{
    Alignment, Background, Element, Length,
    widget::{container, row, text},
};
use iced_shadcn_v2::{Progress, Theme};

use crate::{message::Message, state::OperationView, ui::UiTokens};

pub fn view<'a>(operation: &'a OperationView, theme: &'a Theme) -> Element<'a, Message> {
    let progress = Progress::new(theme)
        .value_maybe(operation.percent.map(f32::from))
        .width(Length::Fixed(UiTokens::STATUS_PROGRESS_WIDTH))
        .height(Length::Fixed(UiTokens::STATUS_PROGRESS_HEIGHT));

    let percent = operation
        .percent
        .map_or_else(|| "…".to_owned(), |value| format!("{value}%"));

    container(
        row![
            row![
                text("状态").size(UiTokens::STATUS_TEXT_SIZE),
                text(&operation.label)
                    .size(UiTokens::STATUS_TEXT_SIZE)
                    .color(theme.palette.muted_foreground),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
            row![progress, text(percent).size(UiTokens::STATUS_TEXT_SIZE)]
                .spacing(8)
                .align_y(Alignment::Center),
        ]
        .align_y(Alignment::Center)
        .spacing(12)
        .width(Length::Fill),
    )
    .height(Length::Fixed(UiTokens::STATUS_BAR_HEIGHT))
    .padding([6, 12])
    .style(move |_| iced::widget::container::Style {
        background: Some(Background::Color(theme.palette.muted)),
        text_color: Some(theme.palette.foreground),
        border: iced::Border {
            color: theme.palette.border,
            width: UiTokens::WINDOW_BORDER,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}
