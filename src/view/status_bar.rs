use gpui::{Context, IntoElement, Render, Window, div, prelude::*, px};

use crate::theme::ActiveTheme;

pub struct StatusBar {
    // TODO: Add vim_mode, sync_state, last_sync, etc.
}

impl StatusBar {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {}
    }
}

impl Render for StatusBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();

        div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .h(px(30.0))
            .px_4()
            .py_1()
            .bg(theme.panel)
            .border_b_1()
            .border_color(theme.border)
    }
}
