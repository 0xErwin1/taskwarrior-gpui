use gpui::prelude::*;
use std::sync::Arc;

use crate::components::dialog::{Dialog, DialogButton, DialogButtonVariant};
use crate::theme::Theme;

#[derive(Clone)]
pub struct ConfirmDialog {
    title: String,
    hint: Option<String>,
    cancel: Option<(
        String,
        Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>,
    )>,
    confirm: Option<(
        String,
        DialogButtonVariant,
        Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>,
    )>,
    on_backdrop_click:
        Option<Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>>,
}

impl ConfirmDialog {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            hint: None,
            cancel: None,
            confirm: None,
            on_backdrop_click: None,
        }
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn cancel(
        mut self,
        label: impl Into<String>,
        on_click: Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>,
    ) -> Self {
        self.cancel = Some((label.into(), on_click));
        self
    }

    pub fn primary(
        mut self,
        label: impl Into<String>,
        on_click: Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>,
    ) -> Self {
        self.confirm = Some((label.into(), DialogButtonVariant::Primary, on_click));
        self
    }

    pub fn danger(
        mut self,
        label: impl Into<String>,
        on_click: Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>,
    ) -> Self {
        self.confirm = Some((label.into(), DialogButtonVariant::Danger, on_click));
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
        let mut dialog = Dialog::new(self.title);
        if let Some(hint) = self.hint {
            dialog = dialog.hint(hint);
        }
        if let Some((label, on_click)) = self.cancel {
            dialog = dialog.button(DialogButton::default(label, on_click));
        }
        if let Some((label, variant, on_click)) = self.confirm {
            let button = match variant {
                DialogButtonVariant::Primary => DialogButton::primary(label, on_click),
                DialogButtonVariant::Danger => DialogButton::danger(label, on_click),
                DialogButtonVariant::Default => DialogButton::default(label, on_click),
            };
            dialog = dialog.button(button);
        }
        if let Some(handler) = self.on_backdrop_click {
            dialog = dialog.on_backdrop_click(handler);
        }

        dialog.render(theme)
    }
}
