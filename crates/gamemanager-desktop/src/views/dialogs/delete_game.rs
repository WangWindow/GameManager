use iced::{
    Alignment, Element, Length,
    widget::{Space, column, container, row, text},
};
use iced_shadcn_v2::{Button, ButtonSize, ButtonVariant, Theme};

use crate::{
    message::Message,
    state::DeleteDialogState,
    ui::{UiTokens, dialog_surface, icons},
};

pub fn view<'a>(state: &'a DeleteDialogState, theme: &'a Theme) -> Element<'a, Message> {
    let close = Button::icon(icons::x(), theme)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::IconSm)
        .on_press_maybe((!state.deleting).then_some(Message::CloseDeleteGame));
    let mut content = column![
        row![
            text("移除游戏").size(20).color(theme.palette.foreground),
            container(close)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right),
        ]
        .align_y(Alignment::Center),
        text(format!("确定要从游戏库移除“{}”吗？", state.title))
            .size(14)
            .color(theme.palette.foreground),
        text("只会移除入库记录，不会删除游戏文件。")
            .size(13)
            .color(theme.palette.muted_foreground),
    ]
    .spacing(12);

    if let Some(error) = state.error.as_deref() {
        content = content.push(text(error).size(13).color(theme.palette.destructive));
    }

    content = content.push(
        row![
            Space::new().width(Length::Fill),
            Button::text("取消", theme)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .on_press_maybe((!state.deleting).then_some(Message::CloseDeleteGame)),
            Button::text(
                if state.deleting {
                    "移除中…"
                } else {
                    "移除"
                },
                theme
            )
            .variant(ButtonVariant::Destructive)
            .size(ButtonSize::Sm)
            .loading(state.deleting)
            .on_press_maybe((!state.deleting).then_some(Message::ConfirmDeleteGame)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    );

    container(content.spacing(UiTokens::DIALOG_GAP))
        .width(Length::Fixed(UiTokens::DIALOG_MAX_NARROW))
        .padding(UiTokens::DIALOG_PADDING)
        .style(move |_| dialog_surface(theme))
        .into()
}
