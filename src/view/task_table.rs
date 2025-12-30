use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::sync::Arc;

use gpui::prelude::*;

use crate::{
    components::{
        self,
        button::{Dropdown, DropdownItem},
        input::Input,
    },
    keymap::{Command, CommandDispatcher},
    models::{DueFilter, FilterState, PriorityFilter, StatusFilter},
    task::{self, TaskFilter, TaskService, TaskSummary},
    theme::{self, ActiveTheme},
    ui::{
        DATE_FORMAT, TABLE_FILTER_BAR_INITIAL_HEIGHT, TABLE_MAX_DESCRIPTION_LENGTH, priority_badge,
        table_col_desc_min_width, table_col_due_width, table_col_id_width,
        table_col_priority_width, table_col_project_width, table_col_status_width,
    },
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
    const COLUMN_ORDER: [Self; 6] = [
        Self::Id,
        Self::Description,
        Self::Project,
        Self::Due,
        Self::Priority,
        Self::Status,
    ];

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

    fn next(self) -> Self {
        let idx = Self::COLUMN_ORDER
            .iter()
            .position(|&c| c == self)
            .unwrap_or(0);
        Self::COLUMN_ORDER[(idx + 1) % Self::COLUMN_ORDER.len()]
    }

    fn prev(self) -> Self {
        let idx = Self::COLUMN_ORDER
            .iter()
            .position(|&c| c == self)
            .unwrap_or(0);
        Self::COLUMN_ORDER[(idx + Self::COLUMN_ORDER.len() - 1) % Self::COLUMN_ORDER.len()]
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
                    dt.format(DATE_FORMAT).to_string()
                }
            }
        }
    }
}

