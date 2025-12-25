use std::collections::HashSet;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DueFilter {
    All,
    Overdue,
    Today,
    ThisWeek,
    NoDate,
}

impl Default for DueFilter {
    fn default() -> Self {
        Self::All
    }
}

impl DueFilter {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Overdue => "Overdue",
            Self::Today => "Today",
            Self::ThisWeek => "This Week",
            Self::NoDate => "No Date",
        }
    }

    pub fn all_variants() -> &'static [Self] {
        &[
            Self::All,
            Self::Overdue,
            Self::Today,
            Self::ThisWeek,
            Self::NoDate,
        ]
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
