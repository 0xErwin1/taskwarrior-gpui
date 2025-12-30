use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskPriority {
    High,
    Medium,
    Low,
    None,
}

impl Into<usize> for TaskPriority {
    fn into(self) -> usize {
        match self {
            TaskPriority::High => 0,
            TaskPriority::Medium => 1,
            TaskPriority::Low => 2,
            TaskPriority::None => 3,
        }
    }
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::None
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskPriority::High => write!(f, "High"),
            TaskPriority::Medium => write!(f, "Medium"),
            TaskPriority::Low => write!(f, "Low"),
            TaskPriority::None => write!(f, "None"),
        }
    }
}

impl Into<String> for TaskPriority {
    fn into(self) -> String {
        match self {
            TaskPriority::High => "High".to_string(),
            TaskPriority::Medium => "Medium".to_string(),
            TaskPriority::Low => "Low".to_string(),
            TaskPriority::None => "None".to_string(),
        }
    }
}

impl From<&str> for TaskPriority {
    fn from(s: &str) -> Self {
        match s {
            "high" | "High" | "H" | "h" => TaskPriority::High,
            "medium" | "Medium" | "M" | "m" => TaskPriority::Medium,
            "low" | "Low" | "L" | "l" => TaskPriority::Low,
            _ => TaskPriority::None,
        }
    }
}

