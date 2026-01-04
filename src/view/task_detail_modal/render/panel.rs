use gpui::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::components::action_button::ActionButton;
use crate::components::button::Dropdown;
use crate::components::chip::{Chip, ChipVariant};
use crate::components::confirm_dialog::ConfirmDialog;
use crate::components::field_row::{FieldRow, KvRow};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::section_card::SectionCard;
use crate::components::toast::{ToastGlobal, ToastKind};
use crate::keymap::Command;
use crate::task::model::TaskLinkVm;
use crate::task::{self, TaskDetailVm};
use crate::theme::Theme;
use crate::ui::{DATE_FORMAT, DATE_TIME_FORMAT};

use super::super::TaskDetailModal;
use super::super::annotations::{AnnotationOrigin, AnnotationState};
use super::super::form::{FieldId, TaskForm};
use super::super::state::{ConfirmAction, EditState, ModalFocus, ModalMode};

pub(super) fn render_task_detail_placeholder_panel<OnCloseClick>(
    title: &str,
    message: &str,
    theme: &Theme,
    on_close_click: OnCloseClick,
) -> gpui::AnyElement
where
    OnCloseClick: Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
{
    let on_close_click = Arc::new(on_close_click);
    let on_close_header = on_close_click.clone();
    let close_button = gpui::div()
        .id("task-detail-close")
        .px(gpui::rems(0.5))
        .py(gpui::rems(0.25))
        .rounded_md()
        .text_color(theme.muted)
        .cursor_pointer()
        .hover(|s| s.bg(theme.hover).text_color(theme.foreground))
        .on_mouse_down(gpui::MouseButton::Left, move |event, window, app| {
            (on_close_header)(event, window, app);
        })
        .child("X");

    let header = gpui::div()
        .flex()
        .items_center()
        .justify_between()
        .px(gpui::rems(1.0))
        .py(gpui::rems(0.75))
        .border_b_1()
        .border_color(theme.divider)
        .child(Label::new(title.to_string()).text_color(theme.foreground))
        .child(close_button);

    let body = gpui::div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .items_center()
        .justify_center()
        .text_color(theme.muted)
        .child(message.to_string());

    let on_close_footer = on_close_click.clone();
    let footer_button = ActionButton::new("Close (Esc)")
        .id("task-detail-cancel")
        .on_click(move |event, window, app| {
            (on_close_footer)(event, window, app);
        });

    gpui::div()
        .id("task-detail-panel")
        .flex()
        .flex_col()
        .w(gpui::rems(48.0))
        .h(gpui::rems(40.0))
        .bg(theme.panel)
        .border_1()
        .border_color(theme.border)
        .rounded_md()
        .block_mouse_except_scroll()
        .child(header)
        .child(body)
        .child(
            gpui::div()
                .flex()
                .items_center()
                .justify_end()
                .px(gpui::rems(1.0))
                .py(gpui::rems(0.5))
                .border_t_1()
                .border_color(theme.divider)
                .child(footer_button),
        )
        .into_any_element()
}

