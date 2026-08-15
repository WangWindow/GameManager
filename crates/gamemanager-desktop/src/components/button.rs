use iced::{
    Element, Length,
    widget::{button, text},
};

pub fn action_button<'a, Message: Clone + 'a>(
    label: &'a str,
    message: Message,
) -> Element<'a, Message> {
    button(text(label))
        .padding([6, 12])
        .width(Length::Shrink)
        .on_press(message)
        .into()
}