impl From<String> for TaskPriority {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Completed,
    Deleted,
    Unknown(String),
    Recurring,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

impl Into<String> for TaskStatus {
    fn into(self) -> String {
        match self {
            TaskStatus::Pending => "Pending".to_string(),
            TaskStatus::Completed => "Completed".to_string(),
            TaskStatus::Deleted => "Deleted".to_string(),
            TaskStatus::Unknown(reason) => format!("Unknown({})", reason),
            TaskStatus::Recurring => "Recurring".to_string(),
        }
    }
}

impl From<&str> for TaskStatus {
    fn from(s: &str) -> Self {
        match s {
            "pending" | "Pending" | "P" | "p" => TaskStatus::Pending,
            "completed" | "Completed" | "C" | "c" => TaskStatus::Completed,
            "deleted" | "Deleted" | "D" | "d" => TaskStatus::Deleted,
            "recurring" | "Recurring" | "R" | "r" => TaskStatus::Recurring,
            _ => TaskStatus::Unknown(String::new()),
        }
    }
}

impl From<String> for TaskStatus {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl From<taskchampion::Status> for TaskStatus {
    fn from(status: taskchampion::Status) -> Self {
        match status {
            taskchampion::Status::Pending => TaskStatus::Pending,
            taskchampion::Status::Completed => TaskStatus::Completed,
            taskchampion::Status::Deleted => TaskStatus::Deleted,
            taskchampion::Status::Unknown(reason) => TaskStatus::Unknown(reason),
            taskchampion::Status::Recurring => TaskStatus::Recurring,
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "Pending"),
            TaskStatus::Completed => write!(f, "Completed"),
            TaskStatus::Deleted => write!(f, "Deleted"),
            TaskStatus::Unknown(reason) => write!(f, "Unknown ({})", reason),
            TaskStatus::Recurring => write!(f, "Recurring"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TaskAnnotation {
    pub entry: DateTime<Utc>,
    pub content: String,
}

impl From<taskchampion::Annotation> for TaskAnnotation {
    fn from(annotation: taskchampion::Annotation) -> Self {
        TaskAnnotation {
            entry: annotation.entry,
            content: annotation.description,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Task {
    pub uuid: uuid::Uuid,
    pub id: Option<usize>,
    pub status: TaskStatus,
    pub description: String,
    pub project: Option<String>,
    pub priority: TaskPriority,
    pub tags: HashSet<String>,
    pub due: Option<DateTime<Utc>>,
    pub wait: Option<DateTime<Utc>>,
    pub entry: Option<DateTime<Utc>>,
    pub modified: Option<DateTime<Utc>>,
    pub annotations: Vec<TaskAnnotation>,
    pub dependencies: HashSet<uuid::Uuid>,
    pub is_active: bool,
    pub is_blocked: bool,
    pub working_id: Option<usize>,
}

impl Task {
    pub fn new(
        uuid: uuid::Uuid,
        id: Option<usize>,
        status: TaskStatus,
        description: String,
        project: Option<String>,
        priority: TaskPriority,
        tags: HashSet<String>,
        due: Option<DateTime<Utc>>,
        wait: Option<DateTime<Utc>>,
        entry: Option<DateTime<Utc>>,
        modified: Option<DateTime<Utc>>,
        annotations: Vec<TaskAnnotation>,
        dependencies: HashSet<uuid::Uuid>,
        is_active: bool,
        is_blocked: bool,
        working_id: Option<usize>,
    ) -> Self {
        Self {
            uuid,
            status,
            description,
            project,
            priority,
            tags,
            due,
            wait,
            entry,
            modified,
            annotations,
            dependencies,
            is_active,
            is_blocked,
            id,
            working_id,
        }
    }

    pub fn is_overdue(&self) -> bool {
        self.due.map_or(false, |due| due < Utc::now())
    }

    pub fn is_due_today(&self) -> bool {
        self.due
            .map_or(false, |due| due.date_naive() == Utc::now().date_naive())
    }
}

#[derive(Debug, Clone)]
pub struct TaskSummary {
    pub uuid: uuid::Uuid,
    pub id: Option<usize>,
    pub working_id: Option<usize>,
    pub status: TaskStatus,
    pub description: String,
    pub project: Option<String>,
    pub priority: TaskPriority,
    pub tags: HashSet<String>,
    pub due: Option<DateTime<Utc>>,
    pub wait: Option<DateTime<Utc>>,
    pub dependencies: HashSet<uuid::Uuid>,
    pub is_active: bool,
    pub is_blocked: bool,
}

impl TaskSummary {
    pub fn is_overdue(&self) -> bool {
        self.due.map_or(false, |due| due < Utc::now())
    }

    pub fn is_due_today(&self) -> bool {
        self.due
            .map_or(false, |due| due.date_naive() == Utc::now().date_naive())
    }
}

impl From<&Task> for TaskSummary {
    fn from(task: &Task) -> Self {
        Self {
            uuid: task.uuid,
            id: task.id,
            working_id: task.working_id,
            status: task.status.clone(),
            description: task.description.clone(),
            project: task.project.clone(),
            priority: task.priority,
            tags: task.tags.clone(),
            due: task.due,
            wait: task.wait,
            dependencies: task.dependencies.clone(),
            is_active: task.is_active,
            is_blocked: task.is_blocked,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskIdentityVm {
    pub uuid: uuid::Uuid,
    pub id: Option<usize>,
    pub working_id: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct TaskOverviewVm {
    pub description: String,
    pub status: TaskStatus,
    pub project: Option<String>,
    pub priority: TaskPriority,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct TaskDatesVm {
    pub entry: Option<DateTime<Utc>>,
    pub modified: Option<DateTime<Utc>>,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub due: Option<DateTime<Utc>>,
    pub scheduled: Option<DateTime<Utc>>,
    pub wait: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct TaskTagsVm {
    pub tags: Vec<String>,
    pub virtual_tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TaskLinkVm {
    pub uuid: uuid::Uuid,
    pub id: Option<usize>,
    pub description: String,
    pub status: TaskStatus,
}

#[derive(Debug, Clone)]
pub struct TaskDependenciesVm {
    pub depends_on: Vec<TaskLinkVm>,
    pub blocked_by: Vec<TaskLinkVm>,
    pub blocking: Vec<TaskLinkVm>,
}

#[derive(Debug, Clone, Default)]
pub struct TaskMetricsVm {
    pub urgency: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct TaskDetailVm {
    pub identity: TaskIdentityVm,
    pub overview: TaskOverviewVm,
    pub dates: TaskDatesVm,
    pub tags: TaskTagsVm,
    pub dependencies: TaskDependenciesVm,
    pub annotations: Vec<TaskAnnotation>,
    pub udas: Vec<(String, String)>,
    pub metrics: TaskMetricsVm,
}

impl TaskDetailVm {
    pub fn from_task(task: &Task, all_tasks: &[TaskSummary]) -> Self {
        let mut tags: Vec<String> = task.tags.iter().cloned().collect();
        tags.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

        let task_map: HashMap<uuid::Uuid, &TaskSummary> =
            all_tasks.iter().map(|t| (t.uuid, t)).collect();

        let to_link = |uuid: &uuid::Uuid| -> TaskLinkVm {
            match task_map.get(uuid) {
                Some(summary) => TaskLinkVm {
                    uuid: *uuid,
                    id: summary.working_id.or(summary.id),
                    description: summary.description.clone(),
                    status: summary.status.clone(),
                },
                None => TaskLinkVm {
                    uuid: *uuid,
                    id: None,
                    description: "Unknown task".to_string(),
                    status: TaskStatus::Unknown("Missing".to_string()),
                },
            }
        };

        let mut depends_on: Vec<TaskLinkVm> = task.dependencies.iter().map(to_link).collect();
        depends_on.sort_by(|a, b| a.uuid.cmp(&b.uuid));

        let mut blocked_by: Vec<TaskLinkVm> = task
            .dependencies
            .iter()
            .filter(|uuid| {
                task_map
                    .get(uuid)
                    .map(|summary| !matches!(summary.status, TaskStatus::Completed))
                    .unwrap_or(true)
            })
            .map(to_link)
            .collect();
        blocked_by.sort_by(|a, b| a.uuid.cmp(&b.uuid));

        let mut blocking: Vec<TaskLinkVm> = all_tasks
            .iter()
            .filter(|summary| summary.dependencies.contains(&task.uuid))
            .map(|summary| TaskLinkVm {
                uuid: summary.uuid,
                id: summary.working_id.or(summary.id),
                description: summary.description.clone(),
                status: summary.status.clone(),
            })
            .collect();
        blocking.sort_by(|a, b| a.uuid.cmp(&b.uuid));

        let mut virtual_tags = Vec::new();
        if !blocked_by.is_empty() || task.is_blocked {
            virtual_tags.push("BLOCKED".to_string());
        }
        if !blocking.is_empty() {
            virtual_tags.push("BLOCKING".to_string());
        }
        if let Some(due) = task.due {
            let today = Utc::now().date_naive();
            let tomorrow = (Utc::now() + Duration::days(1)).date_naive();
            let due_date = due.date_naive();
            if due_date == today {
                virtual_tags.push("TODAY".to_string());
            } else if due_date == tomorrow {
                virtual_tags.push("TOMORROW".to_string());
            } else {
                virtual_tags.push("DUE".to_string());
            }
        }

        let mut annotations = task.annotations.clone();
        annotations.sort_by(|a, b| a.entry.cmp(&b.entry));

        TaskDetailVm {
            identity: TaskIdentityVm {
                uuid: task.uuid,
                id: task.id,
                working_id: task.working_id,
            },
            overview: TaskOverviewVm {
                description: task.description.clone(),
                status: task.status.clone(),
                project: task.project.clone(),
                priority: task.priority,
                is_active: task.is_active,
            },
            dates: TaskDatesVm {
                entry: task.entry,
                modified: task.modified,
                start: None,
                end: None,
                due: task.due,
                scheduled: None,
                wait: task.wait,
                until: None,
            },
            tags: TaskTagsVm { tags, virtual_tags },
            dependencies: TaskDependenciesVm {
                depends_on,
                blocked_by,
                blocking,
            },
            annotations,
            udas: Vec::new(),
            metrics: TaskMetricsVm::default(),
        }
    }
}

impl From<taskchampion::Task> for Task {
    fn from(task: taskchampion::Task) -> Self {
        Self {
            uuid: task.get_uuid(),
            id: task.get_value("ID").map(|value| value.parse().unwrap()),
            status: task.get_status().into(),
            description: task.get_description().to_string(),
            project: task.get_value("project").map(|v| v.to_string()),
            priority: task.get_priority().into(),
            tags: task.get_tags().map(|t| t.to_string()).collect(),
            due: task.get_due().map(Into::into),
            wait: task.get_wait().map(Into::into),
            entry: task.get_entry().map(Into::into),
            modified: task.get_modified().map(Into::into),
            annotations: task.get_annotations().map(Into::into).collect(),
            dependencies: task.get_dependencies().map(Into::into).collect(),
            is_active: task.is_active(),
            is_blocked: task.is_blocked(),
            working_id: None,
        }
    }
}

pub struct TaskUpdate {
    pub description: Option<String>,
    pub project: Option<String>,
    pub priority: Option<String>,
    pub tags: Option<HashSet<String>>,
    pub due: Option<DateTime<Utc>>,
    pub wait: Option<DateTime<Utc>>,
    pub annotations: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct TaskOverview {
    pub tasks: Vec<Task>,
    pub projects: Vec<(String, usize)>,
    pub tags: Vec<(String, usize)>,
    pub total_tasks: usize,
    pub pending_tasks: usize,
    pub completed_tasks: usize,
}

#[derive(Debug, Clone)]
pub enum TaskDetailState {
    Idle,
    Loading(uuid::Uuid),
    Ready(TaskDetailVm),
    Error(uuid::Uuid, String),
}

impl Default for TaskDetailState {
    fn default() -> Self {
        TaskDetailState::Idle
    }
}
