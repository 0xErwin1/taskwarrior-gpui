use crate::{
    app::App,
    keymap::{Command, CommandDispatcher, FocusTarget},
};

impl App {
    fn close_task_detail(&mut self, cx: &mut gpui::Context<Self>) {
        self.task_detail_modal.update(cx, |modal, cx| {
            modal.close(cx);
        });
    }

    fn scroll_task_detail(&self, delta: i32, cx: &mut gpui::Context<Self>) {
        self.task_detail_modal.update(cx, |modal, cx| {
            modal.scroll(delta, cx);
        });
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
                        self.open_selected_task(None, cx);
                    }
                }
                true
            }
            Command::CloseModal => {
                self.close_task_detail(cx);
                true
            }
            Command::SaveModal => {
                self.close_task_detail(cx);
                true
            }
            Command::ModalScrollUp => {
                self.scroll_task_detail(-1, cx);
                true
            }
            Command::ModalScrollDown => {
                self.scroll_task_detail(2, cx);
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
