use gpui::prelude::*;

use crate::components::label::Label;
use crate::theme::{ActiveTheme, Theme};

#[derive(IntoElement)]
pub struct SectionCard {
    title: gpui::SharedString,
    content: Vec<gpui::AnyElement>,
    style: gpui::StyleRefinement,
}

impl SectionCard {
    pub fn new(title: impl Into<gpui::SharedString>) -> Self {
        Self {
            title: title.into(),
            content: Vec::new(),
            style: gpui::StyleRefinement::default(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.content.push(child.into_any_element());
        self
    }

    pub fn children<E>(mut self, children: impl IntoIterator<Item = E>) -> Self
    where
        E: IntoElement,
    {
        self.content
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl Styled for SectionCard {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for SectionCard {
    fn render(mut self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = cx.theme();
        let title = self.title.to_string().to_uppercase();
        let section_title_color = Theme::alpha(theme.foreground, 0.88);

        let header = Label::new(title)
            .text_sm()
            .text_color(section_title_color)
            .font_weight(gpui::FontWeight::BOLD);

        let children: Vec<gpui::AnyElement> = self.content.drain(..).collect();

        let mut container = gpui::div()
            .flex()
            .flex_col()
            .gap_2()
            .bg(theme.raised)
            .border_1()
            .border_color(theme.divider)
            .rounded_md()
            .px(gpui::rems(0.75))
            .py(gpui::rems(0.5))
            .child(header)
            .children(children);

        container.style().refine(&self.style);

        container
    }
}
