use gamemanager_core::{ThemeMode, WindowBackend};
use iced::{
    Alignment, Element, Length,
    widget::{Space, column, container, row, rule, text},
};
use iced_shadcn_v2::{
    Button, ButtonSize, ButtonVariant, Input, InputSize, Select, SelectItem, SelectSize, Switch,
    Theme,
};

use crate::{
    components::form_row,
    message::Message,
    state::{AppearanceDialogState, PreferencesState},
    ui::{UiTokens, card_surface, dialog_surface, icons},
};

pub fn view<'a>(
    preferences: &'a PreferencesState,
    state: &'a AppearanceDialogState,
    theme: &'a Theme,
) -> Element<'a, Message> {
    let disabled = state.is_busy();
    let theme_select = Select::new(theme)
        .item((ThemeMode::System, "系统"))
        .item((ThemeMode::Light, "浅色"))
        .item((ThemeMode::Dark, "深色"))
        .selected(preferences.value().theme_mode)
        .deselectable(false)
        .size(SelectSize::Sm)
        .width(Length::Fixed(112.0))
        .disabled(disabled)
        .on_select(Message::ThemeModeChanged);

    let display_backends = crate::platform::DisplayBackendAvailability::detect();
    let selected_backend = match display_backends.resolve(preferences.value().window_backend) {
        WindowBackend::Auto => None,
        backend => Some(backend),
    };
    let backend_select = Select::new(theme)
        .item(
            SelectItem::new(WindowBackend::Wayland, "Wayland").disabled(!display_backends.wayland),
        )
        .item(SelectItem::new(WindowBackend::X11, "X11").disabled(!display_backends.x11))
        .selected_maybe(selected_backend)
        .placeholder("未检测到可用后端")
        .deselectable(false)
        .size(SelectSize::Sm)
        .width(Length::Fixed(112.0))
        .disabled(disabled)
        .on_select(Message::WindowBackendChanged);

    let root_controls = row![
        Input::new(theme)
            .value(&state.container_root)
            .placeholder("选择容器根目录")
            .size(InputSize::Sm)
            .disabled(disabled)
            .on_input(Message::AppearanceContainerRootChanged),
        Button::icon(icons::folder_open(), theme)
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::IconSm)
            .on_press_maybe((!disabled).then_some(Message::PickContainerRoot)),
        Button::text(
            if state.saving_root {
                "保存中…"
            } else {
                "保存"
            },
            theme
        )
        .size(ButtonSize::Sm)
        .loading(state.saving_root)
        .on_press_maybe((!disabled).then_some(Message::SaveContainerRoot)),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let removal: Element<'a, Message> = if state.confirm_remove_all {
        container(
            column![
                text("将移除游戏库中的全部记录，游戏文件和容器不会被删除。")
                    .size(12)
                    .color(theme.palette.destructive),
                row![
                    Space::new().width(Length::Fill),
                    Button::text("取消", theme)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .on_press_maybe((!disabled).then_some(Message::CancelRemoveAllGames)),
                    Button::text(
                        if state.removing_games {
                            "移除中…"
                        } else {
                            "确认移除"
                        },
                        theme
                    )
                    .variant(ButtonVariant::Destructive)
                    .size(ButtonSize::Sm)
                    .loading(state.removing_games)
                    .on_press_maybe((!disabled).then_some(Message::ConfirmRemoveAllGames)),
                ]
                .spacing(8),
            ]
            .spacing(10),
        )
        .padding(10)
        .style(move |_| card_surface(theme))
        .into()
    } else {
        row![
            Button::text("清理容器", theme)
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::Sm)
                .loading(state.cleaning_profiles)
                .on_press_maybe((!disabled).then_some(Message::CleanupUnusedProfiles)),
            Space::new().width(Length::Fill),
            Button::text("移除所有游戏", theme)
                .variant(ButtonVariant::Destructive)
                .size(ButtonSize::Sm)
                .on_press_maybe((!disabled).then_some(Message::RequestRemoveAllGames)),
        ]
        .align_y(Alignment::Center)
        .into()
    };

    let mut body = column![
        form_row(
            text("主题").size(14).color(theme.palette.muted_foreground),
            theme_select,
        ),
        form_row(
            text("窗口后端")
                .size(14)
                .color(theme.palette.muted_foreground),
            backend_select,
        ),
        form_row(
            text("显示状态栏")
                .size(14)
                .color(theme.palette.muted_foreground),
            Switch::new(theme)
                .checked(preferences.value().show_status_bar)
                .disabled(disabled)
                .on_toggle(Message::StatusBarChanged),
        ),
        form_row(
            text("记住窗口大小")
                .size(14)
                .color(theme.palette.muted_foreground),
            Switch::new(theme)
                .checked(preferences.value().remember_window_size)
                .disabled(disabled)
                .on_toggle(Message::RememberWindowSizeChanged),
        ),
        rule::horizontal(1),
        form_row(
            text("容器根目录")
                .size(14)
                .color(theme.palette.muted_foreground),
            root_controls,
        ),
        rule::horizontal(1),
        removal,
    ]
    .spacing(12);
    if let Some(error) = state.error.as_deref() {
        body = body.push(text(error).size(13).color(theme.palette.destructive));
    }

    container(
        column![
            dialog_header(
                "设置",
                (!disabled).then_some(Message::CloseUtilityDialog),
                theme,
            ),
            body,
        ]
        .spacing(UiTokens::DIALOG_GAP),
    )
    .width(Length::Fixed(UiTokens::DIALOG_MAX_NARROW))
    .padding(UiTokens::DIALOG_PADDING)
    .style(move |_| dialog_surface(theme))
    .into()
}

fn dialog_header<'a>(
    title: &'a str,
    close: Option<Message>,
    theme: &'a Theme,
) -> Element<'a, Message> {
    row![
        text(title).size(20).color(theme.palette.foreground),
        container(
            Button::icon(icons::x(), theme)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .on_press_maybe(close),
        )
        .width(Length::Fill)
        .align_x(iced::alignment::Horizontal::Right),
    ]
    .align_y(Alignment::Center)
    .into()
}
