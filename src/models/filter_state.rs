use std::collections::HashSet;

use chrono::NaiveDate;

#[derive(Debug, Clone, Default)]
pub struct FilterState {
    pub selected_project: Option<String>,
    pub active_tags: HashSet<String>,
    pub search_text: String,
    pub status_filter: StatusFilter,
    pub priority_filter: PriorityFilter,
    pub due_filter: DueFilter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusFilter {
    All,
    Pending,
    Completed,
    Waiting,
    Deleted,
}

impl Default for StatusFilter {
    fn default() -> Self {
        Self::Pending
    }
}

impl StatusFilter {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Pending => "Pending",
            Self::Completed => "Completed",
            Self::Waiting => "Waiting",
            Self::Deleted => "Deleted",
        }
    }

    pub fn all_variants() -> &'static [Self] {
        &[
            Self::All,
            Self::Pending,
            Self::Completed,
            Self::Waiting,
            Self::Deleted,
        ]
    }

    pub fn from_index(index: usize) -> Self {
        Self::all_variants().get(index).copied().unwrap_or_default()
    }

    pub fn to_index(&self) -> usize {
        Self::all_variants()
            .iter()
            .position(|v| v == self)
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriorityFilter {
    All,
    High,
    Medium,
    Low,
    None,
}

impl Default for PriorityFilter {
    fn default() -> Self {
        Self::All
    }
}

impl PriorityFilter {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::High => "High (H)",
            Self::Medium => "Medium (M)",
            Self::Low => "Low (L)",
            Self::None => "None",
        }
    }

    pub fn all_variants() -> &'static [Self] {
        &[Self::All, Self::High, Self::Medium, Self::Low, Self::None]
    }

    pub fn from_index(index: usize) -> Self {
        Self::all_variants().get(index).copied().unwrap_or_default()
    }

    pub fn to_index(&self) -> usize {
        Self::all_variants()
            .iter()
            .position(|v| v == self)
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DueFilter {
    All,
    Overdue,
    Today,
    ThisWeek,
    NoDate,
    OnDate(NaiveDate),
}

impl Default for DueFilter {
    fn default() -> Self {
        Self::All
    }
}

impl DueFilter {
    pub fn all_variants() -> &'static [Self] {
        &[
            Self::All,
            Self::Overdue,
            Self::Today,
            Self::ThisWeek,
            Self::NoDate,
        ]
    }

    pub fn from_index(index: usize) -> Self {
        Self::all_variants().get(index).copied().unwrap_or_default()
    }

    pub fn to_index(&self) -> usize {
        Self::all_variants()
            .iter()
            .position(|v| v == self)
            .unwrap_or(0)
    }

    pub fn label(&self) -> String {
        match self {
            Self::All => "All".to_string(),
            Self::Overdue => "Overdue".to_string(),
            Self::Today => "Today".to_string(),
            Self::ThisWeek => "This Week".to_string(),
            Self::NoDate => "No Date".to_string(),
            Self::OnDate(date) => date.format("%d-%m-%Y").to_string(),
        }
    }

    pub fn value_key(&self) -> String {
        match self {
            Self::All => "all".to_string(),
            Self::Overdue => "overdue".to_string(),
            Self::Today => "today".to_string(),
            Self::ThisWeek => "this_week".to_string(),
            Self::NoDate => "none".to_string(),
            Self::OnDate(date) => format!("date:{}", date.format("%Y-%m-%d")),
        }
    }

    pub fn from_value(value: &str) -> Option<Self> {
        match value {
            "all" => Some(Self::All),
            "overdue" => Some(Self::Overdue),
            "today" => Some(Self::Today),
            "this_week" => Some(Self::ThisWeek),
            "none" => Some(Self::NoDate),
            _ => value.strip_prefix("date:").and_then(|date| {
                NaiveDate::parse_from_str(date, "%Y-%m-%d")
                    .ok()
                    .map(Self::OnDate)
            }),
        }
    }
}

impl FilterState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn select_project(&mut self, project: Option<String>) {
        self.selected_project = project;
    }

    pub fn toggle_tag(&mut self, tag: String) {
        if self.active_tags.contains(&tag) {
            self.active_tags.remove(&tag);
        } else {
            self.active_tags.insert(tag);
        }
    }

    pub fn clear(&mut self) {
        self.selected_project = None;
        self.active_tags.clear();
        self.search_text.clear();
        self.status_filter = StatusFilter::default();
        self.priority_filter = PriorityFilter::default();
        self.due_filter = DueFilter::default();
    }

    pub fn has_active_filters(&self) -> bool {
        self.selected_project.is_some()
            || !self.active_tags.is_empty()
            || !self.search_text.is_empty()
            || self.status_filter != StatusFilter::default()
            || self.priority_filter != PriorityFilter::default()
            || self.due_filter != DueFilter::default()
    }
}
