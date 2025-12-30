use std::collections::HashMap;

use gpui::prelude::*;

use crate::{
    components::modal::ModalState,
    keymap::{Command, CommandDispatcher, ContextId, FocusTarget, KeyChord, KeymapStack},
    models::{FilterState, ProjectTree},
    task::{self, TaskDetailState, TaskOverview, TaskService, TaskSummary},
    theme::ActiveTheme,
    view::{
        app_layout,
        sidebar::{Sidebar, SidebarEvent, SidebarSection, TagItem},
        status_bar::{StatusBar, StatusBarEvent, SyncState},
        task_detail_modal,
        task_table::{TaskTable, TaskTableEvent},
    },
};

pub(super) struct App {
    pub(super) focus_handle: gpui::FocusHandle,
    pub(super) focus_target: FocusTarget,
    pub(super) keymap: KeymapStack,
    pub(super) sidebar: gpui::Entity<Sidebar>,
    pub(super) filter_state: gpui::Entity<FilterState>,
    pub(super) status_bar: gpui::Entity<StatusBar>,
    pub(super) task_table: gpui::Entity<TaskTable>,
    pub(super) task_service: TaskService,
    pub(super) tasks: Vec<TaskSummary>,
    pub(super) selected_task_id: Option<uuid::Uuid>,
    pub(super) task_detail_state: TaskDetailState,
    pub(super) modal_state: ModalState,
    pub(super) modal_focus_handle: gpui::FocusHandle,
    pub(super) focus_before_modal: FocusTarget,
    pub(super) modal_scroll_handle: gpui::ScrollHandle,
}

impl gpui::Render for App {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme = cx.theme();

        let on_root_key_down = cx.listener(|app, event: &gpui::KeyDownEvent, window, cx| {
            app.handle_key_down(event, window, cx);
        });
        let on_sidebar_mouse_down =
            cx.listener(|app, _event: &gpui::MouseDownEvent, _window, cx| {
                if !app.focus_target.is_sidebar() {
                    app.focus_target = FocusTarget::SidebarProjects;
                    app.sidebar.update(cx, |sidebar, cx| {
                        sidebar.set_section(crate::view::sidebar::SidebarSection::Projects, cx);
                    });
                    cx.notify();
                }
            });
        let on_table_mouse_down = cx.listener(|app, _event: &gpui::MouseDownEvent, _window, cx| {
            if !matches!(app.focus_target, FocusTarget::Table) {
                app.focus_target = FocusTarget::Table;
                cx.notify();
            }
        });

        let modal = if self.modal_state.open {
            let on_close_backdrop =
                cx.listener(|app, _event: &gpui::MouseDownEvent, window, cx| {
                    app.close_task_detail(Some(window), cx);
                });
            let on_close_click = cx.listener(|app, _event: &gpui::MouseDownEvent, window, cx| {
                app.close_task_detail(Some(window), cx);
            });
            Some(task_detail_modal::render_task_detail_modal(
                &self.task_detail_state,
                &self.modal_focus_handle,
                &self.modal_scroll_handle,
                theme,
                on_close_backdrop,
                on_close_click,
            ))
        } else {
            None
        };

        app_layout::render_app_layout(
            theme,
            &self.focus_handle,
            self.focus_target,
            self.sidebar.clone(),
            self.task_table.clone(),
            self.status_bar.clone(),
            on_root_key_down,
            on_sidebar_mouse_down,
            on_table_mouse_down,
            modal,
        )
    }
}

impl App {
    fn build_sidebar_data(tasks: &[task::TaskSummary]) -> (Vec<(String, usize)>, Vec<TagItem>) {
        let mut project_counts: HashMap<String, usize> = HashMap::new();
        let mut tag_counts: HashMap<String, usize> = HashMap::new();

        for task in tasks {
            if !matches!(task.status, task::TaskStatus::Pending) {
                continue;
            }

            if let Some(project) = &task.project {
                *project_counts.entry(project.clone()).or_insert(0) += 1;
            }

            for tag in &task.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }

        let mut projects: Vec<(String, usize)> = project_counts.into_iter().collect();
        projects.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        let mut tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
        tags.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        let tag_items = tags
            .into_iter()
            .map(|(name, task_count)| TagItem { name, task_count })
            .collect();

        (projects, tag_items)
    }