pub(super) fn render_task_detail_panel<OnCloseClick>(
    detail: &task::TaskDetailVm,
    mode: ModalMode,
    edit_state: EditState,
    form: &TaskForm,
    errors: &HashMap<FieldId, gpui::SharedString>,
    annotations: &AnnotationState,
    modal_focus: ModalFocus,
    tag_selected: Option<usize>,
    annotation_selected: Option<usize>,
    pending_confirm: &Option<ConfirmAction>,
    annotation_input: &gpui::Entity<Input>,
    description_input: &gpui::Entity<Input>,
    project_input: &gpui::Entity<Input>,
    due_input: &gpui::Entity<Input>,
    tags_input: &gpui::Entity<Input>,
    status_dropdown: &gpui::Entity<Dropdown>,
    priority_dropdown: &gpui::Entity<Dropdown>,
    form_focus_handle: &gpui::FocusHandle,
    scroll_handle: &gpui::ScrollHandle,
    theme: &Theme,
    cx: &mut gpui::Context<TaskDetailModal>,
    on_close_click: OnCloseClick,
) -> gpui::AnyElement
where
    OnCloseClick: Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
{
    let is_editing = mode == ModalMode::Edit;
    let is_navigating = edit_state == EditState::Navigating;
    let status_label = if is_editing {
        form.status.clone().into()
    } else if detail.overview.is_active {
        "Active".to_string()
    } else {
        detail.overview.status.clone().into()
    };

    let priority_label: String = if is_editing {
        form.priority.into()
    } else {
        detail.overview.priority.into()
    };

    let priority_value = if is_editing {
        form.priority
    } else {
        detail.overview.priority
    };

    let status_variant = match status_label.as_str() {
        "Active" => ChipVariant::Success,
        "Pending" => ChipVariant::Warning,
        "Completed" => ChipVariant::Muted,
        "Deleted" => ChipVariant::Danger,
        "Recurring" => ChipVariant::Info,
        _ => ChipVariant::Muted,
    };

    let priority_variant = match priority_value {
        task::TaskPriority::High => ChipVariant::Danger,
        task::TaskPriority::Medium => ChipVariant::Warning,
        task::TaskPriority::Low => ChipVariant::Success,
        task::TaskPriority::None => ChipVariant::Muted,
    };

    let mut badges: Vec<gpui::AnyElement> = vec![
        Chip::new(status_label.clone())
            .variant(status_variant)
            .into_any_element(),
    ];

    if priority_label != "None" {
        badges.push(
            Chip::new(priority_label.clone())
                .variant(priority_variant)
                .into_any_element(),
        );
    }

    if let Some(project) = &detail.overview.project {
        if !is_editing || !form.project.trim().is_empty() {
            let label = if is_editing {
                form.project.trim()
            } else {
                project.as_str()
            };

            if !label.is_empty() {
                badges.push(
                    Chip::new(label.to_string())
                        .variant(ChipVariant::Accent)
                        .into_any_element(),
                );
            }
        }
    } else if is_editing && !form.project.trim().is_empty() {
        badges.push(
            Chip::new(form.project.trim().to_string())
                .variant(ChipVariant::Accent)
                .into_any_element(),
        );
    }

    if is_editing {
        badges.push(
            Chip::new("Editing")
                .variant(ChipVariant::Info)
                .into_any_element(),
        );
    }

    let id_label = detail
        .identity
        .working_id
        .or(detail.identity.id)
        .map(|id| format!("#{}", id))
        .unwrap_or_else(|| format!("#{}", detail.identity.uuid));

    let title_description = if is_editing {
        form.description.clone()
    } else {
        detail.overview.description.clone()
    };
    let title = format!("{} {}", id_label, title_description);

    let on_close_click = Arc::new(on_close_click);
    let header = render_header(title, badges, is_editing, theme, cx, on_close_click.clone());

    let overview_section = render_overview_section(
        detail,
        is_editing,
        form,
        errors,
        modal_focus,
        &status_label,
        &priority_label,
        description_input,
        project_input,
        due_input,
        status_dropdown,
        priority_dropdown,
        theme,
        cx,
    );

    let tags_section = render_tags_section(
        detail,
        is_editing,
        edit_state,
        form,
        modal_focus,
        tag_selected,
        tags_input,
        theme,
        cx,
    );

    let deps_section = render_dependencies_section(detail, theme.foreground);
    let annotations_section = render_annotations_section(
        annotations,
        is_editing,
        edit_state,
        modal_focus,
        annotation_selected,
        annotation_input,
        theme,
        theme.foreground,
        cx,
    );
    let dates_section = render_dates_section(detail, theme.foreground);
    let meta_section = render_metadata_section(detail, theme.foreground);
    let extras_section = render_extras_section(detail, theme.foreground);

    let mut sections = vec![
        overview_section,
        tags_section,
        deps_section,
        annotations_section,
    ];
    sections.push(dates_section);
    sections.push(meta_section);
    if let Some(extras_section) = extras_section {
        sections.push(extras_section);
    }

    let body = gpui::div()
        .id("task-detail-body")
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .track_scroll(scroll_handle)
        .track_focus(form_focus_handle)
        .px(gpui::rems(1.0))
        .py(gpui::rems(0.75))
        .gap_4()
        .children(sections);

    let footer = render_footer(
        detail,
        is_editing,
        is_navigating,
        errors,
        form,
        annotations,
        modal_focus,
        pending_confirm,
        theme,
        cx,
        on_close_click,
    );

    // Panel uses deferred() + anchored() for dropdowns, so clipping is not an issue
    gpui::div()
        .id("task-detail-panel")
        .relative()
        .flex()
        .flex_col()
        .w(gpui::rems(48.0))
        .h(gpui::rems(40.0))
        .bg(theme.panel)
        .border_1()
        .border_color(theme.border)
        .rounded_md()
        .block_mouse_except_scroll()
        .child(header)
        .child(body)
        .child(footer)
        .when(pending_confirm.is_some(), |div| {
            let (title, button_label) = match pending_confirm {
                Some(ConfirmAction::DeleteTag { text, .. }) => {
                    (format!("Delete tag '{}'?", text), "Delete (Enter)")
                }
                Some(ConfirmAction::DeleteAnnotation { text, .. }) => {
                    (format!("Delete annotation '{}'?", text), "Delete (Enter)")
                }
                Some(ConfirmAction::DiscardUnsavedChanges)
                | Some(ConfirmAction::DiscardUnsavedChangesAndClose) => {
                    ("Discard unsaved changes?".to_string(), "Discard (Enter)")
                }
                None => (String::new(), ""),
            };

            let cancel_handler = Arc::new(cx.listener(|modal, _event, _window, cx| {
                modal.dispatch_command(Command::ConfirmNo, None, cx);
            }));

            let confirm_handler = Arc::new(cx.listener(|modal, _event, _window, cx| {
                modal.dispatch_command(Command::ConfirmYes, None, cx);
            }));

            let backdrop_handler = cancel_handler.clone();

            div.child(
                ConfirmDialog::new(title)
                    .hint("Enter/y = confirm • Esc/n = cancel")
                    .cancel("Cancel (Esc)", cancel_handler)
                    .danger(button_label, confirm_handler)
                    .on_backdrop_click(backdrop_handler)
                    .render(theme),
            )
        })
        .into_any_element()
}

