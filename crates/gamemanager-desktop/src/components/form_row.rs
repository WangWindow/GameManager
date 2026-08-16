use iced::{
    Alignment, Element, Length,
    widget::{container, row},
};

pub fn form_row<'a, Message: 'a>(
    label: impl Into<Element<'a, Message>>,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    row![
        container(label).width(Length::Fixed(100.0)),
        container(control).width(Length::Fill),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}
