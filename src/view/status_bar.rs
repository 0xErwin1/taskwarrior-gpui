use gpui::{Context, IntoElement, Render, Window, div, prelude::*, px};

use crate::theme::ActiveTheme;
use crate::ui::{card_style, CARD_RADIUS};

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

        card_style(div(), &theme)
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .h(px(28.0))
            .px_3()
            .py_1()
            .rounded(CARD_RADIUS)
    }
}
