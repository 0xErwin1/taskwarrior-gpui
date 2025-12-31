use gpui::prelude::*;

use crate::components::label::Label;
use crate::theme::{ActiveTheme, Theme};
use crate::ui::mix_color;

pub struct ToastGlobal {
    pub host: gpui::Entity<ToastHost>,
}

impl gpui::Global for ToastGlobal {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Error,
}

struct Toast {
    id: u64,
    kind: ToastKind,
    message: String,
}

pub struct ToastHost {
    toasts: Vec<Toast>,
    next_id: u64,
}

impl ToastHost {
    pub fn new(_cx: &mut gpui::Context<Self>) -> Self {
        Self {
            toasts: Vec::new(),
            next_id: 1,
        }
    }

    pub fn push(&mut self, kind: ToastKind, message: impl Into<String>, cx: &mut Context<Self>) {
        let toast = Toast {
            id: self.next_id,
            kind,
            message: message.into(),
        };

        self.next_id += 1;
        self.toasts.push(toast);

        cx.notify();
    }

    fn dismiss(&mut self, id: u64, cx: &mut Context<Self>) {
        self.toasts.retain(|toast| toast.id != id);
        cx.notify();
    }

    fn kind_color(kind: ToastKind, theme: &Theme) -> gpui::Rgba {
        match kind {
            ToastKind::Info => theme.info,
            ToastKind::Success => theme.success,
            ToastKind::Error => theme.error,
        }
    }
}

impl gpui::Render for ToastHost {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        if self.toasts.is_empty() {
            return gpui::div().into_any_element();
        }

        let theme = cx.theme();
        let items = self.toasts.iter().map(|toast| {
            let toast_id = toast.id;
            let accent = Self::kind_color(toast.kind, theme);

            let close_button = gpui::div()
                .flex()
                .items_center()
                .justify_center()
                .w(gpui::rems(1.5))
                .h(gpui::rems(1.5))
                .text_sm()
                .text_color(theme.muted)
                .cursor_pointer()
                .hover(|s| s.text_color(theme.accent))
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |host, _event, _window, cx| {
                        host.dismiss(toast_id, cx);
                    }),
                )
                .child(Label::new("X"));

            let background = mix_color(theme.background, accent, 0.2);
            let border = Theme::alpha(accent, 0.45);

            gpui::div()
                .flex()
                .items_center()
                .gap_3()
                .px_5()
                .py_3()
                .occlude()
                .border_1()
                .border_color(border)
                .bg(background)
                .rounded_md()
                .shadow_lg()
                .child(
                    gpui::div()
                        .w(gpui::px(6.0))
                        .h_full()
                        .bg(Theme::alpha(accent, 0.9))
                        .rounded_md(),
                )
                .child(
                    gpui::div().flex_1().min_w(gpui::rems(18.0)).child(
                        Label::new(toast.message.clone())
                            .text_sm()
                            .text_color(theme.foreground)
                            .font_weight(gpui::FontWeight::MEDIUM),
                    ),
                )
                .child(close_button)
                .into_any_element()
        });

        gpui::div()
            .id("toast-host")
            .absolute()
            .top(gpui::rems(1.0))
            .right(gpui::rems(1.0))
            .flex()
            .flex_col()
            .gap_2()
            .children(items)
            .into_any_element()
    }
}
