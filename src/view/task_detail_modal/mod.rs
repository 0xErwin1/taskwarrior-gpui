use chrono::{NaiveDate, TimeZone, Utc};
use gpui::prelude::*;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::components::button::{Dropdown, DropdownItem};
use crate::components::input::Input;
use crate::components::toast::{ToastGlobal, ToastKind};
use crate::keymap::{Command, CommandDispatcher, ContextId};
use crate::task::{self, TaskDetailVm};
use crate::ui::DATE_FORMAT;

mod annotations;
mod bindings;
mod form;
mod history;
mod render;
mod state;

pub use form::TaskEditUpdate;
pub use state::{EditState, ModalFocus};

use self::annotations::{AnnotationId, AnnotationOrigin, AnnotationState};
use self::bindings::FocusMap;
use self::form::{FieldId, TaskForm, build_task_update};
use self::history::FormHistory;
use self::state::{ConfirmAction, InlineEditTarget, ModalMode, TaskModalState};

pub enum TaskDetailModalEvent {
    Closed {
        task_id: Option<uuid::Uuid>,
        was_creating: bool,
    },
    SaveEdits {
        task_id: uuid::Uuid,
        update: TaskEditUpdate,
    },
    CreateTask {
        draft: task::TaskDraft,
        annotations: Vec<String>,
    },
}

struct ModalEntities {
    annotation_input: gpui::Entity<Input>,
    description_input: gpui::Entity<Input>,
    project_input: gpui::Entity<Input>,
    due_input: gpui::Entity<Input>,
    tags_input: gpui::Entity<Input>,
    status_dropdown: gpui::Entity<Dropdown>,
    priority_dropdown: gpui::Entity<Dropdown>,
}

enum CommandResult {
    Handled,
    NotHandled,
}

pub struct TaskDetailModal {
    state: TaskModalState,
    focus_handle: gpui::FocusHandle,
    form_focus_handle: gpui::FocusHandle,
    scroll_handle: gpui::ScrollHandle,
    entities: ModalEntities,
    project_suggestions: Arc<Mutex<Vec<String>>>,
    tag_suggestions: Arc<Mutex<Vec<String>>>,
    form_history: FormHistory,
}

