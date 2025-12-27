use std::cmp::Ordering;

use gpui::prelude::*;

use crate::{
    components,
    models::FilterState,
    task::{self, TaskFilter, TaskService},
    theme::{self, ActiveTheme},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Id,
    Description,
    Project,
    Due,
    Priority,
    Status,
}

impl SortColumn {
    pub fn label(&self) -> &'static str {
        match self {
            SortColumn::Id => "ID",
            SortColumn::Description => "Description",
            SortColumn::Project => "Project",
            SortColumn::Due => "Due",
            SortColumn::Priority => "Priority",
            SortColumn::Status => "Status",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    pub fn toggle(&self) -> Self {
        match self {
            SortDirection::Asc => SortDirection::Desc,
            SortDirection::Desc => SortDirection::Asc,
        }
    }

    pub fn arrow(&self) -> &'static str {
        match self {
            SortDirection::Asc => "↑",
            SortDirection::Desc => "↓",
        }
    }
}

impl Default for SortDirection {
    fn default() -> Self {
        Self::Desc
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SortState {
    pub column: SortColumn,
    pub direction: SortDirection,
}

impl Default for SortState {
    fn default() -> Self {
        Self {
            column: SortColumn::Priority,
            direction: SortDirection::Desc,
        }
    }
}

pub struct PaginationState {
    current_page: usize,
    page_size: usize,
    total_items: usize,
}

impl Default for PaginationState {
    fn default() -> Self {
        Self {
            current_page: 1,
            page_size: 20,
            total_items: 0,
        }
    }
}

impl PaginationState {
    pub fn new(current_page: usize, page_size: usize, total_items: usize) -> Self {
        Self {
            current_page,
            page_size,
            total_items,
        }
    }

    pub fn total_items(&mut self, total_items: usize) {
        self.total_items = total_items;
    }

    pub fn page_size(&mut self, page_size: usize) {
        self.page_size = page_size;
    }

    pub fn current_page(&mut self, current_page: usize) {
        self.current_page = current_page;
    }

    #[inline]
    pub fn can_next(&self) -> bool {
        self.current_page < self.total_items / self.page_size + 1
    }

    #[inline]
    pub fn can_previous(&self) -> bool {
        self.current_page > 1
    }

    pub fn next_page(&mut self) {
        if self.current_page < self.total_items / self.page_size + 1 {
            self.current_page += 1;
        }
    }

    pub fn previous_page(&mut self) {
        if self.current_page > 1 {
            self.current_page -= 1;
        }
    }

    pub fn first_item_index(&self) -> usize {
        (self.current_page - 1) * self.page_size
    }

    pub fn last_item_index(&self) -> usize {
        (self.first_item_index() + self.page_size).min(self.total_items)
    }

    pub fn last_item_display(&self) -> usize {
        self.last_item_index()
    }
}

pub struct TaskRow {
    pub uuid: uuid::Uuid,
    pub id_display: String,
    pub description: String,
    pub project: String,
    pub due: String,
    pub priority: String,
    pub status: String,
    pub is_due_today: bool,
    pub is_overdue: bool,
    pub is_active: bool,
}

impl TaskRow {
    fn truncate(desc: &str, max_len: usize) -> String {
        if desc.len() <= max_len {
            desc.to_string()
        } else {
            format!("{}...", &desc[..max_len - 3])
        }
    }

    fn format_date(due: &Option<chrono::DateTime<chrono::Utc>>, is_today: bool) -> String {
        match due {
            None => "-".to_string(),
            Some(dt) => {
                if is_today {
                    "Today".to_string()
                } else {
                    dt.format("%d-%m-%Y").to_string()
                }
            }
        }
    }
}

impl From<&task::Task> for TaskRow {
    fn from(value: &task::Task) -> Self {
        let status = if value.is_active {
            "Active".to_string()
        } else {
            value.status.clone().into()
        };

        Self {
            uuid: value.uuid,
            id_display: value.working_id.unwrap_or(0).to_string(),
            description: Self::truncate(&value.description, 50),
            project: value.project.clone().unwrap_or(String::new()),
            due: Self::format_date(&value.due, value.is_due_today()),
            priority: value.priority.into(),
            status,
            is_due_today: value.is_due_today(),
            is_overdue: value.is_overdue(),
            is_active: value.is_active,
        }
    }
}

pub struct TaskTable {
    id: gpui::ElementId,
    filter_state: gpui::Entity<FilterState>,
    cached_tasks: Vec<task::Task>,
    cached_rows: Vec<TaskRow>,
    sort_state: SortState,
    pagination: PaginationState,
    selected_page_idx: Option<usize>,
    selected_global_idx: Option<usize>,
    need_reload: bool,
}

impl TaskTable {
    pub fn new(
        id: impl Into<gpui::ElementId>,
        filter_state: gpui::Entity<FilterState>,
        _cx: &mut gpui::Context<Self>,
    ) -> Self {
        Self {
            id: id.into(),
            filter_state,
            cached_tasks: vec![],
            cached_rows: vec![],
            sort_state: SortState::default(),
            pagination: PaginationState::default(),
            selected_page_idx: None,
            selected_global_idx: None,
            need_reload: true,
        }
    }

    pub fn set_sort(&mut self, column: SortColumn, cx: &mut gpui::Context<Self>) {
        if self.sort_state.column == column {
            self.sort_state.direction = self.sort_state.direction.toggle();
        } else {
            self.sort_state.column = column;
            self.sort_state.direction = SortDirection::Desc;
        }
        self.apply_sort();
        self.recalculate_rows();
        cx.notify();
    }

    fn apply_sort(&mut self) {
        let direction = self.sort_state.direction;
        let column = self.sort_state.column;

        self.cached_tasks.sort_by(|a, b| {
            let ordering = match column {
                SortColumn::Id => a.working_id.unwrap_or(0).cmp(&b.working_id.unwrap_or(0)),
                SortColumn::Description => a.description.cmp(&b.description),
                SortColumn::Project => {
                    let a_proj = a.project.as_deref().unwrap_or("");
                    let b_proj = b.project.as_deref().unwrap_or("");
                    a_proj.cmp(b_proj)
                }
                SortColumn::Due => match (&a.due, &b.due) {
                    (Some(a_due), Some(b_due)) => a_due.cmp(b_due),
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => Ordering::Equal,
                },
                SortColumn::Priority => {
                    let a_order: usize = a.priority.into();
                    let b_order: usize = b.priority.into();
                    a_order.cmp(&b_order)
                }
                SortColumn::Status => {
                    let a_status: String = a.status.clone().into();
                    let b_status: String = b.status.clone().into();
                    a_status.cmp(&b_status)
                }
            };

            match direction {
                SortDirection::Asc => ordering,
                SortDirection::Desc => ordering.reverse(),
            }
        });
    }

    fn get_current_page_rows(&self) -> &[TaskRow] {
        let start = self.pagination.first_item_index();
        let end = self.pagination.last_item_index();

        &self.cached_rows[start..end]
    }

    pub fn reload_tasks(&mut self, task_service: &mut TaskService, cx: &mut gpui::Context<Self>) {
        let filter_state = self.filter_state.read(cx).clone();

        let task_filter = TaskFilter::from(&filter_state);

        let filtered_tasks = task_service
            .get_filtered_tasks(&task_filter)
            .unwrap_or_else(|e| {
                log::error!("[TaskTable] Failed to load filtered tasks: {}", e);
                vec![]
            });

        self.cached_tasks = filtered_tasks;
        self.apply_sort();
        self.pagination.total_items(self.cached_tasks.len());
        self.pagination.current_page(1);
        self.selected_global_idx = None;
        self.selected_page_idx = None;

        self.recalculate_rows();

        self.need_reload = false;

        cx.notify();
    }

    fn recalculate_rows(&mut self) {
        self.cached_rows = self.cached_tasks.iter().map(TaskRow::from).collect();
    }

    fn priority_color(&self, row: &TaskRow, cx: &gpui::Context<Self>) -> theme::Color {
        let theme = cx.theme();

        match row.priority.as_str() {
            "High" => theme.high,
            "Medium" => theme.medium,
            "Low" => theme.low,
            _ => theme.foreground,
        }
    }

    fn due_color(&self, row: &TaskRow, cx: &gpui::Context<Self>) -> theme::Color {
        let theme = cx.theme();

        if row.is_due_today {
            theme.accent
        } else if row.is_overdue {
            theme.error
        } else {
            theme.foreground
        }
    }

    fn status_color(&self, row: &TaskRow, cx: &gpui::Context<Self>) -> theme::Color {
        let theme = cx.theme();

        match row.status.as_str() {
            "Active" => theme.success,
            "Pending" => theme.warning,
            "Completed" => theme.muted,
            "Deleted" => theme.error,
            "Recurring" => theme.info,
            _ => theme.muted,
        }
    }

    pub fn select_row(&mut self, idx: usize, cx: &mut gpui::Context<Self>) {
        self.selected_page_idx = Some(idx);
        self.selected_global_idx = Some(self.pagination.first_item_index() + idx);
        cx.notify();
    }

    pub fn go_previous_page(&mut self, cx: &mut gpui::Context<Self>) {
        self.pagination.previous_page();
        cx.notify();
    }

    pub fn go_next_page(&mut self, cx: &mut gpui::Context<Self>) {
        self.pagination.next_page();
        cx.notify();
    }

    fn render_header_column(
        &self,
        column: SortColumn,
        id: &'static str,
        cx: &gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme = cx.theme();
        let is_sorted = self.sort_state.column == column;
        let arrow = if is_sorted {
            self.sort_state.direction.arrow()
        } else {
            ""
        };

        gpui::div()
            .id(id)
            .flex()
            .items_center()
            .gap_1()
            .cursor_pointer()
            .hover(|s| s.text_color(theme.foreground))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |table, _, _, cx| {
                    table.set_sort(column, cx);
                }),
            )
            .child(
                components::label::Label::new(column.label()).text_color(if is_sorted {
                    theme.accent
                } else {
                    theme.muted
                }),
            )
            .when(!arrow.is_empty(), |div| {
                div.child(components::label::Label::new(arrow).text_color(theme.accent))
            })
    }

