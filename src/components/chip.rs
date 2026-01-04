use gpui::prelude::*;
use std::sync::Arc;

use crate::theme::{ActiveTheme, Theme};

#[derive(Debug, Clone, Copy)]
pub enum ChipVariant {
    Info,
    Success,
    Warning,
    Danger,
    Muted,
    Accent,
    Custom {
        background: gpui::Rgba,
        foreground: gpui::Rgba,
    },
}

impl ChipVariant {
    fn colors(self, theme: &Theme) -> (gpui::Rgba, gpui::Rgba) {
        match self {
            ChipVariant::Info => (Theme::alpha(theme.info, 0.18), theme.info),
            ChipVariant::Success => (Theme::alpha(theme.success, 0.18), theme.success),
            ChipVariant::Warning => (Theme::alpha(theme.warning, 0.18), theme.warning),
            ChipVariant::Danger => (Theme::alpha(theme.error, 0.18), theme.error),
            ChipVariant::Muted => (Theme::alpha(theme.muted, 0.18), theme.muted),
            ChipVariant::Accent => (Theme::alpha(theme.accent, 0.15), theme.accent),
            ChipVariant::Custom {
                background,
                foreground,
            } => (background, foreground),
        }
    }
}

#[derive(Clone, IntoElement)]
pub struct Chip {
    label: gpui::SharedString,
    variant: ChipVariant,
    selected: bool,
    on_remove:
        Option<Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>>,
}

impl Chip {
    pub fn new(label: impl Into<gpui::SharedString>) -> Self {
        Self {
            label: label.into(),
            variant: ChipVariant::Info,
            selected: false,
            on_remove: None,
        }
    }

    pub fn variant(mut self, variant: ChipVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn custom(self, background: gpui::Rgba, foreground: gpui::Rgba) -> Self {
        self.variant(ChipVariant::Custom {
            background,
            foreground,
        })
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn removable(
        mut self,
        on_remove: Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>,
    ) -> Self {
        self.on_remove = Some(on_remove);
        self
    }
}

impl RenderOnce for Chip {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = cx.theme();
        let (bg, fg) = self.variant.colors(theme);

        let mut chip = gpui::div()
            .flex()
            .items_center()
            .gap_1()
            .px(gpui::rems(0.5))
            .py(gpui::rems(0.125))
            .rounded(gpui::rems(0.25))
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM);

        if self.selected {
            chip = chip
                .bg(theme.accent)
                .text_color(theme.selection_foreground)
                .border_2()
                .border_color(theme.focus_ring);
        } else {
            chip = chip.bg(bg).text_color(fg);
        }

        chip = chip.child(self.label);

        if let Some(on_remove) = self.on_remove {
            chip = chip.child(
                gpui::div()
                    .text_color(theme.muted)
                    .cursor_pointer()
                    .hover(|s| s.text_color(theme.error))
                    .on_mouse_down(gpui::MouseButton::Left, move |event, window, app| {
                        (on_remove)(event, window, app);
                    })
                    .child("×"),
            );
        }

        chip
    }
}
