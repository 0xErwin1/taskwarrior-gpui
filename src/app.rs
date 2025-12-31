use std::collections::HashMap;

use gpui::prelude::*;

use crate::{
    components::toast::{ToastGlobal, ToastHost},
    keymap::{Command, CommandDispatcher, ContextId, FocusTarget, KeyChord, KeymapStack},
    models::{FilterState, ProjectTree},
    task::{self, TaskOverview, TaskService, TaskSummary},
    theme::ActiveTheme,
    view::{
        app_layout,
        sidebar::{Sidebar, SidebarEvent, SidebarSection, TagItem},
        status_bar::{StatusBar, StatusBarEvent, SyncState},
        task_detail_modal::{TaskDetailModal, TaskDetailModalEvent},
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
    pub(super) task_detail_modal: gpui::Entity<TaskDetailModal>,
    pub(super) toast_host: gpui::Entity<ToastHost>,
    pub(super) task_service: TaskService,
    pub(super) tasks: Vec<TaskSummary>,
    pub(super) focus_before_modal: FocusTarget,
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

        let modal = if self.task_detail_modal.read(cx).is_open() {
            Some(self.task_detail_modal.clone().into_any_element())
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
            self.toast_host.clone(),
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
                let modal_is_open = self.task_detail_modal.read(cx).is_open();

                if modal_is_open {
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
        if self.task_detail_modal.read(cx).is_open() {
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

    fn active_context(&self, cx: &gpui::Context<Self>) -> ContextId {
        if self.task_detail_modal.read(cx).is_open() {
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
                        let toast_host = cx.new(|cx| ToastHost::new(cx));
                        cx.set_global(ToastGlobal {
                            host: toast_host.clone(),
                        });

                        let sidebar =
                            cx.new(|cx| Sidebar::new(project_tree, tags, filter_state.clone(), cx));

                        let task_table = cx
                            .new(|cx| TaskTable::new("main-task-table", filter_state.clone(), cx));

                        let task_detail_modal = cx.new(|cx| TaskDetailModal::new(cx));

                        let task_table_events = task_table.clone();
                        let sidebar_events = sidebar.clone();
                        let modal_events = task_detail_modal.clone();

                        task_table.update(cx, |table, cx| {
                            table.reload_tasks_from_all(task_summaries.clone(), cx);
                        });

                        let mut keymap = KeymapStack::new();
                        keymap.push_layer(crate::keymap::defaults::build_default_keymap());

                        let app_instance = App {
                            focus_handle: cx.focus_handle(),
                            focus_target: FocusTarget::Table,
                            keymap,
                            sidebar,
                            filter_state: filter_state.clone(),
                            status_bar: status_bar.clone(),
                            task_table,
                            task_detail_modal,
                            toast_host,
                            task_service,
                            tasks: task_summaries,
                            focus_before_modal: FocusTarget::Table,
                        };

                        window.focus(&app_instance.focus_handle);

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
                                if !app.task_detail_modal.read(cx).is_open() {
                                    app.open_task_detail(*task_id, None, cx);
                                }
                            }
                        })
                        .detach();

                        cx.subscribe(&modal_events, |app, _modal, event, cx| match event {
                            TaskDetailModalEvent::Closed => {
                                app.focus_target = app.focus_before_modal;
                                cx.notify();
                            }
                        })
                        .detach();

                        app_instance
                    })
                },
            )
            .unwrap();
        });
    }
}