fn render_header(
    title: String,
    badges: Vec<gpui::AnyElement>,
    is_editing: bool,
    theme: &Theme,
    cx: &mut gpui::Context<TaskDetailModal>,
    on_close_click: Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>,
) -> gpui::Div {
    let on_close_header = on_close_click.clone();
    let close_button = gpui::div()
        .id("task-detail-close")
        .px(gpui::rems(0.5))
        .py(gpui::rems(0.25))
        .rounded_md()
        .text_color(theme.muted)
        .cursor_pointer()
        .hover(|s| s.bg(theme.hover).text_color(theme.foreground))
        .on_mouse_down(gpui::MouseButton::Left, move |event, window, app| {
            (on_close_header)(event, window, app);
        })
        .child("X");

    let edit_button = if is_editing {
        None
    } else {
        Some(
            gpui::div()
                .id("task-detail-edit")
                .px(gpui::rems(0.6))
                .py(gpui::rems(0.25))
                .rounded_md()
                .border_1()
                .border_color(theme.divider)
                .bg(theme.raised)
                .text_color(theme.foreground)
                .text_xs()
                .cursor_pointer()
                .hover(|s| s.bg(theme.hover))
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(|modal, _event, window, cx| {
                        modal.enter_edit_mode(Some(window), cx);
                    }),
                )
                .child(Label::new("Edit")),
        )
    };

    let mut header_actions = gpui::div().flex().items_center().gap_2();
    if let Some(edit_button) = edit_button {
        header_actions = header_actions.child(edit_button);
    }
    header_actions = header_actions.child(close_button);

    gpui::div()
        .flex()
        .items_start()
        .justify_between()
        .gap_4()
        .px(gpui::rems(1.0))
        .py(gpui::rems(0.75))
        .border_b_1()
        .border_color(theme.divider)
        .child(
            gpui::div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    Label::new(title)
                        .text_color(theme.foreground)
                        .font_weight(gpui::FontWeight::BOLD),
                )
                .child(gpui::div().flex().gap_2().children(badges)),
        )
        .child(header_actions)
}

