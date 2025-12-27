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
    pub warning: Color,
    pub info: Color,
    pub selection: Color,
    pub selection_foreground: Color,
    pub text: Color,
    pub text_size: Option<gpui::Size<u32>>,

    pub high: Color,
    pub medium: Color,
    pub low: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            // ayu-dark
            background: gpui::rgb(0x0A0E14),
            panel: gpui::rgb(0x0F1419),
            foreground: gpui::rgb(0xB3B1AD),
            muted: gpui::rgb(0x5C6773),
            accent: gpui::rgb(0xFFB454),
            border: gpui::rgb(0x1F2430),

            error: gpui::rgb(0xF07178),
            success: gpui::rgb(0xAAD94C),
            warning: gpui::rgb(0xFFB454),
            info: gpui::rgb(0x59C2FF),

            selection: gpui::rgb(0x273747),
            selection_foreground: gpui::rgb(0xE6E1CF),

            text: gpui::rgb(0xB3B1AD),
            text_size: Some(gpui::Size::new(14, 14)),

            high: gpui::rgb(0xF07178),
            medium: gpui::rgb(0xFFB454),
            low: gpui::rgb(0xAAD94C),
        }
    }

    pub fn light() -> Self {
        Self {
            // ayu-light
            background: gpui::rgb(0xFAFAFA),
            panel: gpui::rgb(0xFFFFFF),
            foreground: gpui::rgb(0x5C6166),
            muted: gpui::rgb(0x8A9199),
            accent: gpui::rgb(0xFF8F40),
            border: gpui::rgb(0xE6E6E6),

            error: gpui::rgb(0xE65050),
            success: gpui::rgb(0x86B300),
            warning: gpui::rgb(0xF2AE49),
            info: gpui::rgb(0x399EE6),

            selection: gpui::rgb(0xD3EBFF),
            selection_foreground: gpui::rgb(0x5C6166),

            text: gpui::rgb(0x5C6166),
            text_size: Some(gpui::Size::new(14, 14)),

            high: gpui::rgb(0xE65050),
            medium: gpui::rgb(0xF2AE49),
            low: gpui::rgb(0x86B300),
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
