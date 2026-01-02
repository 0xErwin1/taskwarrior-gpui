use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use gpui::prelude::*;
use std::collections::{HashMap, HashSet, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use crate::components::button::{Dropdown, DropdownItem};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::modal::ModalFrame;
use crate::components::toast::{ToastGlobal, ToastKind};
use crate::keymap::{Command, CommandDispatcher, ContextId};
use crate::task::model::TaskLinkVm;
use crate::task::{self, TaskDetailVm};
use crate::theme::{ActiveTheme, Theme};
use crate::ui::{DATE_FORMAT, DATE_TIME_FORMAT};

pub enum TaskDetailModalEvent {
    Closed,
    SaveEdits {
        task_id: uuid::Uuid,
        update: TaskEditUpdate,
    },
}

type AnnotationId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModalMode {
    View,
    Edit,
}

impl Default for ModalMode {
    fn default() -> Self {
        Self::View
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FieldId {
    Description,
    Due,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalFocus {
    None,
    Description,
    Project,
    StatusDropdown,
    PriorityDropdown,
    Due,
    TagsInput,
    AnnotationsInput,
}

impl Default for ModalFocus {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone)]
struct TaskModalState {
    open: bool,
    task_id: Option<uuid::Uuid>,
    loading: bool,
    original: Option<TaskDetailVm>,
    form: TaskForm,
    errors: HashMap<FieldId, gpui::SharedString>,
    mode: ModalMode,
    annotations: AnnotationState,
    error: Option<gpui::SharedString>,
    modal_focus: ModalFocus,
}

impl Default for TaskModalState {
    fn default() -> Self {
        Self {
            open: false,
            task_id: None,
            loading: false,
            original: None,
            form: TaskForm::default(),
            errors: HashMap::new(),
            mode: ModalMode::default(),
            annotations: AnnotationState::default(),
            error: None,
            modal_focus: ModalFocus::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct TaskForm {
    description: String,
    project: String,
    priority: task::TaskPriority,
    status: task::TaskStatus,
    due: String,
    tags: Vec<String>,
    tag_draft: String,
}

impl TaskForm {
    fn from_detail(detail: &TaskDetailVm) -> Self {
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

    fn is_dirty(&self, original: &TaskDetailVm) -> bool {
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

    fn validate_field(&self, field: FieldId) -> Option<gpui::SharedString> {
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
                    Some("Use YYYY-MM-DD".into())
                } else {
                    None
                }
            }
        }
    }

    fn validate(&self) -> HashMap<FieldId, gpui::SharedString> {
        let mut errors = HashMap::new();
        for field in [FieldId::Description, FieldId::Due] {
            if let Some(message) = self.validate_field(field) {
                errors.insert(field, message);
            }
        }
        errors
    }

    fn add_tags(&mut self, raw: &str) -> bool {
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

    fn remove_tag(&mut self, tag: &str) -> bool {
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
    fn is_empty(&self) -> bool {
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

#[derive(Debug, Clone)]
struct AnnotationState {
    items: Vec<AnnotationView>,
    draft: gpui::SharedString,
}

impl Default for AnnotationState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            draft: gpui::SharedString::default(),
        }
    }
}

#[derive(Debug, Clone)]
struct AnnotationView {
    id: AnnotationId,
    created_at: DateTime<Utc>,
    text: gpui::SharedString,
    origin: AnnotationOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnnotationOrigin {
    Original,
    Added,
    Deleted,
}

fn annotation_id(entry: DateTime<Utc>, text: &str, index: usize) -> AnnotationId {
    let mut hasher = DefaultHasher::new();
    entry.timestamp_millis().hash(&mut hasher);
    text.hash(&mut hasher);
    index.hash(&mut hasher);
    hasher.finish()
}

impl AnnotationState {
    fn from_detail(detail: &TaskDetailVm) -> Self {
        let items: Vec<AnnotationView> = detail
            .annotations
            .iter()
            .enumerate()
            .map(|(index, annotation)| AnnotationView {
                id: annotation_id(annotation.entry, &annotation.content, index),
                created_at: annotation.entry,
                text: annotation.content.clone().into(),
                origin: AnnotationOrigin::Original,
            })
            .collect();

        Self {
            items,
            draft: gpui::SharedString::default(),
        }
    }

    fn set_draft(&mut self, value: &str) {
        self.draft = value.to_string().into();
    }

    fn clear_draft(&mut self) {
        self.draft = gpui::SharedString::default();
    }

    fn add_local(&mut self, text: gpui::SharedString, created_at: DateTime<Utc>) {
        let id = annotation_id(created_at, text.as_ref(), self.items.len());
        self.items.push(AnnotationView {
            id,
            created_at,
            text,
            origin: AnnotationOrigin::Added,
        });
    }

    fn mark_deleted(&mut self, id: AnnotationId) -> Option<DateTime<Utc>> {
        let index = self.items.iter().position(|item| item.id == id)?;
        let item = &mut self.items[index];
        if item.origin == AnnotationOrigin::Added {
            self.items.remove(index);
            return None;
        }
        if item.origin == AnnotationOrigin::Deleted {
            return None;
        }
        item.origin = AnnotationOrigin::Deleted;
        Some(item.created_at)
    }
}

pub struct TaskDetailModal {
    state: TaskModalState,
    focus_handle: gpui::FocusHandle,
    form_focus_handle: gpui::FocusHandle,
    scroll_handle: gpui::ScrollHandle,
    annotation_input: gpui::Entity<Input>,
    description_input: gpui::Entity<Input>,
    project_input: gpui::Entity<Input>,
    due_input: gpui::Entity<Input>,
    tags_input: gpui::Entity<Input>,
    status_dropdown: gpui::Entity<Dropdown>,
    priority_dropdown: gpui::Entity<Dropdown>,
    project_suggestions: Arc<Mutex<Vec<String>>>,
}

impl TaskDetailModal {
    pub fn new(cx: &mut gpui::Context<Self>) -> Self {
        let modal_entity = cx.entity().clone();
        let project_suggestions: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let annotation_input = cx.new(|cx| {
            let on_change_entity = modal_entity.clone();
            let on_submit_entity = modal_entity.clone();

            Input::new("annotation-input", cx, "Add annotation...")
                .multiline()
                .with_on_change(Arc::new(
                    move |value: &str, cx: &mut gpui::Context<Input>| {
                        cx.update_entity(&on_change_entity, |modal, cx| {
                            modal.update_annotation_draft(value, cx);
                        });
                    },
                ))
                .with_on_submit(Arc::new(
                    move |_value: &str, cx: &mut gpui::Context<Input>| {
                        cx.update_entity(&on_submit_entity, |modal, cx| {
                            modal.submit_annotation(cx);
                        });
                    },
                ))
        });

        let description_input = {
            let modal_entity = modal_entity.clone();
            cx.new(|cx| {
                Input::new("task-edit-description", cx, "Description").with_on_change(Arc::new(
                    move |value: &str, cx: &mut gpui::Context<Input>| {
                        cx.update_entity(&modal_entity, |modal, cx| {
                            modal.update_edit_description(value, cx);
                        });
                    },
                ))
            })
        };

        let project_input = {
            let modal_entity = modal_entity.clone();
            let suggestions = project_suggestions.clone();
            cx.new(|cx| {
                Input::new("task-edit-project", cx, "Project")
                    .with_suggest(Arc::new(move |query| {
                        let query = query.trim();
                        if query.is_empty() {
                            return Vec::new();
                        }

                        let Ok(list) = suggestions.lock() else {
                            return Vec::new();
                        };

                        let needle = query.to_lowercase();
                        let mut level_matches = Vec::new();
                        let mut prefix_matches = Vec::new();
                        let mut contains_matches = Vec::new();

                        for project in list.iter() {
                            let hay = project.to_lowercase();
                            if hay.starts_with(&needle) {
                                let boundary = hay.len() == needle.len()
                                    || hay.as_bytes().get(needle.len()) == Some(&b'.')
                                    || needle.ends_with('.');
                                if boundary {
                                    level_matches.push(project.clone());
                                } else {
                                    prefix_matches.push(project.clone());
                                }
                            } else if hay.contains(&needle) {
                                contains_matches.push(project.clone());
                            }
                        }

                        level_matches.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
                        prefix_matches.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
                        contains_matches.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

                        level_matches
                            .into_iter()
                            .chain(prefix_matches)
                            .chain(contains_matches)
                            .take(8)
                            .map(crate::components::input::Suggestion::simple)
                            .collect()
                    }))
                    .with_on_change(Arc::new(
                        move |value: &str, cx: &mut gpui::Context<Input>| {
                            cx.update_entity(&modal_entity, |modal, cx| {
                                modal.update_edit_project(value, cx);
                            });
                        },
                    ))
            })
        };

        let due_input = {
            let modal_entity = modal_entity.clone();
            cx.new(|cx| {
                Input::new("task-edit-due", cx, "Due (YYYY-MM-DD)").with_on_change(Arc::new(
                    move |value: &str, cx: &mut gpui::Context<Input>| {
                        cx.update_entity(&modal_entity, |modal, cx| {
                            modal.update_edit_due(value, cx);
                        });
                    },
                ))
            })
        };

        let tags_input = {
            let modal_entity = modal_entity.clone();
            cx.new(|cx| {
                let on_submit_entity = modal_entity.clone();
                Input::new("task-edit-tags", cx, "Add tag")
                    .with_on_change(Arc::new(
                        move |value: &str, cx: &mut gpui::Context<Input>| {
                            cx.update_entity(&modal_entity, |modal, cx| {
                                modal.update_edit_tags(value, cx);
                            });
                        },
                    ))
                    .with_on_submit(Arc::new(
                        move |_value: &str, cx: &mut gpui::Context<Input>| {
                            cx.update_entity(&on_submit_entity, |modal, cx| {
                                modal.submit_tag_draft(cx);
                            });
                        },
                    ))
            })
        };

        let status_items = vec![
            DropdownItem::new("Pending"),
            DropdownItem::new("Completed"),
            DropdownItem::new("Deleted"),
        ];
        let status_dropdown = {
            let modal_entity = modal_entity.clone();
            cx.new(|_cx| {
                Dropdown::new("task-edit-status")
                    .items(status_items)
                    .on_select(Arc::new(move |index, _item, cx| {
                        cx.update_entity(&modal_entity, |modal, cx| {
                            modal.update_edit_status(index, cx);
                        });
                    }))
            })
        };

        let priority_items = vec![
            DropdownItem::new("High"),
            DropdownItem::new("Medium"),
            DropdownItem::new("Low"),
            DropdownItem::new("None"),
        ];
        let priority_dropdown = {
            let modal_entity = modal_entity.clone();
            cx.new(|_cx| {
                Dropdown::new("task-edit-priority")
                    .items(priority_items)
                    .on_select(Arc::new(move |index, _item, cx| {
                        cx.update_entity(&modal_entity, |modal, cx| {
                            modal.update_edit_priority(index, cx);
                        });
                    }))
            })
        };

        Self {
            state: TaskModalState::default(),
            focus_handle: cx.focus_handle(),
            form_focus_handle: cx.focus_handle(),
            scroll_handle: gpui::ScrollHandle::new(),
            annotation_input,
            description_input,
            project_input,
            due_input,
            tags_input,
            status_dropdown,
            priority_dropdown,
            project_suggestions,
        }
    }

    pub fn is_open(&self) -> bool {
        self.state.open
    }

    pub fn focus_handle(&self) -> &gpui::FocusHandle {
        &self.focus_handle
    }

    pub fn set_project_suggestions(
        &mut self,
        projects: Vec<String>,
        _cx: &mut gpui::Context<Self>,
    ) {
        if let Ok(mut list) = self.project_suggestions.lock() {
            *list = projects;
        }
    }

    pub fn open_with_detail(
        &mut self,
        detail: TaskDetailVm,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        if let Some(window) = window {
            window.focus(&self.focus_handle);
        }

        self.scroll_handle = gpui::ScrollHandle::new();
        self.scroll_handle.scroll_to_item(0);
        self.state.open = true;
        self.state.task_id = Some(detail.identity.uuid);
        self.state.loading = false;
        self.state.error = None;
        self.state.original = Some(detail.clone());
        self.state.mode = ModalMode::View;
        self.state.errors.clear();
        self.sync_from_detail(&detail, cx, true);
        cx.notify();
    }

    pub fn open_with_error(
        &mut self,
        task_id: uuid::Uuid,
        error: String,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        if let Some(window) = window {
            window.focus(&self.focus_handle);
        }

        self.scroll_handle = gpui::ScrollHandle::new();
        self.scroll_handle.scroll_to_item(0);
        self.state.open = true;
        self.state.task_id = Some(task_id);
        self.state.loading = false;
        self.state.original = None;
        self.state.form = TaskForm::default();
        self.state.errors.clear();
        self.state.mode = ModalMode::View;
        self.state.annotations = AnnotationState::default();
        self.state.error = Some(error.into());
        cx.notify();
    }

    pub fn open_loading(
        &mut self,
        task_id: uuid::Uuid,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        if let Some(window) = window {
            window.focus(&self.focus_handle);
        }

        self.scroll_handle = gpui::ScrollHandle::new();
        self.scroll_handle.scroll_to_item(0);
        self.state.open = true;
        self.state.task_id = Some(task_id);
        self.state.loading = true;
        self.state.original = None;
        self.state.form = TaskForm::default();
        self.state.errors.clear();
        self.state.mode = ModalMode::View;
        self.state.annotations = AnnotationState::default();
        self.state.error = None;
        cx.notify();
    }

    pub fn set_detail(&mut self, detail: TaskDetailVm, cx: &mut gpui::Context<Self>) {
        let reset_form = self.state.mode == ModalMode::View;
        self.state.task_id = Some(detail.identity.uuid);
        self.state.loading = false;
        self.state.error = None;
        self.state.original = Some(detail.clone());
        self.sync_from_detail(&detail, cx, reset_form);
        cx.notify();
    }

    pub fn set_error(&mut self, task_id: uuid::Uuid, error: String, cx: &mut gpui::Context<Self>) {
        self.state.task_id = Some(task_id);
        self.state.loading = false;
        self.state.original = None;
        self.state.form = TaskForm::default();
        self.state.errors.clear();
        self.state.mode = ModalMode::View;
        self.state.annotations = AnnotationState::default();
        self.state.error = Some(error.into());
        cx.notify();
    }

    fn sync_from_detail(
        &mut self,
        detail: &TaskDetailVm,
        cx: &mut gpui::Context<Self>,
        reset_edit: bool,
    ) {
        self.state.annotations = AnnotationState::from_detail(detail);
        self.annotation_input.update(cx, |input, cx| {
            input.clear(cx);
        });

        if reset_edit {
            self.state.form = TaskForm::from_detail(detail);
            self.state.errors.clear();
            self.apply_form_inputs(cx);
        }
    }

    pub fn apply_saved_detail(&mut self, detail: TaskDetailVm, cx: &mut gpui::Context<Self>) {
        self.state.mode = ModalMode::View;
        self.sync_from_detail(&detail, cx, true);
        self.state.task_id = Some(detail.identity.uuid);
        self.state.loading = false;
        self.state.original = Some(detail);
        self.state.error = None;
        cx.notify();
    }

    fn enter_edit_mode(&mut self, cx: &mut gpui::Context<Self>) {
        let Some(detail) = self.state.original.clone() else {
            return;
        };

        self.state.mode = ModalMode::Edit;
        self.state.form = TaskForm::from_detail(&detail);
        self.state.errors.clear();
        self.state.annotations = AnnotationState::from_detail(&detail);
        self.annotation_input.update(cx, |input, cx| {
            input.clear(cx);
        });
        self.apply_form_inputs(cx);
        cx.notify();
    }

    pub fn cancel_edit(&mut self, cx: &mut gpui::Context<Self>) {
        let Some(detail) = self.state.original.clone() else {
            return;
        };

        self.state.mode = ModalMode::View;
        self.state.form = TaskForm::from_detail(&detail);
        self.state.errors.clear();
        self.state.annotations = AnnotationState::from_detail(&detail);
        self.annotation_input.update(cx, |input, cx| {
            input.clear(cx);
        });
        self.apply_form_inputs(cx);
        cx.notify();
    }

    pub fn is_editing(&self) -> bool {
        self.state.mode == ModalMode::Edit
    }

    fn apply_form_inputs(&mut self, cx: &mut gpui::Context<Self>) {
        let form = self.state.form.clone();

        self.description_input.update(cx, |input, cx| {
            input.set_value_silent(form.description, cx);
        });

        self.project_input.update(cx, |input, cx| {
            input.set_value_silent(form.project, cx);
        });

        self.due_input.update(cx, |input, cx| {
            input.set_value_silent(form.due, cx);
        });

        self.tags_input.update(cx, |input, cx| {
            input.set_value_silent(form.tag_draft, cx);
        });

        let status_index = match form.status {
            task::TaskStatus::Pending => Some(0),
            task::TaskStatus::Completed => Some(1),
            task::TaskStatus::Deleted => Some(2),
            _ => None,
        };
        if let Some(status_index) = status_index {
            self.status_dropdown.update(cx, |dropdown, cx| {
                dropdown.set_selected_index(status_index, cx);
            });
        }

        let priority_index = match form.priority {
            task::TaskPriority::High => 0,
            task::TaskPriority::Medium => 1,
            task::TaskPriority::Low => 2,
            task::TaskPriority::None => 3,
        };
        self.priority_dropdown.update(cx, |dropdown, cx| {
            dropdown.set_selected_index(priority_index, cx);
        });
    }

    fn update_annotation_draft(&mut self, value: &str, cx: &mut gpui::Context<Self>) {
        self.state.annotations.set_draft(value);
        cx.notify();
    }

    fn update_edit_description(&mut self, value: &str, cx: &mut gpui::Context<Self>) {
        self.state.form.description = value.to_string();
        self.update_field_error(FieldId::Description);
        cx.notify();
    }

    fn update_edit_project(&mut self, value: &str, cx: &mut gpui::Context<Self>) {
        self.state.form.project = value.to_string();
        cx.notify();
    }

    fn update_edit_due(&mut self, value: &str, cx: &mut gpui::Context<Self>) {
        self.state.form.due = value.to_string();
        self.update_field_error(FieldId::Due);
        cx.notify();
    }

    fn update_edit_tags(&mut self, value: &str, cx: &mut gpui::Context<Self>) {
        self.state.form.tag_draft = value.to_string();
        cx.notify();
    }

    fn update_edit_status(&mut self, index: usize, cx: &mut gpui::Context<Self>) {
        let status = match index {
            0 => task::TaskStatus::Pending,
            1 => task::TaskStatus::Completed,
            2 => task::TaskStatus::Deleted,
            _ => task::TaskStatus::Pending,
        };
        self.state.form.status = status;
        cx.notify();
    }

    fn update_edit_priority(&mut self, index: usize, cx: &mut gpui::Context<Self>) {
        let priority = match index {
            0 => task::TaskPriority::High,
            1 => task::TaskPriority::Medium,
            2 => task::TaskPriority::Low,
            _ => task::TaskPriority::None,
        };
        self.state.form.priority = priority;
        cx.notify();
    }

    pub fn submit_edits(&mut self, cx: &mut gpui::Context<Self>) {
        if self.state.mode != ModalMode::Edit {
            self.close(cx);
            return;
        }

        let Some(detail) = self.state.original.as_ref() else {
            return;
        };

        self.state.errors = self.state.form.validate();
        if !self.state.errors.is_empty() {
            cx.notify();
            return;
        }

        let update = self.build_task_update(detail);

        if update.is_empty() {
            self.cancel_edit(cx);
            return;
        }

        cx.emit(TaskDetailModalEvent::SaveEdits {
            task_id: detail.identity.uuid,
            update,
        });
    }

    fn build_task_update(&self, original: &TaskDetailVm) -> TaskEditUpdate {
        let draft = &self.state.form;
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
            } else {
                if let Ok(date) = NaiveDate::parse_from_str(draft_due, DATE_FORMAT) {
                    let due = Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap());
                    update.due = Some(Some(due));
                }
            }
        }

        let draft_tags: HashSet<String> = draft.tags.iter().cloned().collect();
        let original_tags: HashSet<String> = original.tags.tags.iter().cloned().collect();
        if draft_tags != original_tags {
            update.tags = Some(draft_tags);
        }

        for annotation in &self.state.annotations.items {
            match annotation.origin {
                AnnotationOrigin::Added => {
                    update.annotations_add.push(annotation.text.to_string());
                }
                AnnotationOrigin::Deleted => {
                    update.annotations_delete.push(annotation.created_at);
                }
                AnnotationOrigin::Original => {}
            }
        }

        update
    }

    fn update_field_error(&mut self, field: FieldId) {
        match self.state.form.validate_field(field) {
            Some(message) => {
                self.state.errors.insert(field, message);
            }
            None => {
                self.state.errors.remove(&field);
            }
        }
    }

    fn submit_tag_draft(&mut self, cx: &mut gpui::Context<Self>) {
        let draft = self.state.form.tag_draft.clone();
        if draft.trim().is_empty() {
            return;
        }

        self.state.form.add_tags(&draft);
        self.state.form.tag_draft.clear();
        self.tags_input.update(cx, |input, cx| {
            input.clear(cx);
        });
        cx.notify();
    }

    fn remove_tag(&mut self, tag: String, cx: &mut gpui::Context<Self>) {
        if self.state.form.remove_tag(&tag) {
            cx.notify();
        }
    }

    fn submit_annotation(&mut self, cx: &mut gpui::Context<Self>) {
        let text = self.state.annotations.draft.to_string();
        if text.trim().is_empty() {
            let toast_host = cx.global::<ToastGlobal>().host.clone();
            cx.update_entity(&toast_host, |host, cx| {
                host.push(ToastKind::Error, "Annotation cannot be empty", cx);
            });
            return;
        }

        self.state
            .annotations
            .add_local(text.clone().into(), Utc::now());
        self.state.annotations.clear_draft();
        self.annotation_input.update(cx, |input, cx| {
            input.clear(cx);
        });
        cx.notify();
    }

    fn delete_annotation(&mut self, id: AnnotationId, cx: &mut gpui::Context<Self>) {
        if self.state.annotations.mark_deleted(id).is_some() {
            cx.notify();
        }
    }

    pub fn close(&mut self, cx: &mut gpui::Context<Self>) {
        if !self.state.open {
            return;
        }

        self.state = TaskModalState::default();
        cx.emit(TaskDetailModalEvent::Closed);
        cx.notify();
    }

    // Focus management methods
    
    pub fn set_modal_focus(
        &mut self,
        focus: ModalFocus,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        // Close dropdowns if changing focus (not if clicking same dropdown)
        if self.state.modal_focus != focus {
            self.close_all_dropdowns(cx);
        }
        self.state.modal_focus = focus;

        if let Some(window) = window {
            match focus {
                ModalFocus::Description => {
                    self.description_input.update(cx, |i, cx| i.focus(window, cx));
                }
                ModalFocus::Project => {
                    self.project_input.update(cx, |i, cx| i.focus(window, cx));
                }
                ModalFocus::Due => {
                    self.due_input.update(cx, |i, cx| i.focus(window, cx));
                }
                ModalFocus::TagsInput => {
                    self.tags_input.update(cx, |i, cx| i.focus(window, cx));
                }
                ModalFocus::AnnotationsInput => {
                    self.annotation_input.update(cx, |i, cx| i.focus(window, cx));
                }
                ModalFocus::StatusDropdown | ModalFocus::PriorityDropdown => {
                    window.focus(&self.form_focus_handle);
                }
                ModalFocus::None => {
                    window.focus(&self.focus_handle);
                }
            }
        }

        cx.notify();
    }

    pub fn get_modal_focus(&self) -> ModalFocus {
        self.state.modal_focus
    }

    fn close_all_dropdowns(&mut self, cx: &mut gpui::Context<Self>) {
        self.status_dropdown.update(cx, |d, cx| d.close(cx));
        self.priority_dropdown.update(cx, |d, cx| d.close(cx));
    }

    fn has_open_dropdown(&self, cx: &gpui::Context<Self>) -> bool {
        let status_open = self.status_dropdown.read(cx).is_open();
        let priority_open = self.priority_dropdown.read(cx).is_open();
        status_open || priority_open
    }

    pub fn toggle_focused_dropdown(&mut self, cx: &mut gpui::Context<Self>) {
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

        match self.state.modal_focus {
            ModalFocus::StatusDropdown => toggle(&self.status_dropdown, cx),
            ModalFocus::PriorityDropdown => toggle(&self.priority_dropdown, cx),
            _ => {}
        }
    }

    pub fn select_next_dropdown_option(&mut self, cx: &mut gpui::Context<Self>) {
        let select_next = |d: &gpui::Entity<Dropdown>, cx: &mut gpui::Context<Self>| {
            d.update(cx, |d, cx| {
                if !d.is_open() {
                    d.open(cx);
                }
                d.select_next_item(cx);
            });
            cx.notify();
        };

        match self.state.modal_focus {
            ModalFocus::StatusDropdown => select_next(&self.status_dropdown, cx),
            ModalFocus::PriorityDropdown => select_next(&self.priority_dropdown, cx),
            _ => {}
        }
    }

    pub fn select_prev_dropdown_option(&mut self, cx: &mut gpui::Context<Self>) {
        let select_prev = |d: &gpui::Entity<Dropdown>, cx: &mut gpui::Context<Self>| {
            d.update(cx, |d, cx| {
                if !d.is_open() {
                    d.open(cx);
                }
                d.select_prev_item(cx);
            });
            cx.notify();
        };

        match self.state.modal_focus {
            ModalFocus::StatusDropdown => select_prev(&self.status_dropdown, cx),
            ModalFocus::PriorityDropdown => select_prev(&self.priority_dropdown, cx),
            _ => {}
        }
    }

    pub fn focus_next_field(
        &mut self,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        use ModalFocus::*;
        
        let next = match self.state.modal_focus {
            None | Description => Project,
            Project => StatusDropdown,
            StatusDropdown => PriorityDropdown,
            PriorityDropdown => Due,
            Due => TagsInput,
            TagsInput => AnnotationsInput,
            AnnotationsInput => Description,
        };

        self.set_modal_focus(next, window, cx);
    }

    pub fn focus_prev_field(
        &mut self,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        use ModalFocus::*;
        
        let prev = match self.state.modal_focus {
            None | Description => AnnotationsInput,
            AnnotationsInput => TagsInput,
            TagsInput => Due,
            Due => PriorityDropdown,
            PriorityDropdown => StatusDropdown,
            StatusDropdown => Project,
            Project => Description,
        };

        self.set_modal_focus(prev, window, cx);
    }

    pub fn active_context(&self) -> ContextId {
        if !self.state.open {
            return ContextId::Global;
        }

        if self.state.mode != ModalMode::Edit {
            return ContextId::Modal;
        }

        match self.state.modal_focus {
            ModalFocus::Description
            | ModalFocus::Project
            | ModalFocus::Due
            | ModalFocus::TagsInput
            | ModalFocus::AnnotationsInput => ContextId::ModalInput,
            ModalFocus::StatusDropdown | ModalFocus::PriorityDropdown => ContextId::ModalDropdown,
            ModalFocus::None => ContextId::Modal,
        }
    }

    pub fn dispatch_command(
        &mut self,
        command: Command,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) -> bool {
        if !self.state.open {
            return false;
        }

        match command {
            Command::CloseModal => {
                // Escape behavior: dropdown > editing > close
                if self.state.mode == ModalMode::Edit && self.has_open_dropdown(cx) {
                    // First: close dropdown if open
                    self.close_all_dropdowns(cx);
                } else if self.state.mode == ModalMode::Edit {
                    // Second: cancel edit if editing
                    self.cancel_edit(cx);
                } else {
                    // Third: close modal
                    self.close(cx);
                }
                true
            }
            Command::SaveModal => {
                if self.state.mode == ModalMode::Edit {
                    self.submit_edits(cx);
                    true
                } else {
                    false
                }
            }
            Command::ModalScrollDown => {
                self.scroll(1, cx);
                true
            }
            Command::ModalScrollUp => {
                self.scroll(-1, cx);
                true
            }
            Command::FocusFilterNext => {
                if self.state.mode == ModalMode::Edit {
                    self.focus_next_field(window, cx);
                    true
                } else {
                    false
                }
            }
            Command::FocusFilterPrev => {
                if self.state.mode == ModalMode::Edit {
                    self.focus_prev_field(window, cx);
                    true
                } else {
                    false
                }
            }
            Command::ToggleDropdown => {
                if self.state.mode == ModalMode::Edit {
                    self.toggle_focused_dropdown(cx);
                    true
                } else {
                    false
                }
            }
            Command::SelectNextOption => {
                if self.state.mode == ModalMode::Edit {
                    self.select_next_dropdown_option(cx);
                    true
                } else {
                    false
                }
            }
            Command::SelectPrevOption => {
                if self.state.mode == ModalMode::Edit {
                    self.select_prev_dropdown_option(cx);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn scroll(&self, delta: i32, cx: &mut gpui::Context<Self>) {
        let handle = &self.scroll_handle;
        let current = if delta > 0 {
            handle.bottom_item()
        } else {
            handle.top_item()
        };
        let next = if delta > 0 {
            current.saturating_add(1)
        } else {
            current.saturating_sub(1)
        };

        handle.scroll_to_item(next);
        cx.notify();
    }
}

impl CommandDispatcher for TaskDetailModal {
    fn dispatch(&mut self, command: Command, cx: &mut gpui::Context<Self>) -> bool {
        self.dispatch_command(command, None, cx)
    }
}

impl gpui::EventEmitter<TaskDetailModalEvent> for TaskDetailModal {}

impl gpui::Render for TaskDetailModal {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        if !self.state.open {
            return gpui::div().into_any_element();
        }

        let on_close_backdrop = cx.listener(|modal, _event: &gpui::MouseDownEvent, _window, cx| {
            modal.close(cx);
        });
        let on_close_click = cx.listener(|modal, _event: &gpui::MouseDownEvent, _window, cx| {
            modal.close(cx);
        });

        render_task_detail_modal(
            &self.state,
            &self.annotation_input,
            &self.description_input,
            &self.project_input,
            &self.due_input,
            &self.tags_input,
            &self.status_dropdown,
            &self.priority_dropdown,
            &self.focus_handle,
            &self.scroll_handle,
            cx,
            on_close_backdrop,
            on_close_click,
        )
    }
}

fn render_task_detail_modal(
    state: &TaskModalState,
    annotation_input: &gpui::Entity<Input>,
    description_input: &gpui::Entity<Input>,
    project_input: &gpui::Entity<Input>,
    due_input: &gpui::Entity<Input>,
    tags_input: &gpui::Entity<Input>,
    status_dropdown: &gpui::Entity<Dropdown>,
    priority_dropdown: &gpui::Entity<Dropdown>,
    focus_handle: &gpui::FocusHandle,
    scroll_handle: &gpui::ScrollHandle,
    cx: &mut gpui::Context<TaskDetailModal>,
    on_close_out: impl Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
    on_close_click: impl Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> gpui::AnyElement {
    let theme = cx.theme().clone();
    let panel = if let Some(message) = state.error.as_ref() {
        render_task_detail_placeholder_panel(
            "Task Details",
            message.as_ref(),
            &theme,
            on_close_click,
        )
    } else if state.loading || state.original.is_none() {
        render_task_detail_placeholder_panel(
            "Task Details",
            "Loading task...",
            &theme,
            on_close_click,
        )
    } else if let Some(detail) = state.original.as_ref() {
        render_task_detail_panel(
            detail,
            state.mode,
            &state.form,
            &state.errors,
            &state.annotations,
            state.modal_focus,
            annotation_input,
            description_input,
            project_input,
            due_input,
            tags_input,
            status_dropdown,
            priority_dropdown,
            scroll_handle,
            &theme,
            cx,
            on_close_click,
        )
    } else {
        render_task_detail_placeholder_panel(
            "Task Details",
            "Loading task...",
            &theme,
            on_close_click,
        )
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
        .child(Label::new("Close (Esc)"));

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
    mode: ModalMode,
    form: &TaskForm,
    errors: &HashMap<FieldId, gpui::SharedString>,
    annotations: &AnnotationState,
    modal_focus: ModalFocus,
    annotation_input: &gpui::Entity<Input>,
    description_input: &gpui::Entity<Input>,
    project_input: &gpui::Entity<Input>,
    due_input: &gpui::Entity<Input>,
    tags_input: &gpui::Entity<Input>,
    status_dropdown: &gpui::Entity<Dropdown>,
    priority_dropdown: &gpui::Entity<Dropdown>,
    scroll_handle: &gpui::ScrollHandle,
    theme: &Theme,
    cx: &mut gpui::Context<TaskDetailModal>,
    on_close_click: OnCloseClick,
) -> gpui::AnyElement
where
    OnCloseClick: Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
{
    let is_editing = mode == ModalMode::Edit;
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

    let priority_value = if is_editing {
        form.priority
    } else {
        detail.overview.priority
    };
    let priority_color = match priority_value {
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
        if !is_editing || !form.project.trim().is_empty() {
            let label = if is_editing {
                form.project.trim()
            } else {
                project.as_str()
            };
            if !label.is_empty() {
                badges.push(chip(label, Theme::alpha(theme.accent, 0.15), theme.accent));
            }
        }
    } else if is_editing && !form.project.trim().is_empty() {
        badges.push(chip(
            form.project.trim(),
            Theme::alpha(theme.accent, 0.15),
            theme.accent,
        ));
    }

    if is_editing {
        badges.push(chip("Editing", Theme::alpha(theme.info, 0.18), theme.info));
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
                    cx.listener(|modal, _event, _window, cx| {
                        modal.enter_edit_mode(cx);
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
        .child(header_actions);

    let label_color = Theme::alpha(theme.foreground, 0.72);
    let value_color = theme.foreground;
    let section_title_color = Theme::alpha(theme.foreground, 0.88);
    let label_width = gpui::rems(10.0);

    let value_label = |value: String| Label::new(value).text_color(value_color).into_any_element();

    let field_row = |label: &str, value: gpui::AnyElement, error: Option<&gpui::SharedString>| {
        let row = gpui::div()
            .flex()
            .items_start()
            .gap_3()
            .child(
                Label::new(label.to_string())
                    .text_color(label_color)
                    .text_sm()
                    .w(label_width),
            )
            .child(gpui::div().flex_1().min_w_0().child(value));

        let mut container = gpui::div().flex().flex_col().gap_1().child(row);
        if let Some(error) = error {
            container = container.child(
                gpui::div()
                    .flex()
                    .items_start()
                    .gap_3()
                    .child(gpui::div().w(label_width))
                    .child(
                        Label::new(error.to_string())
                            .text_xs()
                            .text_color(theme.error),
                    ),
            );
        }
        container
    };

    let kv_row = |label: &str, value: gpui::AnyElement| field_row(label, value, None);

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
                    // Open dropdown in defer so focus is set first
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
        value_label(status_label.clone())
    };
    let description_value = if is_editing {
        let focused = modal_focus == ModalFocus::Description;
        crate::ui::focus_wrap(description_input.clone(), focused, theme)
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|modal, _, window, cx| {
                    modal.set_modal_focus(ModalFocus::Description, Some(window), cx);
                }),
            )
            .into_any_element()
    } else {
        value_label(detail.overview.description.clone())
    };
    let project_value = if is_editing {
        let focused = modal_focus == ModalFocus::Project;
        crate::ui::focus_wrap(project_input.clone(), focused, theme)
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|modal, _, window, cx| {
                    modal.set_modal_focus(ModalFocus::Project, Some(window), cx);
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
                    // Open dropdown in defer so focus is set first
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
        value_label(priority_label.clone())
    };
    let due_value = if is_editing {
        let focused = modal_focus == ModalFocus::Due;
        crate::ui::focus_wrap(due_input.clone(), focused, theme)
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|modal, _, window, cx| {
                    modal.set_modal_focus(ModalFocus::Due, Some(window), cx);
                }),
            )
            .into_any_element()
    } else {
        value_label(due_text)
    };

    let overview_grid = gpui::div()
        .flex()
        .flex_col()
        .gap_2()
        .child(field_row("Status", status_value, None))
        .child(field_row(
            "Description",
            description_value,
            errors.get(&FieldId::Description),
        ))
        .child(field_row("Project", project_value, None))
        .child(field_row("Priority", priority_value, None))
        .child(field_row("Due", due_value, errors.get(&FieldId::Due)));

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

    let tag_chip = |label: &str, removable: bool| {
        let mut chip = gpui::div()
            .flex()
            .items_center()
            .gap_1()
            .px(gpui::rems(0.5))
            .py(gpui::rems(0.125))
            .rounded(gpui::rems(0.25))
            .bg(Theme::alpha(theme.info, 0.18))
            .text_color(theme.info)
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .child(label.to_string());

        if removable {
            let tag_label = label.to_string();
            chip = chip.child(
                gpui::div()
                    .text_color(theme.muted)
                    .cursor_pointer()
                    .hover(|s| s.text_color(theme.error))
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(move |modal, _event, _window, cx| {
                            modal.remove_tag(tag_label.clone(), cx);
                        }),
                    )
                    .child("×"),
            );
        }

        chip
    };

    let tags_content = if is_editing {
        let chips = form
            .tags
            .iter()
            .map(|tag| tag_chip(tag, true).into_any_element());
        let focused = modal_focus == ModalFocus::TagsInput;
        let tags_input_wrapped = crate::ui::focus_wrap(tags_input.clone(), focused, theme)
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|modal, _, window, cx| {
                    modal.set_modal_focus(ModalFocus::TagsInput, Some(window), cx);
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

    let visible_annotations: Vec<&AnnotationView> = annotations
        .items
        .iter()
        .filter(|item| item.origin != AnnotationOrigin::Deleted)
        .collect();

    let mut annotations_content = gpui::div().flex().flex_col().gap_3();
    if visible_annotations.is_empty() {
        annotations_content = annotations_content.child(
            gpui::div()
                .text_sm()
                .text_color(theme.muted)
                .child("No annotations"),
        );
    } else {
        let count = visible_annotations.len();
        let items = visible_annotations
            .iter()
            .enumerate()
            .map(|(index, annotation)| {
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
                    .child(
                        gpui::div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(Label::new(timestamp).text_xs().text_color(theme.muted))
                            .child(actions),
                    )
                    .child(gpui::div().flex().flex_col().gap_1().children(lines));

                if index + 1 < count {
                    item = item.child(gpui::div().mt_2().h(gpui::px(1.0)).bg(theme.divider));
                }

                item.into_any_element()
            });

        annotations_content = annotations_content.children(items);
    }

    if is_editing {
        let can_add = !annotations.draft.as_ref().trim().is_empty();
        let add_button = if can_add {
            gpui::div()
                .px(gpui::rems(0.75))
                .py(gpui::rems(0.35))
                .rounded_md()
                .border_1()
                .border_color(theme.divider)
                .bg(theme.raised)
                .text_color(theme.foreground)
                .text_sm()
                .cursor_pointer()
                .hover(|s| s.bg(theme.hover))
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(|modal, _event, _window, cx| {
                        modal.submit_annotation(cx);
                    }),
                )
                .child(Label::new("Add"))
        } else {
            gpui::div()
                .px(gpui::rems(0.75))
                .py(gpui::rems(0.35))
                .rounded_md()
                .border_1()
                .border_color(theme.divider)
                .bg(theme.raised)
                .text_color(theme.muted)
                .text_sm()
                .child(Label::new("Add"))
        };

        let focused = modal_focus == ModalFocus::AnnotationsInput;
        let annotation_input_wrapped = crate::ui::focus_wrap(annotation_input.clone(), focused, theme)
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|modal, _, window, cx| {
                    modal.set_modal_focus(ModalFocus::AnnotationsInput, Some(window), cx);
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

    let annotations_section = section("Annotations", annotations_content);
    sections.push(annotations_section);

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
    let action_button = |label: &str,
                         enabled: bool,
                         on_click: Option<
        Arc<dyn Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static>,
    >| {
        let label = label.to_string();
        let mut button = gpui::div()
            .px(gpui::rems(0.75))
            .py(gpui::rems(0.35))
            .rounded_md()
            .border_1()
            .border_color(theme.divider)
            .bg(theme.raised)
            .text_sm();

        if enabled {
            if let Some(on_click) = on_click {
                button = button
                    .text_color(theme.foreground)
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.hover))
                    .on_mouse_down(gpui::MouseButton::Left, move |event, window, app| {
                        (on_click)(event, window, app);
                    });
            } else {
                button = button.text_color(theme.foreground);
            }
        } else {
            button = button.text_color(theme.muted);
        }

        button.child(Label::new(label))
    };

    let mut action_row = gpui::div().flex().items_center().gap_2();
    if is_editing {
        let cancel_edit_handler = Arc::new(cx.listener(|modal, _event, _window, cx| {
            modal.cancel_edit(cx);
        }));
        let save_handler = Arc::new(cx.listener(|modal, _event, _window, cx| {
            modal.submit_edits(cx);
        }));
        let annotations_dirty = annotations
            .items
            .iter()
            .any(|item| item.origin != AnnotationOrigin::Original);
        let can_save = errors.is_empty() && (form.is_dirty(detail) || annotations_dirty);
        let cancel_button = action_button("Cancel (Esc)", true, Some(cancel_edit_handler));
        let save_button = action_button("Save (Ctrl+Enter)", can_save, Some(save_handler));
        action_row = action_row.child(cancel_button).child(save_button);
    } else {
        let close_button = action_button("Close (Esc)", true, Some(on_close_footer));
        action_row = action_row.child(close_button);
    }

    let footer = gpui::div()
        .flex()
        .items_center()
        .justify_end()
        .px(gpui::rems(1.0))
        .py(gpui::rems(0.5))
        .border_t_1()
        .border_color(theme.divider)
        .child(action_row);

    // Panel uses deferred() + anchored() for dropdowns, so clipping is not an issue
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
