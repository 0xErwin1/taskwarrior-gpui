use gpui::prelude::*;
use std::sync::Arc;

use crate::components::label::Label;
use crate::theme::{ActiveTheme, Theme};

#[derive(Debug, Clone, Copy)]
pub enum ActionButtonVariant {
    Normal,
    Danger,
    Ghost,
}

#[derive(Clone, IntoElement)]
pub struct ActionButton {
    id: Option<gpui::ElementId>,
    label: gpui::SharedString,
    variant: ActionButtonVariant,
    enabled: bool,
    on_click:
        Option<Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>>,
}

impl ActionButton {
    pub fn new(label: impl Into<gpui::SharedString>) -> Self {
        Self {
            id: None,
            label: label.into(),
            variant: ActionButtonVariant::Normal,
            enabled: true,
            on_click: None,
        }
    }

    pub fn id(mut self, id: impl Into<gpui::ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn variant(mut self, variant: ActionButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_click = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for ActionButton {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = cx.theme();
        let (bg, fg, hover_bg) = match self.variant {
            ActionButtonVariant::Normal => (theme.raised, theme.foreground, theme.hover),
            ActionButtonVariant::Danger => (
                theme.error,
                theme.selection_foreground,
                Theme::alpha(theme.error, 0.8),
            ),
            ActionButtonVariant::Ghost => (gpui::rgba(0x00000000), theme.foreground, theme.hover),
        };

        if let Some(id) = self.id {
            let mut button = gpui::div()
                .id(id)
                .px(gpui::rems(0.75))
                .py(gpui::rems(0.35))
                .rounded_md()
                .text_sm();

            match self.variant {
                ActionButtonVariant::Normal | ActionButtonVariant::Ghost => {
                    button = button.border_1().border_color(theme.divider);
                }
                ActionButtonVariant::Danger => {}
            }

            if self.enabled {
                button = button.bg(bg).text_color(fg);
                if let Some(on_click) = self.on_click {
                    button = button
                        .cursor_pointer()
                        .hover(move |s| s.bg(hover_bg))
                        .on_mouse_down(gpui::MouseButton::Left, move |event, window, app| {
                            (on_click)(event, window, app);
                        });
                }
            } else {
                button = button.bg(bg).text_color(theme.muted);
            }

            return button.child(Label::new(self.label)).into_any_element();
        }

        let mut button = gpui::div()
            .px(gpui::rems(0.75))
            .py(gpui::rems(0.35))
            .rounded_md()
            .text_sm();

        match self.variant {
            ActionButtonVariant::Normal | ActionButtonVariant::Ghost => {
                button = button.border_1().border_color(theme.divider);
            }
            ActionButtonVariant::Danger => {}
        }

        if self.enabled {
            button = button.bg(bg).text_color(fg);
            if let Some(on_click) = self.on_click {
                button = button
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_bg))
                    .on_mouse_down(gpui::MouseButton::Left, move |event, window, app| {
                        (on_click)(event, window, app);
                    });
            }
        } else {
            button = button.bg(bg).text_color(theme.muted);
        }

        button.child(Label::new(self.label)).into_any_element()
    }
}
