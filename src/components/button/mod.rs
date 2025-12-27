pub mod dropdown;
pub use dropdown::{Dropdown, DropdownItem};

use gpui::{
    App, ClickEvent, ElementId, Pixels, SharedString, StyleRefinement, Window, div, prelude::*, px,
};
use std::sync::Arc;

use crate::components::icon::{Icon, IconSize};
use crate::components::label::Label;
use crate::theme::ActiveTheme;

fn darken(color: gpui::Rgba, amount: f32) -> gpui::Rgba {
    gpui::Rgba {
        r: (color.r * (1.0 - amount)).max(0.0),
        g: (color.g * (1.0 - amount)).max(0.0),
        b: (color.b * (1.0 - amount)).max(0.0),
        a: color.a,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ButtonSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl ButtonSize {
    fn height(self) -> Pixels {
        match self {
            ButtonSize::Small => px(28.),
            ButtonSize::Medium => px(36.),
            ButtonSize::Large => px(44.),
        }
    }

    fn px(self) -> Pixels {
        match self {
            ButtonSize::Small => px(8.),
            ButtonSize::Medium => px(12.),
            ButtonSize::Large => px(16.),
        }
    }

    fn icon_size(self) -> IconSize {
        match self {
            ButtonSize::Small => IconSize::Small,
            ButtonSize::Medium => IconSize::Medium,
            ButtonSize::Large => IconSize::Large,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    #[default]
    Secondary,
    Ghost,
    Danger,
    Success,
    Text,
}

pub trait ButtonVariants: Sized {
    fn with_variant(self, variant: ButtonVariant) -> Self;

    fn primary(self) -> Self {
        self.with_variant(ButtonVariant::Primary)
    }

    fn secondary(self) -> Self {
        self.with_variant(ButtonVariant::Secondary)
    }

    fn ghost(self) -> Self {
        self.with_variant(ButtonVariant::Ghost)
    }

    fn danger(self) -> Self {
        self.with_variant(ButtonVariant::Danger)
    }

    fn success(self) -> Self {
        self.with_variant(ButtonVariant::Success)
    }

    fn text(self) -> Self {
        self.with_variant(ButtonVariant::Text)
    }
}

#[derive(Clone, IntoElement)]
pub struct Button {
    id: ElementId,
    label: Option<SharedString>,
    icon: Option<Icon>,
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
    loading: bool,
    style: StyleRefinement,
    on_click: Option<Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            icon: None,
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            disabled: false,
            loading: false,
            style: StyleRefinement::default(),
            on_click: None,
        }
    }

    pub fn label(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id).with_label(label)
    }

    pub fn icon(id: impl Into<ElementId>, icon: impl Into<Icon>) -> Self {
        Self::new(id).with_icon(icon)
    }

    pub fn with_label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn with_size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn small(self) -> Self {
        self.with_size(ButtonSize::Small)
    }

    pub fn medium(self) -> Self {
        self.with_size(ButtonSize::Medium)
    }

    pub fn large(self) -> Self {
        self.with_size(ButtonSize::Large)
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Arc::new(handler));
        self
    }
}

impl ButtonVariants for Button {
    fn with_variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
}

impl Styled for Button {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let is_icon_only = self.label.is_none();

        let transparent = gpui::rgba(0x00000000);

        let (bg, fg, border, hover_bg) = match self.variant {
            ButtonVariant::Primary => (
                theme.accent,
                theme.background,
                theme.accent,
                darken(theme.accent, 0.1),
            ),
            ButtonVariant::Secondary => {
                (theme.panel, theme.foreground, theme.border, theme.selection)
            }
            ButtonVariant::Ghost => (transparent, theme.foreground, transparent, theme.selection),
            ButtonVariant::Danger => (
                theme.error,
                theme.background,
                theme.error,
                darken(theme.error, 0.1),
            ),
            ButtonVariant::Success => (
                theme.success,
                theme.background,
                theme.success,
                darken(theme.success, 0.1),
            ),
            ButtonVariant::Text => (transparent, theme.foreground, transparent, transparent),
        };

        let (bg, fg, border) = if self.disabled {
            (transparent, theme.muted, theme.border)
        } else {
            (bg, fg, border)
        };

        let height = self.size.height();
        let px = self.size.px();
        let icon_size = self.size.icon_size();

        let mut base = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .gap_2()
            .h(height)
            .border_1()
            .rounded_md()
            .cursor_pointer()
            .bg(bg)
            .border_color(border)
            .text_color(fg);

        base = if is_icon_only {
            base.px(px)
        } else {
            base.px(px * 1.5)
        };

        if !self.disabled && self.variant != ButtonVariant::Text {
            base = base
                .hover(|s: gpui::StyleRefinement| s.bg(hover_bg))
                .active(|s: gpui::StyleRefinement| s.opacity(0.8));
        }

        if self.disabled {
            base = base.cursor_not_allowed();
        }

        if let Some(icon) = self.icon {
            base = base.child(icon.size(icon_size));
        }

        if let Some(label) = self.label {
            base = base.child(Label::new(label).text_color(fg));
        }

        if self.loading {
            base = base.opacity(0.7);
        }

        if let Some(on_click) = self.on_click {
            let disabled = self.disabled || self.loading;
            base = base.on_click(move |event, window, cx| {
                if !disabled {
                    on_click(event, window, cx);
                }
            });
        }

        *base.style() = self.style;

        base
    }
}
