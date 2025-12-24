use gpui::prelude::*;
use std::sync::Arc;

use crate::components::{
    self,
    input::{Input, Suggestion},
};

pub(crate) struct App {
    input: gpui::Entity<Input>,
    current_text: gpui::SharedString,
}

impl gpui::Render for App {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        components::panel::Panel::new()
            .child(self.input.clone())
            .child(components::label::Label::new(format!(
                "Text: {}",
                &self.current_text
            )))
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
                        let input = cx.new(|cx| {
                            Input::new("input", cx, "Type here...").with_suggest(Arc::new(|text| {
                                let suggestions = vec![
                                    ("project:", "Filter by project"),
                                    ("+work", "Tag: work"),
                                    ("+personal", "Tag: personal"),
                                    ("due:today", "Tasks for today"),
                                    ("due:tomorrow", "Tasks for tomorrow"),
                                    ("priority:H", "Priority high"),
                                    ("priority:M", "Priority medium"),
                                    ("priority:L", "Priority low"),
                                ];

                                suggestions
                                    .into_iter()
                                    .filter(|(insert, _)| {
                                        text.is_empty()
                                            || insert.to_lowercase().contains(&text.to_lowercase())
                                    })
                                    .map(|(insert, label)| {
                                        Suggestion::new(format!("{} - {}", insert, label), insert)
                                    })
                                    .collect()
                            }))
                        });

                        cx.observe(&input, |this, input, cx| {
                            let value = input.read(cx).value().to_string();
                            this.current_text = value.into();
                            cx.notify();
                        })
                        .detach();

                        App {
                            input,
                            current_text: "".into(),
                        }
                    })
                },
            )
            .unwrap();
        });
    }
}