#[allow(clippy::too_many_arguments)]
fn render_overview_section(
    detail: &TaskDetailVm,
    is_editing: bool,
    form: &TaskForm,
    errors: &HashMap<FieldId, gpui::SharedString>,
    modal_focus: ModalFocus,
    status_label: &str,
    priority_label: &str,
    description_input: &gpui::Entity<Input>,
    project_input: &gpui::Entity<Input>,
    due_input: &gpui::Entity<Input>,
    status_dropdown: &gpui::Entity<Dropdown>,
    priority_dropdown: &gpui::Entity<Dropdown>,
    theme: &Theme,
    cx: &mut gpui::Context<TaskDetailModal>,
) -> SectionCard {
    let due_text = detail
        .dates
        .due
        .map(|d| d.format(DATE_FORMAT).to_string())
        .unwrap_or_else(|| "-".to_string());

    let status_editable = matches!(
        form.status,
        task::TaskStatus::Pending | task::TaskStatus::Completed | task::TaskStatus::Deleted
    );
    let status_value = if is_editing && status_editable {
        let focused = modal_focus == ModalFocus::StatusDropdown;
        let theme_clone = theme.clone();
        let dropdown_clone = status_dropdown.clone();
        gpui::div()
            .relative()
            .when(focused, |d| {
                d.border_2()
                    .border_color(theme_clone.focus_ring)
                    .rounded_md()
                    .p_px()
            })
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |modal, _, window, cx| {
                    modal.set_modal_focus(ModalFocus::StatusDropdown, Some(window), cx);
                    let dropdown = dropdown_clone.clone();

                    cx.defer(move |cx| {
                        dropdown.update(cx, |d, cx| {
                            if !d.is_open() {
                                d.open(cx);
                            }
                        });
                    });
                }),
            )
            .child(status_dropdown.clone())
            .into_any_element()
    } else {
        value_label(status_label.to_string(), theme.foreground)
    };

    let description_value = if is_editing {
        let focused = modal_focus == ModalFocus::Description;
        crate::ui::focus_wrap(description_input.clone(), focused, theme)
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|modal, _, window, cx| {
                    modal.set_modal_focus_and_edit(ModalFocus::Description, Some(window), cx);
                }),
            )
            .into_any_element()
    } else {
        value_label(detail.overview.description.clone(), theme.foreground)
    };

    let project_value = if is_editing {
        let focused = modal_focus == ModalFocus::Project;
        crate::ui::focus_wrap(project_input.clone(), focused, theme)
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|modal, _, window, cx| {
                    modal.set_modal_focus_and_edit(ModalFocus::Project, Some(window), cx);
                }),
            )
            .into_any_element()
    } else {
        value_label(
            detail
                .overview
                .project
                .clone()
                .unwrap_or_else(|| "-".to_string()),
            theme.foreground,
        )
    };

    let priority_value = if is_editing {
        let focused = modal_focus == ModalFocus::PriorityDropdown;
        let theme_clone = theme.clone();
        let dropdown_clone = priority_dropdown.clone();

        gpui::div()
            .relative()
            .when(focused, |d| {
                d.border_2()
                    .border_color(theme_clone.focus_ring)
                    .rounded_md()
                    .p_px()
            })
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |modal, _, window, cx| {
                    modal.set_modal_focus(ModalFocus::PriorityDropdown, Some(window), cx);
                    let dropdown = dropdown_clone.clone();

                    cx.defer(move |cx| {
                        dropdown.update(cx, |d, cx| {
                            if !d.is_open() {
                                d.open(cx);
                            }
                        });
                    });
                }),
            )
            .child(priority_dropdown.clone())
            .into_any_element()
    } else {
        value_label(priority_label.to_string(), theme.foreground)
    };

    let due_value = if is_editing {
        let focused = modal_focus == ModalFocus::Due;

        crate::ui::focus_wrap(due_input.clone(), focused, theme)
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|modal, _, window, cx| {
                    modal.set_modal_focus_and_edit(ModalFocus::Due, Some(window), cx);
                }),
            )
            .into_any_element()
    } else {
        value_label(due_text, theme.foreground)
    };

    let overview_grid = gpui::div()
        .flex()
        .flex_col()
        .gap_2()
        .child(FieldRow::new("Status", status_value))
        .child(
            FieldRow::new("Description", description_value)
                .error(errors.get(&FieldId::Description).cloned()),
        )
        .child(FieldRow::new("Project", project_value))
        .child(FieldRow::new("Priority", priority_value))
        .child(FieldRow::new("Due", due_value).error(errors.get(&FieldId::Due).cloned()));

    let mut overview_section = SectionCard::new("Overview").child(overview_grid);

    if !detail.dependencies.blocked_by.is_empty() || !detail.dependencies.blocking.is_empty() {
        let mut info = Vec::new();

        if !detail.dependencies.blocked_by.is_empty() {
            info.push(format!(
                "Blocked by {} task(s)",
                detail.dependencies.blocked_by.len()
            ));
        }

        if !detail.dependencies.blocking.is_empty() {
            info.push(format!(
                "Blocking {} task(s)",
                detail.dependencies.blocking.len()
            ));
        }

        overview_section = overview_section.child(
            gpui::div()
                .text_sm()
                .text_color(Theme::alpha(theme.foreground, 0.72))
                .child(info.join(" / ")),
        );
    }

    overview_section
}