    fn update_ui_from_tasks(
        &mut self,
        all_tasks: Vec<task::TaskSummary>,
        cx: &mut gpui::Context<Self>,
    ) {
        self.tasks = all_tasks;
        let (projects, tags) = Self::build_sidebar_data(&self.tasks);

        let mut project_tree = ProjectTree::new();
        project_tree.build_from_projects(&projects);

        self.sidebar.update(cx, |sidebar, cx| {
            sidebar.update_projects(project_tree, cx);
            sidebar.update_tags(tags, cx);
        });

        let tasks = self.tasks.clone();
        self.task_table
            .update(cx, |table, cx| table.reload_tasks_from_all(tasks, cx));
    }

    fn reload_tasks(&mut self, cx: &mut gpui::Context<Self>) {
        match self.task_service.get_all_tasks() {
            Ok(all_tasks) => {
                self.status_bar.update(cx, |bar, cx| {
                    bar.clear_error(cx);
                });
                let summaries: Vec<TaskSummary> = all_tasks.iter().map(TaskSummary::from).collect();
                self.update_ui_from_tasks(summaries, cx);
            }
            Err(e) => {
                log::error!("[App] Failed to load tasks: {}", e);
                self.status_bar.update(cx, |bar, cx| {
                    bar.set_error(format!("Failed to load tasks: {}", e), cx);
                });
                self.update_ui_from_tasks(vec![], cx);
            }
        }
    }

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

