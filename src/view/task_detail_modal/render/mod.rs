use gpui::prelude::*;

use crate::components::button::Dropdown;
use crate::components::input::Input;
use crate::components::modal::ModalFrame;
use crate::theme::ActiveTheme;

use super::TaskDetailModal;
use super::state::TaskModalState;

mod panel;

pub(super) fn render_task_detail_modal(
    state: &TaskModalState,
    annotation_input: &gpui::Entity<Input>,
    description_input: &gpui::Entity<Input>,
    project_input: &gpui::Entity<Input>,
    due_input: &gpui::Entity<Input>,
    tags_input: &gpui::Entity<Input>,
    status_dropdown: &gpui::Entity<Dropdown>,
    priority_dropdown: &gpui::Entity<Dropdown>,
    focus_handle: &gpui::FocusHandle,
    form_focus_handle: &gpui::FocusHandle,
    scroll_handle: &gpui::ScrollHandle,
    cx: &mut gpui::Context<TaskDetailModal>,
    on_close_out: impl Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    on_close_click: impl Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> gpui::AnyElement {
    let theme = cx.theme().clone();
    let panel = if let Some(message) = state.error.as_ref() {
        panel::render_task_detail_placeholder_panel(
            "Task Details",
            message.as_ref(),
            &theme,
            on_close_click,
        )
    } else if state.loading || state.original.is_none() {
        panel::render_task_detail_placeholder_panel(
            "Task Details",
            "Loading task...",
            &theme,
            on_close_click,
        )
    } else if let Some(detail) = state.original.as_ref() {
        panel::render_task_detail_panel(
            detail,
            state.mode,
            state.is_create,
            state.edit_state,
            &state.form,
            &state.errors,
            &state.annotations,
            state.modal_focus,
            state.tag_selected,
            state.annotation_selected,
            &state.pending_confirm,
            annotation_input,
            description_input,
            project_input,
            due_input,
            tags_input,
            status_dropdown,
            priority_dropdown,
            form_focus_handle,
            scroll_handle,
            &theme,
            cx,
            on_close_click,
        )
    } else {
        panel::render_task_detail_placeholder_panel(
            "Task Details",
            "Loading task...",
            &theme,
            on_close_click,
        )
    };

    ModalFrame::new("task-detail-modal", focus_handle.clone(), theme.backdrop)
        .panel(panel)
        .on_close(on_close_out)
        .into_any_element()
}
