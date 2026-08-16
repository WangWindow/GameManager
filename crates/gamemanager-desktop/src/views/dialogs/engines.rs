use iced::{
    Alignment, Element, Length,
    widget::{column, container, responsive, row, rule, scrollable, text},
};
use iced_shadcn_v2::{Button, ButtonSize, ButtonVariant, Switch, Theme};

use crate::{
    message::Message,
    state::{EngineListState, EngineRow},
    ui::{UiTokens, card_surface, dialog_surface, icons},
};

pub fn view<'a>(engines: &'a EngineListState, theme: &'a Theme) -> Element<'a, Message> {
    responsive(move |viewport| {
        container(dialog(engines, theme, viewport.height))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    })
    .into()
}

fn dialog<'a>(
    engines: &'a EngineListState,
    theme: &'a Theme,
    viewport_height: f32,
) -> Element<'a, Message> {
    let mut entries = column![].spacing(8);
    if engines.entries().is_empty() {
        entries = entries.push(
            text("未找到引擎规则")
                .size(13)
                .color(theme.palette.muted_foreground),
        );
    } else {
        for engine in engines.entries() {
            entries = entries.push(engine_entry(engine, engines.is_expanded(&engine.id), theme));
        }
    }

    let natural_height = engines.entries().len().max(1) as f32 * 64.0;
    let list_height = natural_height.min((viewport_height * 0.6).clamp(180.0, 520.0));

    container(
        column![
            dialog_header("引擎插件", Message::CloseUtilityDialog, theme),
            scrollable(entries).height(Length::Fixed(list_height)),
        ]
        .spacing(UiTokens::DIALOG_GAP),
    )
    .width(Length::Fixed(UiTokens::DIALOG_MAX_NARROW))
    .padding(UiTokens::DIALOG_PADDING)
    .style(move |_| dialog_surface(theme))
    .into()
}

fn engine_entry<'a>(
    engine: &'a EngineRow,
    expanded: bool,
    theme: &'a Theme,
) -> Element<'a, Message> {
    let engine_id = engine.id.clone();
    let toggle_id = engine.id.clone();
    let chevron = if expanded {
        icons::chevron_up()
    } else {
        icons::chevron_down()
    };
    let details = column![
        text(engine.name.as_str())
            .size(14)
            .color(theme.palette.foreground),
        text(format!(
            "{} 条规则 · {}",
            engine.rule_count, engine.strategy
        ))
        .size(12)
        .color(theme.palette.muted_foreground),
    ]
    .spacing(3)
    .width(Length::Fill);
    let header = Button::new(
        row![
            icons::engine(engine.id.as_str())
                .size(17)
                .color(theme.palette.muted_foreground),
            details,
            chevron.size(16).color(theme.palette.muted_foreground),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        theme,
    )
    .variant(ButtonVariant::Ghost)
    .size(ButtonSize::Sm)
    .width(Length::Fill)
    .on_press(Message::ToggleEngineExpanded(toggle_id));

    let mut body = column![
        row![
            header,
            Switch::new(theme)
                .checked(engine.enabled)
                .disabled(!engine.valid)
                .on_toggle(move |enabled| Message::EngineEnabledChanged {
                    id: engine_id.clone(),
                    enabled,
                }),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    ]
    .spacing(10);

    if expanded {
        body = body.push(expanded_details(engine, theme));
    } else if !engine.valid {
        body = body.push(
            text(engine.errors.join("；"))
                .size(12)
                .color(theme.palette.destructive),
        );
    }

    container(body)
        .width(Length::Fill)
        .padding([10, 12])
        .style(move |_| card_surface(theme))
        .into()
}

fn expanded_details<'a>(engine: &'a EngineRow, theme: &'a Theme) -> Element<'a, Message> {
    let mut rules = column![
        text(format!("检测规则（最低 {} 分）", engine.minimum_score))
            .size(13)
            .color(theme.palette.foreground),
    ]
    .spacing(6);
    for detection_rule in &engine.rules {
        let weight = detection_rule
            .weight
            .map_or_else(String::new, |weight| format!("+{weight}"));
        rules = rules.push(
            row![
                tag(detection_rule.requirement.label(), theme),
                tag(detection_rule.rule_type.as_str(), theme),
                container(
                    text(detection_rule.target.as_str())
                        .size(12)
                        .color(theme.palette.muted_foreground)
                        .wrapping(text::Wrapping::None),
                )
                .width(Length::Fill)
                .clip(true),
                text(weight).size(12).color(theme.palette.muted_foreground),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        );
    }

    let mut launch = column![
        text("启动配置").size(13).color(theme.palette.foreground),
        value_line("策略", engine.strategy.as_str(), theme),
    ]
    .spacing(6);
    if !engine.entry_patterns.is_empty() {
        let entry_patterns = engine.entry_patterns.join(", ");
        launch = launch.push(value_line("入口", &entry_patterns, theme));
    }
    if !engine.exclude_patterns.is_empty() {
        let exclude_patterns = engine.exclude_patterns.join(", ");
        launch = launch.push(value_line("排除", &exclude_patterns, theme));
    }

    let mut content = column![rule::horizontal(1), rules, launch].spacing(10);
    if !engine.errors.is_empty() {
        content = content.push(
            text(engine.errors.join("；"))
                .size(12)
                .color(theme.palette.destructive),
        );
    }

    container(content).padding([0, 4]).into()
}

fn tag<'a>(label: &'a str, theme: &'a Theme) -> Element<'a, Message> {
    container(text(label).size(10).color(theme.palette.muted_foreground))
        .padding([2, 4])
        .style(move |_| card_surface(theme))
        .into()
}

fn value_line<'a>(
    label: impl Into<String>,
    value: impl Into<String>,
    theme: &'a Theme,
) -> Element<'a, Message> {
    row![
        text(format!("{}:", label.into()))
            .size(12)
            .color(theme.palette.muted_foreground),
        container(
            text(value.into())
                .size(12)
                .color(theme.palette.muted_foreground)
                .wrapping(text::Wrapping::None),
        )
        .width(Length::Fill)
        .clip(true),
    ]
    .spacing(6)
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
