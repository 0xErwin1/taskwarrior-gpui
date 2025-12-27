pub type Color = gpui::Rgba;

#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub panel: Color,
    pub card: Color,
    pub raised: Color,

    pub foreground: Color,
    pub muted: Color,
    pub disabled_fg: Color,

    pub accent: Color,
    pub focus_ring: Color,

    pub border: Color,
    pub divider: Color,

    pub field_bg: Color,
    pub field_border: Color,
    pub field_placeholder: Color,

    pub hover: Color,
    pub selection: Color,
    pub selection_foreground: Color,

    pub backdrop: Color,

    pub error: Color,
    pub success: Color,
    pub warning: Color,
    pub info: Color,

    pub high: Color,
    pub medium: Color,
    pub low: Color,

    pub text_size: Option<gpui::Size<u32>>,
}

impl Theme {
    pub fn alpha(c: Color, a: f32) -> Color {
        gpui::Rgba {
            r: c.r,
            g: c.g,
            b: c.b,
            a: a.clamp(0.0, 1.0),
        }
    }

    pub fn dark() -> Self {
        let background = gpui::rgb(0x0A0E14);
        let panel = gpui::rgb(0x0F1419);
        let foreground = gpui::rgb(0xB3B1AD);
        let muted = gpui::rgb(0x5C6773);
        let accent = gpui::rgb(0xFFB454);
        let border = gpui::rgb(0x1F2430);

        let card = gpui::rgb(0x111823);
        let raised = gpui::rgb(0x151E2B);

        let divider = Self::alpha(foreground, 0.10);
        let hover = Self::alpha(foreground, 0.05);
        let field_bg = raised;
        let field_border = Self::alpha(foreground, 0.14);
        let field_placeholder = Self::alpha(foreground, 0.45);
        let focus_ring = Self::alpha(accent, 0.75);
        let disabled_fg = Self::alpha(foreground, 0.35);
        let backdrop = gpui::rgba(0x0000008C);

        let selection = gpui::rgb(0x273747);
        let selection_foreground = gpui::rgb(0xE6E1CF);

        let error = gpui::rgb(0xF07178);
        let success = gpui::rgb(0xAAD94C);
        let warning = gpui::rgb(0xFFB454);
        let info = gpui::rgb(0x59C2FF);

        Self {
            background,
            panel,
            card,
            raised,

            foreground,
            muted,
            disabled_fg,

            accent,
            focus_ring,

            border,
            divider,

            field_bg,
            field_border,
            field_placeholder,

            hover,
            selection,
            selection_foreground,

            backdrop,

            error,
            success,
            warning,
            info,

            high: error,
            medium: warning,
            low: success,

            text_size: Some(gpui::Size::new(14, 14)),
        }
    }

    pub fn light() -> Self {
        let background = gpui::rgb(0xFAFAFA);
        let panel = gpui::rgb(0xFFFFFF);
        let foreground = gpui::rgb(0x5C6166);
        let muted = gpui::rgb(0x8A9199);
        let accent = gpui::rgb(0xFF8F40);
        let border = gpui::rgb(0xE6E6E6);

        let card = gpui::rgb(0xF6F7F9);
        let raised = gpui::rgb(0xEFF1F5);

        let divider = Self::alpha(foreground, 0.12);
        let hover = Self::alpha(foreground, 0.06);
        let field_bg = gpui::rgb(0xFFFFFF);
        let field_border = Self::alpha(foreground, 0.18);
        let field_placeholder = Self::alpha(foreground, 0.55);
        let focus_ring = Self::alpha(accent, 0.70);
        let disabled_fg = Self::alpha(foreground, 0.40);
        let backdrop = gpui::rgba(0x00000040);

        let selection = gpui::rgb(0xD3EBFF);
        let selection_foreground = foreground;

        let error = gpui::rgb(0xE65050);
        let success = gpui::rgb(0x86B300);
        let warning = gpui::rgb(0xF2AE49);
        let info = gpui::rgb(0x399EE6);

        Self {
            background,
            panel,
            card,
            raised,

            foreground,
            muted,
            disabled_fg,

            accent,
            focus_ring,

            border,
            divider,

            field_bg,
            field_border,
            field_placeholder,

            hover,
            selection,
            selection_foreground,

            backdrop,

            error,
            success,
            warning,
            info,

            high: error,
            medium: warning,
            low: success,

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
