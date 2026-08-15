use gamemanager_core::Runner;
use iced::{
    Element, Length,
    widget::{button, checkbox, column, container, row, text, text_input},
};

use crate::{message::Message, state::GameSettingsState};

pub fn view(state: &GameSettingsState) -> Element<'_, Message> {
    let mut content = column![
        row![
            text("游戏设置").size(22).width(Length::Fill),
            button(text("×")).on_press(Message::CloseGameSettings),
        ],
        text_input("名称", &state.title)
            .on_input(Message::GameSettingsTitleChanged)
            .padding([8, 12]),
        text_input("入口文件", &state.entry_path)
            .on_input(Message::GameSettingsEntryChanged)
            .padding([8, 12]),
    ]
    .spacing(12)
    .padding(24);

    let mut runners = row![].spacing(6);
    for runner in state.runner_choices(true, true, true) {
        let label = runner.as_str();
        let button = button(text(label)).on_press(Message::GameSettingsRunnerChanged(runner));
        runners = runners.push(button);
    }
    content = content.push(text("启动方式")).push(runners);

    if state.runner == Runner::Bottles {
        content = content.push(
            text_input("Bottle", state.bottle_name.as_deref().unwrap_or_default())
                .on_input(Message::GameSettingsBottleChanged)
                .padding([8, 12]),
        );
    }
    if state.shows_sandbox_home() {
        content = content.push(
            checkbox(state.sandbox_home)
                .label("沙盒主目录")
                .on_toggle(Message::GameSettingsSandboxChanged),
        );
    }

    if let Some(error) = state.error.as_deref() {
        content = content.push(text(error).size(14));
    }
    content = content.push(
        row![
            button(text("取消")).on_press(Message::CloseGameSettings),
            button(text(if state.saving {
                "保存中…"
            } else {
                "保存"
            }))
            .on_press_maybe((!state.saving).then_some(Message::SaveGameSettings)),
        ]
        .spacing(8),
    );

    container(content).width(Length::Fixed(560.0)).into()
}
