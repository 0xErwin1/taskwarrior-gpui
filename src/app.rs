use gpui::prelude::*;

use crate::models::{FilterState, ProjectTree};
use crate::task::{TaskOverview, TaskService};
use crate::theme::ActiveTheme;
use crate::view::sidebar::{Sidebar, TagItem};
use crate::view::status_bar::StatusBar;
use gpui::div;

pub(crate) struct App {
    sidebar: gpui::Entity<Sidebar>,
    filter_state: gpui::Entity<FilterState>,
    status_bar: gpui::Entity<StatusBar>,
    task_service: TaskService,
}

impl gpui::Render for App {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme = _cx.theme();

        let content = div()
            .flex()
            .flex_1()
            .child(div().w(gpui::px(250.)).h_full().child(self.sidebar.clone()))
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_color(theme.muted)
                            .child("Main panel - TaskTable will go here"),
                    ),
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

                        let sidebar = cx.new(|cx| {
                            Sidebar::new(project_tree, tags, filter_state.clone(), cx)
                        });

                        App {
                            sidebar,
                            filter_state,
                            status_bar,
                            task_service,
                        }
                    })
                },
            )
            .unwrap();
        });
    }
}
