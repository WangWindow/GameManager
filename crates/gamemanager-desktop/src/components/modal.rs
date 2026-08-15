use iced::{
    Element, Length,
    widget::{column, container, opaque, stack},
};

pub struct Modal<'a, Message> {
    content: Element<'a, Message>,
}

impl<'a, Message: 'a> Modal<'a, Message> {
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
        }
    }

    pub fn overlay(self, base: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
        stack![
            base.into(),
            opaque(
                container(column![self.content])
                    .width(Length::Fill)
                    .height(Length::Fill)
            )
        ]
        .into()
    }
}
