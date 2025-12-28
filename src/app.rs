use std::collections::HashMap;

use gpui::prelude::*;

use crate::keymap::{Command, CommandDispatcher, ContextId, FocusTarget, KeyChord, KeymapStack};
use crate::models::{FilterState, ProjectTree};
use crate::task::{self, TaskOverview, TaskService};
use crate::theme::ActiveTheme;
use crate::ui::{ROOT_PADDING, SECTION_GAP, SIDEBAR_WIDTH};
use crate::view::sidebar::{Sidebar, TagItem};
use crate::view::status_bar::{StatusBar, StatusBarEvent, SyncState};
use crate::view::task_table::TaskTable;
use gpui::div;

pub(crate) struct App {
    focus_handle: gpui::FocusHandle,
    focus_target: FocusTarget,
    keymap: KeymapStack,
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

        let sidebar_focused = self.focus_target.is_sidebar();

        let sidebar_border_color = if sidebar_focused {
            theme.focus_ring
        } else {
            theme.divider
        };

        let sidebar = div()
            .bg(theme.card)
            .border_2()
            .border_color(sidebar_border_color)
            .rounded(crate::ui::CARD_RADIUS)
            .p(crate::ui::CARD_PADDING)
            .w(SIDEBAR_WIDTH)
            .h_full()
            .flex_shrink_0()
            .overflow_hidden()
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|app, _event, _window, cx| {
                    if !app.focus_target.is_sidebar() {
                        app.focus_target = FocusTarget::SidebarProjects;
                        app.sidebar.update(cx, |sidebar, cx| {
                            sidebar.set_section(crate::view::sidebar::SidebarSection::Projects, cx);
                        });
                        cx.notify();
                    }
                }),
            )
            .child(self.sidebar.clone());

        let table_focused = matches!(
            self.focus_target,
            FocusTarget::Table | FocusTarget::TableHeaders
        );

        let table_border_color = if table_focused {
            theme.focus_ring
        } else {
            theme.divider
        };

        let main = div()
            .bg(theme.card)
            .border_2()
            .border_color(table_border_color)
            .rounded(crate::ui::CARD_RADIUS)
            .p(crate::ui::CARD_PADDING)
            .flex_1()
            .h_full()
            .min_w_0()
            .overflow_hidden()
            .p_0()
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|app, _event, _window, cx| {
                    if !matches!(app.focus_target, FocusTarget::Table) {
                        app.focus_target = FocusTarget::Table;
                        cx.notify();
                    }
                }),
            )
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
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|app, event, window, cx| {
                app.handle_key_down(event, window, cx);
            }))
            .child(content)
            .child(self.status_bar.clone())
    }
}