impl TaskDetailModal {
    pub fn new(cx: &mut gpui::Context<Self>) -> Self {
        let modal_entity = cx.entity().clone();
        let project_suggestions: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let tag_suggestions: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        let annotation_input = cx.new(|cx| {
            let on_change_entity = modal_entity.clone();

            Input::new("annotation-input", cx, "Add annotation...")
                .multiline()
                .with_on_change(Arc::new(move |value, cx| {
                    cx.defer({
                        let entity = on_change_entity.clone();
                        let v = value.to_string();
                        move |cx| {
                            let _ = entity.update(cx, |modal, cx| {
                                modal.state.annotations.draft = v.into();
                                cx.notify();
                            });
                        }
                    });
                }))
        });

        let description_input =
            cx.new(|cx| Input::new("task-edit-description", cx, "Description").multiline());

        let project_input = {
            let suggestions = project_suggestions.clone();

            cx.new(|cx| {
                Input::new("task-edit-project", cx, "Project").with_suggest(Arc::new(
                    move |query| {
                        let query = query.trim();
                        if query.is_empty() {
                            return Vec::new();
                        }

                        let Ok(list) = suggestions.lock() else {
                            return Vec::new();
                        };

                        let needle = query.to_lowercase();
                        let mut candidates = Vec::new();
                        let mut seen = HashSet::new();

                        for project in list.iter() {
                            let parts: Vec<&str> = project
                                .split('.')
                                .filter(|part| !part.trim().is_empty())
                                .collect();
                            if parts.is_empty() {
                                continue;
                            }

                            let mut prefix = String::new();
                            for (idx, part) in parts.iter().enumerate() {
                                if !prefix.is_empty() {
                                    prefix.push('.');
                                }
                                prefix.push_str(part);

                                let key = prefix.to_lowercase();
                                if seen.insert(key) {
                                    candidates.push(prefix.clone());
                                }

                                if idx + 1 < parts.len() {
                                    let mut with_dot = prefix.clone();
                                    with_dot.push('.');
                                    let key = with_dot.to_lowercase();
                                    if seen.insert(key) {
                                        candidates.push(with_dot);
                                    }
                                }
                            }
                        }

                        let mut level_matches = Vec::new();
                        let mut prefix_matches = Vec::new();
                        let mut contains_matches = Vec::new();

                        for candidate in candidates {
                            let hay = candidate.to_lowercase();
                            if hay.starts_with(&needle) {
                                let boundary = hay.len() == needle.len()
                                    || hay.as_bytes().get(needle.len()) == Some(&b'.')
                                    || needle.ends_with('.');
                                if boundary {
                                    level_matches.push(candidate);
                                } else {
                                    prefix_matches.push(candidate);
                                }
                            } else if hay.contains(&needle) {
                                contains_matches.push(candidate);
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
                    },
                ))
            })
        };

        let due_input = cx.new(|cx| Input::new("task-edit-due", cx, "Due (YYYY-MM-DD)"));

        let tags_input = {
            let on_change_entity = modal_entity.clone();
            let suggestions = tag_suggestions.clone();

            cx.new(|cx| {
                Input::new("task-edit-tags", cx, "Add tag")
                    .with_suggest(Arc::new(move |query| {
                        let (prefix, needle) = match query
                            .char_indices()
                            .rev()
                            .find(|(_, ch)| ch.is_whitespace() || *ch == ',')
                        {
                            Some((idx, _)) => (&query[..=idx], &query[idx + 1..]),
                            None => ("", query),
                        };
                        let needle = needle.trim();
                        if needle.is_empty() {
                            return Vec::new();
                        }

                        let Ok(list) = suggestions.lock() else {
                            return Vec::new();
                        };

                        let needle_lower = needle.to_lowercase();
                        let mut prefix_matches = Vec::new();
                        let mut contains_matches = Vec::new();

                        for tag in list.iter() {
                            let hay = tag.to_lowercase();
                            if hay.starts_with(&needle_lower) {
                                prefix_matches.push(tag.clone());
                            } else if hay.contains(&needle_lower) {
                                contains_matches.push(tag.clone());
                            }
                        }

                        prefix_matches.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
                        contains_matches.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

                        prefix_matches
                            .into_iter()
                            .chain(contains_matches)
                            .take(8)
                            .map(|tag| {
                                let mut insert = String::new();
                                insert.push_str(prefix);
                                insert.push_str(&tag);
                                crate::components::input::Suggestion::new(tag, insert)
                            })
                            .collect()
                    }))
                    .with_on_change(Arc::new(move |value, cx| {
                        cx.defer({
                            let entity = on_change_entity.clone();
                            let v = value.to_string();
                            move |cx| {
                                let _ = entity.update(cx, |modal, cx| {
                                    modal.state.form.tag_draft = v;
                                    cx.notify();
                                });
                            }
                        });
                    }))
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
                        let modal_entity = modal_entity.clone();
                        cx.defer(move |cx| {
                            let _ = modal_entity.update(cx, |modal, cx| {
                                modal.state.form.status = match index {
                                    0 => task::TaskStatus::Pending,
                                    1 => task::TaskStatus::Completed,
                                    2 => task::TaskStatus::Deleted,
                                    _ => task::TaskStatus::Pending,
                                };
                                cx.notify();
                            });
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
                        let modal_entity = modal_entity.clone();
                        cx.defer(move |cx| {
                            let _ = modal_entity.update(cx, |modal, cx| {
                                modal.state.form.priority = match index {
                                    0 => task::TaskPriority::High,
                                    1 => task::TaskPriority::Medium,
                                    2 => task::TaskPriority::Low,
                                    3 => task::TaskPriority::None,
                                    _ => task::TaskPriority::None,
                                };
                                cx.notify();
                            });
                        });
                    }))
            })
        };

        let entities = ModalEntities {
            annotation_input,
            description_input,
            project_input,
            due_input,
            tags_input,
            status_dropdown,
            priority_dropdown,
        };

        Self {
            state: TaskModalState::default(),
            focus_handle: cx.focus_handle(),
            form_focus_handle: cx.focus_handle(),
            scroll_handle: gpui::ScrollHandle::new(),
            entities,
            project_suggestions,
            tag_suggestions,
            form_history: FormHistory::default(),
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

    pub fn set_tag_suggestions(&mut self, tags: Vec<String>, _cx: &mut gpui::Context<Self>) {
        if let Ok(mut list) = self.tag_suggestions.lock() {
            *list = tags;
        }
    }

    fn placeholder_detail() -> TaskDetailVm {
        let task = task::Task::new(
            uuid::Uuid::nil(),
            None,
            task::TaskStatus::Pending,
            String::new(),
            None,
            task::TaskPriority::None,
            HashSet::new(),
            None,
            None,
            None,
            None,
            vec![],
            HashSet::new(),
            false,
            false,
            None,
        );

        TaskDetailVm::from_task(&task, &[])
    }

    pub fn open_with_detail(
        &mut self,
        detail: TaskDetailVm,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        self.open_with_detail_mode(detail, false, window, cx);
    }

    pub fn open_with_detail_edit(
        &mut self,
        detail: TaskDetailVm,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        self.open_with_detail_mode(detail, true, window, cx);
    }

    pub fn open_create(&mut self, window: Option<&mut gpui::Window>, cx: &mut gpui::Context<Self>) {
        self.reset_open_state();
        self.state.task_id = None;
        self.state.loading = false;
        self.state.error = None;
        self.state.original = Some(Self::placeholder_detail());
        self.state.form = TaskForm::default();
        self.state.errors.clear();
        self.state.is_create = true;
        self.state.mode = ModalMode::Edit;
        self.state.edit_state = EditState::Navigating;
        self.state.modal_focus = ModalFocus::Description;
        self.state.annotations = AnnotationState::default();
        self.reset_pending_state();
        self.apply_form_inputs(cx);
        self.form_history.clear();
        self.form_history.push(self.state.form.clone());

        if let Some(window) = window {
            window.focus(&self.form_focus_handle);
        }

        cx.notify();
    }

    fn reset_open_state(&mut self) {
        self.scroll_handle = gpui::ScrollHandle::new();
        self.scroll_handle.scroll_to_item(0);
        self.state.open = true;
    }

    fn reset_pending_state(&mut self) {
        self.state.pending_confirm = None;
        self.form_history.clear();
    }

    fn apply_empty_state(
        &mut self,
        task_id: uuid::Uuid,
        loading: bool,
        error: Option<gpui::SharedString>,
    ) {
        self.state.task_id = Some(task_id);
        self.state.loading = loading;
        self.state.original = None;
        self.state.form = TaskForm::default();
        self.state.errors.clear();
        self.state.mode = ModalMode::View;
        self.state.is_create = false;
        self.state.edit_state = EditState::Navigating;
        self.state.annotations = AnnotationState::default();
        self.state.error = error;
        self.reset_pending_state();
    }

    fn open_with_detail_mode(
        &mut self,
        detail: TaskDetailVm,
        edit_mode: bool,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        self.reset_open_state();
        self.state.task_id = Some(detail.identity.uuid);
        self.state.loading = false;
        self.state.error = None;
        self.state.original = Some(detail.clone());
        self.state.errors.clear();
        self.state.is_create = false;
        self.reset_pending_state();
        self.sync_from_detail(&detail, cx, true);

        if edit_mode {
            self.state.mode = ModalMode::Edit;
            self.state.edit_state = EditState::Navigating;
            self.state.modal_focus = ModalFocus::StatusDropdown;
            self.form_history.push(self.state.form.clone());

            if let Some(window) = window {
                window.focus(&self.form_focus_handle);
            }
        } else {
            self.state.mode = ModalMode::View;
            self.state.edit_state = EditState::Navigating;

            if let Some(window) = window {
                window.focus(&self.focus_handle);
            }
        }

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

        self.reset_open_state();
        self.apply_empty_state(task_id, false, Some(error.into()));
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

        self.reset_open_state();
        self.apply_empty_state(task_id, true, None);
        cx.notify();
    }

    pub fn set_detail(&mut self, detail: TaskDetailVm, cx: &mut gpui::Context<Self>) {
        let reset_form = self.state.mode == ModalMode::View;
        self.state.task_id = Some(detail.identity.uuid);
        self.state.loading = false;
        self.state.error = None;
        self.state.original = Some(detail.clone());
        self.state.is_create = false;
        self.sync_from_detail(&detail, cx, reset_form);
        cx.notify();
    }

    pub fn set_error(&mut self, task_id: uuid::Uuid, error: String, cx: &mut gpui::Context<Self>) {
        self.apply_empty_state(task_id, false, Some(error.into()));
        cx.notify();
    }

    fn sync_from_detail(
        &mut self,
        detail: &TaskDetailVm,
        cx: &mut gpui::Context<Self>,
        reset_edit: bool,
    ) {
        self.state.annotations = AnnotationState::from_detail(detail);
        self.entities.annotation_input.update(cx, |input, cx| {
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
        self.state.edit_state = EditState::Navigating;
        self.sync_from_detail(&detail, cx, true);
        self.state.task_id = Some(detail.identity.uuid);
        self.state.loading = false;
        self.state.original = Some(detail);
        self.state.error = None;
        self.state.is_create = false;
        self.reset_pending_state();
        cx.notify();
    }

    fn enter_edit_mode(&mut self, window: Option<&mut gpui::Window>, cx: &mut gpui::Context<Self>) {
        if self.state.is_create {
            return;
        }

        let Some(detail) = self.state.original.clone() else {
            return;
        };

        self.state.mode = ModalMode::Edit;
        self.state.edit_state = EditState::Navigating;
        self.state.modal_focus = ModalFocus::StatusDropdown;
        self.sync_from_detail(&detail, cx, true);
        self.form_history.clear();
        self.form_history.push(self.state.form.clone());

        // Focus the form handle so we can navigate with j/k
        if let Some(window) = window {
            window.focus(&self.form_focus_handle);
        }

        cx.notify();
    }

    pub fn cancel_edit(&mut self, window: Option<&mut gpui::Window>, cx: &mut gpui::Context<Self>) {
        if self.state.is_create {
            self.close(window, cx);
            return;
        }

        let Some(detail) = self.state.original.clone() else {
            return;
        };

        self.state.mode = ModalMode::View;
        self.state.edit_state = EditState::Navigating;
        self.state.modal_focus = ModalFocus::None;
        self.sync_from_detail(&detail, cx, true);
        self.reset_pending_state();

        // Restore focus to modal main handle
        if let Some(window) = window {
            window.focus(&self.focus_handle);
        }

        cx.notify();
    }

    pub fn is_editing(&self) -> bool {
        self.state.mode == ModalMode::Edit && !self.state.is_create
    }

    pub fn is_creating(&self) -> bool {
        self.state.mode == ModalMode::Edit && self.state.is_create
    }

    pub fn task_id(&self) -> Option<uuid::Uuid> {
        self.state.task_id
    }

    fn apply_form_inputs(&mut self, cx: &mut gpui::Context<Self>) {
        let form = self.state.form.clone();

        self.entities.description_input.update(cx, |input, cx| {
            input.set_value_silent(form.description, cx);
        });

        self.entities.project_input.update(cx, |input, cx| {
            input.set_value_silent(form.project, cx);
        });

        self.entities.due_input.update(cx, |input, cx| {
            input.set_value_silent(form.due, cx);
        });

        self.entities.tags_input.update(cx, |input, cx| {
            input.set_value_silent(form.tag_draft, cx);
        });

        let status_index = match form.status {
            task::TaskStatus::Pending => Some(0),
            task::TaskStatus::Completed => Some(1),
            task::TaskStatus::Deleted => Some(2),
            _ => None,
        };
        if let Some(status_index) = status_index {
            self.entities.status_dropdown.update(cx, |dropdown, cx| {
                dropdown.set_selected_index(status_index, cx);
            });
        }

        let priority_index = match form.priority {
            task::TaskPriority::High => 0,
            task::TaskPriority::Medium => 1,
            task::TaskPriority::Low => 2,
            task::TaskPriority::None => 3,
        };
        self.entities.priority_dropdown.update(cx, |dropdown, cx| {
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

    fn sync_form_from_inputs(&mut self, cx: &gpui::Context<Self>) {
        self.state.form.description = self.entities.description_input.read(cx).value().to_string();
        self.state.form.project = self.entities.project_input.read(cx).value().to_string();
        self.state.form.due = self.entities.due_input.read(cx).value().to_string();
        self.state.form.tag_draft = self.entities.tags_input.read(cx).value().to_string();
        self.state.annotations.draft = self
            .entities
            .annotation_input
            .read(cx)
            .value()
            .to_string()
            .into();
    }

    pub fn submit_edits(&mut self, cx: &mut gpui::Context<Self>) {
        if self.state.mode != ModalMode::Edit {
            self.close(None, cx);
            return;
        }

        if self.state.is_create {
            self.submit_create(cx);
            return;
        }

        if self.state.original.is_none() {
            return;
        }

        // Sync form from inputs before validating
        self.sync_form_from_inputs(cx);

        self.state.errors = self.state.form.validate();
        if !self.state.errors.is_empty() {
            cx.notify();
            return;
        }

        let detail = self.state.original.as_ref().unwrap();
        let update = build_task_update(detail, &self.state.form, &self.state.annotations);

        if update.is_empty() {
            self.cancel_edit(None, cx);
            return;
        }

        cx.emit(TaskDetailModalEvent::SaveEdits {
            task_id: detail.identity.uuid,
            update,
        });
    }

    fn submit_create(&mut self, cx: &mut gpui::Context<Self>) {
        if self.state.mode != ModalMode::Edit || !self.state.is_create {
            return;
        }

        self.sync_form_from_inputs(cx);
        self.state.errors = self.state.form.validate();
        if !self.state.errors.is_empty() {
            cx.notify();
            return;
        }

        let description = self.state.form.description.trim().to_string();
        let project = match self.state.form.project.trim() {
            "" => None,
            value => Some(value.to_string()),
        };

        let due = if self.state.form.due.trim().is_empty() {
            None
        } else {
            NaiveDate::parse_from_str(self.state.form.due.trim(), DATE_FORMAT)
                .ok()
                .and_then(|date| date.and_hms_opt(0, 0, 0))
                .map(|date| Utc.from_utc_datetime(&date))
        };

        let priority = match self.state.form.priority {
            task::TaskPriority::None => None,
            value => Some(value),
        };

        let status = match self.state.form.status.clone() {
            task::TaskStatus::Pending => None,
            value => Some(value),
        };

        let tags = self.state.form.tags.iter().cloned().collect();

        let annotations = self
            .state
            .annotations
            .items
            .iter()
            .filter_map(|item| match item.origin {
                AnnotationOrigin::Added => Some(item.text.to_string()),
                _ => None,
            })
            .collect();

        cx.emit(TaskDetailModalEvent::CreateTask {
            draft: task::TaskDraft {
                description,
                project,
                priority,
                status,
                tags,
                due,
            },
            annotations,
        });
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
        self.entities.tags_input.update(cx, |input, cx| {
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
        let text = self.state.annotations.draft.trim().to_string();
        if text.is_empty() {
            let toast_host = cx.global::<ToastGlobal>().host.clone();
            cx.update_entity(&toast_host, |host, cx| {
                host.push(ToastKind::Error, "Annotation cannot be empty", cx);
            });
            return;
        }

        self.state.annotations.add_local(text.into(), Utc::now());
        self.state.annotations.clear_draft();
        self.entities.annotation_input.update(cx, |input, cx| {
            input.clear(cx);
        });
        cx.notify();
    }

    fn delete_annotation(&mut self, id: AnnotationId, cx: &mut gpui::Context<Self>) {
        if self.state.annotations.mark_deleted(id).is_some() {
            cx.notify();
        }
    }

    pub fn close(&mut self, window: Option<&mut gpui::Window>, cx: &mut gpui::Context<Self>) {
        if !self.state.open {
            return;
        }

        if let Some(window) = window {
            self.entities
                .description_input
                .update(cx, |input, cx| input.blur(window, cx));
            self.entities
                .project_input
                .update(cx, |input, cx| input.blur(window, cx));
            self.entities
                .due_input
                .update(cx, |input, cx| input.blur(window, cx));
            self.entities
                .tags_input
                .update(cx, |input, cx| input.blur(window, cx));
            self.entities
                .annotation_input
                .update(cx, |input, cx| input.blur(window, cx));
        }

        let task_id = self.state.task_id;
        let was_creating = self.state.is_create;
        self.state = TaskModalState::default();
        cx.emit(TaskDetailModalEvent::Closed {
            task_id,
            was_creating,
        });
        cx.notify();
    }

    pub fn set_modal_focus(
        &mut self,
        focus: ModalFocus,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        self.set_focus(focus, window, cx);
    }

    pub fn set_modal_focus_and_edit(
        &mut self,
        focus: ModalFocus,
        mut window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        let window_ref = window.as_deref_mut();
        self.set_modal_focus(focus, window_ref, cx);

        if self.state.edit_state == EditState::Navigating {
            self.enter_edit_field(window, cx);
        }
    }

    fn set_focus(
        &mut self,
        focus: ModalFocus,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        if self.state.modal_focus != focus {
            self.close_all_dropdowns(cx);
        }

        self.state.modal_focus = focus;

        if let Some(window) = window {
            self.apply_focus(focus, window, cx);
        }

        self.apply_scroll(focus);
        cx.notify();
    }

    fn apply_focus(
        &mut self,
        focus: ModalFocus,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        if self.state.edit_state == EditState::Navigating {
            window.focus(&self.form_focus_handle);
            return;
        }

        if FocusMap::wants_input_focus(focus) {
            match focus {
                ModalFocus::Description => {
                    self.entities
                        .description_input
                        .update(cx, |i, cx| i.focus(window, cx));
                }
                ModalFocus::Project => {
                    self.entities
                        .project_input
                        .update(cx, |i, cx| i.focus(window, cx));
                }
                ModalFocus::Due => {
                    self.entities
                        .due_input
                        .update(cx, |i, cx| i.focus(window, cx));
                }
                ModalFocus::TagsInput => {
                    self.entities
                        .tags_input
                        .update(cx, |i, cx| i.focus(window, cx));
                }
                ModalFocus::AnnotationsInput => {
                    self.entities
                        .annotation_input
                        .update(cx, |i, cx| i.focus(window, cx));
                }
                ModalFocus::None | ModalFocus::StatusDropdown | ModalFocus::PriorityDropdown => {}
            }
            return;
        }

        match focus {
            ModalFocus::StatusDropdown | ModalFocus::PriorityDropdown => {
                window.focus(&self.form_focus_handle);
            }
            ModalFocus::None => {
                window.focus(&self.focus_handle);
            }
            ModalFocus::Description
            | ModalFocus::Project
            | ModalFocus::Due
            | ModalFocus::TagsInput
            | ModalFocus::AnnotationsInput => {}
        }
    }

    fn apply_scroll(&mut self, focus: ModalFocus) {
        let section_index = FocusMap::section_index(focus);
        self.scroll_handle.scroll_to_item(section_index);
    }

    pub fn get_modal_focus(&self) -> ModalFocus {
        self.state.modal_focus
    }

    fn close_all_dropdowns(&mut self, cx: &mut gpui::Context<Self>) {
        self.entities
            .status_dropdown
            .update(cx, |d, cx| d.close(cx));
        self.entities
            .priority_dropdown
            .update(cx, |d, cx| d.close(cx));
    }

    fn has_open_dropdown(&self, cx: &gpui::Context<Self>) -> bool {
        let status_open = self.entities.status_dropdown.read(cx).is_open();
        let priority_open = self.entities.priority_dropdown.read(cx).is_open();
        status_open || priority_open
    }

    pub fn toggle_focused_dropdown(&mut self, cx: &mut gpui::Context<Self>) {
        match self.state.modal_focus {
            ModalFocus::StatusDropdown => {
                let was_open = self.entities.status_dropdown.read(cx).is_open();
                self.entities.status_dropdown.update(cx, |d, cx| {
                    if d.is_open() {
                        d.accept_selection(cx);
                    } else {
                        d.open(cx);
                    }
                });
                if was_open {
                    if let Some(index) = self
                        .entities
                        .status_dropdown
                        .read(cx)
                        .selected_index_value()
                    {
                        self.update_edit_status(index, cx);
                    }
                }
            }
            ModalFocus::PriorityDropdown => {
                let was_open = self.entities.priority_dropdown.read(cx).is_open();
                self.entities.priority_dropdown.update(cx, |d, cx| {
                    if d.is_open() {
                        d.accept_selection(cx);
                    } else {
                        d.open(cx);
                    }
                });
                if was_open {
                    if let Some(index) = self
                        .entities
                        .priority_dropdown
                        .read(cx)
                        .selected_index_value()
                    {
                        self.update_edit_priority(index, cx);
                    }
                }
            }
            _ => {}
        }
        cx.notify();
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
            ModalFocus::StatusDropdown => select_next(&self.entities.status_dropdown, cx),
            ModalFocus::PriorityDropdown => select_next(&self.entities.priority_dropdown, cx),
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
            ModalFocus::StatusDropdown => select_prev(&self.entities.status_dropdown, cx),
            ModalFocus::PriorityDropdown => select_prev(&self.entities.priority_dropdown, cx),
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
            None | StatusDropdown => Description,
            Description => Project,
            Project => PriorityDropdown,
            PriorityDropdown => Due,
            Due => TagsInput,
            TagsInput => AnnotationsInput,
            AnnotationsInput => StatusDropdown,
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
            None | StatusDropdown => AnnotationsInput,
            AnnotationsInput => TagsInput,
            TagsInput => Due,
            Due => PriorityDropdown,
            PriorityDropdown => Project,
            Project => Description,
            Description => StatusDropdown,
        };

        if matches!(self.state.modal_focus, None | StatusDropdown)
            && prev == AnnotationsInput
            && self.state.annotation_selected.is_none()
        {
            let visible_count = self
                .state
                .annotations
                .items
                .iter()
                .filter(|a| a.origin != AnnotationOrigin::Deleted)
                .count();
            if visible_count > 0 {
                self.state.annotation_selected = Some(visible_count - 1);
            }
        }

        self.set_modal_focus(prev, window, cx);
    }

    pub fn active_context(&self) -> ContextId {
        if !self.state.open {
            return ContextId::Global;
        }

        match self.state.mode {
            ModalMode::View => ContextId::Modal,
            ModalMode::Edit => match self.state.edit_state {
                EditState::Navigating => ContextId::ModalEditNav,
                EditState::Editing => ContextId::ModalInput,
                EditState::DropdownOpen => ContextId::ModalDropdown,
            },
        }
    }

    pub fn get_edit_state(&self) -> EditState {
        self.state.edit_state
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

        matches!(
            self.handle_command(command, window, cx),
            CommandResult::Handled
        )
    }

    fn handle_command(
        &mut self,
        command: Command,
        mut window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) -> CommandResult {
        let command = if self.state.pending_confirm.is_some() {
            match command {
                Command::ConfirmYes => Command::ConfirmYes,
                Command::EditSelectedItem => Command::ConfirmYes,
                Command::ConfirmNo | Command::CloseModal => {
                    self.state.pending_confirm = None;
                    cx.notify();
                    return CommandResult::Handled;
                }
                _ => {
                    return CommandResult::NotHandled;
                }
            }
        } else {
            command
        };

        match command {
            Command::CloseModal => {
                self.handle_close_or_escape(window.as_deref_mut(), cx);
                CommandResult::Handled
            }
            Command::SaveModal => self.handle_save_modal(cx),
            Command::EnterEditMode => self.handle_enter_edit_mode(window.as_deref_mut(), cx),
            Command::EnterEditField => self.handle_enter_edit_field(window.as_deref_mut(), cx),
            Command::ExitEditField => self.handle_exit_edit_field(window.as_deref_mut(), cx),
            Command::SubmitOrExitField => {
                self.handle_submit_or_exit_field(window.as_deref_mut(), cx)
            }
            Command::ModalFocusNext => self.handle_modal_focus_next(window.as_deref_mut(), cx),
            Command::ModalFocusPrev => self.handle_modal_focus_prev(window.as_deref_mut(), cx),
            Command::Undo => self.handle_undo(cx),
            Command::Redo => self.handle_redo(cx),
            Command::ModalItemNext => self.handle_modal_item_next(cx),
            Command::ModalItemPrev => self.handle_modal_item_prev(cx),
            Command::DeleteSelectedItem => self.handle_delete_selected_item(cx),
            Command::ConfirmYes => self.handle_confirm_yes(window.as_deref_mut(), cx),
            Command::ConfirmNo => self.handle_confirm_no(cx),
            Command::CopySelectedItem => self.handle_copy_selected_item(cx),
            Command::EditSelectedItem => self.handle_edit_selected_item(window.as_deref_mut(), cx),
            Command::ModalScrollDown => self.handle_scroll_command(1, cx),
            Command::ModalScrollUp => self.handle_scroll_command(-1, cx),
            Command::FocusFilterNext => self.handle_focus_filter_next(window.as_deref_mut(), cx),
            Command::FocusFilterPrev => self.handle_focus_filter_prev(window.as_deref_mut(), cx),
            Command::ToggleDropdown => self.handle_toggle_dropdown(cx),
            Command::SelectNextOption => self.handle_select_next_option(cx),
            Command::SelectPrevOption => self.handle_select_prev_option(cx),
            _ => CommandResult::NotHandled,
        }
    }

    fn handle_save_modal(&mut self, cx: &mut gpui::Context<Self>) -> CommandResult {
        if self.state.mode == ModalMode::Edit {
            self.submit_edits(cx);
            CommandResult::Handled
        } else {
            CommandResult::NotHandled
        }
    }

    fn handle_enter_edit_mode(
        &mut self,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) -> CommandResult {
        if self.state.mode == ModalMode::View {
            self.enter_edit_mode(window, cx);
            CommandResult::Handled
        } else {
            CommandResult::NotHandled
        }
    }

    fn handle_enter_edit_field(
        &mut self,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) -> CommandResult {
        if self.state.mode == ModalMode::Edit && self.state.edit_state == EditState::Navigating {
            self.enter_edit_field(window, cx);
            CommandResult::Handled
        } else {
            CommandResult::NotHandled
        }
    }

    fn handle_exit_edit_field(
        &mut self,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) -> CommandResult {
        if self.state.mode == ModalMode::Edit && self.state.edit_state != EditState::Navigating {
            self.exit_edit_field(window, cx);
            CommandResult::Handled
        } else {
            CommandResult::NotHandled
        }
    }

    fn handle_submit_or_exit_field(
        &mut self,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) -> CommandResult {
        if self.state.mode != ModalMode::Edit || self.state.edit_state != EditState::Editing {
            return CommandResult::NotHandled;
        }

        match self.state.modal_focus {
            ModalFocus::TagsInput => {
                self.sync_form_from_inputs(cx);

                if let Some(InlineEditTarget::Tag(i)) = self.state.inline_edit {
                    let new_value = self.state.form.tag_draft.trim();
                    if !new_value.is_empty() && i < self.state.form.tags.len() {
                        self.state.form.tags[i] = new_value.to_string();
                    }

                    self.state.inline_edit = None;
                    self.state.form.tag_draft.clear();
                    self.entities
                        .tags_input
                        .update(cx, |input, cx| input.clear(cx));

                    self.exit_edit_field(window, cx);
                } else {
                    self.submit_tag_draft(cx);
                }
                CommandResult::Handled
            }
            ModalFocus::AnnotationsInput => {
                self.sync_form_from_inputs(cx);

                if let Some(InlineEditTarget::Annotation(i)) = self.state.inline_edit {
                    let new_value = self.state.annotations.draft.trim().to_string();
                    if !new_value.is_empty() && i < self.state.annotations.items.len() {
                        self.state.annotations.items[i].text = new_value.into();
                    }

                    self.state.inline_edit = None;
                    self.state.annotations.clear_draft();
                    self.entities
                        .annotation_input
                        .update(cx, |input, cx| input.clear(cx));

                    self.exit_edit_field(window, cx);
                } else {
                    self.submit_annotation(cx);
                }
                CommandResult::Handled
            }
            ModalFocus::Project => {
                let accepted = self
                    .entities
                    .project_input
                    .update(cx, |input, _cx| input.consume_suggestion_accept());
                if accepted {
                    CommandResult::Handled
                } else if self.project_suggestions_open(cx) {
                    CommandResult::NotHandled
                } else {
                    self.exit_edit_field(window, cx);
                    CommandResult::Handled
                }
            }
            ModalFocus::Description => CommandResult::NotHandled,
            _ => {
                self.exit_edit_field(window, cx);
                CommandResult::Handled
            }
        }
    }

    fn handle_modal_focus_next(
        &mut self,
        mut window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) -> CommandResult {
        if self.state.mode != ModalMode::Edit {
            return CommandResult::NotHandled;
        }

        if self.state.edit_state == EditState::Editing {
            self.exit_edit_field(window.as_deref_mut(), cx);
        }

        if self.state.edit_state == EditState::Navigating {
            if self.state.modal_focus == ModalFocus::AnnotationsInput {
                if self.move_annotation_selection(true, cx) {
                    return CommandResult::Handled;
                }
            }
            self.focus_next_field(window.as_deref_mut(), cx);
        }

        CommandResult::Handled
    }

    fn handle_modal_focus_prev(
        &mut self,
        mut window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) -> CommandResult {
        if self.state.mode != ModalMode::Edit {
            return CommandResult::NotHandled;
        }

        if self.state.edit_state == EditState::Editing {
            self.exit_edit_field(window.as_deref_mut(), cx);
        }

        if self.state.edit_state == EditState::Navigating {
            if self.state.modal_focus == ModalFocus::AnnotationsInput {
                if self.move_annotation_selection(false, cx) {
                    return CommandResult::Handled;
                }
            }
            self.focus_prev_field(window.as_deref_mut(), cx);
        }

        CommandResult::Handled
    }

    fn handle_undo(&mut self, cx: &mut gpui::Context<Self>) -> CommandResult {
        if self.state.mode == ModalMode::Edit {
            self.undo(cx);
            CommandResult::Handled
        } else {
            CommandResult::NotHandled
        }
    }

    fn handle_redo(&mut self, cx: &mut gpui::Context<Self>) -> CommandResult {
        if self.state.mode == ModalMode::Edit {
            self.redo(cx);
            CommandResult::Handled
        } else {
            CommandResult::NotHandled
        }
    }

    fn handle_modal_item_next(&mut self, cx: &mut gpui::Context<Self>) -> CommandResult {
        if self.state.mode != ModalMode::Edit || self.state.edit_state != EditState::Navigating {
            return CommandResult::NotHandled;
        }

        match self.state.modal_focus {
            ModalFocus::TagsInput => {
                if self.state.form.tags.is_empty() {
                    return CommandResult::NotHandled;
                }

                let len = self.state.form.tags.len();
                self.state.tag_selected = match self.state.tag_selected {
                    None => Some(0),
                    Some(i) => {
                        let next = i + 1;
                        if next >= len { None } else { Some(next) }
                    }
                };
                cx.notify();
                CommandResult::Handled
            }
            _ => CommandResult::NotHandled,
        }
    }

    fn handle_modal_item_prev(&mut self, cx: &mut gpui::Context<Self>) -> CommandResult {
        if self.state.mode != ModalMode::Edit || self.state.edit_state != EditState::Navigating {
            return CommandResult::NotHandled;
        }

        match self.state.modal_focus {
            ModalFocus::TagsInput => {
                if self.state.form.tags.is_empty() {
                    return CommandResult::NotHandled;
                }

                let len = self.state.form.tags.len();
                self.state.tag_selected = match self.state.tag_selected {
                    None => Some(len - 1),
                    Some(0) => None,
                    Some(i) => Some(i - 1),
                };
                cx.notify();
                CommandResult::Handled
            }
            _ => CommandResult::NotHandled,
        }
    }

    fn handle_delete_selected_item(&mut self, cx: &mut gpui::Context<Self>) -> CommandResult {
        if self.state.mode != ModalMode::Edit
            || self.state.edit_state != EditState::Navigating
            || self.state.pending_confirm.is_some()
        {
            return CommandResult::NotHandled;
        }

        match self.state.modal_focus {
            ModalFocus::TagsInput => {
                if let Some(i) = self.state.tag_selected {
                    if i < self.state.form.tags.len() {
                        let text = self.state.form.tags[i].clone();
                        self.state.pending_confirm =
                            Some(ConfirmAction::DeleteTag { index: i, text });
                        cx.notify();
                        return CommandResult::Handled;
                    }
                }
                CommandResult::NotHandled
            }
            ModalFocus::AnnotationsInput => {
                if let Some(i) = self.state.annotation_selected {
                    let visible: Vec<_> = self
                        .state
                        .annotations
                        .items
                        .iter()
                        .enumerate()
                        .filter(|(_, a)| a.origin != AnnotationOrigin::Deleted)
                        .collect();

                    if i < visible.len() {
                        let (actual_index, ann) = visible[i];
                        let text = ann.text.to_string();
                        let text_single_line = text.replace('\n', " ").replace('\r', " ");
                        let text_single_line = text_single_line.trim().to_string();
                        let text_preview = if text_single_line.len() > 50 {
                            format!("{}...", &text_single_line[..47])
                        } else {
                            text_single_line
                        };
                        self.state.pending_confirm = Some(ConfirmAction::DeleteAnnotation {
                            index: actual_index,
                            text: text_preview,
                        });
                        cx.notify();
                        return CommandResult::Handled;
                    }
                }
                CommandResult::NotHandled
            }
            _ => CommandResult::NotHandled,
        }
    }

    fn handle_confirm_yes(
        &mut self,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) -> CommandResult {
        if let Some(confirm) = self.state.pending_confirm.take() {
            match confirm {
                ConfirmAction::DeleteTag { index, .. } => {
                    if index < self.state.form.tags.len() {
                        self.state.form.tags.remove(index);
                        // Adjust selection
                        if self.state.form.tags.is_empty() {
                            self.state.tag_selected = None;
                        } else if let Some(sel) = self.state.tag_selected {
                            self.state.tag_selected = Some(sel.min(self.state.form.tags.len() - 1));
                        }
                    }
                }
                ConfirmAction::DeleteAnnotation { index, .. } => {
                    if index < self.state.annotations.items.len() {
                        self.state.annotations.items[index].origin = AnnotationOrigin::Deleted;

                        let visible_count = self
                            .state
                            .annotations
                            .items
                            .iter()
                            .filter(|a| a.origin != AnnotationOrigin::Deleted)
                            .count();

                        if visible_count == 0 {
                            self.state.annotation_selected = None;
                        } else if let Some(sel) = self.state.annotation_selected {
                            self.state.annotation_selected = Some(sel.min(visible_count - 1));
                        }
                    }
                }
                ConfirmAction::DiscardUnsavedChanges => {
                    self.cancel_edit(window, cx);
                }
                ConfirmAction::DiscardUnsavedChangesAndClose => {
                    self.close(window, cx);
                }
            }
            cx.notify();
            return CommandResult::Handled;
        }

        self.handle_command(Command::CopySelectedItem, window, cx)
    }

    fn handle_confirm_no(&mut self, cx: &mut gpui::Context<Self>) -> CommandResult {
        if self.state.pending_confirm.take().is_some() {
            cx.notify();
            CommandResult::Handled
        } else {
            CommandResult::NotHandled
        }
    }

    fn handle_copy_selected_item(&mut self, cx: &mut gpui::Context<Self>) -> CommandResult {
        if self.state.mode != ModalMode::Edit || self.state.edit_state != EditState::Navigating {
            return CommandResult::NotHandled;
        }

        match self.state.modal_focus {
            ModalFocus::AnnotationsInput => {
                if let Some(i) = self.state.annotation_selected {
                    let visible: Vec<_> = self
                        .state
                        .annotations
                        .items
                        .iter()
                        .filter(|a| a.origin != AnnotationOrigin::Deleted)
                        .collect();

                    if i < visible.len() {
                        let text = visible[i].text.to_string();
                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));

                        let toast_host = cx.global::<ToastGlobal>().host.clone();
                        cx.update_entity(&toast_host, |host, cx| {
                            host.push(ToastKind::Info, "Annotation copied", cx);
                        });

                        return CommandResult::Handled;
                    }
                }
                CommandResult::NotHandled
            }
            _ => CommandResult::NotHandled,
        }
    }

    fn handle_edit_selected_item(
        &mut self,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) -> CommandResult {
        if self.state.mode != ModalMode::Edit || self.state.edit_state != EditState::Navigating {
            return CommandResult::NotHandled;
        }

        match self.state.modal_focus {
            ModalFocus::TagsInput => {
                if let Some(i) = self.state.tag_selected {
                    if i < self.state.form.tags.len() {
                        self.state.inline_edit = Some(InlineEditTarget::Tag(i));

                        let tag_value = self.state.form.tags[i].clone();
                        self.entities.tags_input.update(cx, |input, cx| {
                            input.set_value(tag_value, cx);
                        });

                        self.enter_edit_field(window, cx);

                        return CommandResult::Handled;
                    }
                }

                self.enter_edit_field(window, cx);
                CommandResult::Handled
            }
            ModalFocus::AnnotationsInput => {
                if let Some(i) = self.state.annotation_selected {
                    let visible: Vec<_> = self
                        .state
                        .annotations
                        .items
                        .iter()
                        .enumerate()
                        .filter(|(_, a)| a.origin != AnnotationOrigin::Deleted)
                        .collect();

                    if i < visible.len() {
                        let (actual_index, ann) = visible[i];

                        match ann.origin {
                            AnnotationOrigin::Added => {
                                self.state.inline_edit =
                                    Some(InlineEditTarget::Annotation(actual_index));

                                let ann_value = ann.text.to_string();
                                self.entities.annotation_input.update(cx, |input, cx| {
                                    input.set_value(ann_value, cx);
                                });

                                self.enter_edit_field(window, cx);

                                return CommandResult::Handled;
                            }
                            AnnotationOrigin::Original => {
                                let ann_id = ann.id;
                                let ann_value = ann.text.to_string();
                                self.state.annotations.mark_modified(ann_id);

                                self.state.inline_edit =
                                    Some(InlineEditTarget::Annotation(actual_index));

                                self.entities.annotation_input.update(cx, |input, cx| {
                                    input.set_value(ann_value, cx);
                                });

                                self.enter_edit_field(window, cx);

                                return CommandResult::Handled;
                            }
                            AnnotationOrigin::Modified { .. } => {
                                self.state.inline_edit =
                                    Some(InlineEditTarget::Annotation(actual_index));

                                let ann_value = ann.text.to_string();
                                self.entities.annotation_input.update(cx, |input, cx| {
                                    input.set_value(ann_value, cx);
                                });

                                self.enter_edit_field(window, cx);

                                return CommandResult::Handled;
                            }
                            AnnotationOrigin::Deleted => {}
                        }
                    }
                }

                self.enter_edit_field(window, cx);
                CommandResult::Handled
            }
            _ => {
                self.enter_edit_field(window, cx);
                CommandResult::Handled
            }
        }
    }

    fn handle_scroll_command(&mut self, delta: i32, cx: &mut gpui::Context<Self>) -> CommandResult {
        if self.state.mode == ModalMode::View {
            self.scroll(delta, cx);
            CommandResult::Handled
        } else {
            CommandResult::NotHandled
        }
    }

    fn handle_focus_filter_next(
        &mut self,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) -> CommandResult {
        if self.state.mode == ModalMode::Edit {
            self.focus_next_field(window, cx);
            CommandResult::Handled
        } else {
            CommandResult::NotHandled
        }
    }

    fn handle_focus_filter_prev(
        &mut self,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) -> CommandResult {
        if self.state.mode == ModalMode::Edit {
            self.focus_prev_field(window, cx);
            CommandResult::Handled
        } else {
            CommandResult::NotHandled
        }
    }

    fn handle_toggle_dropdown(&mut self, cx: &mut gpui::Context<Self>) -> CommandResult {
        if self.state.mode == ModalMode::Edit
            && matches!(
                self.state.modal_focus,
                ModalFocus::StatusDropdown | ModalFocus::PriorityDropdown
            )
        {
            if self.state.edit_state == EditState::DropdownOpen {
                self.toggle_focused_dropdown(cx);
                self.state.edit_state = EditState::Navigating;
            } else {
                self.state.edit_state = EditState::DropdownOpen;
                self.toggle_focused_dropdown(cx);
            }
            CommandResult::Handled
        } else {
            CommandResult::NotHandled
        }
    }

    fn handle_select_next_option(&mut self, cx: &mut gpui::Context<Self>) -> CommandResult {
        if self.state.mode == ModalMode::Edit && self.state.edit_state == EditState::DropdownOpen {
            self.select_next_dropdown_option(cx);
            CommandResult::Handled
        } else {
            CommandResult::NotHandled
        }
    }

    fn handle_select_prev_option(&mut self, cx: &mut gpui::Context<Self>) -> CommandResult {
        if self.state.mode == ModalMode::Edit && self.state.edit_state == EditState::DropdownOpen {
            self.select_prev_dropdown_option(cx);
            CommandResult::Handled
        } else {
            CommandResult::NotHandled
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

    fn move_annotation_selection(&mut self, next: bool, cx: &mut gpui::Context<Self>) -> bool {
        if self.state.mode != ModalMode::Edit || self.state.edit_state != EditState::Navigating {
            return false;
        }

        let visible_count = self
            .state
            .annotations
            .items
            .iter()
            .filter(|a| a.origin != AnnotationOrigin::Deleted)
            .count();

        if visible_count == 0 {
            return false;
        }

        let (next_selection, consumed) = match self.state.annotation_selected {
            None => {
                if next {
                    (Some(0), true)
                } else {
                    (None, false)
                }
            }
            Some(i) => {
                if next {
                    let next_index = i + 1;
                    if next_index >= visible_count {
                        (None, false)
                    } else {
                        (Some(next_index), true)
                    }
                } else if i == 0 {
                    (None, true)
                } else {
                    (Some(i - 1), true)
                }
            }
        };

        self.state.annotation_selected = next_selection;
        cx.notify();
        consumed
    }

    fn enter_edit_field(
        &mut self,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        if self.state.mode != ModalMode::Edit {
            return;
        }

        match self.state.modal_focus {
            ModalFocus::StatusDropdown | ModalFocus::PriorityDropdown => {
                self.state.edit_state = EditState::DropdownOpen;
                self.toggle_focused_dropdown(cx);
            }
            ModalFocus::Description
            | ModalFocus::Project
            | ModalFocus::Due
            | ModalFocus::TagsInput
            | ModalFocus::AnnotationsInput => {
                self.state.edit_state = EditState::Editing;

                if let Some(window) = window {
                    match self.state.modal_focus {
                        ModalFocus::Description => {
                            self.entities
                                .description_input
                                .update(cx, |i, cx| i.focus(window, cx));
                        }
                        ModalFocus::Project => {
                            self.entities
                                .project_input
                                .update(cx, |i, cx| i.focus(window, cx));
                        }
                        ModalFocus::Due => {
                            self.entities
                                .due_input
                                .update(cx, |i, cx| i.focus(window, cx));
                        }
                        ModalFocus::TagsInput => {
                            self.entities
                                .tags_input
                                .update(cx, |i, cx| i.focus(window, cx));
                        }
                        ModalFocus::AnnotationsInput => {
                            self.entities
                                .annotation_input
                                .update(cx, |i, cx| i.focus(window, cx));
                        }
                        _ => {}
                    }
                }
            }
            ModalFocus::None => {
                self.state.modal_focus = ModalFocus::Description;
                self.state.edit_state = EditState::Editing;
                if let Some(window) = window {
                    self.entities
                        .description_input
                        .update(cx, |i, cx| i.focus(window, cx));
                }
            }
        }
        cx.notify();
    }

    fn project_suggestions_open(&self, cx: &gpui::Context<Self>) -> bool {
        self.entities.project_input.read(cx).has_suggestions_open()
    }

    fn exit_edit_field(&mut self, window: Option<&mut gpui::Window>, cx: &mut gpui::Context<Self>) {
        if self.state.mode != ModalMode::Edit {
            return;
        }

        match self.state.edit_state {
            EditState::Editing => {
                if self.state.inline_edit.is_some() {
                    self.state.inline_edit = None;

                    match self.state.modal_focus {
                        ModalFocus::TagsInput => {
                            self.state.form.tag_draft.clear();
                            self.entities
                                .tags_input
                                .update(cx, |input, cx| input.clear(cx));
                        }
                        ModalFocus::AnnotationsInput => {
                            self.state.annotations.clear_draft();
                            self.entities
                                .annotation_input
                                .update(cx, |input, cx| input.clear(cx));
                        }
                        _ => {}
                    }
                } else {
                    self.sync_form_from_inputs(cx);
                }

                self.state.edit_state = EditState::Navigating;
                if let Some(window) = window {
                    window.focus(&self.form_focus_handle);
                }

                if self.state.inline_edit.is_none() {
                    self.form_history.push(self.state.form.clone());
                }
            }
            EditState::DropdownOpen => {
                self.close_all_dropdowns(cx);
                self.state.edit_state = EditState::Navigating;
                if let Some(window) = window {
                    window.focus(&self.form_focus_handle);
                }
            }
            EditState::Navigating => {
                // Already navigating, nothing to do
            }
        }
        cx.notify();
    }

    fn undo(&mut self, cx: &mut gpui::Context<Self>) {
        if let Some(form) = self.form_history.undo().cloned() {
            self.state.form = form;
            self.apply_form_inputs(cx);
            cx.notify();
        }
    }

    fn redo(&mut self, cx: &mut gpui::Context<Self>) {
        if let Some(form) = self.form_history.redo().cloned() {
            self.state.form = form;
            self.apply_form_inputs(cx);
            cx.notify();
        }
    }

    fn has_unsaved_changes(&mut self, cx: &gpui::Context<Self>) -> bool {
        self.sync_form_from_inputs(cx);
        if let Some(original) = &self.state.original {
            self.state.form.is_dirty(original)
        } else {
            false
        }
    }

    fn handle_close_or_escape(
        &mut self,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        match self.state.mode {
            ModalMode::View => {
                self.close(window, cx);
            }
            ModalMode::Edit => match self.state.edit_state {
                EditState::Editing => {
                    self.exit_edit_field(window, cx);
                }
                EditState::DropdownOpen => {
                    self.close_all_dropdowns(cx);
                    self.state.edit_state = EditState::Navigating;
                    if let Some(window) = window {
                        window.focus(&self.form_focus_handle);
                    }
                    cx.notify();
                }
                EditState::Navigating => {
                    if self.has_unsaved_changes(cx) {
                        self.state.pending_confirm = Some(ConfirmAction::DiscardUnsavedChanges);
                        cx.notify();
                    } else {
                        self.cancel_edit(window, cx);
                    }
                }
            },
        }
    }

    fn request_close(&mut self, window: Option<&mut gpui::Window>, cx: &mut gpui::Context<Self>) {
        if self.state.pending_confirm.is_some() {
            self.state.pending_confirm = None;
            cx.notify();
            return;
        }

        if self.state.mode == ModalMode::Edit && self.has_unsaved_changes(cx) {
            self.state.pending_confirm = Some(ConfirmAction::DiscardUnsavedChangesAndClose);
            cx.notify();
        } else {
            self.close(window, cx);
        }
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

        let on_close_backdrop = cx.listener(|modal, _event: &gpui::MouseDownEvent, window, cx| {
            modal.request_close(Some(window), cx);
        });
        let on_close_click = cx.listener(|modal, _event: &gpui::MouseDownEvent, window, cx| {
            modal.request_close(Some(window), cx);
        });

        render::render_task_detail_modal(
            &self.state,
            &self.entities.annotation_input,
            &self.entities.description_input,
            &self.entities.project_input,
            &self.entities.due_input,
            &self.entities.tags_input,
            &self.entities.status_dropdown,
            &self.entities.priority_dropdown,
            &self.focus_handle,
            &self.form_focus_handle,
            &self.scroll_handle,
            cx,
            on_close_backdrop,
            on_close_click,
        )
    }
}