fn render_tags_section(
    detail: &TaskDetailVm,
    is_editing: bool,
    edit_state: EditState,
    form: &TaskForm,
    modal_focus: ModalFocus,
    tag_selected: Option<usize>,
    tags_input: &gpui::Entity<Input>,
    theme: &Theme,
    cx: &mut gpui::Context<TaskDetailModal>,
) -> SectionCard {
    let tags_content = if is_editing {
        let chips = form.tags.iter().enumerate().map(|(i, tag)| {
            let selected = modal_focus == ModalFocus::TagsInput && tag_selected == Some(i);
            let tag_value = tag.clone();
            let on_remove = Arc::new(cx.listener(move |modal, _event, _window, cx| {
                modal.remove_tag(tag_value.clone(), cx);
            }));

            Chip::new(tag)
                .variant(ChipVariant::Info)
                .selected(selected)
                .removable(on_remove)
                .into_any_element()
        });

        let focused = modal_focus == ModalFocus::TagsInput
            && (tag_selected.is_none() || edit_state == EditState::Editing);

        let tags_input_wrapped = crate::ui::focus_wrap(tags_input.clone(), focused, theme)
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|modal, _, window, cx| {
                    modal.set_modal_focus_and_edit(ModalFocus::TagsInput, Some(window), cx);
                }),
            );

        gpui::div()
            .flex()
            .items_center()
            .gap_2()
            .children(chips)
            .child(gpui::div().min_w(gpui::rems(8.0)).child(tags_input_wrapped))
            .into_any_element()
    } else if detail.tags.tags.is_empty() {
        value_label("-".to_string(), theme.foreground)
    } else {
        let chips = detail
            .tags
            .tags
            .iter()
            .map(|tag| Chip::new(tag).variant(ChipVariant::Info).into_any_element());

        gpui::div()
            .flex()
            .gap_2()
            .children(chips)
            .into_any_element()
    };

    SectionCard::new("Tags").child(
        gpui::div()
            .flex()
            .flex_col()
            .gap_2()
            .child(tags_content)
            .when(!detail.tags.virtual_tags.is_empty(), |div| {
                let vchips = detail.tags.virtual_tags.iter().map(|tag| {
                    Chip::new(tag)
                        .custom(Theme::alpha(theme.muted, 0.2), theme.muted)
                        .into_any_element()
                });
                div.child(gpui::div().flex().gap_2().children(vchips).text_sm())
            }),
    )
}

