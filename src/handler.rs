use crate::{
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
        if self.task_detail_modal.read(cx).is_open() {
            return;
        }

        let task_id = self.task_table.read(cx).selected_task_uuid();
        let Some(task_id) = task_id else {
            return;
        };

        self.open_task_detail(task_id, window, cx);
    }

    pub(super) fn open_task_detail(
        &mut self,
        task_id: uuid::Uuid,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        self.focus_before_modal = self.focus_target;

        let tasks = self.tasks.clone();
        match self.task_service.get_task_detail(task_id, &tasks) {
            Ok(detail) => {
                self.task_detail_modal.update(cx, |modal, cx| {
                    modal.open_with_detail(detail, window, cx);
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
        let mut latest_task: Option<task::Task> = None;

        if description.is_some()
            || project.is_some()
            || priority.is_some()
            || tags.is_some()
            || due.is_some()
        {
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

        if let Some(status) = status {
            let status_result = match status {
                task::TaskStatus::Completed => self.task_service.complete_task(task_id),
                task::TaskStatus::Pending => self.task_service.reopen_task(task_id),
                task::TaskStatus::Deleted => {
                    self.task_service.delete_task(task_id).and_then(|_| {
                        self.task_service
                            .get_task(task_id)
                            .and_then(|task| task.ok_or(task::TaskError::NotFound(task_id)))
                    })
                }
                _ => self
                    .task_service
                    .get_task(task_id)
                    .and_then(|task| task.ok_or(task::TaskError::NotFound(task_id))),
            };

            match status_result {
                Ok(task) => latest_task = Some(task),
                Err(e) => {
                    log::error!("[App] Failed to update status: {}", e);
                    self.toast_host.update(cx, |host, cx| {
                        host.push(
                            ToastKind::Error,
                            format!("Failed to update status: {}", e),
                            cx,
                        );
                    });
                    if let Some(task) = latest_task {
                        self.apply_task_update(task, cx);
                    }
                    return;
                }
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
                    if let Some(task) = latest_task {
                        self.sync_task_detail(task, cx);
                    }
                    return;
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
                    if let Some(task) = latest_task {
                        self.sync_task_detail(task, cx);
                    }
                    return;
                }
            }
        }

        if let Some(task) = latest_task {
            self.apply_task_update(task, cx);
        } else {
            self.task_detail_modal.update(cx, |modal, cx| {
                modal.cancel_edit(cx);
            });
        }
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
