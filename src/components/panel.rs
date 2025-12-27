use gpui::prelude::*;

use crate::theme::ActiveTheme;

#[derive(gpui::IntoElement)]
pub struct Panel {
    id: gpui::ElementId,
    content: Vec<gpui::AnyElement>,
    title: Option<String>,
    border: f32,
    padding: f32,
    style: gpui::StyleRefinement,
}

impl Panel {
    pub fn new(id: impl Into<gpui::ElementId>) -> Self {
        Self {
            id: id.into(),
            content: Vec::new(),
            title: None,
            border: 0.0,
            padding: 0.0,
            style: gpui::StyleRefinement::default(),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn border(mut self, border: f32) -> Self {
        self.border = border;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn child(mut self, child: impl gpui::IntoElement) -> Self {
        self.content.push(child.into_any_element());
        self
    }

    pub fn children<E>(mut self, children: impl IntoIterator<Item = E>) -> Self
    where
        E: gpui::IntoElement,
    {
        self.content
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl gpui::Styled for Panel {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl gpui::RenderOnce for Panel {
    fn render(mut self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = cx.theme();

        let header = self.title.as_ref().map(|title| {
            gpui::div()
                .px_2()
                .py_1()
                .text_sm()
                .text_color(theme.muted)
                .border_b(gpui::px(self.border))
                .border_color(theme.border)
                .child(title.clone())
                .into_any_element()
        });

        let children: Vec<gpui::AnyElement> = self.content.drain(..).collect();

        let mut base = gpui::div()
            .relative()
            .size_full()
            .bg(theme.panel)
            .border(gpui::px(self.border))
            .border_color(theme.border)
            .rounded_md()
            .p(gpui::px(self.padding))
            .overflow_hidden()
            .children(header)
            .children(children);

        base.style().refine(&self.style);

        base
    }
}
