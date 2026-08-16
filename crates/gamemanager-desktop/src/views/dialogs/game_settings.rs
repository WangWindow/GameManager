use gamemanager_core::Runner;
use iced::{
    Alignment, Element, Length,
    widget::{Space, column, container, responsive, row, rule, scrollable, text},
};
use iced_shadcn_v2::{
    Button, ButtonSize, ButtonVariant, Input, InputSize, Select, SelectSelection, SelectSize,
    Switch, Theme,
};

use crate::{
    components::form_row,
    message::Message,
    state::{EngineListState, GameSettingsState, MaintenanceState},
    ui::{UiTokens, dialog_surface, icons},
};

pub fn view<'a>(
    state: &'a GameSettingsState,
    engines: &'a EngineListState,
    maintenance: &'a MaintenanceState,
    theme: &'a Theme,
) -> Element<'a, Message> {
    responsive(move |viewport| {
        let body_height = body_height(viewport.height, state.natural_body_height());
        container(dialog(state, engines, maintenance, theme, body_height))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    })
    .into()
}

pub fn body_height(viewport_height: f32, natural_height: f32) -> f32 {
    natural_height.min((viewport_height * 0.6).clamp(240.0, 560.0))
}

fn dialog<'a>(
    state: &'a GameSettingsState,
    engines: &'a EngineListState,
    maintenance: &'a MaintenanceState,
    theme: &'a Theme,
    body_height: f32,
) -> Element<'a, Message> {
    let disabled = state.saving;
    let engine_select = engines.entries().iter().fold(
        Select::new(theme).size(SelectSize::Sm).width(Length::Fill),
        |select, engine| select.item((engine.id.clone(), engine.name.clone())),
    );
    let engine_select = engine_select
        .selected(state.engine_type.clone())
        .deselectable(false)
        .disabled(disabled)
        .on_select(Message::GameSettingsEngineChanged);

    let runner_select = state.runner_choices(
        maintenance.bottles_enabled() && maintenance.bottles_available(),
        maintenance.nwjs_available(),
        maintenance.mkxpz_available(),
    );
    let runner_select = runner_select.into_iter().fold(
        Select::new(theme).size(SelectSize::Sm).width(Length::Fill),
        |select, runner| select.item((runner, runner_label(runner))),
    );
    let runner_select = runner_select
        .selected(state.runner)
        .deselectable(false)
        .disabled(disabled)
        .on_select(Message::GameSettingsRunnerChanged);

    let nwjs_versions = GameSettingsState::nwjs_versions(maintenance.runtimes());
    let nwjs_select = nwjs_versions.iter().fold(
        Select::<String, Message>::new(theme)
            .size(SelectSize::Sm)
            .width(Length::Fill)
            .placeholder("使用最新已安装版本"),
        |select, version| select.item((version.clone(), version.clone())),
    );
    let nwjs_select = nwjs_select
        .selected_maybe(state.runtime_version.clone())
        .placeholder("未安装 NW.js 运行时")
        .disabled(disabled || nwjs_versions.is_empty())
        .on_selection_change(|selection| {
            Message::GameSettingsRuntimeVersionSelected(selection.as_single().cloned())
        });

    let bottle_select = maintenance.bottles().iter().fold(
        Select::<String, Message>::new(theme)
            .size(SelectSize::Sm)
            .width(Length::Fill)
            .placeholder("选择 Bottle"),
        |select, bottle| select.item((bottle.clone(), bottle.clone())),
    );
    let bottle_select = bottle_select
        .selected_maybe(state.bottle_name.clone())
        .disabled(disabled || !maintenance.can_select_bottles())
        .on_selection_change(|selection: SelectSelection<String>| {
            Message::GameSettingsBottleSelected(selection.as_single().cloned())
        });

    let mut fields = column![
        form_row(
            text("游戏名称")
                .size(14)
                .color(theme.palette.muted_foreground),
            Input::new(theme)
                .value(&state.title)
                .size(InputSize::Sm)
                .disabled(disabled)
                .on_input(Message::GameSettingsTitleChanged),
        ),
        form_row(
            text("引擎类型")
                .size(14)
                .color(theme.palette.muted_foreground),
            engine_select,
        ),
        form_row(
            text("游戏路径")
                .size(14)
                .color(theme.palette.muted_foreground),
            Input::new(theme)
                .value(&state.game_path)
                .size(InputSize::Sm)
                .disabled(true),
        ),
        form_row(
            text("打开目录")
                .size(14)
                .color(theme.palette.muted_foreground),
            row![
                Button::text("游戏目录", theme)
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Xs)
                    .on_press_maybe((!disabled).then_some(Message::OpenGameSettingsDirectory)),
                Button::text("Profile 目录", theme)
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Xs)
                    .on_press_maybe((!disabled).then_some(Message::OpenGameProfileDirectory)),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        ),
        form_row(
            text("图标/封面")
                .size(14)
                .color(theme.palette.muted_foreground),
            row![
                Input::new(theme)
                    .value(state.cover_file.as_deref().unwrap_or_default())
                    .placeholder("可选")
                    .size(InputSize::Sm)
                    .disabled(disabled)
                    .on_input(Message::GameSettingsCoverChanged),
                Button::icon(icons::folder_open(), theme)
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::IconSm)
                    .on_press_maybe((!disabled).then_some(Message::PickGameSettingsCover)),
                Button::icon(
                    if state.refreshing_cover {
                        icons::loader_circle()
                    } else {
                        icons::refresh_cw()
                    },
                    theme,
                )
                .variant(ButtonVariant::Outline)
                .size(ButtonSize::IconSm)
                .loading(state.refreshing_cover)
                .on_press_maybe(
                    (!disabled && !state.refreshing_cover).then_some(Message::RefreshGameCover),
                ),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        ),
        rule::horizontal(1),
        form_row(
            text("入口文件/目录")
                .size(14)
                .color(theme.palette.muted_foreground),
            row![
                Input::new(theme)
                    .value(&state.entry_path)
                    .placeholder("选择入口文件或目录")
                    .size(InputSize::Sm)
                    .disabled(disabled)
                    .on_input(Message::GameSettingsEntryChanged),
                Button::icon(icons::file(), theme)
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::IconSm)
                    .on_press_maybe((!disabled).then_some(Message::PickGameSettingsEntryFile)),
                Button::icon(icons::folder(), theme)
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::IconSm)
                    .on_press_maybe((!disabled).then_some(Message::PickGameSettingsEntryDirectory)),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        ),
        form_row(
            text("启动方式")
                .size(14)
                .color(theme.palette.muted_foreground),
            runner_select,
        ),
    ]
    .spacing(12);

    if state.runner == Runner::Nwjs {
        fields = fields.push(form_row(
            text("NW.js 版本")
                .size(14)
                .color(theme.palette.muted_foreground),
            nwjs_select,
        ));
    }

    if state.runner == Runner::Bottles {
        let mut bottle_body = column![bottle_select].spacing(4);
        if let Some(status) = bottles_status(maintenance, theme) {
            bottle_body = bottle_body.push(status);
        }
        fields = fields.push(form_row(
            text("Bottle")
                .size(14)
                .color(theme.palette.muted_foreground),
            bottle_body,
        ));
    }

    fields = fields.push(form_row(
        text("启动参数")
            .size(14)
            .color(theme.palette.muted_foreground),
        Input::new(theme)
            .value(state.arguments_text())
            .placeholder("--debug")
            .size(InputSize::Sm)
            .disabled(disabled)
            .on_input(Message::GameSettingsArgumentsChanged),
    ));

    if state.shows_sandbox_home() {
        fields = fields.push(form_row(
            text("沙盒主目录")
                .size(14)
                .color(theme.palette.muted_foreground),
            Switch::new(theme)
                .checked(state.sandbox_home)
                .disabled(disabled)
                .on_toggle(Message::GameSettingsSandboxChanged),
        ));
    }

    let footer = row![
        Space::new().width(Length::Fill),
        Button::text("取消", theme)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .on_press_maybe((!state.saving).then_some(Message::CloseGameSettings)),
        Button::text(
            if state.saving {
                "保存中…"
            } else {
                "保存"
            },
            theme
        )
        .variant(ButtonVariant::Default)
        .size(ButtonSize::Sm)
        .loading(state.saving)
        .on_press_maybe((!state.saving).then_some(Message::SaveGameSettings)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let mut content = column![
        row![
            text("游戏设置").size(20).color(theme.palette.foreground),
            container(
                Button::icon(icons::x(), theme)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .on_press_maybe((!state.saving).then_some(Message::CloseGameSettings)),
            )
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right),
        ]
        .align_y(Alignment::Center),
        scrollable(fields).height(Length::Fixed(body_height)),
    ]
    .spacing(UiTokens::DIALOG_GAP);
    if let Some(error) = state.error.as_deref() {
        content = content.push(text(error).size(13).color(theme.palette.destructive));
    }
    content = content.push(footer);

    container(content)
        .width(Length::Fixed(UiTokens::DIALOG_MAX_NARROW))
        .padding(UiTokens::DIALOG_PADDING)
        .style(move |_| dialog_surface(theme))
        .into()
}

fn bottles_status<'a>(
    maintenance: &'a MaintenanceState,
    theme: &'a Theme,
) -> Option<Element<'a, Message>> {
    let message = if maintenance.bottles_loading() {
        Some("正在读取 Bottle…")
    } else if let Some(error) = maintenance.bottles_error() {
        return Some(text(error).size(12).color(theme.palette.destructive).into());
    } else if !maintenance.bottles_enabled() {
        Some("Bottles 未启用")
    } else if !maintenance.bottles_available() {
        Some("未检测到 Bottles")
    } else if maintenance.bottles().is_empty() {
        Some("未找到可用 Bottle")
    } else {
        None
    };

    message.map(|message| {
        text(message)
            .size(12)
            .color(theme.palette.muted_foreground)
            .into()
    })
}

fn runner_label(runner: Runner) -> &'static str {
    match runner {
        Runner::Native => "Linux 原生",
        Runner::Bottles => "Bottles",
        Runner::Nwjs => "NW.js",
        Runner::Mkxpz => "mkxp-z",
        Runner::External => "外部命令",
        Runner::Auto => "自动",
    }
}
