use gpui::prelude::*;

use crate::components::toast::ToastHost;
use crate::keymap::FocusTarget;
use crate::theme::Theme;
use crate::ui::{CARD_PADDING, CARD_RADIUS, ROOT_PADDING, SECTION_GAP, SIDEBAR_WIDTH};
use crate::view::sidebar::Sidebar;
use crate::view::status_bar::StatusBar;
use crate::view::task_table::TaskTable;

pub fn render_app_layout(
    theme: &Theme,
    focus_handle: &gpui::FocusHandle,
    focus_target: FocusTarget,
    sidebar: gpui::Entity<Sidebar>,
    task_table: gpui::Entity<TaskTable>,
    status_bar: gpui::Entity<StatusBar>,
    toast_host: gpui::Entity<ToastHost>,
    on_root_key_down: impl Fn(&gpui::KeyDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    on_sidebar_mouse_down: impl Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    on_table_mouse_down: impl Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    modal: Option<gpui::AnyElement>,
) -> gpui::AnyElement {
    let sidebar_focused = focus_target.is_sidebar();

    let sidebar_border_color = if sidebar_focused {
        theme.focus_ring
    } else {
        theme.divider
    };

    let sidebar = gpui::div()
        .bg(theme.card)
        .border_2()
        .border_color(sidebar_border_color)
        .rounded(CARD_RADIUS)
        .p(CARD_PADDING)
        .w(SIDEBAR_WIDTH)
        .h_full()
        .flex_shrink_0()
        .overflow_hidden()
        .on_mouse_down(gpui::MouseButton::Left, on_sidebar_mouse_down)
        .child(sidebar);

    let table_focused = matches!(focus_target, FocusTarget::Table | FocusTarget::TableHeaders);

    let table_border_color = if table_focused {
        theme.focus_ring
    } else {
        theme.divider
    };

    let main = gpui::div()
        .bg(theme.card)
        .border_2()
        .border_color(table_border_color)
        .rounded(CARD_RADIUS)
        .p(CARD_PADDING)
        .flex_1()
        .h_full()
        .min_w_0()
        .overflow_hidden()
        .p_0()
        .on_mouse_down(gpui::MouseButton::Left, on_table_mouse_down)
        .child(task_table);

    let content = gpui::div()
        .flex()
        .flex_1()
        .min_h_0()
        .gap(SECTION_GAP)
        .child(sidebar)
        .child(main);

    let mut root = gpui::div()
        .flex()
        .flex_col()
        .size_full()
        .relative()
        .bg(theme.background)
        .p(ROOT_PADDING)
        .gap(SECTION_GAP)
        .track_focus(focus_handle)
        .on_key_down(on_root_key_down)
        .child(content)
        .child(status_bar);

    if let Some(modal) = modal {
        root = root.child(modal);
    }

    let toast_layer = gpui::div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .child(toast_host);

    root = root.child(toast_layer);

    root.into_any_element()
}