fn render_dates_section(detail: &TaskDetailVm, value_color: gpui::Rgba) -> SectionCard {
    let format_dt = |value: Option<chrono::DateTime<chrono::Utc>>| {
        value
            .map(|d| d.format(DATE_TIME_FORMAT).to_string())
            .unwrap_or_else(|| "-".to_string())
    };

    let dates_grid = gpui::div()
        .flex()
        .flex_col()
        .gap_2()
        .child(KvRow::new(
            "Entry",
            value_label(format_dt(detail.dates.entry), value_color),
        ))
        .child(KvRow::new(
            "Modified",
            value_label(format_dt(detail.dates.modified), value_color),
        ))
        .child(KvRow::new(
            "Start",
            value_label(format_dt(detail.dates.start), value_color),
        ))
        .child(KvRow::new(
            "End",
            value_label(format_dt(detail.dates.end), value_color),
        ))
        .child(KvRow::new(
            "Scheduled",
            value_label(format_dt(detail.dates.scheduled), value_color),
        ))
        .child(KvRow::new(
            "Wait",
            value_label(format_dt(detail.dates.wait), value_color),
        ))
        .child(KvRow::new(
            "Until",
            value_label(format_dt(detail.dates.until), value_color),
        ));

    SectionCard::new("Dates").child(dates_grid)
}

fn render_metadata_section(detail: &TaskDetailVm, value_color: gpui::Rgba) -> SectionCard {
    let uuid_value = detail.identity.uuid.to_string();
    let id_value = detail
        .identity
        .working_id
        .or(detail.identity.id)
        .map(|id| id.to_string())
        .unwrap_or_else(|| "-".to_string());

    let mut meta_grid = gpui::div()
        .flex()
        .flex_col()
        .gap_2()
        .child(KvRow::new("UUID", value_label(uuid_value, value_color)))
        .child(KvRow::new("ID", value_label(id_value, value_color)));

    if let Some(urgency) = detail.metrics.urgency {
        meta_grid = meta_grid.child(KvRow::new(
            "Urgency",
            value_label(format!("{:.2}", urgency), value_color),
        ));
    }

    SectionCard::new("Metadata").child(meta_grid)
}

fn render_dependencies_section(detail: &TaskDetailVm, value_color: gpui::Rgba) -> SectionCard {
    let format_link = |link: &TaskLinkVm| {
        let id = link
            .id
            .map(|id| format!("#{}", id))
            .unwrap_or_else(|| link.uuid.to_string());
        let status: String = link.status.clone().into();
        format!("{} {} ({})", id, link.description, status)
    };

    let render_links = |links: &[TaskLinkVm]| {
        if links.is_empty() {
            value_label("-".to_string(), value_color)
        } else {
            let items = links.iter().map(|link| {
                Label::new(format_link(link))
                    .text_sm()
                    .text_color(value_color)
                    .into_any_element()
            });

            gpui::div()
                .flex()
                .flex_col()
                .gap_1()
                .min_w_0()
                .children(items)
                .into_any_element()
        }
    };

    let deps_grid = gpui::div()
        .flex()
        .flex_col()
        .gap_2()
        .child(KvRow::new(
            "Depends On",
            render_links(&detail.dependencies.depends_on),
        ))
        .child(KvRow::new(
            "Blocked By",
            render_links(&detail.dependencies.blocked_by),
        ))
        .child(KvRow::new(
            "Blocking",
            render_links(&detail.dependencies.blocking),
        ));

    SectionCard::new("Dependencies").child(deps_grid)
}

