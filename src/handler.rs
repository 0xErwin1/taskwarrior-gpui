use crate::{
    app::DeleteConfirmState,
    components::toast::ToastKind,
    keymap::{Command, CommandDispatcher, ContextId, FocusTarget, KeyChord},
    task::{self, TaskSummary},
    view::{status_bar::SyncState, task_detail_modal::TaskEditUpdate},
};

use super::App;

impl App {
    pub(super) fn handle_sync(&mut self, cx: &mut gpui::Context<Self>) {
        self.status_bar.update(cx, |bar, cx| {
            bar.set_sync_state(SyncState::Syncing, cx);
            bar.set_last_sync_message("Syncing...".to_string(), cx);
        });

        match self.task_service.get_all_tasks() {
            Ok(all_tasks) => {
                let summaries: Vec<TaskSummary> = all_tasks.iter().map(TaskSummary::from).collect();
                self.update_ui_from_tasks(summaries, cx);

                self.status_bar.update(cx, |bar, cx| {
                    bar.set_sync_state(SyncState::Success, cx);
                    bar.set_last_sync_message("Synced".to_string(), cx);
                });
            }
            Err(e) => {
                log::error!("[App] Sync failed: {}", e);
                self.status_bar.update(cx, |bar, cx| {
                    bar.set_sync_state(SyncState::Error, cx);
                    bar.set_last_sync_message(format!("Error: {}", e), cx);
                });
            }
        }
    }

    pub(super) fn handle_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        if self.delete_confirm.is_some() {
            let key = event.keystroke.key.as_str().to_lowercase();
            let mods = &event.keystroke.modifiers;
            let has_mods = mods.control || mods.alt || mods.shift;

            if !has_mods {
                match key.as_str() {
                    "enter" | "y" => {
                        self.confirm_delete_task(cx);
                        return;
                    }
                    "escape" | "n" => {
                        self.cancel_delete_confirm(cx);
                        return;
                    }
                    _ => {}
                }
            }

            return;
        }