impl From<&task::TaskSummary> for TaskRow {
    fn from(value: &task::TaskSummary) -> Self {
        let status = if value.is_active {
            "Active".to_string()
        } else {
            value.status.clone().into()
        };

        Self {
            uuid: value.uuid,
            id_display: value.working_id.unwrap_or(0).to_string(),
            description: Self::truncate(&value.description, TABLE_MAX_DESCRIPTION_LENGTH),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterBarFocus {
    None,
    SearchInput,
    StatusDropdown,
    PriorityDropdown,
    DueDropdown,
}

impl Default for FilterBarFocus {
    fn default() -> Self {
        Self::None
    }
}

pub struct TaskTable {
    id: gpui::ElementId,
    filter_state: gpui::Entity<FilterState>,
    cached_tasks: Vec<task::TaskSummary>,
    cached_rows: Vec<TaskRow>,
    sort_state: SortState,
    pagination: PaginationState,
    selected_page_idx: Option<usize>,
    selected_global_idx: Option<usize>,
    need_reload: bool,
    filter_bar_height: gpui::Pixels,
    search_input: gpui::Entity<Input>,
    status_dropdown: gpui::Entity<Dropdown>,
    priority_dropdown: gpui::Entity<Dropdown>,
    due_dropdown: gpui::Entity<Dropdown>,
    filter_bar_focus: FilterBarFocus,
    filter_bar_focus_handle: gpui::FocusHandle,
    focused_header: Option<SortColumn>,
    header_focus_handle: gpui::FocusHandle,
}

impl TaskTable {
    pub fn new(
        id: impl Into<gpui::ElementId>,
        filter_state: gpui::Entity<FilterState>,
        cx: &mut gpui::Context<Self>,
    ) -> Self {
        let search_input = {
            let filter_state = filter_state.clone();
            cx.new(|cx| {
                Input::new("filter-search", cx, "Search...").with_on_change(Arc::new(
                    move |value: &str, cx: &mut gpui::Context<Input>| {
                        cx.update_entity(&filter_state, |filter, cx| {
                            filter.search_text = value.to_string();
                            cx.notify();
                        });
                    },
                ))
            })
        };

        let status_items = StatusFilter::all_variants()
            .iter()
            .map(|status| DropdownItem::new(status.as_str()))
            .collect::<Vec<_>>();
        let status_dropdown = {
            let filter_state = filter_state.clone();
            cx.new(|_cx| {
                Dropdown::new("filter-status")
                    .items(status_items)
                    .label_prefix("Status")
                    .selected_index(StatusFilter::default().to_index())
                    .on_select(Arc::new(move |index, _item, cx| {
                        let selected = StatusFilter::from_index(index);
                        cx.update_entity(&filter_state, |filter, cx| {
                            filter.status_filter = selected;
                            cx.notify();
                        });
                    }))
            })
        };

        let priority_items = PriorityFilter::all_variants()
            .iter()
            .map(|priority| DropdownItem::new(priority.as_str()))
            .collect::<Vec<_>>();
        let priority_dropdown = {
            let filter_state = filter_state.clone();
            cx.new(|_cx| {
                Dropdown::new("filter-priority")
                    .items(priority_items)
                    .label_prefix("Priority")
                    .selected_index(PriorityFilter::default().to_index())
                    .on_select(Arc::new(move |index, _item, cx| {
                        let selected = PriorityFilter::from_index(index);
                        cx.update_entity(&filter_state, |filter, cx| {
                            filter.priority_filter = selected;
                            cx.notify();
                        });
                    }))
            })
        };

        let due_dropdown = {
            let filter_state = filter_state.clone();
            cx.new(|_cx| {
                Dropdown::new("filter-due")
                    .items(vec![DropdownItem::with_value("All", "all")])
                    .label_prefix("Due")
                    .selected_index(0)
                    .on_select(Arc::new(move |_index, item, cx| {
                        let selected =
                            DueFilter::from_value(item.value.as_ref()).unwrap_or(DueFilter::All);
                        cx.update_entity(&filter_state, |filter, cx| {
                            filter.due_filter = selected;
                            cx.notify();
                        });
                    }))
            })
        };

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
            filter_bar_height: TABLE_FILTER_BAR_INITIAL_HEIGHT,
            search_input,
            status_dropdown,
            priority_dropdown,
            due_dropdown,
            filter_bar_focus: FilterBarFocus::None,
            filter_bar_focus_handle: cx.focus_handle(),
            focused_header: None,
            header_focus_handle: cx.focus_handle(),
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

    pub fn reload_tasks_from_all(
        &mut self,
        all_tasks: Vec<task::TaskSummary>,
        cx: &mut gpui::Context<Self>,
    ) {
        let filter_state = self.filter_state.read(cx).clone();

        let task_filter = TaskFilter::from(&filter_state);
        let mut due_filter = task_filter.clone();
        due_filter.due_filter = None;

        let filtered_tasks = task_filter.apply(&all_tasks);
        let due_tasks = due_filter.apply(&all_tasks);

        self.cached_tasks = filtered_tasks;
        self.apply_sort();
        self.pagination.total_items(self.cached_tasks.len());
        self.pagination.current_page(1);
        self.selected_global_idx = None;
        self.selected_page_idx = None;

        self.recalculate_rows();

        if !self.cached_rows.is_empty() {
            self.selected_page_idx = Some(0);
            self.selected_global_idx = Some(0);
        }

        self.sync_filter_dropdowns(&due_tasks, &filter_state, cx);

        self.need_reload = false;

        cx.notify();
    }

    fn recalculate_rows(&mut self) {
        self.cached_rows = self.cached_tasks.iter().map(TaskRow::from).collect();
    }

    fn sync_filter_dropdowns(
        &mut self,
        due_tasks: &[task::TaskSummary],
        filter_state: &FilterState,
        cx: &mut gpui::Context<Self>,
    ) {
        let status_index = filter_state.status_filter.to_index();
        self.status_dropdown.update(cx, |dropdown, cx| {
            dropdown.set_selected_index(status_index, cx);
        });

        let priority_index = filter_state.priority_filter.to_index();
        self.priority_dropdown.update(cx, |dropdown, cx| {
            dropdown.set_selected_index(priority_index, cx);
        });

        let mut due_items = Self::build_due_items(due_tasks);
        let selected_key = filter_state.due_filter.value_key();
        let mut selected_index = due_items
            .iter()
            .position(|item| item.value.as_ref() == selected_key);

        if selected_index.is_none() && filter_state.due_filter != DueFilter::All {
            if let Some(item) = Self::due_item_from_filter(&filter_state.due_filter) {
                due_items.push(item);
                selected_index = Some(due_items.len() - 1);
            }
        }

        self.due_dropdown.update(cx, |dropdown, cx| {
            dropdown.set_items(due_items, cx);
            if let Some(index) = selected_index {
                dropdown.set_selected_index(index, cx);
            }
        });
    }

    fn build_due_items(tasks: &[task::TaskSummary]) -> Vec<DropdownItem> {
        let now = chrono::Utc::now();
        let today = now.date_naive();
        let week_end = now + chrono::Duration::days(7);

        let mut dates = BTreeSet::new();
        let mut has_no_date = false;
        let mut has_overdue = false;
        let mut has_today = false;
        let mut has_this_week = false;

        for task in tasks {
            match task.due {
                None => {
                    has_no_date = true;
                }
                Some(due) => {
                    let date = due.date_naive();
                    dates.insert(date);
                    if due < now {
                        has_overdue = true;
                    }
                    if date == today {
                        has_today = true;
                    }
                    if due >= now && due <= week_end {
                        has_this_week = true;
                    }
                }
            }
        }

        let mut items = vec![DropdownItem::with_value("All", "all")];
        if has_no_date {
            items.push(DropdownItem::with_value("No Date", "none"));
        }
        if has_overdue {
            items.push(DropdownItem::with_value("Overdue", "overdue"));
        }
        if has_today {
            items.push(DropdownItem::with_value("Today", "today"));
        }
        if has_this_week {
            items.push(DropdownItem::with_value("This Week", "this_week"));
        }

        for date in dates {
            if date == today {
                continue;
            }
            let label = Self::format_due_label(date);
            let value = format!("date:{}", date.format(DATE_FORMAT));
            items.push(DropdownItem::with_value(label, value));
        }

        items
    }

    fn due_item_from_filter(filter: &DueFilter) -> Option<DropdownItem> {
        match filter {
            DueFilter::All => None,
            DueFilter::OnDate(date) => Some(DropdownItem::with_value(
                Self::format_due_label(*date),
                filter.value_key(),
            )),
            _ => Some(DropdownItem::with_value(filter.label(), filter.value_key())),
        }
    }

    fn format_due_label(date: chrono::NaiveDate) -> String {
        let today = chrono::Utc::now().date_naive();
        if date == today {
            "Today".to_string()
        } else {
            date.format(DATE_FORMAT).to_string()
        }
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

    pub fn select_next_row(&mut self, cx: &mut gpui::Context<Self>) {
        if self.cached_rows.is_empty() {
            return;
        }

        if self.selected_page_idx.is_none() {
            self.selected_page_idx = Some(0);
            self.selected_global_idx = Some(self.pagination.first_item_index());
            cx.notify();
            return;
        }

        let current_page_idx = self.selected_page_idx.unwrap();
        let current_global_idx = self.selected_global_idx.unwrap();

        if current_global_idx + 1 >= self.cached_rows.len() {
            return;
        }

        let next_global_idx = current_global_idx + 1;
        let page_last_idx = self.pagination.last_item_index() - 1;

        if next_global_idx > page_last_idx {
            self.pagination.next_page();
            self.selected_page_idx = Some(0);
            self.selected_global_idx = Some(next_global_idx);
        } else {
            self.selected_page_idx = Some(current_page_idx + 1);
            self.selected_global_idx = Some(next_global_idx);
        }

        cx.notify();
    }

    pub fn select_previous_row(&mut self, cx: &mut gpui::Context<Self>) {
        if self.cached_rows.is_empty() {
            return;
        }

        if self.selected_page_idx.is_none() {
            let last_global_idx = self.cached_rows.len() - 1;
            let last_page = (self.cached_rows.len() + self.pagination.page_size - 1)
                / self.pagination.page_size;

            self.pagination.current_page(last_page);

            let page_first_idx = self.pagination.first_item_index();
            let page_idx = last_global_idx - page_first_idx;

            self.selected_page_idx = Some(page_idx);
            self.selected_global_idx = Some(last_global_idx);
            cx.notify();
            return;
        }

        let current_page_idx = self.selected_page_idx.unwrap();
        let current_global_idx = self.selected_global_idx.unwrap();

        if current_global_idx == 0 {
            return;
        }

        let prev_global_idx = current_global_idx - 1;
        let page_first_idx = self.pagination.first_item_index();

        if prev_global_idx < page_first_idx {
            self.pagination.previous_page();
            let new_page_size = (self.pagination.last_item_index()
                - self.pagination.first_item_index())
            .min(self.pagination.page_size);
            self.selected_page_idx = Some(new_page_size - 1);
            self.selected_global_idx = Some(prev_global_idx);
        } else {
            self.selected_page_idx = Some(current_page_idx.saturating_sub(1));
            self.selected_global_idx = Some(prev_global_idx);
        }

        cx.notify();
    }

    pub fn select_first_row(&mut self, cx: &mut gpui::Context<Self>) {
        if self.cached_rows.is_empty() {
            return;
        }

        self.pagination.current_page(1);
        self.selected_page_idx = Some(0);
        self.selected_global_idx = Some(0);
        cx.notify();
    }

    pub fn select_last_row(&mut self, cx: &mut gpui::Context<Self>) {
        if self.cached_rows.is_empty() {
            return;
        }

        let last_global_idx = self.cached_rows.len() - 1;
        let last_page =
            (self.cached_rows.len() + self.pagination.page_size - 1) / self.pagination.page_size;

        self.pagination.current_page(last_page);

        let page_first_idx = self.pagination.first_item_index();
        let page_idx = last_global_idx - page_first_idx;

        self.selected_page_idx = Some(page_idx);
        self.selected_global_idx = Some(last_global_idx);
        cx.notify();
    }

    pub fn clear_selection(&mut self, cx: &mut gpui::Context<Self>) {
        self.selected_page_idx = None;
        self.selected_global_idx = None;
        cx.notify();
    }

    pub fn selected_task_uuid(&self) -> Option<uuid::Uuid> {
        self.selected_global_idx
            .and_then(|idx| self.cached_tasks.get(idx))
            .map(|task| task.uuid)
    }

    pub fn focus_search_input(&mut self, window: &mut gpui::Window, cx: &mut gpui::Context<Self>) {
        self.filter_bar_focus = FilterBarFocus::SearchInput;
        self.search_input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
        cx.notify();
    }

    pub fn blur_search_input(&mut self, window: &mut gpui::Window, cx: &mut gpui::Context<Self>) {
        window.focus(&self.filter_bar_focus_handle);
        cx.notify();
    }

    pub fn clear_search_input(&mut self, cx: &mut gpui::Context<Self>) {
        self.search_input.update(cx, |input, cx| {
            input.clear(cx);
        });
    }

    pub fn reset_dropdowns(&mut self, cx: &mut gpui::Context<Self>) {
        self.status_dropdown.update(cx, |dropdown, cx| {
            dropdown.set_selected_index(1, cx);
        });
        self.priority_dropdown.update(cx, |dropdown, cx| {
            dropdown.set_selected_index(0, cx);
        });
        self.due_dropdown.update(cx, |dropdown, cx| {
            dropdown.set_selected_index(0, cx);
        });
    }

    pub fn get_filter_bar_focus(&self) -> FilterBarFocus {
        self.filter_bar_focus
    }

    pub fn set_filter_bar_focus(&mut self, focus: FilterBarFocus, cx: &mut gpui::Context<Self>) {
        self.filter_bar_focus = focus;
        cx.notify();
    }

    pub fn focus_table_headers(&mut self, window: &mut gpui::Window, cx: &mut gpui::Context<Self>) {
        self.focused_header = Some(SortColumn::Id);
        window.focus(&self.header_focus_handle);
        cx.notify();
    }

    pub fn blur_table_headers(&mut self, cx: &mut gpui::Context<Self>) {
        self.focused_header = None;
        cx.notify();
    }

    pub fn header_move_next(&mut self, cx: &mut gpui::Context<Self>) {
        self.focused_header = Some(self.focused_header.unwrap_or(SortColumn::Id).next());
        cx.notify();
    }

    pub fn header_move_prev(&mut self, cx: &mut gpui::Context<Self>) {
        self.focused_header = Some(self.focused_header.unwrap_or(SortColumn::Id).prev());
        cx.notify();
    }

    pub fn header_cycle_sort_order(&mut self, cx: &mut gpui::Context<Self>) {
        if let Some(column) = self.focused_header {
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
    }

    fn close_all_dropdowns(&mut self, cx: &mut gpui::Context<Self>) {
        self.status_dropdown.update(cx, |d, cx| d.close(cx));
        self.priority_dropdown.update(cx, |d, cx| d.close(cx));
        self.due_dropdown.update(cx, |d, cx| d.close(cx));
    }

    pub fn focus_filter_next(&mut self, cx: &mut gpui::Context<Self>) {
        use FilterBarFocus::*;
        self.close_all_dropdowns(cx);

        self.filter_bar_focus = match self.filter_bar_focus {
            None | SearchInput => StatusDropdown,
            StatusDropdown => PriorityDropdown,
            PriorityDropdown => DueDropdown,
            DueDropdown => SearchInput,
        };
        cx.notify();
    }

    pub fn focus_filter_prev(&mut self, cx: &mut gpui::Context<Self>) {
        use FilterBarFocus::*;
        self.close_all_dropdowns(cx);

        self.filter_bar_focus = match self.filter_bar_focus {
            None | SearchInput => DueDropdown,
            DueDropdown => PriorityDropdown,
            PriorityDropdown => StatusDropdown,
            StatusDropdown => SearchInput,
        };
        cx.notify();
    }

    pub fn toggle_focused_dropdown(&mut self, cx: &mut gpui::Context<Self>) {
        use FilterBarFocus::*;
        let toggle = |d: &gpui::Entity<Dropdown>, cx: &mut gpui::Context<Self>| {
            d.update(cx, |d, cx| {
                if d.is_open() {
                    d.accept_selection(cx);
                } else {
                    d.open(cx);
                }
            });
            cx.notify();
        };
        match self.filter_bar_focus {
            StatusDropdown => toggle(&self.status_dropdown, cx),
            PriorityDropdown => toggle(&self.priority_dropdown, cx),
            DueDropdown => toggle(&self.due_dropdown, cx),
            _ => {}
        }
    }

    pub fn select_next_dropdown_option(&mut self, cx: &mut gpui::Context<Self>) {
        use FilterBarFocus::*;
        let select_next = |d: &gpui::Entity<Dropdown>, cx: &mut gpui::Context<Self>| {
            d.update(cx, |d, cx| {
                if !d.is_open() {
                    d.open(cx);
                }
                d.select_next_item(cx);
            });
            cx.notify();
        };
        match self.filter_bar_focus {
            StatusDropdown => select_next(&self.status_dropdown, cx),
            PriorityDropdown => select_next(&self.priority_dropdown, cx),
            DueDropdown => select_next(&self.due_dropdown, cx),
            _ => {}
        }
    }

    pub fn select_prev_dropdown_option(&mut self, cx: &mut gpui::Context<Self>) {
        use FilterBarFocus::*;
        let select_prev = |d: &gpui::Entity<Dropdown>, cx: &mut gpui::Context<Self>| {
            d.update(cx, |d, cx| {
                if !d.is_open() {
                    d.open(cx);
                }
                d.select_prev_item(cx);
            });
            cx.notify();
        };
        match self.filter_bar_focus {
            StatusDropdown => select_prev(&self.status_dropdown, cx),
            PriorityDropdown => select_prev(&self.priority_dropdown, cx),
            DueDropdown => select_prev(&self.due_dropdown, cx),
            _ => {}
        }
    }

    pub fn blur_filter_bar(&mut self, cx: &mut gpui::Context<Self>) {
        self.close_all_dropdowns(cx);
        self.filter_bar_focus = FilterBarFocus::None;
        cx.notify();
    }

    pub fn get_active_filter_context(&self) -> Option<crate::keymap::ContextId> {
        use FilterBarFocus::*;
        match self.filter_bar_focus {
            SearchInput => Some(crate::keymap::ContextId::TextInput),
            StatusDropdown | PriorityDropdown | DueDropdown => {
                Some(crate::keymap::ContextId::FilterBar)
            }
            FilterBarFocus::None => Option::None,
        }
    }

    fn handle_clear_filters(&mut self, cx: &mut gpui::Context<Self>) {
        self.search_input.update(cx, |input, cx| {
            input.clear(cx);
        });
        self.filter_state.update(cx, |filter, cx| {
            filter.clear();
            cx.notify();
        });
    }

    fn render_filter_bar(&self, cx: &gpui::Context<Self>) -> impl gpui::IntoElement {
        let theme = cx.theme();
        let filter = self.filter_state.read(cx);
        let has_filters = filter.has_active_filters();
        let view = cx.entity().clone();

        let clear_button = {
            let mut btn = gpui::div()
                .id("clear-all-filters")
                .flex_shrink_0()
                .min_w(gpui::rems(5.5))
                .px_2()
                .py_1()
                .rounded_md()
                .text_sm();

            if has_filters {
                btn = btn
                    .text_color(theme.error)
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.hover))
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|table, _, _, cx| {
                            table.handle_clear_filters(cx);
                        }),
                    )
                    .child("✕ Clear");
            } else {
                btn = btn.child(gpui::div().opacity(0.0).child("✕ Clear"));
            }

            btn
        };

        use FilterBarFocus::*;

        let status_has_focus = matches!(self.filter_bar_focus, StatusDropdown);
        let priority_has_focus = matches!(self.filter_bar_focus, PriorityDropdown);
        let due_has_focus = matches!(self.filter_bar_focus, DueDropdown);

        let status_wrapper = gpui::div()
            .min_w(gpui::rems(11.0))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _event, _window, cx| {
                    this.filter_bar_focus = FilterBarFocus::StatusDropdown;
                    cx.notify();
                }),
            )
            .when(status_has_focus, |div| {
                div.border_2()
                    .border_color(theme.focus_ring)
                    .rounded_md()
                    .p_px()
            })
            .child(self.status_dropdown.clone());

        let priority_wrapper = gpui::div()
            .min_w(gpui::rems(10.0))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _event, _window, cx| {
                    this.filter_bar_focus = FilterBarFocus::PriorityDropdown;
                    cx.notify();
                }),
            )
            .when(priority_has_focus, |div| {
                div.border_2()
                    .border_color(theme.focus_ring)
                    .rounded_md()
                    .p_px()
            })
            .child(self.priority_dropdown.clone());

        let due_wrapper = gpui::div()
            .min_w(gpui::rems(8.0))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _event, _window, cx| {
                    this.filter_bar_focus = FilterBarFocus::DueDropdown;
                    cx.notify();
                }),
            )
            .when(due_has_focus, |div| {
                div.border_2()
                    .border_color(theme.focus_ring)
                    .rounded_md()
                    .p_px()
            })
            .child(self.due_dropdown.clone());

        let bar = gpui::div()
            .id("filter-bar")
            .flex()
            .gap_3()
            .items_center()
            .px_4()
            .py_2()
            .child(
                gpui::div()
                    .flex_1()
                    .min_w(gpui::rems(12.0))
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _event, _window, cx| {
                            this.close_all_dropdowns(cx);
                            this.filter_bar_focus = FilterBarFocus::SearchInput;
                            cx.notify();
                        }),
                    )
                    .child(self.search_input.clone()),
            )
            .child(status_wrapper)
            .child(priority_wrapper)
            .child(due_wrapper)
            .child(clear_button);

        gpui::div()
            .child(bar)
            .on_children_prepainted(move |bounds, _, cx| {
                let Some(bounds) = bounds.first() else {
                    return;
                };
                let height = bounds.size.height;
                cx.update_entity(&view, |table, cx| {
                    if table.filter_bar_height != height {
                        table.filter_bar_height = height;
                        cx.notify();
                    }
                });
            })
    }

    fn render_header_column(
        &self,
        column: SortColumn,
        id: &'static str,
        cx: &gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme = cx.theme();
        let is_sorted = self.sort_state.column == column;
        let is_focused = self.focused_header == Some(column);
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
            .when(is_focused, |div| {
                div.border_2()
                    .border_color(theme.focus_ring)
                    .rounded_md()
                    .px_1()
                    .mx(gpui::px(-1.0))
            })
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
            .track_focus(&self.header_focus_handle)
            .flex()
            .flex_shrink_0()
            .items_center()
            .gap_2()
            .px_4()
            .py_2()
            .bg(theme.raised)
            .border_b_1()
            .border_color(theme.divider)
            .text_sm()
            .font_weight(gpui::FontWeight::MEDIUM)
            .child(
                gpui::div()
                    .min_w(table_col_id_width())
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(components::label::Label::new(" ").text_color(theme.muted))
                    .child(self.render_header_column(SortColumn::Id, "header-id", cx)),
            )
            .child(
                gpui::div()
                    .flex_1()
                    .min_w(table_col_desc_min_width())
                    .child(self.render_header_column(SortColumn::Description, "header-desc", cx)),
            )
            .child(
                gpui::div()
                    .w(table_col_project_width())
                    .child(self.render_header_column(SortColumn::Project, "header-project", cx)),
            )
            .child(
                gpui::div()
                    .w(table_col_due_width())
                    .child(self.render_header_column(SortColumn::Due, "header-due", cx)),
            )
            .child(
                gpui::div()
                    .w(table_col_priority_width())
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
        let row_uuid = row.uuid;

        gpui::div()
            .flex()
            .items_center()
            .gap_2()
            .px_4()
            .py_1()
            .border_b_1()
            .border_color(theme.divider)
            .text_color(theme.foreground)
            .when(selected, |d| {
                d.bg(theme.selection).text_color(theme.selection_foreground)
            })
            .when(!selected, |d| d.hover(|s| s.bg(theme.hover)))
            .cursor_pointer()
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |table, event: &gpui::MouseDownEvent, _window, cx| {
                    table.select_row(idx, cx);
                    if event.click_count >= 2 {
                        cx.emit(TaskTableEvent::OpenTask(row_uuid));
                    }
                }),
            )
            .child(
                gpui::div()
                    .min_w(table_col_id_width())
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
                    .min_w(table_col_desc_min_width())
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
                gpui::div()
                    .w(table_col_priority_width())
                    .child(priority_badge(&row.priority, theme)),
            )
            .child(
                gpui::div().w(table_col_status_width()).child(
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
            .border_color(theme.divider)
            .bg(theme.raised)
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
                                    .border_1()
                                    .border_color(theme.divider)
                                    .text_color(if can_prev {
                                        theme.foreground
                                    } else {
                                        theme.disabled_fg
                                    })
                                    .when(can_prev, |d| {
                                        d.cursor_pointer().hover(|s| s.bg(theme.hover))
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
                                    .border_1()
                                    .border_color(theme.divider)
                                    .text_color(if can_next {
                                        theme.foreground
                                    } else {
                                        theme.disabled_fg
                                    })
                                    .when(can_next, |d| {
                                        d.cursor_pointer().hover(|s| s.bg(theme.hover))
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

impl CommandDispatcher for TaskTable {
    fn dispatch(&mut self, command: Command, cx: &mut gpui::Context<Self>) -> bool {
        match command {
            Command::SelectNextRow => {
                self.select_next_row(cx);
                true
            }
            Command::SelectPrevRow => {
                self.select_previous_row(cx);
                true
            }
            Command::SelectFirstRow => {
                self.select_first_row(cx);
                true
            }
            Command::SelectLastRow => {
                self.select_last_row(cx);
                true
            }
            Command::NextPage => {
                self.go_next_page(cx);
                true
            }
            Command::PrevPage => {
                self.go_previous_page(cx);
                true
            }
            Command::ClearSelection => {
                self.clear_selection(cx);
                true
            }
            Command::FocusFilterNext | Command::FocusFilterPrev => false,
            Command::ToggleDropdown => {
                self.toggle_focused_dropdown(cx);
                true
            }
            Command::SelectNextOption => {
                self.select_next_dropdown_option(cx);
                true
            }
            Command::SelectPrevOption => {
                self.select_prev_dropdown_option(cx);
                true
            }
            Command::BlurInput => {
                self.blur_filter_bar(cx);
                true
            }
            Command::ExpandProject | Command::CollapseProject => false,
            _ => false,
        }
    }
}

pub enum TaskTableEvent {
    OpenTask(uuid::Uuid),
}

impl gpui::EventEmitter<TaskTableEvent> for TaskTable {}

impl gpui::Render for TaskTable {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme = cx.theme();

        let panel = components::panel::Panel::new(self.id.clone())
            .flex()
            .flex_col();

        if self.need_reload {
            return gpui::div().size_full().child(
                panel
                    .flex()
                    .flex_col()
                    .size_full()
                    .items_center()
                    .justify_center()
                    .child(
                        components::label::Label::new("Loading...").text_color(theme.foreground),
                    ),
            );
        }

        let current_page = self.get_current_page_rows();
        let rows: Vec<gpui::Div> = current_page
            .iter()
            .enumerate()
            .map(|(index, row)| self.render_row(index, row, cx))
            .collect();

        let filter_bar = self.render_filter_bar(cx);
        let header = self.render_header(cx);
        let footer = self.render_footer(cx);

        let body = gpui::div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .overflow_hidden()
            .bg(theme.background)
            .child(gpui::div().h(self.filter_bar_height))
            .child(gpui::div().h_4())
            .child(header)
            .child(
                gpui::div()
                    .id("task-table-content")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .child(gpui::div().flex().flex_col().children(rows)),
            )
            .child(footer);

        gpui::div()
            .size_full()
            .track_focus(&self.filter_bar_focus_handle)
            .child(
                panel.child(body).child(
                    gpui::div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .child(filter_bar),
                ),
            )
    }
}
