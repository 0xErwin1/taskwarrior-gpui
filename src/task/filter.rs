use std::collections::HashSet;

use chrono::{DateTime, NaiveDate, Utc};

use super::model::{Task, TaskPriority, TaskStatus};
use crate::models::{DueFilter, FilterState, PriorityFilter, StatusFilter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TagsFilterMode {
    #[default]
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DueDateFilter {
    Overdue,
    Today,
    ThisWeek,
    NoDate,
    Before(DateTime<Utc>),
    After(DateTime<Utc>),
    OnDate(NaiveDate),
}

#[derive(Debug, Clone, Default)]
pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub project: Option<String>,
    pub project_include_children: bool,
    pub tags: HashSet<String>,
    pub tags_mode: TagsFilterMode,
    pub priority: Option<TaskPriority>,
    pub due_filter: Option<DueDateFilter>,
    pub search_text: Option<String>,
    pub is_active: Option<bool>,
    pub is_blocked: Option<bool>,
}

impl TaskFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_project(mut self, project: String, include_children: bool) -> Self {
        self.project = Some(project);
        self.project_include_children = include_children;
        self
    }

    pub fn with_tags(mut self, tags: HashSet<String>, mode: TagsFilterMode) -> Self {
        self.tags = tags;
        self.tags_mode = mode;
        self
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn with_due(mut self, filter: DueDateFilter) -> Self {
        self.due_filter = Some(filter);
        self
    }

    pub fn with_search(mut self, text: String) -> Self {
        if !text.is_empty() {
            self.search_text = Some(text.to_lowercase());
        }
        self
    }
}

impl From<&FilterState> for TaskFilter {
    fn from(state: &FilterState) -> Self {
        let mut filter = Self::new();

        filter.status = match state.status_filter {
            StatusFilter::All => None,
            StatusFilter::Pending => Some(TaskStatus::Pending),
            StatusFilter::Completed => Some(TaskStatus::Completed),
            StatusFilter::Waiting => Some(TaskStatus::Pending),
            StatusFilter::Deleted => Some(TaskStatus::Deleted),
        };

        if let Some(ref project) = state.selected_project {
            filter.project = Some(project.clone());
            filter.project_include_children = true;
        }

        if !state.active_tags.is_empty() {
            filter.tags = state.active_tags.clone();
            filter.tags_mode = TagsFilterMode::And;
        }

        filter.priority = match state.priority_filter {
            PriorityFilter::All => None,
            PriorityFilter::High => Some(TaskPriority::High),
            PriorityFilter::Medium => Some(TaskPriority::Medium),
            PriorityFilter::Low => Some(TaskPriority::Low),
            PriorityFilter::None => Some(TaskPriority::None),
        };

        filter.due_filter = match state.due_filter {
            DueFilter::All => None,
            DueFilter::Overdue => Some(DueDateFilter::Overdue),
            DueFilter::Today => Some(DueDateFilter::Today),
            DueFilter::ThisWeek => Some(DueDateFilter::ThisWeek),
            DueFilter::NoDate => Some(DueDateFilter::NoDate),
            DueFilter::OnDate(date) => Some(DueDateFilter::OnDate(date)),
        };

        if !state.search_text.is_empty() {
            filter.search_text = Some(state.search_text.to_lowercase());
        }

        filter
    }
}

impl From<FilterState> for TaskFilter {
    fn from(state: FilterState) -> Self {
        Self::from(&state)
    }
}

impl TaskFilter {
    pub fn matches(&self, task: &Task) -> bool {
        if let Some(status) = &self.status {
            match status {
                TaskStatus::Pending => {
                    if !matches!(task.status, TaskStatus::Pending) {
                        return false;
                    }
                    if task.wait.map(|w| w > Utc::now()).unwrap_or(false) {
                        return false;
                    }
                }
                _ => {
                    if &task.status != status {
                        return false;
                    }
                }
            }
        }

        if let Some(project) = &self.project {
            match &task.project {
                None => return false,
                Some(task_project) => {
                    if self.project_include_children {
                        if !task_project.starts_with(project) {
                            return false;
                        }
                    } else if task_project != project {
                        return false;
                    }
                }
            }
        }

        if !self.tags.is_empty() {
            match self.tags_mode {
                TagsFilterMode::And => {
                    for tag in &self.tags {
                        if !task.tags.contains(tag) {
                            return false;
                        }
                    }
                }
                TagsFilterMode::Or => {
                    let has_any = self.tags.iter().any(|t| task.tags.contains(t));
                    if !has_any {
                        return false;
                    }
                }
            }
        }

        if let Some(priority) = &self.priority {
            if &task.priority != priority {
                return false;
            }
        }

        if let Some(due_filter) = &self.due_filter {
            match due_filter {
                DueDateFilter::Overdue => {
                    if !task.is_overdue() {
                        return false;
                    }
                }
                DueDateFilter::Today => {
                    if !task.is_due_today() {
                        return false;
                    }
                }
                DueDateFilter::ThisWeek => {
                    let is_due_this_week = task
                        .due
                        .map(|d| {
                            let now = Utc::now();
                            let week_end = now + chrono::Duration::days(7);
                            d >= now && d <= week_end
                        })
                        .unwrap_or(false);

                    if !is_due_this_week {
                        return false;
                    }
                }
                DueDateFilter::NoDate => {
                    if task.due.is_some() {
                        return false;
                    }
                }
                DueDateFilter::Before(dt) => {
                    if task.due.map(|d| d >= *dt).unwrap_or(true) {
                        return false;
                    }
                }
                DueDateFilter::After(dt) => {
                    if task.due.map(|d| d <= *dt).unwrap_or(true) {
                        return false;
                    }
                }
                DueDateFilter::OnDate(date) => {
                    if !task.due.map(|d| d.date_naive() == *date).unwrap_or(false) {
                        return false;
                    }
                }
            }
        }

        if let Some(search) = &self.search_text {
            let desc_match = task.description.to_lowercase().contains(search);
            let proj_match = task
                .project
                .as_ref()
                .map(|p| p.to_lowercase().contains(search))
                .unwrap_or(false);
            let tag_match = task.tags.iter().any(|t| t.to_lowercase().contains(search));

            if !desc_match && !proj_match && !tag_match {
                return false;
            }
        }

        if let Some(is_active) = self.is_active {
            if task.is_active != is_active {
                return false;
            }
        }

        if let Some(is_blocked) = self.is_blocked {
            if task.is_blocked != is_blocked {
                return false;
            }
        }

        true
    }

    pub fn apply(&self, tasks: &[Task]) -> Vec<Task> {
        tasks.iter().filter(|t| self.matches(t)).cloned().collect()
    }
}
