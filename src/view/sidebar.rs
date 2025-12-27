use crate::models::{FilterState, ProjectTree};
use crate::theme::ActiveTheme;
use crate::ui::{divider_h, section_header};
use gpui::{Context, Div, Entity, IntoElement, Window, div, prelude::*, px};

#[derive(Debug, Clone)]
pub struct TagItem {
    pub name: String,
    pub task_count: usize,
}

pub struct Sidebar {
    project_tree: ProjectTree,
    tags: Vec<TagItem>,
    filter_state: Entity<FilterState>,
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
        }
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
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.filter_state.update(cx, |filter, cx| {
            filter.select_project(full_path);
            cx.notify();
        });

        cx.notify();
    }

    fn handle_expand_toggle(&mut self, full_path: String, cx: &mut Context<Self>) {
        self.project_tree.toggle_expansion(&full_path);
        cx.notify();
    }

    fn handle_tag_click(&mut self, tag_name: String, _window: &mut Window, cx: &mut Context<Self>) {
        self.filter_state.update(cx, |filter, cx| {
            filter.toggle_tag(tag_name);
            cx.notify();
        });
        cx.notify();
    }

    fn handle_clear_tags(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.filter_state.update(cx, |filter, cx| {
            filter.active_tags.clear();
            cx.notify();
        });
        cx.notify();
    }

    fn handle_clear_project(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.filter_state.update(cx, |filter, cx| {
            filter.selected_project = None;
            cx.notify();
        });
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
                .rounded_sm()
                .cursor_pointer()
                .when(is_all_selected, |this| this.bg(theme.selection))
                .when(!is_all_selected, |this| this.hover(|style| style.bg(theme.hover)))
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
                    .rounded_sm()
                    .cursor_pointer()
                    .when(is_selected, |this| this.bg(theme.selection))
                    .when(!is_selected, |this| this.hover(|style| style.bg(theme.hover)))
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
                    .rounded_sm()
                    .cursor_pointer()
                    .when(is_active, |this| this.bg(theme.selection))
                    .when(!is_active, |this| this.hover(|style| style.bg(theme.hover)))
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
        let filter = self.filter_state.read(cx);
        let has_project = filter.selected_project.is_some();
        let has_tags = !filter.active_tags.is_empty();

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_2()
                            .py_2()
                            .child(section_header("Projects", &theme))
                            .when(has_project, |this| {
                                this.child(
                                    div()
                                        .id("clear-project")
                                        .text_xs()
                                        .text_color(theme.muted)
                                        .cursor_pointer()
                                        .hover(|s| s.text_color(theme.accent))
                                        .on_mouse_down(
                                            gpui::MouseButton::Left,
                                            cx.listener(|view, _, window, cx| {
                                                view.handle_clear_project(window, cx);
                                            }),
                                        )
                                        .child("Clear"),
                                )
                            }),
                    )
                    .child(
                        div()
                            .id("sidebar-projects")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .children(projects),
                    ),
            )
            .child(divider_h(&theme).my_1())
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_2()
                            .py_2()
                            .child(section_header("Tags", &theme))
                            .when(has_tags, |this| {
                                this.child(
                                    div()
                                        .id("clear-tags")
                                        .text_xs()
                                        .text_color(theme.muted)
                                        .cursor_pointer()
                                        .hover(|s| s.text_color(theme.accent))
                                        .on_mouse_down(
                                            gpui::MouseButton::Left,
                                            cx.listener(|view, _, window, cx| {
                                                view.handle_clear_tags(window, cx);
                                            }),
                                        )
                                        .child("Clear"),
                                )
                            }),
                    )
                    .child(
                        div()
                            .id("sidebar-tags")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .children(tags),
                    ),
            )
    }
}
