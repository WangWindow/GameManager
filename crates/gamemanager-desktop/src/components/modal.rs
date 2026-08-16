use iced::{
    Background, Color, Element, Length,
    widget::{Space, container, mouse_area, opaque, stack},
};

pub struct Modal<'a, Message> {
    content: Element<'a, Message>,
}

impl<'a, Message: Clone + 'a> Modal<'a, Message> {
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
        }
    }

    pub fn overlay(
        self,
        base: impl Into<Element<'a, Message>>,
        dismiss: Option<Message>,
    ) -> Element<'a, Message> {
        let backdrop = container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.4))),
                ..Default::default()
            });
        let backdrop: Element<'a, Message> = match dismiss {
            Some(message) => opaque(mouse_area(backdrop).on_press(message)),
            None => opaque(backdrop),
        };

        stack![
            base.into(),
            backdrop,
            // Keep the content surface opaque to the layers below it, while
            // leaving the surrounding viewport available for backdrop
            // dismissal. Wrapping the full-size centering container would
            // swallow clicks everywhere and make the backdrop ineffective.
            container(opaque(self.content))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        ]
        .into()
    }
}
