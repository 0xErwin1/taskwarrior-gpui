use crate::keymap::{Command, CommandDispatcher};
use crate::models::{FilterState, ProjectTree};
use crate::theme::ActiveTheme;
use crate::ui::{divider_h, section_header};
use gpui::{
    Context, Div, Entity, IntoElement, ScrollHandle, Stateful, Window, div, prelude::*, px,
};

#[derive(Debug, Clone)]
pub struct TagItem {
    pub name: String,
    pub task_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarSection {
    Projects,
    Tags,
}

pub enum SidebarEvent {
    Focused(SidebarSection),
}

pub struct Sidebar {
    project_tree: ProjectTree,
    tags: Vec<TagItem>,
    filter_state: Entity<FilterState>,
    selected_section: SidebarSection,
    selected_index: Option<usize>,
    projects_scroll_handle: ScrollHandle,
    tags_scroll_handle: ScrollHandle,
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
            selected_section: SidebarSection::Projects,
            selected_index: Some(0),
            projects_scroll_handle: ScrollHandle::new(),
            tags_scroll_handle: ScrollHandle::new(),
        }
    }

    pub fn update_projects(&mut self, mut project_tree: ProjectTree, cx: &mut Context<Self>) {
        let expanded_paths = self.project_tree.get_expanded_paths();
        project_tree.restore_expanded_paths(expanded_paths);

        self.project_tree = project_tree;
        cx.notify();
    }

    pub fn update_tags(&mut self, tags: Vec<TagItem>, cx: &mut Context<Self>) {
        self.tags = tags;
        cx.notify();
    }

    pub fn set_section(&mut self, section: SidebarSection, cx: &mut Context<Self>) {
        self.selected_section = section;
        self.selected_index = Some(0);
        self.scroll_to_selected();
        cx.notify();
    }

    fn handle_project_click(
        &mut self,
        full_path: Option<String>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.emit(SidebarEvent::Focused(SidebarSection::Projects));
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
        cx.emit(SidebarEvent::Focused(SidebarSection::Tags));
        self.filter_state.update(cx, |filter, cx| {
            filter.toggle_tag(tag_name);
            cx.notify();
        });
        cx.notify();
    }

    fn handle_clear_tags(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        cx.emit(SidebarEvent::Focused(SidebarSection::Tags));
        self.filter_state.update(cx, |filter, cx| {
            filter.active_tags.clear();
            cx.notify();
        });
        cx.notify();
    }

    fn handle_clear_project(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        cx.emit(SidebarEvent::Focused(SidebarSection::Projects));
        self.filter_state.update(cx, |filter, cx| {
            filter.selected_project = None;
            cx.notify();
        });
        cx.notify();
    }

    fn get_items_count(&self) -> usize {
        match self.selected_section {
            SidebarSection::Projects => 1 + self.project_tree.iter_visible().len(),
            SidebarSection::Tags => self.tags.len(),
        }
    }

    fn scroll_to_selected(&mut self) {
        if let Some(idx) = self.selected_index {
            match self.selected_section {
                SidebarSection::Projects => {
                    self.projects_scroll_handle.scroll_to_item(idx);
                }
                SidebarSection::Tags => {
                    self.tags_scroll_handle.scroll_to_item(idx);
                }
            }
        }
    }

    fn select_next(&mut self, cx: &mut Context<Self>) {
        let count = self.get_items_count();
        if count == 0 {
            return;
        }

        if let Some(idx) = self.selected_index {
            if idx + 1 < count {
                self.selected_index = Some(idx + 1);
            }
        } else {
            self.selected_index = Some(0);
        }
        self.scroll_to_selected();
        cx.notify();
    }

    fn select_prev(&mut self, cx: &mut Context<Self>) {
        if let Some(idx) = self.selected_index {
            if idx > 0 {
                self.selected_index = Some(idx - 1);
            }
        } else {
            let count = self.get_items_count();
            if count > 0 {
                self.selected_index = Some(count - 1);
            }
        }
        self.scroll_to_selected();
        cx.notify();
    }

    fn select_first(&mut self, cx: &mut Context<Self>) {
        self.selected_index = Some(0);
        self.scroll_to_selected();
        cx.notify();
    }

    fn select_last(&mut self, cx: &mut Context<Self>) {
        let count = self.get_items_count();
        if count > 0 {
            self.selected_index = Some(count - 1);
        }
        self.scroll_to_selected();
        cx.notify();
    }

    fn expand_selected(&mut self, cx: &mut Context<Self>) {
        if self.selected_section != SidebarSection::Projects {
            return;
        }

        let idx = match self.selected_index {
            Some(i) => i,
            None => return,
        };

        if idx > 0 {
            let visible = self.project_tree.iter_visible();
            let should_expand = if let Some((_, node)) = visible.get(idx - 1) {
                if node.has_children() && !node.is_expanded {
                    Some(node.full_path.clone())
                } else {
                    None
                }
            } else {
                None
            };
            drop(visible);

            if let Some(path) = should_expand {
                self.project_tree.toggle_expansion(&path);
                cx.notify();
            }
        }
    }

    fn collapse_selected(&mut self, cx: &mut Context<Self>) {
        if self.selected_section != SidebarSection::Projects {
            return;
        }

        let idx = match self.selected_index {
            Some(i) => i,
            None => return,
        };

        if idx > 0 {
            let visible = self.project_tree.iter_visible();
            let should_collapse = if let Some((_, node)) = visible.get(idx - 1) {
                if node.has_children() && node.is_expanded {
                    Some(node.full_path.clone())
                } else {
                    None
                }
            } else {
                None
            };
            drop(visible);

            if let Some(path) = should_collapse {
                self.project_tree.toggle_expansion(&path);
                cx.notify();
            }
        }
    }

    fn activate_selected(&mut self, cx: &mut Context<Self>) {
        let idx = match self.selected_index {
            Some(i) => i,
            None => return,
        };

        match self.selected_section {
            SidebarSection::Projects => {
                if idx == 0 {
                    self.filter_state.update(cx, |filter, cx| {
                        filter.select_project(None);
                        cx.notify();
                    });
                } else {
                    let visible = self.project_tree.iter_visible();
                    if let Some((_, node)) = visible.get(idx - 1) {
                        self.filter_state.update(cx, |filter, cx| {
                            filter.select_project(Some(node.full_path.clone()));
                            cx.notify();
                        });
                    }
                }
            }
            SidebarSection::Tags => {
                if let Some(tag) = self.tags.get(idx) {
                    let tag_name = tag.name.clone();
                    self.filter_state.update(cx, |filter, cx| {
                        filter.toggle_tag(tag_name);
                        cx.notify();
                    });
                }
            }
        }
        cx.notify();
    }

    fn render_projects(&self, cx: &mut Context<Self>) -> Vec<Stateful<Div>> {
        let theme = cx.theme();
        let filter = self.filter_state.read(cx);
        let mut elements = Vec::new();

        let is_all_selected = filter.selected_project.is_none();
        let is_keyboard_selected =
            self.selected_section == SidebarSection::Projects && self.selected_index == Some(0);

        elements.push(
            div()
                .id(("project", 0usize))
                .flex()
                .items_center()
                .gap_1()
                .px_3()
                .py_1()
                .rounded_sm()
                .cursor_pointer()
                .when(is_all_selected, |this| this.bg(theme.selection))
                .when(!is_all_selected && is_keyboard_selected, |this| {
                    this.bg(theme.hover)
                })
                .when(!is_all_selected && !is_keyboard_selected, |this| {
                    this.hover(|s| s.bg(theme.hover))
                })
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(|view, _event, window, cx| {
                        view.handle_project_click(None, window, cx);
                    }),
                )
                .child(
                    div()
                        .w_4()
                        .text_color(theme.accent)
                        .child(if is_keyboard_selected { ">" } else { " " }),
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

        for (idx, (_tree_idx, node)) in self.project_tree.iter_visible().iter().enumerate() {
            let is_selected = filter
                .selected_project
                .as_ref()
                .map(|p| p == &node.full_path)
                .unwrap_or(false);

            let is_keyboard_selected = self.selected_section == SidebarSection::Projects
                && self.selected_index == Some(idx + 1);

            let indent = node.level * 16;
            let full_path = node.full_path.clone();
            let full_path_for_expand = node.full_path.clone();
            let has_children = node.has_children();
            let is_expanded = node.is_expanded;

            elements.push(
                div()
                    .id(("project", idx + 1))
                    .flex()
                    .items_center()
                    .gap_1()
                    .px_3()
                    .py_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .when(is_selected, |this| this.bg(theme.selection))
                    .when(!is_selected && is_keyboard_selected, |this| {
                        this.bg(theme.hover)
                    })
                    .when(!is_selected && !is_keyboard_selected, |this| {
                        this.hover(|s| s.bg(theme.hover))
                    })
                    .child(
                        div()
                            .w_4()
                            .text_color(theme.accent)
                            .child(if is_keyboard_selected { ">" } else { " " }),
                    )
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

    fn render_tags(&self, cx: &mut Context<Self>) -> Vec<Stateful<Div>> {
        let theme = cx.theme();
        let filter = self.filter_state.read(cx);
        let mut elements = Vec::new();

        for (idx, tag) in self.tags.iter().enumerate() {
            let is_active = filter.active_tags.contains(&tag.name);
            let is_keyboard_selected =
                self.selected_section == SidebarSection::Tags && self.selected_index == Some(idx);
            let tag_name = tag.name.clone();

            elements.push(
                div()
                    .id(("tag", idx))
                    .flex()
                    .items_center()
                    .gap_1()
                    .px_3()
                    .py_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .when(is_active, |this| this.bg(theme.selection))
                    .when(!is_active && is_keyboard_selected, |this| {
                        this.bg(theme.hover)
                    })
                    .when(!is_active && !is_keyboard_selected, |this| {
                        this.hover(|s| s.bg(theme.hover))
                    })
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(move |view, _event, window, cx| {
                            view.handle_tag_click(tag_name.clone(), window, cx);
                        }),
                    )
                    .child(
                        div()
                            .w_4()
                            .text_color(theme.accent)
                            .child(if is_keyboard_selected { ">" } else { " " }),
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

impl CommandDispatcher for Sidebar {
    fn dispatch(&mut self, command: Command, cx: &mut Context<Self>) -> bool {
        match command {
            Command::SelectNextRow => {
                self.select_next(cx);
                true
            }
            Command::SelectPrevRow => {
                self.select_prev(cx);
                true
            }
            Command::SelectFirstRow => {
                self.select_first(cx);
                true
            }
            Command::SelectLastRow => {
                self.select_last(cx);
                true
            }
            Command::OpenSelectedTask => {
                self.activate_selected(cx);
                true
            }
            Command::ExpandProject => {
                self.expand_selected(cx);
                true
            }
            Command::CollapseProject => {
                self.collapse_selected(cx);
                true
            }
            _ => false,
        }
    }
}

impl gpui::EventEmitter<SidebarEvent> for Sidebar {}

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
                            .track_scroll(&self.projects_scroll_handle)
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
                            .track_scroll(&self.tags_scroll_handle)
                            .children(tags),
                    ),
            )
    }
}
