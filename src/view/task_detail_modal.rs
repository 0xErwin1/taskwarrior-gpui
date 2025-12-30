use gpui::prelude::*;
use std::sync::Arc;

use crate::components::label::Label;
use crate::components::modal::ModalFrame;
use crate::task::model::TaskLinkVm;
use crate::task::{self, TaskDetailState};
use crate::theme::Theme;
use crate::ui::{DATE_FORMAT, DATE_TIME_FORMAT};

pub fn render_task_detail_modal(
    detail_state: &TaskDetailState,
    focus_handle: &gpui::FocusHandle,
    scroll_handle: &gpui::ScrollHandle,
    theme: &Theme,
    on_close_out: impl Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    on_close_click: impl Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> gpui::AnyElement {
    let panel = match detail_state {
        TaskDetailState::Ready(detail) => {
            render_task_detail_panel(detail, scroll_handle, theme, on_close_click)
        }
        TaskDetailState::Error(_, message) => {
            render_task_detail_placeholder_panel("Task Details", message, theme, on_close_click)
        }
        TaskDetailState::Loading(_) | TaskDetailState::Idle => {
            render_task_detail_placeholder_panel(
                "Task Details",
                "Loading task...",
                theme,
                on_close_click,
            )
        }
    };

    ModalFrame::new("task-detail-modal", focus_handle.clone(), theme.backdrop)
        .panel(panel)
        .on_close(on_close_out)
        .into_any_element()
}

fn render_task_detail_placeholder_panel<OnCloseClick>(
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
    let footer_button = gpui::div()
        .id("task-detail-cancel")
        .px(gpui::rems(0.75))
        .py(gpui::rems(0.35))
        .rounded_md()
        .border_1()
        .border_color(theme.divider)
        .bg(theme.raised)
        .text_color(theme.foreground)
        .cursor_pointer()
        .hover(|s| s.bg(theme.hover))
        .on_mouse_down(gpui::MouseButton::Left, move |event, window, app| {
            (on_close_footer)(event, window, app);
        })
        .child(Label::new("Cancel (Esc)"));

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

fn render_task_detail_panel<OnCloseClick>(
    detail: &task::TaskDetailVm,
    scroll_handle: &gpui::ScrollHandle,
    theme: &Theme,
    on_close_click: OnCloseClick,
) -> gpui::AnyElement
where
    OnCloseClick: Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
{
    let status_label = if detail.overview.is_active {
        "Active".to_string()
    } else {
        detail.overview.status.clone().into()
    };

    let priority_label: String = detail.overview.priority.into();

    let chip = |label: &str, bg: gpui::Rgba, fg: gpui::Rgba| {
        gpui::div()
            .px(gpui::rems(0.5))
            .py(gpui::rems(0.125))
            .rounded(gpui::rems(0.25))
            .bg(bg)
            .text_color(fg)
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .child(label.to_string())
    };

    let status_color = match status_label.as_str() {
        "Active" => theme.success,
        "Pending" => theme.warning,
        "Completed" => theme.muted,
        "Deleted" => theme.error,
        "Recurring" => theme.info,
        _ => theme.muted,
    };

    let priority_color = match detail.overview.priority {
        task::TaskPriority::High => theme.high,
        task::TaskPriority::Medium => theme.medium,
        task::TaskPriority::Low => theme.low,
        task::TaskPriority::None => theme.muted,
    };

    let mut badges = vec![chip(
        &status_label,
        Theme::alpha(status_color, 0.18),
        status_color,
    )];

    if priority_label != "None" {
        badges.push(chip(
            &priority_label,
            Theme::alpha(priority_color, 0.18),
            priority_color,
        ));
    }

    if let Some(project) = &detail.overview.project {
        badges.push(chip(
            project,
            Theme::alpha(theme.accent, 0.15),
            theme.accent,
        ));
    }

    let id_label = detail
        .identity
        .working_id
        .or(detail.identity.id)
        .map(|id| format!("#{}", id))
        .unwrap_or_else(|| format!("#{}", detail.identity.uuid));

    let title = format!("{} {}", id_label, detail.overview.description);

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
        .child(close_button);

    let label_color = Theme::alpha(theme.foreground, 0.72);
    let value_color = theme.foreground;
    let section_title_color = Theme::alpha(theme.foreground, 0.88);

    let value_label = |value: String| Label::new(value).text_color(value_color).into_any_element();

    let kv_row = |label: &str, value: gpui::AnyElement| {
        gpui::div()
            .flex()
            .items_start()
            .gap_3()
            .child(
                Label::new(label.to_string())
                    .text_color(label_color)
                    .text_sm()
                    .w(gpui::rems(10.0)),
            )
            .child(gpui::div().flex_1().min_w_0().child(value))
    };

    let section_header = |title: &str| {
        Label::new(title.to_uppercase())
            .text_sm()
            .text_color(section_title_color)
            .font_weight(gpui::FontWeight::BOLD)
    };

    let section = |title: &str, content: gpui::Div| {
        gpui::div()
            .flex()
            .flex_col()
            .gap_2()
            .bg(theme.raised)
            .border_1()
            .border_color(theme.divider)
            .rounded_md()
            .px(gpui::rems(0.75))
            .py(gpui::rems(0.5))
            .child(section_header(title))
            .child(content)
    };

    let due_text = detail
        .dates
        .due
        .map(|d| d.format(DATE_FORMAT).to_string())
        .unwrap_or_else(|| "-".to_string());

    let overview_grid = gpui::div()
        .flex()
        .flex_col()
        .gap_2()
        .child(kv_row("Status", value_label(status_label.clone())))
        .child(kv_row(
            "Description",
            value_label(detail.overview.description.clone()),
        ))
        .child(kv_row(
            "Project",
            value_label(
                detail
                    .overview
                    .project
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
            ),
        ))
        .child(kv_row("Priority", value_label(priority_label.clone())))
        .child(kv_row("Due", value_label(due_text)));

    let mut overview_section = section("Overview", overview_grid);

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
                .text_color(label_color)
                .child(info.join(" / ")),
        );
    }

    let tags_content = if detail.tags.tags.is_empty() {
        value_label("-".to_string())
    } else {
        let chips =
            detail.tags.tags.iter().map(|tag| {
                chip(tag, Theme::alpha(theme.info, 0.18), theme.info).into_any_element()
            });
        gpui::div()
            .flex()
            .gap_2()
            .children(chips)
            .into_any_element()
    };

    let tags_section = section(
        "Tags",
        gpui::div()
            .flex()
            .flex_col()
            .gap_2()
            .child(tags_content)
            .when(!detail.tags.virtual_tags.is_empty(), |div| {
                let vchips = detail.tags.virtual_tags.iter().map(|tag| {
                    chip(tag, Theme::alpha(theme.muted, 0.2), theme.muted).into_any_element()
                });
                div.child(gpui::div().flex().gap_2().children(vchips).text_sm())
            }),
    );

    let format_dt = |value: Option<chrono::DateTime<chrono::Utc>>| {
        value
            .map(|d| d.format(DATE_TIME_FORMAT).to_string())
            .unwrap_or_else(|| "-".to_string())
    };

    let dates_grid = gpui::div()
        .flex()
        .flex_col()
        .gap_2()
        .child(kv_row("Entry", value_label(format_dt(detail.dates.entry))))
        .child(kv_row(
            "Modified",
            value_label(format_dt(detail.dates.modified)),
        ))
        .child(kv_row("Start", value_label(format_dt(detail.dates.start))))
        .child(kv_row("End", value_label(format_dt(detail.dates.end))))
        .child(kv_row(
            "Scheduled",
            value_label(format_dt(detail.dates.scheduled)),
        ))
        .child(kv_row("Wait", value_label(format_dt(detail.dates.wait))))
        .child(kv_row("Until", value_label(format_dt(detail.dates.until))));

    let dates_section = section("Dates", dates_grid);

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
        .child(kv_row("UUID", value_label(uuid_value)))
        .child(kv_row("ID", value_label(id_value)));

    if let Some(urgency) = detail.metrics.urgency {
        meta_grid = meta_grid.child(kv_row("Urgency", value_label(format!("{:.2}", urgency))));
    }

    let meta_section = section("Metadata", meta_grid);

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
            value_label("-".to_string())
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
        .child(kv_row(
            "Depends On",
            render_links(&detail.dependencies.depends_on),
        ))
        .child(kv_row(
            "Blocked By",
            render_links(&detail.dependencies.blocked_by),
        ))
        .child(kv_row(
            "Blocking",
            render_links(&detail.dependencies.blocking),
        ));

    let deps_section = section("Dependencies", deps_grid);

    let mut sections = vec![overview_section, tags_section, deps_section];

    if !detail.annotations.is_empty() {
        let items = detail.annotations.iter().map(|annotation| {
            gpui::div()
                .flex()
                .flex_col()
                .gap_1()
                .min_w_0()
                .child(
                    Label::new(annotation.entry.format(DATE_TIME_FORMAT).to_string())
                        .text_xs()
                        .text_color(theme.muted),
                )
                .child(
                    Label::new(annotation.content.clone())
                        .text_sm()
                        .text_color(value_color),
                )
                .into_any_element()
        });

        let annotations_section = section(
            "Annotations",
            gpui::div().flex().flex_col().gap_2().children(items),
        );
        sections.push(annotations_section);
    }

    sections.push(dates_section);
    sections.push(meta_section);

    if !detail.udas.is_empty() {
        let rows = detail
            .udas
            .iter()
            .map(|(key, value)| kv_row(key, value_label(value.clone())).into_any_element());
        let udas_section = section(
            "Extras",
            gpui::div().flex().flex_col().gap_2().children(rows),
        );
        sections.push(udas_section);
    }

    let body = gpui::div()
        .id("task-detail-body")
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .track_scroll(scroll_handle)
        .px(gpui::rems(1.0))
        .py(gpui::rems(0.75))
        .gap_4()
        .children(sections);

    let on_close_footer = on_close_click.clone();
    let footer = gpui::div()
        .flex()
        .items_center()
        .justify_end()
        .px(gpui::rems(1.0))
        .py(gpui::rems(0.5))
        .border_t_1()
        .border_color(theme.divider)
        .child(
            gpui::div()
                .id("task-detail-cancel")
                .px(gpui::rems(0.75))
                .py(gpui::rems(0.35))
                .rounded_md()
                .border_1()
                .border_color(theme.divider)
                .bg(theme.raised)
                .text_color(theme.foreground)
                .cursor_pointer()
                .hover(|s| s.bg(theme.hover))
                .on_mouse_down(gpui::MouseButton::Left, move |event, window, app| {
                    (on_close_footer)(event, window, app);
                })
                .child(Label::new("Cancel (Esc)")),
        );

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
        .child(footer)
        .into_any_element()
}
