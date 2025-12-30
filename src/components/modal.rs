use std::sync::Arc;

use gpui::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct ModalState {
    pub open: bool,
}

#[derive(gpui::IntoElement)]
pub struct ModalFrame {
    id: gpui::ElementId,
    focus: gpui::FocusHandle,
    panel: gpui::AnyElement,
    backdrop: gpui::Rgba,
    on_close:
        Option<Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>>,
}

impl ModalFrame {
    pub fn new(
        id: impl Into<gpui::ElementId>,
        focus: gpui::FocusHandle,
        backdrop: gpui::Rgba,
    ) -> Self {
        Self {
            id: id.into(),
            focus,
            panel: gpui::div().into_any_element(),
            backdrop,
            on_close: None,
        }
    }

    pub fn panel(mut self, panel: impl gpui::IntoElement) -> Self {
        self.panel = panel.into_any_element();
        self
    }

    pub fn on_close(
        mut self,
        handler: impl Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_close = Some(Arc::new(handler));
        self
    }
}

impl gpui::RenderOnce for ModalFrame {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl gpui::IntoElement {
        let mut panel_wrap = gpui::div().child(self.panel);

        if let Some(handler) = self.on_close {
            panel_wrap = panel_wrap.on_mouse_down_out(move |event, window, app| {
                (handler)(event, window, app);
            });
        }

        let root = gpui::div()
            .id(self.id)
            .size_full()
            .absolute()
            .top_0()
            .left_0()
            .occlude()
            .track_focus(&self.focus);

        root.child(
            gpui::div()
                .size_full()
                .bg(self.backdrop)
                .absolute()
                .top_0()
                .left_0(),
        )
        .child(
            gpui::div()
                .size_full()
                .absolute()
                .top_0()
                .left_0()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .child(panel_wrap),
        )
    }
}