    fn handle_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        if let Some(chord) = KeyChord::from_gpui(event) {
            let context = self.active_context(cx);

            if let Some(command) = self.keymap.resolve(context, &chord) {
                if self.modal_state.open {
                    match command {
                        Command::CloseModal
                        | Command::SaveModal
                        | Command::Sync
                        | Command::ModalScrollUp
                        | Command::ModalScrollDown => {}
                        _ => return,
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
        if self.modal_state.open {
            return;
        }

        let task_id = self.task_table.read(cx).selected_task_uuid();
        let Some(task_id) = task_id else {
            return;
        };

        self.open_task_detail(task_id, window, cx);
    }

    fn open_task_detail(
        &mut self,
        task_id: uuid::Uuid,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        self.focus_before_modal = self.focus_target;
        self.modal_state.open = true;
        if let Some(window) = window {
            window.focus(&self.modal_focus_handle);
        }
        self.modal_scroll_handle = gpui::ScrollHandle::new();
        self.modal_scroll_handle.scroll_to_item(0);

        if self.selected_task_id == Some(task_id) {
            if matches!(self.task_detail_state, TaskDetailState::Ready(_)) {
                cx.notify();
                return;
            }
        }

        self.selected_task_id = Some(task_id);
        self.task_detail_state = TaskDetailState::Loading(task_id);
        cx.notify();

        let tasks = self.tasks.clone();
        match self.task_service.get_task_detail(task_id, &tasks) {
            Ok(detail) => {
                self.task_detail_state = TaskDetailState::Ready(detail);
            }
            Err(e) => {
                self.task_detail_state = TaskDetailState::Error(task_id, e.to_string());
            }
        }

        cx.notify();
    }

    pub(super) fn close_task_detail(
        &mut self,
        window: Option<&mut gpui::Window>,
        cx: &mut gpui::Context<Self>,
    ) {
        if !self.modal_state.open {
            return;
        }

        self.modal_state.open = false;
        self.selected_task_id = None;
        self.focus_target = self.focus_before_modal;

        if let Some(window) = window {
            window.focus(&self.focus_handle);
        }

        cx.notify();
    }

    pub(super) fn scroll_task_detail(&self, delta: i32, cx: &mut gpui::Context<Self>) {
        let handle = &self.modal_scroll_handle;
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

    fn active_context(&self, cx: &gpui::Context<Self>) -> ContextId {
        if self.modal_state.open {
            return ContextId::Modal;
        }
        if matches!(self.focus_target, FocusTarget::Table) {
            let filter_context = self.task_table.read(cx).get_active_filter_context();
            if let Some(context) = filter_context {
                return context;
            }
        }
        self.focus_target.to_context()
    }

    pub fn run() -> () {
        let app = gpui::Application::new();

        app.run(|app: &mut gpui::App| {
            app.set_global(crate::theme::Theme::dark());
            app.open_window(
                gpui::WindowOptions::default(),
                |window: &mut gpui::Window, app: &mut gpui::App| {
                    app.new(|cx: &mut gpui::Context<'_, App>| {
                        let filter_state = cx.new(|_cx| FilterState::new());

                        let mut task_service = TaskService::new()
                            .unwrap_or_else(|e| panic!("Failed to initialize TaskService: {}", e));

                        let overview = task_service.get_overview().unwrap_or_else(|e| {
                            log::error!("Failed to load tasks: {}", e);
                            TaskOverview {
                                tasks: vec![],
                                projects: vec![],
                                tags: vec![],
                                total_tasks: 0,
                                pending_tasks: 0,
                                completed_tasks: 0,
                            }
                        });

                        let task_summaries: Vec<TaskSummary> =
                            overview.tasks.iter().map(TaskSummary::from).collect();

                        let mut project_tree = ProjectTree::new();
                        project_tree.build_from_projects(&overview.projects);

                        let tags: Vec<TagItem> = overview
                            .tags
                            .into_iter()
                            .map(|(name, task_count)| TagItem { name, task_count })
                            .collect();

                        let status_bar = cx.new(|cx| StatusBar::new(cx));

                        let sidebar =
                            cx.new(|cx| Sidebar::new(project_tree, tags, filter_state.clone(), cx));

                        let task_table = cx
                            .new(|cx| TaskTable::new("main-task-table", filter_state.clone(), cx));
                        let task_table_events = task_table.clone();
                        let sidebar_events = sidebar.clone();

                        task_table.update(cx, |table, cx| {
                            table.reload_tasks_from_all(task_summaries.clone(), cx);
                        });

                        let mut keymap = KeymapStack::new();
                        keymap.push_layer(crate::keymap::defaults::build_default_keymap());

                        let app = App {
                            focus_handle: cx.focus_handle(),
                            focus_target: FocusTarget::Table,
                            keymap,
                            sidebar,
                            filter_state: filter_state.clone(),
                            status_bar: status_bar.clone(),
                            task_table,
                            task_service,
                            tasks: task_summaries,
                            selected_task_id: None,
                            task_detail_state: TaskDetailState::default(),
                            modal_state: ModalState::default(),
                            modal_focus_handle: cx.focus_handle(),
                            focus_before_modal: FocusTarget::Table,
                            modal_scroll_handle: gpui::ScrollHandle::new(),
                        };

                        window.focus(&app.focus_handle);

                        cx.observe(&filter_state, |app, _, cx| {
                            app.reload_tasks(cx);
                        })
                        .detach();

                        cx.subscribe(&status_bar, |app, _bar, event, cx| match event {
                            StatusBarEvent::SyncRequested => {
                                app.handle_sync(cx);
                            }
                        })
                        .detach();
                        cx.subscribe(&sidebar_events, |app, _sidebar, event, cx| match event {
                            SidebarEvent::Focused(section) => {
                                app.focus_target = match section {
                                    SidebarSection::Projects => FocusTarget::SidebarProjects,
                                    SidebarSection::Tags => FocusTarget::SidebarTags,
                                };
                                cx.notify();
                            }
                        })
                        .detach();
                        cx.subscribe(&task_table_events, |app, _table, event, cx| match event {
                            TaskTableEvent::OpenTask(task_id) => {
                                if !app.modal_state.open {
                                    app.open_task_detail(*task_id, None, cx);
                                }
                            }
                        })
                        .detach();

                        app
                    })
                },
            )
            .unwrap();
        });
    }
}
