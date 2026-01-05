use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use gpui::SharedString;
use std::collections::{HashMap, HashSet};

use crate::task::{self, TaskDetailVm};
use crate::ui::DATE_FORMAT;

use super::annotations::{AnnotationOrigin, AnnotationState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum FieldId {
    Description,
    Due,
}

#[derive(Debug, Clone, Default)]
pub(super) struct TaskForm {
    pub(super) description: String,
    pub(super) project: String,
    pub(super) priority: task::TaskPriority,
    pub(super) status: task::TaskStatus,
    pub(super) due: String,
    pub(super) tags: Vec<String>,
    pub(super) tag_draft: String,
}

impl TaskForm {
    pub(super) fn from_detail(detail: &TaskDetailVm) -> Self {
        let project = detail.overview.project.clone().unwrap_or_default();
        let due = detail
            .dates
            .due
            .map(|date| date.format(DATE_FORMAT).to_string())
            .unwrap_or_default();

        Self {
            description: detail.overview.description.clone(),
            project,
            priority: detail.overview.priority,
            status: detail.overview.status.clone(),
            due,
            tags: detail.tags.tags.clone(),
            tag_draft: String::new(),
        }
    }

    pub(super) fn is_dirty(&self, original: &TaskDetailVm) -> bool {
        let original_due = original
            .dates
            .due
            .map(|date| date.format(DATE_FORMAT).to_string())
            .unwrap_or_default();
        let original_project = original.overview.project.clone().unwrap_or_default();

        if self.description != original.overview.description {
            return true;
        }

        if self.project.trim() != original_project.trim() {
            return true;
        }

        if self.priority != original.overview.priority {
            return true;
        }

        if self.status != original.overview.status {
            return true;
        }

        if self.due.trim() != original_due.trim() {
            return true;
        }

        let original_tags: HashSet<String> = original.tags.tags.iter().cloned().collect();
        let draft_tags: HashSet<String> = self.tags.iter().cloned().collect();
        draft_tags != original_tags
    }

    pub(super) fn validate_field(&self, field: FieldId) -> Option<SharedString> {
        match field {
            FieldId::Description => {
                if self.description.trim().is_empty() {
                    Some("Description is required".into())
                } else {
                    None
                }
            }
            FieldId::Due => {
                if self.due.trim().is_empty() {
                    None
                } else if NaiveDate::parse_from_str(self.due.trim(), DATE_FORMAT).is_err() {
                    Some("Invalid date format. Use YYYY-MM-DD".into())
                } else {
                    None
                }
            }
        }
    }

    pub(super) fn validate(&self) -> HashMap<FieldId, SharedString> {
        let mut errors = HashMap::new();

        for field in [FieldId::Description, FieldId::Due] {
            if let Some(message) = self.validate_field(field) {
                errors.insert(field, message);
            }
        }

        errors
    }

    pub(super) fn add_tags(&mut self, raw: &str) -> bool {
        let mut added = false;

        for tag in raw.split(|ch: char| ch.is_whitespace() || ch == ',') {
            let tag = tag.trim();
            if tag.is_empty() {
                continue;
            }

            if !self.tags.iter().any(|t| t == tag) {
                self.tags.push(tag.to_string());
                added = true;
            }
        }

        if added {
            self.tags
                .sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        }

        added
    }

    pub(super) fn remove_tag(&mut self, tag: &str) -> bool {
        let before = self.tags.len();
        self.tags.retain(|t| t != tag);
        before != self.tags.len()
    }
}

#[derive(Debug, Clone, Default)]
pub struct TaskEditUpdate {
    pub description: Option<String>,
    pub project: Option<Option<String>>,
    pub priority: Option<String>,
    pub status: Option<task::TaskStatus>,
    pub due: Option<Option<DateTime<Utc>>>,
    pub tags: Option<HashSet<String>>,
    pub annotations_add: Vec<String>,
    pub annotations_delete: Vec<DateTime<Utc>>,
}

impl TaskEditUpdate {
    pub(super) fn is_empty(&self) -> bool {
        self.description.is_none()
            && self.project.is_none()
            && self.priority.is_none()
            && self.status.is_none()
            && self.due.is_none()
            && self.tags.is_none()
            && self.annotations_add.is_empty()
            && self.annotations_delete.is_empty()
    }
}

pub(super) fn build_task_update(
    original: &TaskDetailVm,
    draft: &TaskForm,
    annotations: &AnnotationState,
) -> TaskEditUpdate {
    let mut update = TaskEditUpdate::default();

    if draft.description != original.overview.description {
        update.description = Some(draft.description.clone());
    }

    let draft_project = draft.project.trim();
    let original_project = original.overview.project.as_deref().unwrap_or("").trim();
    if draft_project != original_project {
        update.project = Some(if draft_project.is_empty() {
            None
        } else {
            Some(draft_project.to_string())
        });
    }

    if draft.priority != original.overview.priority {
        let priority: String = draft.priority.into();
        update.priority = Some(priority);
    }

    if draft.status != original.overview.status {
        update.status = Some(draft.status.clone());
    }

    let draft_due = draft.due.trim();
    let original_due = original
        .dates
        .due
        .map(|date| date.format(DATE_FORMAT).to_string())
        .unwrap_or_default();

    if draft_due != original_due.trim() {
        if draft_due.is_empty() {
            update.due = Some(None);
        } else if let Ok(date) = NaiveDate::parse_from_str(draft_due, DATE_FORMAT) {
            let due = Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap());
            update.due = Some(Some(due));
        }
    }

    let draft_tags: HashSet<String> = draft.tags.iter().cloned().collect();
    let original_tags: HashSet<String> = original.tags.tags.iter().cloned().collect();
    if draft_tags != original_tags {
        update.tags = Some(draft_tags);
    }

    for annotation in &annotations.items {
        match annotation.origin {
            AnnotationOrigin::Added => {
                update.annotations_add.push(annotation.text.to_string());
            }
            AnnotationOrigin::Deleted => {
                update.annotations_delete.push(annotation.created_at);
            }
            AnnotationOrigin::Original => {}
            AnnotationOrigin::Modified {
                original_entry,
                original_text: _,
            } => {
                update.annotations_delete.push(original_entry);
                update.annotations_add.push(annotation.text.to_string());
            }
        }
    }

    update
}

