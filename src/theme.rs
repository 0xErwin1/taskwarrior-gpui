pub type Color = gpui::Rgba;

#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub panel: Color,
    pub muted: Color,
    pub accent: Color,
    pub border: Color,
    pub error: Color,
    pub success: Color,
    pub selection: Color,
    pub selection_foreground: Color,
    pub text: Color,
    pub text_size: Option<gpui::Size<u32>>,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            background: gpui::rgb(0x1E1E2E),
            panel: gpui::rgb(0x181825),
            foreground: gpui::rgb(0xCDD6F4),
            muted: gpui::rgb(0x7F849C),
            accent: gpui::rgb(0x89B4FA),
            border: gpui::rgb(0x313244),
            error: gpui::rgb(0xF38BA8),
            success: gpui::rgb(0xA6E3A1),
            selection: gpui::rgb(0x45475A),
            selection_foreground: gpui::rgb(0xCDD6F4),
            text: gpui::rgb(0xCDD6F4),
            text_size: Some(gpui::Size::new(14, 14)),
        }
    }

    pub fn light() -> Self {
        Self {
            background: gpui::rgb(0xF5F5F5),
            panel: gpui::rgb(0xF0F0F0),
            foreground: gpui::rgb(0x333333),
            muted: gpui::rgb(0x999999),
            accent: gpui::rgb(0x0078D4),
            border: gpui::rgb(0xE0E0E0),
            error: gpui::rgb(0xFF4444),
            success: gpui::rgb(0x4CAF50),
            selection: gpui::rgb(0xD0D0D0),
            selection_foreground: gpui::rgb(0x333333),
            text: gpui::rgb(0x333333),
            text_size: Some(gpui::Size::new(14, 14)),
        }
    }

    pub fn global(app: &gpui::App) -> &Self {
        app.global::<Self>()
    }
}

impl gpui::Global for Theme {}

pub trait ActiveTheme {
    fn theme(&self) -> &Theme;
}

impl ActiveTheme for gpui::App {
    #[inline(always)]
    fn theme(&self) -> &Theme {
        Theme::global(self)
    }
}
