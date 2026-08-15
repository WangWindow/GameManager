use iced::{
    Element, Length,
    widget::{button, column, container, row, text},
};

use crate::{message::Message, state::ScanDialogState};

pub fn view(state: &ScanDialogState) -> Element<'_, Message> {
    let root = if state.root.as_os_str().is_empty() {
        "未选择目录".to_owned()
    } else {
        state.root.display().to_string()
    };
    let progress = state.progress.map_or_else(
        || state.label.clone(),
        |value| format!("{} {value}%", state.label),
    );
    let mut content = column![
        row![
            text("扫描游戏").size(22).width(Length::Fill),
            button(text("×")).on_press(Message::CloseScan),
        ],
        text(root).size(14),
        row![
            button(text("选择目录")).on_press(Message::PickScanRoot),
            button(text("扫描")).on_press(Message::SubmitScan),
        ]
        .spacing(8),
        text(progress).size(14),
    ]
    .spacing(14)
    .padding(24);
    if let Some(error) = state.error.as_deref() {
        content = content.push(text(error).size(14));
    }
    container(content).width(Length::Fixed(480.0)).into()
}