#[allow(clippy::too_many_arguments)]
fn render_annotations_section(
    annotations: &AnnotationState,
    is_editing: bool,
    edit_state: EditState,
    modal_focus: ModalFocus,
    annotation_selected: Option<usize>,
    annotation_input: &gpui::Entity<Input>,
    theme: &Theme,
    value_color: gpui::Rgba,
    cx: &mut gpui::Context<TaskDetailModal>,
) -> SectionCard {
    let visible_count = annotations
        .items
        .iter()
        .filter(|item| item.origin != AnnotationOrigin::Deleted)
        .count();

    let mut annotations_content = gpui::div().flex().flex_col().gap_3();
    if is_editing {
        let can_add = !annotations.draft.as_ref().trim().is_empty();
        let add_button = ActionButton::new("Add")
            .enabled(can_add)
            .on_click(cx.listener(|modal, _event, _window, cx| {
                modal.submit_annotation(cx);
            }));

        let focused = modal_focus == ModalFocus::AnnotationsInput
            && (annotation_selected.is_none() || edit_state == EditState::Editing);

        let annotation_input_wrapped =
            crate::ui::focus_wrap(annotation_input.clone(), focused, theme).on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|modal, _, window, cx| {
                    modal.set_modal_focus_and_edit(ModalFocus::AnnotationsInput, Some(window), cx);
                }),
            );

        annotations_content = annotations_content.child(
            gpui::div()
                .flex()
                .items_start()
                .gap_2()
                .child(
                    gpui::div()
                        .flex_1()
                        .min_w_0()
                        .child(annotation_input_wrapped),
                )
                .child(add_button),
        );
    }

    if visible_count == 0 {
        annotations_content = annotations_content.child(
            gpui::div()
                .text_sm()
                .text_color(theme.muted)
                .child("No annotations"),
        );
    } else {
        let mut visible_index = 0;

        let items = annotations.items.iter().filter_map(|annotation| {
            if annotation.origin == AnnotationOrigin::Deleted {
                return None;
            }

            let index = visible_index;

            visible_index += 1;

            let selected =
                modal_focus == ModalFocus::AnnotationsInput && annotation_selected == Some(index);

            let timestamp = annotation.created_at.format(DATE_TIME_FORMAT).to_string();

            let content_for_copy = annotation.text.to_string();

            let copy_action = gpui::div()
                .text_xs()
                .text_color(theme.muted)
                .cursor_pointer()
                .hover(|s| s.text_color(theme.accent))
                .on_mouse_down(gpui::MouseButton::Left, move |_event, _window, app| {
                    app.write_to_clipboard(gpui::ClipboardItem::new_string(
                        content_for_copy.clone(),
                    ));

                    let toast_host = app.global::<ToastGlobal>().host.clone();

                    app.update_entity(&toast_host, |host, cx| {
                        host.push(ToastKind::Info, "Annotation copied", cx);
                    });
                })
                .child(Label::new("Copy"));

            let delete_id = annotation.id;
            let delete_action = if is_editing {
                Some(
                    gpui::div()
                        .text_xs()
                        .text_color(theme.muted)
                        .cursor_pointer()
                        .hover(|s| s.text_color(theme.error))
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(move |modal, _event, _window, cx| {
                                modal.delete_annotation(delete_id, cx);
                            }),
                        )
                        .child(Label::new("×")),
                )
            } else {
                None
            };

            let lines = annotation.text.as_ref().split('\n').map(|line| {
                let text = if line.is_empty() { " " } else { line };
                Label::new(text.to_string())
                    .text_sm()
                    .text_color(value_color)
                    .into_any_element()
            });

            let mut actions = gpui::div().flex().items_center().gap_2().child(copy_action);
            if let Some(delete_action) = delete_action {
                actions = actions.child(delete_action);
            }

            let mut item = gpui::div()
                .flex()
                .flex_col()
                .gap_1()
                .min_w_0()
                .p(gpui::rems(0.5))
                .rounded(gpui::rems(0.25));

            if selected {
                item = item
                    .bg(Theme::alpha(theme.accent, 0.1))
                    .border_2()
                    .border_color(theme.focus_ring);
            } else {
                item = item.bg(theme.raised).border_1().border_color(theme.divider);
            }

            item = item
                .child(
                    gpui::div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(Label::new(timestamp).text_xs().text_color(theme.muted))
                        .child(actions),
                )
                .child(gpui::div().flex().flex_col().gap_1().children(lines));

            if index + 1 < visible_count {
                item = item.child(gpui::div().mt_2().h(gpui::px(1.0)).bg(theme.divider));
            }

            Some(item.into_any_element())
        });

        annotations_content = annotations_content.children(items);
    }

    SectionCard::new("Annotations").child(annotations_content)
}