        if let Some(chord) = KeyChord::from_gpui(event) {
            let context = self.active_context(cx);

            if let Some(command) = self.keymap.resolve(context, &chord) {
                let modal_is_open = self.task_detail_modal.read(cx).is_open();

                if modal_is_open {
                    let mut handled = false;
                    self.task_detail_modal.update(cx, |modal, cx| {
                        handled = modal.dispatch_command(command, Some(window), cx);
                    });

                    if handled {
                        return;
                    }

                    if command != Command::Sync {
                        return;
                    }
                }

                match command {
                    Command::FocusSearch => {
                        let from_headers = matches!(self.focus_target, FocusTarget::TableHeaders);
                        self.focus_target = FocusTarget::Table;
                        self.task_table.update(cx, |table, cx| {
                            if from_headers {
                                table.blur_table_headers(cx);
                            }
                            table.focus_search_input(window, cx);
                        });
                        cx.notify();
                    }
                    Command::FocusTableHeaders => {
                        self.focus_target = FocusTarget::TableHeaders;
                        self.task_table.update(cx, |table, cx| {
                            table.blur_search_input(window, cx);
                            table.set_filter_bar_focus(
                                crate::view::task_table::FilterBarFocus::None,
                                cx,
                            );
                            table.focus_table_headers(window, cx);
                        });
                        cx.notify();
                    }
                    Command::FocusTable => {
                        self.focus_target = FocusTarget::Table;
                        self.task_table.update(cx, |table, cx| match context {
                            ContextId::TextInput | ContextId::FilterBar => {
                                table.blur_search_input(window, cx);
                                table.set_filter_bar_focus(
                                    crate::view::task_table::FilterBarFocus::None,
                                    cx,
                                );
                            }
                            ContextId::TableHeaders => {
                                table.blur_table_headers(cx);
                            }
                            _ => {}
                        });
                        cx.notify();
                    }
                    Command::FocusFilterNext | Command::FocusFilterPrev => {
                        self.task_table.update(cx, |table, cx| {
                            use crate::view::task_table::FilterBarFocus;
                            let was_on_input =
                                matches!(table.get_filter_bar_focus(), FilterBarFocus::SearchInput);

                            if command == Command::FocusFilterNext {
                                table.focus_filter_next(cx);
                            } else {
                                table.focus_filter_prev(cx);
                            }

                            if was_on_input {
                                table.blur_search_input(window, cx);
                            }

                            let now_on_input =
                                matches!(table.get_filter_bar_focus(), FilterBarFocus::SearchInput);
                            if now_on_input && !was_on_input {
                                table.focus_search_input(window, cx);
                            }
                        });
                    }
                    _ => {
                        self.dispatch(command, cx);
                    }
                }
            }
        }
    }

    pub(super) fn open_selected_task(
        &mut self,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        self.open_selected_task_mode(false, window, cx);
    }

    pub(super) fn open_selected_task_edit(
        &mut self,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        self.open_selected_task_mode(true, window, cx);
    }

    fn open_selected_task_mode(
        &mut self,
        edit_mode: bool,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        if self.task_detail_modal.read(cx).is_open() {
            return;
        }

        let task_id = self.task_table.read(cx).selected_task_uuid();
        let Some(task_id) = task_id else {
            return;
        };

        self.open_task_detail(task_id, edit_mode, window, cx);
    }

    pub(super) fn open_task_detail(
        &mut self,
        task_id: uuid::Uuid,
        edit_mode: bool,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        self.focus_before_modal = self.focus_target;

        let tasks = self.tasks.clone();
        match self.task_service.get_task_detail(task_id, &tasks) {
            Ok(detail) => {
                self.task_detail_modal.update(cx, |modal, cx| {
                    if edit_mode {
                        modal.open_with_detail_edit(detail, window, cx);
                    } else {
                        modal.open_with_detail(detail, window, cx);
                    }
                });
            }
            Err(e) => {
                self.task_detail_modal.update(cx, |modal, cx| {
                    modal.open_with_error(task_id, e.to_string(), window, cx);
                });
            }
        }

        cx.notify();
    }

    pub(super) fn open_task_create(&mut self, cx: &mut gpui::Context<Self>) {
        if self.task_detail_modal.read(cx).is_open() {
            return;
        }

        self.focus_before_modal = self.focus_target;
        self.task_detail_modal.update(cx, |modal, cx| {
            modal.open_create(None, cx);
        });
        cx.notify();
    }

    pub(super) fn request_delete_task(
        &mut self,
        task_id: uuid::Uuid,
        cx: &mut gpui::Context<Self>,
    ) {
        if self.delete_confirm.is_some() {
            return;
        }

        if self.task_detail_modal.read(cx).is_editing() {
            self.task_detail_modal.update(cx, |modal, cx| {
                modal.cancel_edit(None, cx);
            });
        }

        let Some(task) = self.tasks.iter().find(|task| task.uuid == task_id) else {
            return;
        };

        let id_display = task
            .working_id
            .or(task.id)
            .map(|id| format!("#{}", id))
            .unwrap_or_else(|| task_id.to_string());

        let mut tags: Vec<String> = task.tags.iter().cloned().collect();
        tags.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

        self.delete_confirm = Some(DeleteConfirmState {
            task_id,
            id_display,
            description: task.description.clone(),
            project: task.project.clone(),
            tags,
        });
        cx.notify();
    }

    pub(super) fn cancel_delete_confirm(&mut self, cx: &mut gpui::Context<Self>) {
        if self.delete_confirm.take().is_some() {
            cx.notify();
        }
    }

    pub(super) fn confirm_delete_task(&mut self, cx: &mut gpui::Context<Self>) {
        let Some(confirm) = self.delete_confirm.take() else {
            return;
        };

        let next_selection = self
            .task_table
            .read(cx)
            .next_selection_after_delete(confirm.task_id);

        if let Err(e) = self.task_service.delete_task(confirm.task_id) {
            log::error!("[App] Failed to delete task: {}", e);
            self.toast_host.update(cx, |host, cx| {
                host.push(
                    ToastKind::Error,
                    format!("Failed to delete task: {}", e),
                    cx,
                );
            });
            return;
        }

        let updated_task = match self.task_service.get_task(confirm.task_id) {
            Ok(task) => task,
            Err(ref e)
                if e.to_string().contains("not found") || e.to_string().contains("No such") =>
            {
                log::info!(
                    "[App] Task {} not found after deletion (expected)",
                    confirm.task_id
                );
                None
            }
            Err(e) => {
                log::warn!("[App] Failed to reload task after delete: {}", e);
                None
            }
        };

        if let Some(task) = updated_task {
            self.upsert_task_summary(task, cx);
        } else {
            self.tasks.retain(|task| task.uuid != confirm.task_id);
            let summaries = self.tasks.clone();
            self.update_ui_from_tasks(summaries, cx);
        }

        self.task_table.update(cx, |table, cx| {
            if let Some(next_uuid) = next_selection {
                if !table.select_task_by_uuid(next_uuid, cx) {
                    table.clear_selection(cx);
                }
            } else {
                table.clear_selection(cx);
            }
        });

        let modal_task_id = self.task_detail_modal.read(cx).task_id();
        if modal_task_id == Some(confirm.task_id) {
            self.task_detail_modal.update(cx, |modal, cx| {
                modal.close(None, cx);
            });
        }

        self.toast_host.update(cx, |host, cx| {
            host.push(ToastKind::Info, "Task deleted", cx);
        });

        self.status_bar.update(cx, |bar, cx| {
            bar.set_dirty(cx);
        });
    }

    fn apply_task_update(&mut self, task: task::Task, cx: &mut gpui::Context<Self>) {
        let summary = TaskSummary::from(&task);
        if let Some(existing) = self.tasks.iter_mut().find(|t| t.uuid == task.uuid) {
            *existing = summary;
        } else {
            self.tasks.push(summary);
        }

        let summaries = self.tasks.clone();
        self.update_ui_from_tasks(summaries, cx);

        let detail = task::TaskDetailVm::from_task(&task, &self.tasks);
        self.task_detail_modal.update(cx, |modal, cx| {
            modal.apply_saved_detail(detail, cx);
        });
    }

    fn upsert_task_summary(&mut self, task: task::Task, cx: &mut gpui::Context<Self>) {
        let summary = TaskSummary::from(&task);
        if let Some(existing) = self.tasks.iter_mut().find(|t| t.uuid == task.uuid) {
            *existing = summary;
        } else {
            self.tasks.push(summary);
        }

        let summaries = self.tasks.clone();
        self.update_ui_from_tasks(summaries, cx);
    }

    fn sync_task_detail(&mut self, task: task::Task, cx: &mut gpui::Context<Self>) {
        let summary = TaskSummary::from(&task);
        if let Some(existing) = self.tasks.iter_mut().find(|t| t.uuid == task.uuid) {
            *existing = summary;
        } else {
            self.tasks.push(summary);
        }

        let summaries = self.tasks.clone();
        self.update_ui_from_tasks(summaries, cx);

        let detail = task::TaskDetailVm::from_task(&task, &self.tasks);
        self.task_detail_modal.update(cx, |modal, cx| {
            modal.set_detail(detail, cx);
        });
    }

    pub(super) fn handle_save_task_edits(
        &mut self,
        task_id: uuid::Uuid,
        update: TaskEditUpdate,
        cx: &mut gpui::Context<Self>,
    ) {
        let TaskEditUpdate {
            description,
            project,
            priority,
            status,
            due,
            tags,
            annotations_add,
            annotations_delete,
        } = update;

        let has_description = description.is_some();
        let has_project = project.is_some();
        let has_priority = priority.is_some();
        let has_due = due.is_some();
        let has_tags = tags.is_some();
        let has_status = status.is_some();
        let has_annotations = !annotations_add.is_empty() || !annotations_delete.is_empty();

        let mut latest_task: Option<task::Task> = None;

        if has_description || has_project || has_priority || has_tags || has_due {
            match self.task_service.update_task(
                task_id,
                description,
                project,
                priority,
                tags,
                due,
                None,
            ) {
                Ok(task) => latest_task = Some(task),
                Err(e) => {
                    log::error!("[App] Failed to update task: {}", e);
                    self.toast_host.update(cx, |host, cx| {
                        host.push(
                            ToastKind::Error,
                            format!("Failed to update task: {}", e),
                            cx,
                        );
                    });
                    return;
                }
            }
        }

        if has_status {
            match status {
                Some(task::TaskStatus::Completed) => {
                    latest_task = self.task_service.complete_task(task_id).ok()
                }
                Some(task::TaskStatus::Pending) => {
                    latest_task = self.task_service.reopen_task(task_id).ok()
                }
                Some(task::TaskStatus::Deleted) => {
                    let _ = self.task_service.delete_task(task_id);
                    latest_task = self.task_service.get_task(task_id).ok().flatten();
                }
                _ => {
                    latest_task = self.task_service.get_task(task_id).ok().flatten();
                }
                None => {}
            }
        }

        for text in annotations_add {
            match self.task_service.add_annotation(task_id, text) {
                Ok(task) => latest_task = Some(task),
                Err(e) => {
                    log::error!("[App] Failed to add annotation: {}", e);
                    self.toast_host.update(cx, |host, cx| {
                        host.push(
                            ToastKind::Error,
                            format!("Failed to add annotation: {}", e),
                            cx,
                        );
                    });
                }
            }
        }

        for entry in annotations_delete {
            match self.task_service.remove_annotation(task_id, entry) {
                Ok(task) => latest_task = Some(task),
                Err(e) => {
                    log::error!("[App] Failed to delete annotation: {}", e);
                    self.toast_host.update(cx, |host, cx| {
                        host.push(
                            ToastKind::Error,
                            format!("Failed to delete annotation: {}", e),
                            cx,
                        );
                    });
                }
            }
        }

        let task_to_sync = if latest_task.is_some() {
            latest_task.take()
        } else if has_description
            || has_project
            || has_priority
            || has_tags
            || has_due
            || has_status
            || has_annotations
        {
            match self.task_service.get_task(task_id) {
                Ok(task) => task,
                Err(e) => {
                    log::warn!("[App] Failed to sync task after update: {}", e);
                    None
                }
            }
        } else {
            self.task_detail_modal.update(cx, |modal, cx| {
                modal.cancel_edit(None, cx);
            });
            return;
        };

        if let Some(task) = task_to_sync {
            self.apply_task_update(task, cx);
        } else {
            self.task_detail_modal.update(cx, |modal, cx| {
                modal.cancel_edit(None, cx);
            });
        }
    }

    pub(super) fn handle_create_task(
        &mut self,
        draft: task::TaskDraft,
        annotations: Vec<String>,
        cx: &mut gpui::Context<Self>,
    ) {
        let created = match self.task_service.create_task(draft) {
            Ok(task) => task,
            Err(e) => {
                log::error!("[App] Failed to create task: {}", e);
                self.toast_host.update(cx, |host, cx| {
                    host.push(
                        ToastKind::Error,
                        format!("Failed to create task: {}", e),
                        cx,
                    );
                });
                return;
            }
        };

        let mut latest_task = created;
        for text in annotations {
            match self.task_service.add_annotation(latest_task.uuid, text) {
                Ok(task) => {
                    latest_task = task;
                }
                Err(e) => {
                    log::error!("[App] Failed to add annotation: {}", e);
                    self.toast_host.update(cx, |host, cx| {
                        host.push(
                            ToastKind::Error,
                            format!("Failed to add annotation: {}", e),
                            cx,
                        );
                    });
                    break;
                }
            }
        }

        let created_uuid = latest_task.uuid;
        self.created_task_uuid = Some(created_uuid);
        self.upsert_task_summary(latest_task, cx);

        self.task_detail_modal.update(cx, |modal, cx| {
            modal.close(None, cx);
        });

        self.status_bar.update(cx, |bar, cx| {
            bar.set_dirty(cx);
        });
    }

    pub(super) fn handle_modal_closed(
        &mut self,
        task_id: Option<uuid::Uuid>,
        was_creating: bool,
        cx: &mut gpui::Context<Self>,
    ) {
        self.focus_target = self.focus_before_modal;

        self.task_table.update(cx, |table, cx| {
            table.blur_filter_bar(cx);
            table.blur_table_headers(cx);

            let uuid_to_select = if was_creating {
                self.created_task_uuid.take()
            } else {
                task_id
            };

            if let Some(uuid) = uuid_to_select {
                let _ = table.select_task_by_uuid(uuid, cx);
            }
        });

        self.needs_focus_restore = true;
        cx.notify();
    }

    fn active_context(&self, cx: &gpui::Context<Self>) -> ContextId {
        if self.task_detail_modal.read(cx).is_open() {
            return self.task_detail_modal.read(cx).active_context();
        }
        if matches!(self.focus_target, FocusTarget::Table) {
            let filter_context = self.task_table.read(cx).get_active_filter_context();
            if let Some(context) = filter_context {
                return context;
            }
        }
        self.focus_target.to_context()
    }
}
