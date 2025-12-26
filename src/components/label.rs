use gpui::prelude::*;

use crate::theme::ActiveTheme;

#[derive(gpui::IntoElement)]
pub struct Label {
    text: gpui::SharedString,
    style: gpui::StyleRefinement,
}

impl Label {
    pub fn new(text: impl Into<gpui::SharedString>) -> Self {
        Self {
            text: text.into(),
            style: gpui::StyleRefinement::default(),
        }
    }
}

impl gpui::Styled for Label {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Label {
    fn render(mut self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl gpui::IntoElement {
        let theme = cx.theme();

        let mut div = gpui::div()
            .line_height(gpui::rems(1.25))
            .text_color(theme.foreground)
            .child(gpui::StyledText::new(&self.text));

        *div.style() = self.style;

        div
    }
}
