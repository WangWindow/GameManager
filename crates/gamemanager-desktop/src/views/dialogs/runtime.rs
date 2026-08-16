use iced::{
    Alignment, Element, Length,
    widget::{column, container, row, rule, text},
};
use iced_shadcn_v2::{
    Button, ButtonSize, ButtonVariant, Select, SelectSelection, SelectSize, Switch, Theme,
};

use crate::{
    components::form_row,
    message::Message,
    state::MaintenanceState,
    ui::{UiTokens, card_surface, dialog_surface, icons},
};

pub fn view<'a>(maintenance: &'a MaintenanceState, theme: &'a Theme) -> Element<'a, Message> {
    let nwjs = maintenance.runtime("nwjs");
    let mkxpz = maintenance.runtime("mkxpz");
    let nwjs_action = if nwjs.is_some() { "更新" } else { "下载" };

    let mut body = column![
        row![
            text("运行时").size(14).color(theme.palette.foreground),
            container(
                Button::icon(icons::refresh_cw(), theme)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconXs)
                    .on_press(Message::RefreshRuntimes),
            )
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right),
        ]
        .align_y(Alignment::Center),
        runtime_row(
            "NW.js",
            nwjs.map(|runtime| runtime.version.as_str()),
            icons::monitor(),
            Button::text(nwjs_action, theme)
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::Sm)
                .loading(maintenance.runtime_loading())
                .on_press_maybe((!maintenance.runtime_loading()).then_some(Message::DownloadNwjs)),
            theme,
        ),
        runtime_row(
            "mkxp-z",
            mkxpz.map(|runtime| runtime.version.as_str()),
            icons::gamepad_2(),
            row![
                Button::text("导入", theme)
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Sm)
                    .loading(maintenance.runtime_loading())
                    .on_press_maybe(
                        (!maintenance.runtime_loading()).then_some(Message::PickMkxpzArchive)
                    ),
                Button::text("构建页", theme)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .on_press_maybe(
                        (!maintenance.runtime_loading()).then_some(Message::OpenMkxpzBuilds)
                    ),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
            theme,
        ),
    ]
    .spacing(12);

    if maintenance.bottles_available() {
        let bottle_select = maintenance.bottles().iter().fold(
            Select::<String, Message>::new(theme)
                .size(SelectSize::Sm)
                .width(Length::Fixed(180.0))
                .placeholder("选择 Bottle"),
            |select, bottle| select.item((bottle.clone(), bottle.clone())),
        );
        let bottle_select = bottle_select
            .selected_maybe(maintenance.bottles_default().map(ToOwned::to_owned))
            .disabled(!maintenance.can_select_bottles())
            .on_selection_change(|selection: SelectSelection<String>| {
                Message::BottlesDefaultSelected(selection.as_single().cloned())
            });
        body = body
            .push(rule::horizontal(1))
            .push(
                row![
                    text("Bottles").size(14).color(theme.palette.foreground),
                    container(
                        Button::icon(icons::refresh_cw(), theme)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconXs)
                            .loading(maintenance.bottles_loading())
                            .on_press_maybe(
                                (!maintenance.bottles_loading()).then_some(Message::RefreshBottles),
                            ),
                    )
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Right),
                ]
                .align_y(Alignment::Center),
            )
            .push(form_row(
                text("启用 Bottles")
                    .size(14)
                    .color(theme.palette.muted_foreground),
                Switch::new(theme)
                    .checked(maintenance.bottles_enabled())
                    .on_toggle(Message::BottlesEnabledChanged),
            ))
            .push(form_row(
                text("默认 bottle")
                    .size(14)
                    .color(theme.palette.muted_foreground),
                bottle_select,
            ));
    }

    if let Some(error) = maintenance.error.as_deref() {
        body = body.push(text(error).size(13).color(theme.palette.destructive));
    }

    container(
        column![
            dialog_header("运行时", Message::CloseUtilityDialog, theme),
            body,
        ]
        .spacing(UiTokens::DIALOG_GAP),
    )
    .width(Length::Fixed(UiTokens::DIALOG_MAX_NARROW))
    .padding(UiTokens::DIALOG_PADDING)
    .style(move |_| dialog_surface(theme))
    .into()
}

fn runtime_row<'a>(
    name: &'a str,
    version: Option<&'a str>,
    icon: icons::IconText<'a>,
    action: impl Into<Element<'a, Message>>,
    theme: &'a Theme,
) -> Element<'a, Message> {
    container(
        row![
            icon.size(17).color(theme.palette.muted_foreground),
            column![
                text(name).size(14).color(theme.palette.foreground),
                text(version.unwrap_or("未安装"))
                    .size(12)
                    .color(theme.palette.muted_foreground),
            ]
            .spacing(3)
            .width(Length::Fill),
            action.into(),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding([10, 12])
    .style(move |_| card_surface(theme))
    .into()
}

fn dialog_header<'a>(title: &'a str, close: Message, theme: &'a Theme) -> Element<'a, Message> {
    row![
        text(title).size(20).color(theme.palette.foreground),
        container(
            Button::icon(icons::x(), theme)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .on_press(close),
        )
        .width(Length::Fill)
        .align_x(iced::alignment::Horizontal::Right),
    ]
    .align_y(Alignment::Center)
    .into()
}
