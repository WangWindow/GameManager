use iced::{
    Alignment, Element, Length,
    widget::{Space, column, container, row, text},
};
use iced_shadcn_v2::{
    Button, ButtonSize, ButtonVariant, Input, InputGroup, InputGroupAddon, InputGroupAddonAlign,
    InputSize, Theme,
};

use crate::{
    message::Message,
    state::ImportDialogState,
    ui::{UiTokens, dialog_surface, icons},
};

pub fn view<'a>(state: &'a ImportDialogState, theme: &'a Theme) -> Element<'a, Message> {
    let path = state
        .entry_path
        .as_ref()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default();
    let input = InputGroup::new(theme)
        .push_input(
            Input::new(theme)
                .value(path)
                .placeholder("选择入口文件")
                .size(InputSize::Sm)
                .disabled(state.submitting),
        )
        .push_addon(
            InputGroupAddon::new(
                icons::file().size(16).color(theme.palette.muted_foreground),
                theme,
            )
            .align(InputGroupAddonAlign::InlineStart),
        )
        .width(Length::Fill)
        .height(Length::Fixed(UiTokens::CONTROL_HEIGHT))
        .disabled(state.submitting);

    let close = Button::icon(icons::x(), theme)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::IconSm)
        .on_press_maybe((!state.submitting).then_some(Message::CloseImport));
    let picker = Button::icon(icons::folder_open(), theme)
        .variant(ButtonVariant::Outline)
        .size(ButtonSize::IconSm)
        .on_press_maybe((!state.submitting).then_some(Message::PickImportEntry));
    let submit = Button::text(
        if state.submitting {
            "导入中…"
        } else {
            "导入"
        },
        theme,
    )
    .variant(ButtonVariant::Default)
    .size(ButtonSize::Sm)
    .loading(state.submitting)
    .on_press_maybe(state.can_submit().then_some(Message::SubmitImport));

    let mut content = column![
        row![
            text("导入游戏").size(20).color(theme.palette.foreground),
            container(close)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right),
        ]
        .align_y(Alignment::Center),
        column![
            text("入口文件").size(14).color(theme.palette.foreground),
            row![input, picker].spacing(8).align_y(Alignment::Center),
        ]
        .spacing(8),
    ]
    .spacing(UiTokens::DIALOG_GAP);
    if let Some(error) = state.error.as_deref() {
        content = content.push(text(error).size(13).color(theme.palette.destructive));
    }
    content = content.push(
        row![
            Space::new().width(Length::Fill),
            Button::text("取消", theme)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .on_press_maybe((!state.submitting).then_some(Message::CloseImport)),
            submit,
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    );

    container(content)
        .width(Length::Fixed(UiTokens::DIALOG_MAX_NARROW))
        .padding(UiTokens::DIALOG_PADDING)
        .style(move |_| dialog_surface(theme))
        .into()
}
