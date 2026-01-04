use gpui::prelude::*;

use crate::components::label::Label;
use crate::theme::{ActiveTheme, Theme};

#[derive(IntoElement)]
pub struct FieldRow {
    label: gpui::SharedString,
    value: gpui::AnyElement,
    error: Option<gpui::SharedString>,
    label_width: gpui::Length,
    style: gpui::StyleRefinement,
}

impl FieldRow {
    pub fn new(label: impl Into<gpui::SharedString>, value: impl IntoElement) -> Self {
        Self {
            label: label.into(),
            value: value.into_any_element(),
            error: None,
            label_width: gpui::rems(10.0).into(),
            style: gpui::StyleRefinement::default(),
        }
    }

    pub fn error(mut self, error: Option<gpui::SharedString>) -> Self {
        self.error = error;
        self
    }

    pub fn label_width(mut self, width: gpui::Length) -> Self {
        self.label_width = width;
        self
    }
}

impl Styled for FieldRow {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for FieldRow {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = cx.theme();
        let label_color = Theme::alpha(theme.foreground, 0.72);

        let row = gpui::div()
            .flex()
            .items_start()
            .gap_3()
            .child(
                Label::new(self.label)
                    .text_color(label_color)
                    .text_sm()
                    .w(self.label_width),
            )
            .child(gpui::div().flex_1().min_w_0().child(self.value));

        let mut container = gpui::div().flex().flex_col().gap_1().child(row);
        if let Some(error) = self.error {
            container = container.child(
                gpui::div()
                    .flex()
                    .items_start()
                    .gap_3()
                    .child(gpui::div().w(self.label_width))
                    .child(Label::new(error).text_xs().text_color(theme.error)),
            );
        }

        container.style().refine(&self.style);

        container
    }
}

#[derive(IntoElement)]
pub struct KvRow {
    label: gpui::SharedString,
    value: gpui::AnyElement,
    label_width: gpui::Length,
}

impl KvRow {
    pub fn new(label: impl Into<gpui::SharedString>, value: impl IntoElement) -> Self {
        Self {
            label: label.into(),
            value: value.into_any_element(),
            label_width: gpui::rems(10.0).into(),
        }
    }

    pub fn label_width(mut self, width: gpui::Length) -> Self {
        self.label_width = width;
        self
    }
}

impl RenderOnce for KvRow {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        FieldRow::new(self.label, self.value).label_width(self.label_width)
    }
}
