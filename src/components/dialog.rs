use crate::components::label::Label;
use crate::theme::Theme;
use gpui::prelude::*;
use std::sync::Arc;

pub struct DialogButton {
    label: String,
    variant: DialogButtonVariant,
    on_click: Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>,
}

#[derive(Clone, Copy)]
pub enum DialogButtonVariant {
    Default,
    Primary,
    Danger,
}

impl DialogButton {
    pub fn new(
        label: impl Into<String>,
        variant: DialogButtonVariant,
        on_click: Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>,
    ) -> Self {
        Self {
            label: label.into(),
            variant,
            on_click,
        }
    }

    pub fn default(
        label: impl Into<String>,
        on_click: Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>,
    ) -> Self {
        Self::new(label, DialogButtonVariant::Default, on_click)
    }

    pub fn primary(
        label: impl Into<String>,
        on_click: Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>,
    ) -> Self {
        Self::new(label, DialogButtonVariant::Primary, on_click)
    }

    pub fn danger(
        label: impl Into<String>,
        on_click: Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>,
    ) -> Self {
        Self::new(label, DialogButtonVariant::Danger, on_click)
    }
}

pub struct Dialog {
    title: String,
    message: Option<String>,
    hint: Option<String>,
    buttons: Vec<DialogButton>,
    on_backdrop_click:
        Option<Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>>,
}

impl Dialog {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: None,
            hint: None,
            buttons: Vec::new(),
            on_backdrop_click: None,
        }
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn button(mut self, button: DialogButton) -> Self {
        self.buttons.push(button);
        self
    }

    pub fn on_backdrop_click(
        mut self,
        handler: Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>,
    ) -> Self {
        self.on_backdrop_click = Some(handler);
        self
    }

    pub fn render(self, theme: &Theme) -> gpui::Div {
        let card = gpui::div()
            .flex()
            .flex_col()
            .gap_3()
            .p(gpui::rems(1.0))
            .min_w(gpui::rems(20.0))
            .bg(theme.panel)
            .border_2()
            .border_color(theme.error)
            .rounded_md()
            .shadow_lg()
            .child(
                Label::new(&self.title)
                    .text_color(theme.foreground)
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM),
            );

        let card = if let Some(message) = &self.message {
            card.child(Label::new(message).text_color(theme.foreground).text_sm())
        } else {
            card
        };

        let card = if let Some(hint) = &self.hint {
            card.child(
                gpui::div()
                    .flex()
                    .gap_2()
                    .text_xs()
                    .text_color(theme.muted)
                    .child(hint.clone()),
            )
        } else {
            card
        };

        let card = if !self.buttons.is_empty() {
            let button_row = self.buttons.into_iter().fold(
                gpui::div().flex().gap_2().justify_end(),
                |row, button| {
                    let (bg_color, text_color, hover_bg) = match button.variant {
                        DialogButtonVariant::Default => {
                            (theme.raised, theme.foreground, theme.hover)
                        }
                        DialogButtonVariant::Primary => (
                            theme.accent,
                            theme.selection_foreground,
                            Theme::alpha(theme.accent, 0.8),
                        ),
                        DialogButtonVariant::Danger => (
                            theme.error,
                            theme.selection_foreground,
                            Theme::alpha(theme.error, 0.8),
                        ),
                    };

                    let btn = gpui::div()
                        .px(gpui::rems(0.75))
                        .py(gpui::rems(0.35))
                        .rounded_md()
                        .text_sm()
                        .cursor_pointer();

                    let btn = match button.variant {
                        DialogButtonVariant::Default => btn
                            .border_1()
                            .border_color(theme.divider)
                            .bg(bg_color)
                            .text_color(text_color)
                            .hover(move |s| s.bg(hover_bg)),
                        _ => btn
                            .bg(bg_color)
                            .text_color(text_color)
                            .hover(move |s| s.bg(hover_bg)),
                    };

                    let handler = button.on_click.clone();
                    let btn = btn
                        .on_mouse_down(gpui::MouseButton::Left, move |event, window, app| {
                            (handler)(event, window, app);
                        })
                        .child(Label::new(&button.label));

                    row.child(btn)
                },
            );
            card.child(button_row)
        } else {
            card
        };

        let card = card.on_mouse_down(gpui::MouseButton::Left, |_event, _window, _app| {});

        let backdrop = gpui::div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(Theme::alpha(theme.backdrop, 0.7))
            .child(card);

        if let Some(on_backdrop_click) = self.on_backdrop_click {
            backdrop.on_mouse_down(gpui::MouseButton::Left, move |event, window, app| {
                (on_backdrop_click)(event, window, app);
            })
        } else {
            backdrop
        }
    }
}
