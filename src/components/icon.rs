use gpui::{Pixels, SharedString, StyleRefinement, Styled, prelude::*, px, rems};

#[derive(Debug, Clone, Copy, Default)]
pub enum IconSize {
    XSmall, // 12px
    Small,  // 14px
    #[default]
    Medium, // 16px
    Large,  // 20px
    XLarge, // 24px
    Custom(Pixels),
}

impl IconSize {
    pub fn px(self) -> Pixels {
        match self {
            IconSize::XSmall => px(12.),
            IconSize::Small => px(14.),
            IconSize::Medium => px(16.),
            IconSize::Large => px(20.),
            IconSize::XLarge => px(24.),
            IconSize::Custom(size) => size,
        }
    }

    pub fn rems(self) -> gpui::Rems {
        match self {
            IconSize::XSmall => rems(0.75),
            IconSize::Small => rems(0.875),
            IconSize::Medium => rems(1.),
            IconSize::Large => rems(1.25),
            IconSize::XLarge => rems(1.5),
            IconSize::Custom(size) => rems(f32::from(size) / 16.),
        }
    }
}

impl From<Pixels> for IconSize {
    fn from(px: Pixels) -> Self {
        IconSize::Custom(px)
    }
}

impl From<f32> for IconSize {
    fn from(px: f32) -> Self {
        IconSize::Custom(gpui::px(px))
    }
}

pub trait IconNamed {
    fn path(&self) -> SharedString;
}

#[derive(Debug, Clone, Copy)]
pub enum IconName {}

impl IconNamed for IconName {
    fn path(&self) -> SharedString {
        match self {
            _ => todo!(),
        }
    }
}

impl<T: IconNamed> From<T> for Icon {
    fn from(name: T) -> Self {
        Self::new(name)
    }
}

#[derive(Clone, IntoElement)]
pub struct Icon {
    path: SharedString,
    size: IconSize,
    color: Option<gpui::Hsla>,
    style: StyleRefinement,
}

impl Default for Icon {
    fn default() -> Self {
        Self {
            path: SharedString::default(),
            size: IconSize::default(),
            color: None,
            style: StyleRefinement::default(),
        }
    }
}

impl Icon {
    pub fn new(name: impl IconNamed) -> Self {
        Self::default().path(name.path())
    }

    pub fn from_path(path: impl Into<SharedString>) -> Self {
        Self::default().path(path)
    }

    pub fn path(mut self, path: impl Into<SharedString>) -> Self {
        self.path = path.into();
        self
    }

    pub fn size(mut self, size: impl Into<IconSize>) -> Self {
        self.size = size.into();
        self
    }

    pub fn xsmall(self) -> Self {
        self.size(IconSize::XSmall)
    }

    pub fn small(self) -> Self {
        self.size(IconSize::Small)
    }

    pub fn medium(self) -> Self {
        self.size(IconSize::Medium)
    }

    pub fn large(self) -> Self {
        self.size(IconSize::Large)
    }

    pub fn xlarge(self) -> Self {
        self.size(IconSize::XLarge)
    }

    pub fn color(mut self, color: impl Into<gpui::Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn with_style(mut self, style: StyleRefinement) -> Self {
        self.style = style;
        self
    }
}

impl Styled for Icon {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Icon {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let size = self.size.rems();
        let color = self.color.unwrap_or_else(|| window.text_style().color);
        self.style.size.width = Some(size.into());
        self.style.size.height = Some(size.into());

        let mut svg = gpui::svg().path(self.path).flex_none().text_color(color);

        *svg.style() = self.style;

        svg
    }
}
