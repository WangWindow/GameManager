use gamemanager_core::ThemeMode;
use iced::{
    Element, Length,
    widget::{button, checkbox, column, container, row, text},
};

use crate::{
    message::Message,
    state::{EngineListState, MaintenanceState, PreferencesState},
};

pub fn view<'a>(
    preferences: &'a PreferencesState,
    engines: &'a EngineListState,
    maintenance: &'a MaintenanceState,
) -> Element<'a, Message> {
    let mut content = column![
        row![
            text("设置").size(22).width(Length::Fill),
            button(text("×")).on_press(Message::CloseSettings),
        ],
        text("主题"),
        theme_buttons(preferences.value().theme_mode),
        checkbox(preferences.value().show_status_bar)
            .label("状态栏")
            .on_toggle(Message::StatusBarChanged),
        text("引擎"),
    ]
    .spacing(12)
    .padding(24);

    for entry in engines.entries() {
        let id = entry.id.clone();
        let label = format!(
            "{} · {} · {}",
            entry.name,
            entry.strategy,
            if entry.valid { "有效" } else { "无效" }
        );
        content = content.push(
            checkbox(entry.enabled)
                .label(label)
                .on_toggle(move |enabled| Message::EngineEnabledChanged {
                    id: id.clone(),
                    enabled,
                }),
        );
    }

    content = content.push(text("运行时")).push(
        row![
            button(text("导入 mkxp-z")).on_press(Message::PickMkxpzArchive),
            button(text("获取版本")).on_press(Message::OpenMkxpzBuilds),
        ]
        .spacing(8),
    );
    if maintenance.runtimes().is_empty() {
        content = content.push(text("暂无运行时").size(14));
    } else {
        for runtime in maintenance.runtimes() {
            content = content.push(text(format!(
                "{} {} · {}",
                runtime.name, runtime.version, runtime.engine_type
            )));
        }
    }
    if let Some(error) = maintenance.error.as_deref() {
        content = content.push(text(error).size(14));
    }

    container(content).width(Length::Fixed(560.0)).into()
}

fn theme_buttons(mode: ThemeMode) -> Element<'static, Message> {
    let mut buttons = row![].spacing(6);
    for (label, value) in [
        ("系统", ThemeMode::System),
        ("浅色", ThemeMode::Light),
        ("深色", ThemeMode::Dark),
    ] {
        let label = if value == mode {
            format!("✓ {label}")
        } else {
            label.to_owned()
        };
        buttons = buttons.push(button(text(label)).on_press(Message::ThemeModeChanged(value)));
    }
    buttons.into()
}
