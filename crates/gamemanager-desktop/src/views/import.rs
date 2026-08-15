use iced::{
    Element, Length,
    widget::{button, column, container, row, text},
};

use crate::{message::Message, state::ImportDialogState};

pub fn view(state: &ImportDialogState) -> Element<'_, Message> {
    let path = state.entry_path.as_deref().map_or_else(
        || "未选择入口".to_owned(),
        |path| path.display().to_string(),
    );
    let mut content = column![
        row![
            text("导入游戏").size(22).width(Length::Fill),
            button(text("×")).on_press(Message::CloseImport),
        ],
        text(path).size(14),
        row![
            button(text("选择入口")).on_press(Message::PickImportEntry),
            button(text(if state.submitting {
                "导入中…"
            } else {
                "导入"
            }))
            .on_press_maybe(state.can_submit().then_some(Message::SubmitImport)),
        ]
        .spacing(8),
    ]
    .spacing(14)
    .padding(24);
    if let Some(error) = state.error.as_deref() {
        content = content.push(text(error).size(14));
    }
    container(content).width(Length::Fixed(480.0)).into()
}
