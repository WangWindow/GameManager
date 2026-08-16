use iced::{
    Alignment, Element, Length,
    widget::{Space, column, container, progress_bar, row, text},
};
use iced_shadcn_v2::{
    Button, ButtonSize, ButtonVariant, Input, InputGroup, InputGroupAddon, InputGroupAddonAlign,
    InputSize, Theme,
};

use crate::{
    message::Message,
    state::ScanDialogState,
    ui::{UiTokens, dialog_surface, icons},
};

pub fn view<'a>(state: &'a ScanDialogState, theme: &'a Theme) -> Element<'a, Message> {
    let scanning = state.operation_id.is_some();
    let root = if state.root.as_os_str().is_empty() {
        String::new()
    } else {
        state.root.to_string_lossy().into_owned()
    };
    let root_input = InputGroup::new(theme)
        .push_input(
            Input::new(theme)
                .value(root)
                .placeholder("选择扫描根目录")
                .size(InputSize::Sm)
                .disabled(scanning),
        )
        .push_addon(
            InputGroupAddon::new(
                icons::folder()
                    .size(16)
                    .color(theme.palette.muted_foreground),
                theme,
            )
            .align(InputGroupAddonAlign::InlineStart),
        )
        .width(Length::Fill)
        .height(Length::Fixed(UiTokens::CONTROL_HEIGHT))
        .disabled(scanning);

    let depth_input = Input::new(theme)
        .value(state.max_depth.to_string())
        .size(InputSize::Sm)
        .disabled(scanning)
        .on_input(Message::ScanDepthChanged)
        .width(Length::Fill);
    let minus = Button::icon(icons::minus(), theme)
        .variant(ButtonVariant::Outline)
        .size(ButtonSize::IconSm)
        .width(Length::Fixed(UiTokens::CARD_ACTION_SIZE))
        .height(Length::Fixed(UiTokens::CARD_ACTION_SIZE))
        .on_press_maybe((!scanning).then_some(Message::ScanDepthAdjusted(-1)));
    let plus = Button::icon(icons::plus(), theme)
        .variant(ButtonVariant::Outline)
        .size(ButtonSize::IconSm)
        .width(Length::Fixed(UiTokens::CARD_ACTION_SIZE))
        .height(Length::Fixed(UiTokens::CARD_ACTION_SIZE))
        .on_press_maybe((!scanning).then_some(Message::ScanDepthAdjusted(1)));

    let footer: Element<'a, Message> = if scanning {
        row![
            Space::new().width(Length::Fill),
            Button::text("正在扫描…", theme)
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .disabled(true),
        ]
        .align_y(Alignment::Center)
        .into()
    } else {
        row![
            Space::new().width(Length::Fill),
            Button::text("取消", theme)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .on_press(Message::CloseScan),
            Button::text("开始扫描", theme)
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .on_press_maybe(state.can_submit().then_some(Message::SubmitScan)),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    };

    let mut content = column![
        row![
            text("扫描游戏").size(20).color(theme.palette.foreground),
            container(
                Button::icon(icons::x(), theme)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .on_press_maybe((!scanning).then_some(Message::CloseScan)),
            )
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right),
        ]
        .align_y(Alignment::Center),
        text("扫描目录中的可识别游戏并自动加入库。")
            .size(13)
            .color(theme.palette.muted_foreground),
        column![
            text("扫描根目录").size(14).color(theme.palette.foreground),
            row![
                root_input,
                Button::icon(icons::folder_open(), theme)
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::IconSm)
                    .on_press_maybe((!scanning).then_some(Message::PickScanRoot)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(8),
        column![
            text("最大扫描深度")
                .size(14)
                .color(theme.palette.foreground),
            row![depth_input, minus, plus]
                .spacing(8)
                .align_y(Alignment::Center),
        ]
        .spacing(8),
    ]
    .spacing(UiTokens::DIALOG_GAP);
    if let Some(value) = state.progress {
        content = content.push(
            column![
                text(if state.label.is_empty() {
                    "正在扫描…"
                } else {
                    state.label.as_str()
                })
                .size(13)
                .color(theme.palette.muted_foreground),
                progress_bar(0.0..=100.0, f32::from(value)),
            ]
            .spacing(6),
        );
    }
    if let Some(error) = state.error.as_deref() {
        content = content.push(text(error).size(13).color(theme.palette.destructive));
    }
    content = content.push(footer);

    container(content)
        .width(Length::Fixed(UiTokens::DIALOG_MAX_WIDE))
        .padding(UiTokens::DIALOG_PADDING)
        .style(move |_| dialog_surface(theme))
        .into()
}
