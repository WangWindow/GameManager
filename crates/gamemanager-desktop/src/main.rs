use iced::{Element, Task};

fn main() -> iced::Result {
    iced::application(|| (), update, view)
        .title("GameManager")
        .run()
}

fn update(_: &mut (), _: ()) -> Task<()> {
    Task::none()
}

fn view(_: &()) -> Element<'_, ()> {
    iced::widget::text("GameManager").into()
}