fn render_extras_section(detail: &TaskDetailVm, value_color: gpui::Rgba) -> Option<SectionCard> {
    if detail.udas.is_empty() {
        return None;
    }

    let rows = detail.udas.iter().map(|(key, value)| {
        KvRow::new(key, value_label(value.clone(), value_color)).into_any_element()
    });

    Some(SectionCard::new("Extras").child(gpui::div().flex().flex_col().gap_2().children(rows)))
}

fn render_footer(
    detail: &TaskDetailVm,
    is_editing: bool,
    is_navigating: bool,
    errors: &HashMap<FieldId, gpui::SharedString>,
    form: &TaskForm,
    annotations: &AnnotationState,
    modal_focus: ModalFocus,
    pending_confirm: &Option<ConfirmAction>,
    theme: &Theme,
    cx: &mut gpui::Context<TaskDetailModal>,
    on_close_click: Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>,
) -> gpui::Div {
    let mut action_row = gpui::div().flex().items_center().gap_2();

    if is_editing {
        let cancel_edit_handler = Arc::new(cx.listener(|modal, _event, window, cx| {
            modal.cancel_edit(Some(window), cx);
        }));

        let save_handler = Arc::new(cx.listener(|modal, _event, _window, cx| {
            modal.submit_edits(cx);
        }));

        let annotations_dirty = annotations
            .items
            .iter()
            .any(|item| item.origin != AnnotationOrigin::Original);

        let can_save = errors.is_empty() && (form.is_dirty(detail) || annotations_dirty);
        let cancel_button =
            ActionButton::new("Cancel (Esc)").on_click(move |event, window, app| {
                (cancel_edit_handler)(event, window, app);
            });

        let save_button = ActionButton::new("Save (Ctrl+S)")
            .enabled(can_save)
            .on_click(move |event, window, app| {
                (save_handler)(event, window, app);
            });

        action_row = action_row.child(cancel_button).child(save_button);
    } else {
        let close_button = ActionButton::new("Close (Esc)").on_click(move |event, window, app| {
            (on_close_click)(event, window, app);
        });

        action_row = action_row.child(close_button);
    }

    let mode_indicator = if is_editing && is_navigating && pending_confirm.is_none() {
        Some(gpui::div().text_color(theme.muted).text_xs().child(format!(
            "Navigate: j/k | Edit: Enter | Focus: {}",
            match modal_focus {
                ModalFocus::Description => "Description",
                ModalFocus::Project => "Project",
                ModalFocus::StatusDropdown => "Status",
                ModalFocus::PriorityDropdown => "Priority",
                ModalFocus::Due => "Due",
                ModalFocus::TagsInput => "Tags",
                ModalFocus::AnnotationsInput => "Annotations",
                ModalFocus::None => "None",
            }
        )))
    } else {
        None
    };

    gpui::div()
        .flex()
        .items_center()
        .justify_between()
        .px(gpui::rems(1.0))
        .py(gpui::rems(0.5))
        .border_t_1()
        .border_color(theme.divider)
        .child(gpui::div().children(mode_indicator))
        .child(action_row)
}

fn value_label(value: String, value_color: gpui::Rgba) -> gpui::AnyElement {
    Label::new(value).text_color(value_color).into_any_element()
}
