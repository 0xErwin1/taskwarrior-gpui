use std::fmt;

pub enum TaskError {
    Storage(String),
    NotFound(uuid::Uuid),
    InvalidTag(String),
    InvalidProject(String),
    InvalidPriority(String),
    InvalidDue(String),
    InvalidWait(String),
    InvalidAnnotation(String),
    InvalidDependency(String),
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TaskError::Storage(msg) => write!(f, "Storage error: {}", msg),
            TaskError::NotFound(id) => write!(f, "Task not found: {}", id),
            TaskError::InvalidTag(tag) => write!(f, "Invalid tag: {}", tag),
            TaskError::InvalidProject(project) => write!(f, "Invalid project: {}", project),
            TaskError::InvalidPriority(priority) => write!(f, "Invalid priority: {}", priority),
            TaskError::InvalidDue(due) => write!(f, "Invalid due date: {}", due),
            TaskError::InvalidWait(wait) => write!(f, "Invalid wait date: {}", wait),
            TaskError::InvalidAnnotation(annotation) => {
                write!(f, "Invalid annotation: {}", annotation)
            }
            TaskError::InvalidDependency(dependency) => {
                write!(f, "Invalid dependency: {}", dependency)
            }
        }
    }
}

pub type TaskResult<T> = Result<T, TaskError>;