impl CommandDispatcher for App {
    fn dispatch(&mut self, command: Command, cx: &mut gpui::Context<Self>) -> bool {
        match command {
            Command::Sync => {
                self.handle_sync(cx);
                true
            }
            Command::FocusSearch => false,
            Command::FocusTable => {
                self.focus_target = match self.focus_target {
                    FocusTarget::Table => {
                        self.sidebar.update(cx, |sidebar, cx| {
                            sidebar.set_section(crate::view::sidebar::SidebarSection::Projects, cx);
                        });
                        FocusTarget::SidebarProjects
                    }
                    FocusTarget::SidebarProjects => {
                        self.sidebar.update(cx, |sidebar, cx| {
                            sidebar.set_section(crate::view::sidebar::SidebarSection::Tags, cx);
                        });
                        FocusTarget::SidebarTags
                    }
                    _ => FocusTarget::Table,
                };
                cx.notify();
                true
            }
            Command::FocusSidebar => {
                self.focus_target = match self.focus_target {
                    FocusTarget::Table => {
                        self.sidebar.update(cx, |sidebar, cx| {
                            sidebar.set_section(crate::view::sidebar::SidebarSection::Tags, cx);
                        });
                        FocusTarget::SidebarTags
                    }
                    FocusTarget::SidebarTags => {
                        self.sidebar.update(cx, |sidebar, cx| {
                            sidebar.set_section(crate::view::sidebar::SidebarSection::Projects, cx);
                        });
                        FocusTarget::SidebarProjects
                    }
                    _ => FocusTarget::Table,
                };
                cx.notify();
                true
            }
            Command::FocusSidebarProjects => {
                self.focus_target = FocusTarget::SidebarProjects;
                self.sidebar.update(cx, |sidebar, cx| {
                    sidebar.set_section(crate::view::sidebar::SidebarSection::Projects, cx);
                });
                cx.notify();
                true
            }
            Command::FocusSidebarTags => {
                self.focus_target = FocusTarget::SidebarTags;
                self.sidebar.update(cx, |sidebar, cx| {
                    sidebar.set_section(crate::view::sidebar::SidebarSection::Tags, cx);
                });
                cx.notify();
                true
            }
            Command::SelectNextRow
            | Command::SelectPrevRow
            | Command::SelectFirstRow
            | Command::SelectLastRow => {
                match self.focus_target {
                    FocusTarget::SidebarProjects | FocusTarget::SidebarTags => {
                        self.sidebar
                            .update(cx, |sidebar, cx| sidebar.dispatch(command, cx));
                    }
                    _ => {
                        self.task_table
                            .update(cx, |table, cx| table.dispatch(command, cx));
                    }
                }
                true
            }
            Command::OpenSelectedTask => {
                match self.focus_target {
                    FocusTarget::SidebarProjects | FocusTarget::SidebarTags => {
                        self.sidebar
                            .update(cx, |sidebar, cx| sidebar.dispatch(command, cx));
                    }
                    _ => {
                        self.task_table
                            .update(cx, |table, cx| table.dispatch(command, cx));
                    }
                }
                true
            }
            Command::ExpandProject | Command::CollapseProject => {
                match self.focus_target {
                    FocusTarget::SidebarProjects => {
                        self.sidebar
                            .update(cx, |sidebar, cx| sidebar.dispatch(command, cx));
                    }
                    _ => {
                        self.task_table
                            .update(cx, |table, cx| table.dispatch(command, cx));
                    }
                }
                true
            }
            Command::NextPage | Command::PrevPage | Command::ClearSelection => {
                self.task_table
                    .update(cx, |table, cx| table.dispatch(command, cx));
                true
            }
            Command::ToggleDropdown
            | Command::SelectNextOption
            | Command::SelectPrevOption
            | Command::BlurInput => {
                self.task_table
                    .update(cx, |table, cx| table.dispatch(command, cx));
                true
            }
            Command::ClearAllFilters => {
                self.filter_state.update(cx, |state, cx| {
                    state.clear();
                    cx.notify();
                });
                true
            }
            Command::ClearProjectFilter => {
                self.filter_state.update(cx, |state, cx| {
                    state.clear_project();
                    cx.notify();
                });
                true
            }
            Command::ClearTagFilter => {
                self.filter_state.update(cx, |state, cx| {
                    state.clear_tags();
                    cx.notify();
                });
                true
            }
            Command::ClearSearchAndDropdowns => {
                self.filter_state.update(cx, |state, cx| {
                    state.clear_search_and_dropdowns();
                    cx.notify();
                });
                self.task_table.update(cx, |table, cx| {
                    table.clear_search_input(cx);
                    table.reset_dropdowns(cx);
                });
                true
            }
            Command::HeaderMoveNext => {
                self.task_table.update(cx, |table, cx| {
                    table.header_move_next(cx);
                });
                true
            }
            Command::HeaderMovePrev => {
                self.task_table.update(cx, |table, cx| {
                    table.header_move_prev(cx);
                });
                true
            }
            Command::HeaderCycleSortOrder => {
                self.task_table.update(cx, |table, cx| {
                    table.header_cycle_sort_order(cx);
                });
                true
            }
            _ => false,
        }
    }
}

impl App {
    fn build_sidebar_data(tasks: &[task::Task]) -> (Vec<(String, usize)>, Vec<TagItem>) {
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

    fn update_ui_from_tasks(&mut self, all_tasks: Vec<task::Task>, cx: &mut gpui::Context<Self>) {
        let (projects, tags) = Self::build_sidebar_data(&all_tasks);

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

    fn reload_tasks(&mut self, cx: &mut gpui::Context<Self>) {
        match self.task_service.get_all_tasks() {
            Ok(all_tasks) => {
                self.status_bar.update(cx, |bar, cx| {
                    bar.clear_error(cx);
                });
                self.update_ui_from_tasks(all_tasks, cx);
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

    fn handle_sync(&mut self, cx: &mut gpui::Context<Self>) {
        self.status_bar.update(cx, |bar, cx| {
            bar.set_sync_state(SyncState::Syncing, cx);
            bar.set_last_sync_message("Syncing...".to_string(), cx);
        });

        match self.task_service.get_all_tasks() {
            Ok(all_tasks) => {
                self.update_ui_from_tasks(all_tasks, cx);

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

    fn active_context(&self, cx: &gpui::Context<Self>) -> ContextId {
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

                        let mut keymap = KeymapStack::new();
                        keymap.push_layer(crate::keymap::defaults::build_default_keymap());

                        let app = App {
                            focus_handle: cx.focus_handle(),
                            focus_target: FocusTarget::Table,
                            keymap,
                            sidebar,
                            filter_state: filter_state.clone(),
                            status_bar: status_bar.clone(),
                            task_table,
                            task_service,
                        };

                        window.focus(&app.focus_handle);

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
