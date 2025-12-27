use gpui::prelude::*;

use crate::models::{FilterState, ProjectTree};
use crate::task::{TaskOverview, TaskService};
use crate::theme::ActiveTheme;
use crate::view::sidebar::{Sidebar, TagItem};
use crate::view::status_bar::StatusBar;
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

        let content = div()
            .flex()
            .flex_1()
            .min_h_0()
            .overflow_hidden()
            .child(
                div()
                    .w(gpui::px(250.))
                    .h_full()
                    .flex_shrink_0()
                    .child(self.sidebar.clone()),
            )
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .min_w_0()
                    .child(self.task_table.clone()),
            );

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.background)
            .child(content)
            .child(self.status_bar.clone())
    }
}

impl App {
    fn reload_tasks(&mut self, cx: &mut gpui::Context<Self>) {
        let task_service = &mut self.task_service;
        self.task_table.update(cx, |table, cx| {
            table.reload_tasks(task_service, cx);
        });
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
                            status_bar,
                            task_table,
                            task_service,
                        };

                        cx.observe(&filter_state, |app, _, cx| {
                            app.reload_tasks(cx);
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
