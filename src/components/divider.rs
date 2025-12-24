use gpui::Styled as _;

use crate::theme::{ActiveTheme, Color};

pub enum DividerDirection {
    Horizontal,
    Vertical,
}

#[derive(gpui::IntoElement)]
pub struct Divider {
    color: Color,
    direction: DividerDirection,
}

impl Divider {
    pub fn new(color: Color, direction: DividerDirection) -> Self {
        Self { color, direction }
    }

    pub fn build(cx: &mut gpui::Context<Self>) -> Self {
        Self {
            color: cx.theme().border,
            direction: DividerDirection::Horizontal,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn direction(mut self, direction: DividerDirection) -> Self {
        self.direction = direction;
        self
    }
}

impl gpui::RenderOnce for Divider {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl gpui::IntoElement {
        match self.direction {
            DividerDirection::Horizontal => gpui::div().h(gpui::px(1.0)).w_full().bg(self.color),
            DividerDirection::Vertical => gpui::div().w(gpui::px(1.0)).h_full().bg(self.color),
        }
    }
}
