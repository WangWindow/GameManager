use iced::{
    Alignment, Background, Border, Element, Length, Padding,
    alignment::{Horizontal, Vertical},
    widget::{Space, container, mouse_area, row, text},
};
use iced_shadcn_v2::{Button, ButtonSize, ButtonVariant, DropdownMenu, DropdownMenuItem, Theme};

use crate::{
    message::{Message, WindowAction, WindowMessage},
    state::UtilityDialog,
    ui::{UiTokens, brand_icon_surface, icons},
};

pub fn view(menu_open: bool, theme: &Theme) -> Element<'_, Message> {
    let brand = row![
        container(
            icons::gamepad_2()
                .size(UiTokens::BRAND_ICON_SIZE)
                .color(theme.palette.primary_foreground),
        )
        .width(Length::Fixed(UiTokens::BRAND_ICON_BOX_WIDTH))
        .height(Length::Fixed(UiTokens::BRAND_ICON_BOX_HEIGHT))
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .style(move |_| brand_icon_surface(theme)),
        text("GameManager")
            .size(UiTokens::TITLE_TEXT_SIZE)
            .color(theme.palette.foreground),
    ]
    .align_y(Alignment::Center)
    .spacing(8)
    .width(Length::Shrink);

    let drag_region = mouse_area(
        container(Space::new().width(Length::Fill).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(Message::Window(WindowMessage::Action(WindowAction::Drag)));

    // `DropdownMenu` owns the trigger interaction. A shadcn `Button` without
    // an `on_press` message is treated as disabled by the button component,
    // which leaves its disabled surface painted even while the menu is closed.
    // Use a plain surface as the trigger so its resting state is transparent
    // and only the explicitly open state gets the muted background.
    let overflow_trigger = container(
        icons::ellipsis_vertical()
            .size(18)
            .color(theme.palette.foreground),
    )
    .width(Length::Fixed(UiTokens::TITLE_ACTION_WIDTH))
    .height(Length::Fixed(UiTokens::TITLE_ACTION_HEIGHT))
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
    .style(move |_| iced::widget::container::Style {
        background: menu_open.then_some(Background::Color(theme.palette.muted)),
        text_color: Some(theme.palette.foreground),
        border: Border {
            radius: 8.0.into(),
            ..Border::default()
        },
        ..Default::default()
    });

    let overflow_menu = DropdownMenu::new(theme)
        .trigger(overflow_trigger)
        .width(UiTokens::TITLE_MENU_WIDTH)
        .open(menu_open)
        .on_open(Message::OpenAppMenu)
        .on_close(Message::DismissAppMenu)
        .item(
            DropdownMenuItem::new("运行时")
                .on_select(Message::OpenUtilityDialog(UtilityDialog::Runtime)),
        )
        .item(
            DropdownMenuItem::new("引擎")
                .on_select(Message::OpenUtilityDialog(UtilityDialog::Engines)),
        )
        .separator()
        .item(
            DropdownMenuItem::new("设置")
                .on_select(Message::OpenUtilityDialog(UtilityDialog::Appearance)),
        );

    let window_action = |icon, action| {
        Button::icon(icon, theme)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .width(Length::Fixed(UiTokens::WINDOW_ACTION_WIDTH))
            .height(Length::Fixed(UiTokens::TITLE_ACTION_HEIGHT))
            .on_press(Message::Window(WindowMessage::Action(action)))
    };

    let actions = row![
        Button::new(
            row![
                icons::plus().size(17),
                text("导入").size(UiTokens::TITLE_ACTION_TEXT_SIZE)
            ]
            .spacing(6)
            .align_y(Alignment::Center),
            theme
        )
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .height(Length::Fixed(UiTokens::TITLE_ACTION_HEIGHT))
        .on_press(Message::OpenImport),
        Button::new(
            row![
                icons::folder_search().size(17),
                text("扫描").size(UiTokens::TITLE_ACTION_TEXT_SIZE)
            ]
            .spacing(6)
            .align_y(Alignment::Center),
            theme
        )
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Sm)
        .height(Length::Fixed(UiTokens::TITLE_ACTION_HEIGHT))
        .on_press(Message::OpenScan),
        // The menu is positioned from the trigger's left edge. Reserve enough
        // room after the trigger for its fixed-width surface, otherwise the
        // overlay reaches the viewport edge and its right border is clipped.
        container(overflow_menu).padding(Padding {
            top: 0.0,
            right: UiTokens::TITLE_MENU_RIGHT_PADDING,
            bottom: 0.0,
            left: 0.0,
        }),
        window_action(icons::minus(), WindowAction::Minimize),
        window_action(icons::maximize(), WindowAction::ToggleMaximize),
        window_action(icons::x(), WindowAction::Close),
    ]
    .align_y(Alignment::Center)
    .spacing(2);

    container(row![brand, drag_region, actions].align_y(Alignment::Center))
        .width(Length::Fill)
        .height(Length::Fixed(UiTokens::TITLE_BAR_HEIGHT))
        .padding([0.0, UiTokens::TITLE_BAR_PADDING_X])
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme.palette.background)),
            border: iced::Border {
                color: theme.palette.border,
                width: UiTokens::WINDOW_BORDER,
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}