    fn render_header(&self, cx: &gpui::Context<Self>) -> gpui::Div {
        let theme = cx.theme();

        gpui::div()
            .flex()
            .flex_shrink_0()
            .items_center()
            .gap_2()
            .px_4()
            .py_1()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.panel)
            .text_sm()
            .font_weight(gpui::FontWeight::MEDIUM)
            .child(
                gpui::div()
                    .min_w(gpui::rems(3.0))
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(components::label::Label::new(" ").text_color(theme.muted))
                    .child(self.render_header_column(SortColumn::Id, "header-id", cx)),
            )
            .child(
                gpui::div()
                    .flex_1()
                    .min_w(gpui::rems(10.0))
                    .child(self.render_header_column(SortColumn::Description, "header-desc", cx)),
            )
            .child(
                gpui::div()
                    .w(gpui::rems(10.0))
                    .child(self.render_header_column(SortColumn::Project, "header-project", cx)),
            )
            .child(
                gpui::div()
                    .w(gpui::rems(7.0))
                    .child(self.render_header_column(SortColumn::Due, "header-due", cx)),
            )
            .child(
                gpui::div()
                    .w(gpui::rems(5.0))
                    .child(self.render_header_column(SortColumn::Priority, "header-priority", cx)),
            )
            .child(
                gpui::div()
                    .w(gpui::rems(6.0))
                    .child(self.render_header_column(SortColumn::Status, "header-status", cx)),
            )
    }

    fn render_row(&self, idx: usize, row: &TaskRow, cx: &gpui::Context<Self>) -> gpui::Div {
        let theme = cx.theme();
        let selected = self.selected_page_idx == Some(idx);

        gpui::div()
            .flex()
            .items_center()
            .gap_2()
            .px_4()
            .py_1()
            .border_b_1()
            .border_color(theme.border)
            .text_color(theme.foreground)
            .when(selected, |d| {
                d.bg(theme.selection).text_color(theme.selection_foreground)
            })
            .when(!selected, |d| d.hover(|s| s.bg(theme.panel)))
            .cursor_pointer()
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |table, _, _, cx| table.select_row(idx, cx)),
            )
            .child(
                gpui::div()
                    .min_w(gpui::rems(3.0))
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        components::label::Label::new(if selected { ">" } else { " " })
                            .text_color(theme.accent),
                    )
                    .child(components::label::Label::new(row.id_display.clone())),
            )
            .child(
                gpui::div()
                    .flex_1()
                    .min_w(gpui::rems(10.0))
                    .overflow_x_hidden()
                    .child(
                        components::label::Label::new(row.description.clone())
                            .text_ellipsis()
                            .whitespace_nowrap(),
                    ),
            )
            .child(
                gpui::div().w(gpui::rems(10.0)).overflow_x_hidden().child(
                    components::label::Label::new(row.project.clone())
                        .text_color(theme.muted)
                        .text_ellipsis()
                        .whitespace_nowrap(),
                ),
            )
            .child(gpui::div().w(gpui::rems(7.0)).child(
                components::label::Label::new(row.due.clone()).text_color(self.due_color(row, cx)),
            ))
            .child(
                gpui::div().w(gpui::rems(5.0)).child(
                    components::label::Label::new(row.priority.clone())
                        .text_color(self.priority_color(row, cx))
                        .font_weight(gpui::FontWeight::BOLD),
                ),
            )
            .child(
                gpui::div().w(gpui::rems(6.0)).child(
                    components::label::Label::new(row.status.clone())
                        .text_color(self.status_color(row, cx)),
                ),
            )
    }

    fn render_footer(&self, cx: &gpui::Context<Self>) -> gpui::Div {
        let theme = cx.theme();
        let can_prev = self.pagination.can_previous();
        let can_next = self.pagination.can_next();
        let pages = (self.pagination.total_items + self.pagination.page_size - 1)
            / self.pagination.page_size.max(1);

        gpui::div()
            .flex()
            .flex_shrink_0()
            .justify_between()
            .items_center()
            .px_4()
            .py_2()
            .border_t_1()
            .border_color(theme.border)
            .bg(theme.panel)
            .text_sm()
            .child(
                components::label::Label::new(format!(
                    "Showing {}-{} of {}",
                    self.pagination.first_item_index() + 1,
                    self.pagination.last_item_display(),
                    self.pagination.total_items
                ))
                .text_color(theme.muted),
            )
            .child(
                gpui::div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        components::label::Label::new(format!(
                            "Page {} of {}",
                            self.pagination.current_page,
                            pages.max(1)
                        ))
                        .text_color(theme.muted),
                    )
                    .child(
                        gpui::div()
                            .flex()
                            .gap_1()
                            .child(
                                gpui::div()
                                    .id("prev-btn")
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .text_color(if can_prev {
                                        theme.foreground
                                    } else {
                                        theme.muted
                                    })
                                    .when(can_prev, |d| {
                                        d.cursor_pointer().hover(|s| s.bg(theme.selection))
                                    })
                                    .when(!can_prev, |d| d.cursor_not_allowed())
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(|table, _, _, cx| table.go_previous_page(cx)),
                                    )
                                    .child(components::label::Label::new("← Prev")),
                            )
                            .child(
                                gpui::div()
                                    .id("next-btn")
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .text_color(if can_next {
                                        theme.foreground
                                    } else {
                                        theme.muted
                                    })
                                    .when(can_next, |d| {
                                        d.cursor_pointer().hover(|s| s.bg(theme.selection))
                                    })
                                    .when(!can_next, |d| d.cursor_not_allowed())
                                    .on_mouse_down(
                                        gpui::MouseButton::Left,
                                        cx.listener(|table, _, _, cx| table.go_next_page(cx)),
                                    )
                                    .child(components::label::Label::new("Next →")),
                            ),
                    ),
            )
    }
}

impl gpui::Render for TaskTable {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme = cx.theme();

        let panel = components::panel::Panel::new(self.id.clone());

        if self.need_reload {
            return panel
                .flex()
                .flex_col()
                .size_full()
                .items_center()
                .justify_center()
                .child(components::label::Label::new("Loading...").text_color(theme.text));
        }

        let current_page = self.get_current_page_rows();
        let rows: Vec<gpui::Div> = current_page
            .iter()
            .enumerate()
            .map(|(index, row)| self.render_row(index, row, cx))
            .collect();

        panel
            .flex()
            .flex_col()
            .size_full()
            .overflow_hidden()
            .bg(theme.background)
            .child(self.render_header(cx))
            .child(
                gpui::div()
                    .id("task-table-content")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(gpui::div().flex().flex_col().children(rows)),
            )
            .child(self.render_footer(cx))
    }
}