#[cfg(test)]
mod tests {
    use super::super::annotations::AnnotationState;
    use super::*;
    use crate::task::model::{
        TaskAnnotation, TaskDatesVm, TaskDependenciesVm, TaskDetailVm, TaskIdentityVm,
        TaskMetricsVm, TaskOverviewVm, TaskTagsVm,
    };
    use crate::task::{TaskPriority, TaskStatus};
    use chrono::{TimeZone, Utc};

    fn base_detail() -> TaskDetailVm {
        TaskDetailVm {
            identity: TaskIdentityVm {
                uuid: uuid::Uuid::nil(),
                id: None,
                working_id: None,
            },
            overview: TaskOverviewVm {
                description: "Test task".to_string(),
                status: TaskStatus::Pending,
                project: Some("proj".to_string()),
                priority: TaskPriority::None,
                is_active: false,
            },
            dates: TaskDatesVm {
                entry: None,
                modified: None,
                start: None,
                end: None,
                due: None,
                scheduled: None,
                wait: None,
                until: None,
            },
            tags: TaskTagsVm {
                tags: vec!["a".to_string(), "b".to_string()],
                virtual_tags: Vec::new(),
            },
            dependencies: TaskDependenciesVm {
                depends_on: Vec::new(),
                blocked_by: Vec::new(),
                blocking: Vec::new(),
            },
            annotations: Vec::new(),
            udas: Vec::new(),
            metrics: TaskMetricsVm::default(),
        }
    }

    #[test]
    fn build_update_project_cleared() {
        let detail = base_detail();
        let mut draft = TaskForm::from_detail(&detail);
        draft.project = "   ".to_string();

        let annotations = AnnotationState::default();
        let update = build_task_update(&detail, &draft, &annotations);

        assert_eq!(update.project, Some(None));
    }

    #[test]
    fn build_update_due_invalid_is_ignored() {
        let detail = base_detail();
        let mut draft = TaskForm::from_detail(&detail);
        draft.due = "2023-99-99".to_string();

        let annotations = AnnotationState::default();
        let update = build_task_update(&detail, &draft, &annotations);

        assert!(update.due.is_none());
    }

    #[test]
    fn build_update_tags_ignore_order() {
        let detail = base_detail();
        let mut draft = TaskForm::from_detail(&detail);
        draft.tags = vec!["b".to_string(), "a".to_string()];

        let annotations = AnnotationState::default();
        let update = build_task_update(&detail, &draft, &annotations);

        assert!(update.tags.is_none());
    }

    #[test]
    fn build_update_annotations_added_deleted() {
        let mut detail = base_detail();
        let deleted_at = Utc.with_ymd_and_hms(2023, 5, 2, 12, 0, 0).unwrap();
        detail.annotations = vec![TaskAnnotation {
            entry: deleted_at,
            content: "Original".to_string(),
        }];

        let mut annotations = AnnotationState::from_detail(&detail);
        let original_id = annotations.items[0].id;
        annotations.mark_deleted(original_id);
        annotations.add_local("Added".into(), Utc::now());

        let draft = TaskForm::from_detail(&detail);
        let update = build_task_update(&detail, &draft, &annotations);

        assert_eq!(update.annotations_add, vec!["Added".to_string()]);
        assert_eq!(update.annotations_delete, vec![deleted_at]);
    }
}
