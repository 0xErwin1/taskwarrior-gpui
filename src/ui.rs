use gpui::prelude::*;
use gpui::{px, Pixels};

use crate::theme::Theme;

pub const CARD_RADIUS: Pixels = px(6.0);
pub const CARD_PADDING: Pixels = px(8.0);
pub const SECTION_GAP: Pixels = px(12.0);
pub const INSET_GAP: Pixels = px(8.0);
pub const ROOT_PADDING: Pixels = px(12.0);

pub fn card_style(div: gpui::Div, theme: &Theme) -> gpui::Div {
    div.bg(theme.card)
        .border_1()
        .border_color(theme.divider)
        .rounded(CARD_RADIUS)
        .p(CARD_PADDING)
}

pub fn raised_style(div: gpui::Div, theme: &Theme) -> gpui::Div {
    div.bg(theme.raised)
        .border_1()
        .border_color(theme.divider)
        .rounded(CARD_RADIUS)
        .p(CARD_PADDING)
}

pub fn panel_style(div: gpui::Div, theme: &Theme) -> gpui::Div {
    div.bg(theme.panel)
        .border_1()
        .border_color(theme.border)
        .rounded(CARD_RADIUS)
}

pub fn field_style(div: gpui::Div, theme: &Theme, focused: bool) -> gpui::Div {
    div.bg(theme.field_bg)
        .border_1()
        .border_color(if focused {
            theme.focus_ring
        } else {
            theme.field_border
        })
        .rounded(CARD_RADIUS)
        .px_2()
        .py_1()
}

pub fn divider_h(theme: &Theme) -> gpui::Div {
    gpui::div().w_full().h(px(1.0)).bg(theme.divider)
}

pub fn divider_v(theme: &Theme) -> gpui::Div {
    gpui::div().h_full().w(px(1.0)).bg(theme.divider)
}

pub fn section_header(label: &str, theme: &Theme) -> gpui::Div {
    gpui::div()
        .px_2()
        .py_1()
        .text_xs()
        .text_color(theme.muted)
        .child(label.to_uppercase())
}

pub fn priority_badge(priority: &str, theme: &Theme) -> gpui::Div {
    let (bg, fg) = match priority {
        "High" | "H" => (Theme::alpha(theme.high, 0.15), theme.high),
        "Medium" | "M" => (Theme::alpha(theme.medium, 0.15), theme.medium),
        "Low" | "L" => (Theme::alpha(theme.low, 0.15), theme.low),
        _ => (gpui::rgba(0x00000000), theme.muted),
    };

    gpui::div()
        .px_2()
        .py(px(2.0))
        .rounded(px(4.0))
        .bg(bg)
        .text_color(fg)
        .text_xs()
        .font_weight(gpui::FontWeight::MEDIUM)
        .child(priority.to_string())
}
