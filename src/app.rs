use gpui::prelude::*;

use crate::models::{FilterState, ProjectTree};
use crate::theme::ActiveTheme;
use crate::view::sidebar::{Sidebar, TagItem};
use crate::view::status_bar::StatusBar;
use gpui::div;

pub(crate) struct App {
    sidebar: gpui::Entity<Sidebar>,
    filter_state: gpui::Entity<FilterState>,
    status_bar: gpui::Entity<StatusBar>,
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

                        let mut project_tree = ProjectTree::new();
                        project_tree.build_from_projects(&[
                            ("Work.Backend.API".to_string(), 5),
                            ("Work.Backend.DB".to_string(), 7),
                            ("Work.Frontend.React".to_string(), 3),
                            ("Work.Frontend.Styling".to_string(), 2),
                            ("Home.Kitchen".to_string(), 4),
                            ("Home.Garden".to_string(), 2),
                            ("ignis.v0.1.phase0".to_string(), 15),
                            ("free-ai".to_string(), 2),
                        ]);

                        project_tree.expand_path("Work");
                        project_tree.expand_path("Work.Backend");

                        let tags = vec![
                            TagItem {
                                name: "parser".to_string(),
                                task_count: 8,
                            },
                            TagItem {
                                name: "cli".to_string(),
                                task_count: 5,
                            },
                            TagItem {
                                name: "testing".to_string(),
                                task_count: 6,
                            },
                            TagItem {
                                name: "diagnostics".to_string(),
                                task_count: 7,
                            },
                            TagItem {
                                name: "analyzer".to_string(),
                                task_count: 3,
                            },
                        ];

                        let status_bar = cx.new(|cx| StatusBar::new(cx));

                        let sidebar = cx.new(|cx| {
                            Sidebar::new(project_tree, tags, filter_state.clone(), cx)
                                .on_filter_change(|filter, _window, _cx| {
                                    println!("Filter changed:");
                                    if let Some(ref project) = filter.selected_project {
                                        println!("  Project: {}", project);
                                    } else {
                                        println!("  Project: All");
                                    }
                                    println!("  Active tags: {:?}", filter.active_tags);
                                })
                        });

                        App {
                            sidebar,
                            filter_state,
                            status_bar,
                        }
                    })
                },
            )
            .unwrap();
        });
    }
}
