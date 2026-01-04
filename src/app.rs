use std::collections::HashMap;
use std::sync::Arc;

use gpui::prelude::*;

use crate::{
    components::{
        confirm_dialog::ConfirmDialog,
        toast::{ToastGlobal, ToastHost},
    },
    keymap::{FocusTarget, KeymapStack},
    models::{FilterState, ProjectTree},
    task::{self, TaskOverview, TaskService, TaskSummary},
    theme::ActiveTheme,
    view::{
        app_layout,
        sidebar::{Sidebar, SidebarEvent, SidebarSection, TagItem},
        status_bar::{StatusBar, StatusBarEvent},
        task_detail_modal::{TaskDetailModal, TaskDetailModalEvent},
        task_table::{TaskTable, TaskTableEvent},
    },
};

pub(super) struct DeleteConfirmState {
    pub(super) task_id: uuid::Uuid,
    pub(super) id_display: String,
    pub(super) description: String,
    pub(super) project: Option<String>,
    pub(super) tags: Vec<String>,
}

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
    pub(super) delete_confirm: Option<DeleteConfirmState>,
    pub(super) created_task_uuid: Option<uuid::Uuid>,
    pub(super) needs_focus_restore: bool,
}

impl gpui::Render for App {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme = cx.theme();

        if self.needs_focus_restore {
            self.needs_focus_restore = false;
            window.focus(&self.focus_handle);
        }

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

        let delete_dialog = self.delete_confirm.as_ref().map(|state| {
            let cancel_handler = Arc::new(cx.listener(|app, _event, _window, cx| {
                app.cancel_delete_confirm(cx);
            }));
            let confirm_handler = Arc::new(cx.listener(|app, _event, _window, cx| {
                app.confirm_delete_task(cx);
            }));

            let mut message = format!(
                "This will permanently delete: {} \"{}\"",
                state.id_display,
                state.description.trim()
            );

            if let Some(project) = state
                .project
                .as_ref()
                .map(|p| p.trim())
                .filter(|p| !p.is_empty())
            {
                message.push_str(&format!(" | Project: {}", project));
            }

            if !state.tags.is_empty() {
                message.push_str(&format!(" | Tags: {}", state.tags.join(", ")));
            }

            ConfirmDialog::new("Delete task?")
                .message(message)
                .hint("Enter/y = delete | Esc/n = cancel")
                .cancel("Cancel (Esc)", cancel_handler.clone())
                .danger("Delete (Enter)", confirm_handler.clone())
                .on_backdrop_click(cancel_handler)
                .render(theme)
                .into_any_element()
        });

        let overlay = match (modal, delete_dialog) {
            (None, None) => None,
            (Some(modal), None) => Some(modal),
            (None, Some(dialog)) => Some(dialog),
            (Some(modal), Some(dialog)) => {
                Some(gpui::div().child(modal).child(dialog).into_any_element())
            }
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
            overlay,
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

    pub(super) fn update_ui_from_tasks(
        &mut self,
        all_tasks: Vec<task::TaskSummary>,
        cx: &mut gpui::Context<Self>,
    ) {
        let previous_selection = self.task_table.read(cx).selected_task_uuid();
        let filter_state = self.filter_state.read(cx).clone();
        let filtered = task::TaskFilter::from(&filter_state).apply(&all_tasks);
        let reselect_uuid = previous_selection.and_then(|uuid| {
            filtered
                .iter()
                .any(|task| task.uuid == uuid)
                .then_some(uuid)
        });

        self.tasks = all_tasks;
        let (projects, tags) = Self::build_sidebar_data(&self.tasks);

        let mut project_tree = ProjectTree::new();
        project_tree.build_from_projects(&projects);

        self.sidebar.update(cx, |sidebar, cx| {
            sidebar.update_projects(project_tree, cx);
            sidebar.update_tags(tags, cx);
        });

        let tasks = self.tasks.clone();
        self.task_table.update(cx, |table, cx| {
            table.reload_tasks_from_all(tasks, cx);
            if let Some(uuid) = reselect_uuid {
                let _ = table.select_task_by_uuid(uuid, cx);
            }
        });

        self.update_modal_project_suggestions(cx);
        self.update_modal_tag_suggestions(cx);
    }

    fn update_modal_project_suggestions(&self, cx: &mut gpui::Context<Self>) {
        let mut projects: Vec<String> = self
            .tasks
            .iter()
            .filter_map(|task| task.project.clone())
            .collect();

        projects.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        projects.dedup_by(|a, b| a.eq_ignore_ascii_case(b));

        self.task_detail_modal.update(cx, |modal, cx| {
            modal.set_project_suggestions(projects, cx);
        });
    }

    fn update_modal_tag_suggestions(&self, cx: &mut gpui::Context<Self>) {
        let mut tags: Vec<String> = self
            .tasks
            .iter()
            .flat_map(|task| task.tags.iter().cloned())
            .collect();

        tags.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        tags.dedup_by(|a, b| a.eq_ignore_ascii_case(b));

        self.task_detail_modal.update(cx, |modal, cx| {
            modal.set_tag_suggestions(tags, cx);
        });
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
                        let mut project_suggestions: Vec<String> = task_summaries
                            .iter()
                            .filter_map(|task| task.project.clone())
                            .collect();
                        project_suggestions.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
                        project_suggestions.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
                        task_detail_modal.update(cx, |modal, cx| {
                            modal.set_project_suggestions(project_suggestions, cx);
                        });

                        let mut tag_suggestions: Vec<String> = task_summaries
                            .iter()
                            .flat_map(|task| task.tags.iter().cloned())
                            .collect();
                        tag_suggestions.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
                        tag_suggestions.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
                        task_detail_modal.update(cx, |modal, cx| {
                            modal.set_tag_suggestions(tag_suggestions, cx);
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
                            delete_confirm: None,
                            created_task_uuid: None,
                            needs_focus_restore: false,
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
                                    app.open_task_detail(*task_id, false, None, cx);
                                }
                            }
                            TaskTableEvent::CreateRequested => {
                                app.open_task_create(cx);
                            }
                            TaskTableEvent::DeleteRequested(task_id) => {
                                app.request_delete_task(*task_id, cx);
                            }
                        })
                        .detach();

                        cx.subscribe(&modal_events, |app, _modal, event, cx| match event {
                            TaskDetailModalEvent::Closed {
                                task_id,
                                was_creating,
                            } => {
                                app.handle_modal_closed(*task_id, *was_creating, cx);
                            }
                            TaskDetailModalEvent::SaveEdits { task_id, update } => {
                                app.handle_save_task_edits(*task_id, update.clone(), cx);
                            }
                            TaskDetailModalEvent::CreateTask { draft, annotations } => {
                                app.handle_create_task(draft.clone(), annotations.clone(), cx);
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
