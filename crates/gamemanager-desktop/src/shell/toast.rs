#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Toast {
    pub message: String,
}

impl Toast {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}
