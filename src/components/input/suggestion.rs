use gpui::SharedString;

#[derive(Clone, Debug)]
pub struct Suggestion {
    pub label: SharedString,
    pub insert: SharedString,
}

impl Suggestion {
    pub fn new(label: impl Into<SharedString>, insert: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            insert: insert.into(),
        }
    }

    pub fn simple(text: impl Into<SharedString>) -> Self {
        let text = text.into();
        Self {
            label: text.clone(),
            insert: text,
        }
    }
}
