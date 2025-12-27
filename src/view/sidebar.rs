use crate::components::{label::Label, panel::Panel};
use crate::models::{FilterState, ProjectTree};
use crate::theme::ActiveTheme;
use gpui::{Context, Div, Entity, IntoElement, Window, div, prelude::*, px};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TagItem {
    pub name: String,
    pub task_count: usize,
}

pub struct Sidebar {
    project_tree: ProjectTree,
    tags: Vec<TagItem>,
    filter_state: Entity<FilterState>,
    on_filter_change: Option<Arc<dyn Fn(FilterState, &mut Window, &mut Context<Self>) + 'static>>,
}

impl Sidebar {
    pub fn new(
        project_tree: ProjectTree,
        tags: Vec<TagItem>,
        filter_state: Entity<FilterState>,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&filter_state, |_sidebar, _filter, cx| {
            cx.notify();
        })
        .detach();

        Self {
            project_tree,
            tags,
            filter_state,
            on_filter_change: None,
        }
    }

    pub fn on_filter_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(FilterState, &mut Window, &mut Context<Self>) + 'static,
    {
        self.on_filter_change = Some(Arc::new(callback));
        self
    }

    pub fn update_projects(&mut self, project_tree: ProjectTree, cx: &mut Context<Self>) {
        self.project_tree = project_tree;
        cx.notify();
    }

    pub fn update_tags(&mut self, tags: Vec<TagItem>, cx: &mut Context<Self>) {
        self.tags = tags;
        cx.notify();
    }

    fn handle_project_click(
        &mut self,
        full_path: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.filter_state.update(cx, |filter, _cx| {
            filter.select_project(full_path);
        });

        if let Some(callback) = &self.on_filter_change {
            let filter = self.filter_state.read(cx).clone();
            callback(filter, window, cx);
        }

        cx.notify();
    }

    fn handle_expand_toggle(&mut self, full_path: String, cx: &mut Context<Self>) {
        self.project_tree.toggle_expansion(&full_path);
        cx.notify();
    }

    fn handle_tag_click(&mut self, tag_name: String, window: &mut Window, cx: &mut Context<Self>) {
        self.filter_state.update(cx, |filter, _cx| {
            filter.toggle_tag(tag_name);
        });

        if let Some(callback) = &self.on_filter_change {
            let filter = self.filter_state.read(cx).clone();
            callback(filter, window, cx);
        }

        cx.notify();
    }

    fn render_projects(&self, cx: &mut Context<Self>) -> Vec<Div> {
        let theme = cx.theme();
        let filter = self.filter_state.read(cx);
        let mut elements = Vec::new();

        let is_all_selected = filter.selected_project.is_none();
        elements.push(
            div()
                .flex()
                .items_center()
                .gap_2()
                .px_3()
                .py_1()
                .cursor_pointer()
                .when(is_all_selected, |this| this.bg(theme.selection))
                .hover(|style| style.bg(theme.panel))
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(|view, _event, window, cx| {
                        view.handle_project_click(None, window, cx);
                    }),
                )
                .child(
                    div()
                        .w_3()
                        .h_3()
                        .rounded_full()
                        .border_1()
                        .border_color(theme.accent)
                        .when(is_all_selected, |this| this.bg(theme.accent)),
                )
                .child(
                    div()
                        .text_color(if is_all_selected {
                            theme.foreground
                        } else {
                            theme.muted
                        })
                        .child("All"),
                ),
        );

        for (_idx, node) in self.project_tree.iter_visible() {
            let is_selected = filter
                .selected_project
                .as_ref()
                .map(|p| p == &node.full_path)
                .unwrap_or(false);

            let indent = node.level * 16;
            let full_path = node.full_path.clone();
            let full_path_for_expand = node.full_path.clone();
            let has_children = node.has_children();
            let is_expanded = node.is_expanded;

            elements.push(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .px_3()
                    .py_1()
                    .cursor_pointer()
                    .when(is_selected, |this| this.bg(theme.selection))
                    .hover(|style| style.bg(theme.panel))
                    .child(div().w(px(indent as f32)))
                    .child(
                        div()
                            .w_4()
                            .h_4()
                            .flex()
                            .items_center()
                            .justify_center()
                            .when(has_children, |this| {
                                this.on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |view, _event, _window, cx| {
                                        view.handle_expand_toggle(full_path_for_expand.clone(), cx);
                                    }),
                                )
                            })
                            .child(if has_children {
                                div()
                                    .text_color(theme.muted)
                                    .text_xs()
                                    .child(if is_expanded { "▼" } else { "▶" })
                            } else {
                                div()
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .items_center()
                            .gap_2()
                            .on_mouse_down(
                                gpui::MouseButton::Left,
                                cx.listener(move |view, _event, window, cx| {
                                    view.handle_project_click(Some(full_path.clone()), window, cx);
                                }),
                            )
                            .child(
                                div()
                                    .w_3()
                                    .h_3()
                                    .rounded_full()
                                    .border_1()
                                    .border_color(theme.accent)
                                    .when(is_selected, |this| this.bg(theme.accent)),
                            )
                            .child(
                                div()
                                    .text_color(if is_selected {
                                        theme.foreground
                                    } else {
                                        theme.muted
                                    })
                                    .child(format!("{} ({})", node.name, node.task_count)),
                            ),
                    ),
            );
        }

        elements
    }

    fn render_tags(&self, cx: &mut Context<Self>) -> Vec<Div> {
        let theme = cx.theme();
        let filter = self.filter_state.read(cx);
        let mut elements = Vec::new();

        for tag in &self.tags {
            let is_active = filter.active_tags.contains(&tag.name);
            let tag_name = tag.name.clone();

            elements.push(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_1()
                    .cursor_pointer()
                    .when(is_active, |this| this.bg(theme.selection))
                    .hover(|style| style.bg(theme.panel))
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(move |view, _event, window, cx| {
                            view.handle_tag_click(tag_name.clone(), window, cx);
                        }),
                    )
                    .child(
                        div()
                            .w_3()
                            .h_3()
                            .rounded_sm()
                            .border_1()
                            .border_color(theme.accent)
                            .when(is_active, |this| {
                                this.bg(theme.accent)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(div().text_color(theme.background).text_xs().child("✓"))
                            }),
                    )
                    .child(
                        div()
                            .text_color(if is_active {
                                theme.foreground
                            } else {
                                theme.muted
                            })
                            .child(format!("{} ({})", tag.name, tag.task_count)),
                    ),
            );
        }

        elements
    }
}

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let projects = self.render_projects(cx);
        let tags = self.render_tags(cx);

        Panel::new("Sidebar").border(1.0).padding(0.0).child(
            div()
                .flex()
                .flex_col()
                .h_full()
                .bg(theme.background)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .h(gpui::relative(0.5))
                        .overflow_hidden()
                        .child(
                            div()
                                .px_3()
                                .py_2()
                                .border_b_1()
                                .border_color(theme.border)
                                .child(
                                    Label::new("PROJECTS")
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(theme.foreground),
                                ),
                        )
                        .child(
                            div()
                                .id("sidebar-projects")
                                .flex()
                                .flex_col()
                                .flex_1()
                                .min_h_0()
                                .py_2()
                                .overflow_y_scroll()
                                .scrollbar_width(gpui::px(6.0))
                                .children(projects),
                        ),
                )
                .child(div().h_px().bg(theme.border))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .h(gpui::relative(0.5))
                        .overflow_hidden()
                        .child(
                            div()
                                .px_3()
                                .py_2()
                                .border_b_1()
                                .border_color(theme.border)
                                .child(
                                    Label::new("TAGS")
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(theme.foreground),
                                ),
                        )
                        .child(
                            div()
                                .id("sidebar-tags")
                                .flex()
                                .flex_col()
                                .flex_1()
                                .min_h_0()
                                .py_2()
                                .overflow_y_scroll()
                                .scrollbar_width(gpui::px(6.0))
                                .children(tags),
                        ),
                ),
        )
    }
}
