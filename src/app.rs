use std::collections::HashMap;

use gpui::prelude::*;

use crate::models::{FilterState, ProjectTree};
use crate::task::{self, TaskFilter, TaskOverview, TaskService};
use crate::theme::ActiveTheme;
use crate::ui::{ROOT_PADDING, SECTION_GAP, card_style};
use crate::view::sidebar::{Sidebar, TagItem};
use crate::view::status_bar::{StatusBar, StatusBarEvent, SyncState};
use crate::view::task_table::TaskTable;
use gpui::div;

pub(crate) struct App {
    sidebar: gpui::Entity<Sidebar>,
    filter_state: gpui::Entity<FilterState>,
    status_bar: gpui::Entity<StatusBar>,
    task_table: gpui::Entity<TaskTable>,
    task_service: TaskService,
}

impl gpui::Render for App {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme = cx.theme();

        let sidebar = card_style(div(), theme)
            .w(gpui::px(250.))
            .h_full()
            .flex_shrink_0()
            .overflow_hidden()
            .child(self.sidebar.clone());

        let main = card_style(div(), theme)
            .flex_1()
            .h_full()
            .min_w_0()
            .overflow_hidden()
            .p_0()
            .child(self.task_table.clone());

        let content = div()
            .flex()
            .flex_1()
            .min_h_0()
            .gap(SECTION_GAP)
            .child(sidebar)
            .child(main);

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.background)
            .p(ROOT_PADDING)
            .gap(SECTION_GAP)
            .child(content)
            .child(self.status_bar.clone())
    }
}

impl App {
    fn build_sidebar_data(tasks: &[task::Task]) -> (Vec<(String, usize)>, Vec<TagItem>) {
        let mut project_counts: HashMap<String, usize> = HashMap::new();
        let mut tag_counts: HashMap<String, usize> = HashMap::new();

        for task in tasks {
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

    fn reload_tasks(&mut self, cx: &mut gpui::Context<Self>) {
        let all_tasks = self.task_service.get_all_tasks().unwrap_or_else(|e| {
            log::error!("[App] Failed to load tasks: {}", e);
            vec![]
        });

        let filter_state = self.filter_state.read(cx).clone();
        let task_filter = TaskFilter::from(&filter_state);
        let filtered_tasks = task_filter.apply(&all_tasks);
        let (projects, tags) = Self::build_sidebar_data(&filtered_tasks);

        let mut project_tree = ProjectTree::new();
        project_tree.build_from_projects(&projects);

        self.sidebar.update(cx, |sidebar, cx| {
            sidebar.update_projects(project_tree, cx);
            sidebar.update_tags(tags, cx);
        });

        self.task_table.update(cx, |table, cx| {
            table.reload_tasks_from_all(all_tasks, cx);
        });
    }

    fn handle_sync(&mut self, cx: &mut gpui::Context<Self>) {
        self.status_bar.update(cx, |bar, cx| {
            bar.set_sync_state(SyncState::Syncing, cx);
            bar.set_last_sync_message("Syncing...".to_string(), cx);
        });

        match self.task_service.get_all_tasks() {
            Ok(all_tasks) => {
                let filter_state = self.filter_state.read(cx).clone();
                let task_filter = TaskFilter::from(&filter_state);
                let filtered_tasks = task_filter.apply(&all_tasks);
                let (projects, tags) = Self::build_sidebar_data(&filtered_tasks);

                let mut project_tree = ProjectTree::new();
                project_tree.build_from_projects(&projects);

                self.sidebar.update(cx, |sidebar, cx| {
                    sidebar.update_projects(project_tree, cx);
                    sidebar.update_tags(tags, cx);
                });

                self.task_table.update(cx, |table, cx| {
                    table.reload_tasks_from_all(all_tasks, cx);
                });

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

    pub fn run() -> () {
        let app = gpui::Application::new();

        app.run(|app: &mut gpui::App| {
            app.set_global(crate::theme::Theme::dark());
            app.open_window(
                gpui::WindowOptions::default(),
                |_window: &mut gpui::Window, app: &mut gpui::App| {
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

                        task_table.update(cx, |table, cx| {
                            table.reload_tasks(&mut task_service, cx);
                        });

                        let app = App {
                            sidebar,
                            filter_state: filter_state.clone(),
                            status_bar: status_bar.clone(),
                            task_table,
                            task_service,
                        };

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

                        app
                    })
                },
            )
            .unwrap();
        });
    }
}
