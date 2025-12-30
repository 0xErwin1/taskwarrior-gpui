use super::{Command, ContextId, Key, KeyChord, KeymapLayer, Mods};

pub fn build_default_keymap() -> KeymapLayer {
    let mut layer = KeymapLayer::new();

    // Global bindings
    layer.bind(
        ContextId::Global,
        KeyChord::new(Key::Char('r'), Mods::ctrl()),
        Command::Sync,
    );
    layer.bind(
        ContextId::Global,
        KeyChord::new(Key::Char('f'), Mods::ctrl()),
        Command::FocusSearch,
    );
    layer.bind(
        ContextId::Global,
        KeyChord::new(Key::Escape, Mods::none()),
        Command::CloseModal,
    );
    layer.bind(
        ContextId::Global,
        KeyChord::new(Key::Tab, Mods::none()),
        Command::FocusTable,
    );
    layer.bind(
        ContextId::Global,
        KeyChord::new(Key::Tab, Mods::shift()),
        Command::FocusSidebar,
    );
    layer.bind(
        ContextId::Global,
        KeyChord::new(Key::Char('c'), Mods::ctrl()),
        Command::ClearAllFilters,
    );
    layer.bind(
        ContextId::Global,
        KeyChord::new(Key::Char('p'), Mods::ctrl()),
        Command::ClearProjectFilter,
    );
    layer.bind(
        ContextId::Global,
        KeyChord::new(Key::Char('t'), Mods::ctrl()),
        Command::ClearTagFilter,
    );
    layer.bind(
        ContextId::Global,
        KeyChord::new(Key::Char('x'), Mods::ctrl()),
        Command::ClearSearchAndDropdowns,
    );

    // Table navigation
    layer.bind(
        ContextId::Table,
        KeyChord::new(Key::Char('h'), Mods::ctrl()),
        Command::FocusSidebarProjects,
    );
    layer.bind(
        ContextId::Table,
        KeyChord::new(Key::Char('k'), Mods::ctrl()),
        Command::FocusTableHeaders,
    );
    layer.bind(
        ContextId::Table,
        KeyChord::new(Key::Char('j'), Mods::none()),
        Command::SelectNextRow,
    );
    layer.bind(
        ContextId::Table,
        KeyChord::new(Key::Char('k'), Mods::none()),
        Command::SelectPrevRow,
    );
    layer.bind(
        ContextId::Table,
        KeyChord::new(Key::ArrowDown, Mods::none()),
        Command::SelectNextRow,
    );
    layer.bind(
        ContextId::Table,
        KeyChord::new(Key::ArrowUp, Mods::none()),
        Command::SelectPrevRow,
    );
    layer.bind(
        ContextId::Table,
        KeyChord::new(Key::Char('g'), Mods::none()),
        Command::SelectFirstRow,
    );
    layer.bind(
        ContextId::Table,
        KeyChord::new(Key::Char('g'), Mods::shift()),
        Command::SelectLastRow,
    );
    layer.bind(
        ContextId::Table,
        KeyChord::new(Key::Home, Mods::none()),
        Command::SelectFirstRow,
    );
    layer.bind(
        ContextId::Table,
        KeyChord::new(Key::End, Mods::none()),
        Command::SelectLastRow,
    );
    layer.bind(
        ContextId::Table,
        KeyChord::new(Key::PageDown, Mods::none()),
        Command::NextPage,
    );
    layer.bind(
        ContextId::Table,
        KeyChord::new(Key::PageUp, Mods::none()),
        Command::PrevPage,
    );
    layer.bind(
        ContextId::Table,
        KeyChord::new(Key::Char('l'), Mods::none()),
        Command::NextPage,
    );
    layer.bind(
        ContextId::Table,
        KeyChord::new(Key::Char('h'), Mods::none()),
        Command::PrevPage,
    );
    layer.bind(
        ContextId::Table,
        KeyChord::new(Key::Escape, Mods::none()),
        Command::ClearSelection,
    );
    layer.bind(
        ContextId::Table,
        KeyChord::new(Key::Enter, Mods::none()),
        Command::OpenSelectedTask,
    );
    layer.bind(
        ContextId::Table,
        KeyChord::new(Key::ArrowLeft, Mods::none()),
        Command::CollapseProject,
    );
    layer.bind(
        ContextId::Table,
        KeyChord::new(Key::ArrowRight, Mods::none()),
        Command::ExpandProject,
    );

    // Table Headers navigation
    layer.bind(
        ContextId::TableHeaders,
        KeyChord::new(Key::Char('h'), Mods::none()),
        Command::HeaderMovePrev,
    );
    layer.bind(
        ContextId::TableHeaders,
        KeyChord::new(Key::Char('l'), Mods::none()),
        Command::HeaderMoveNext,
    );
    layer.bind(
        ContextId::TableHeaders,
        KeyChord::new(Key::ArrowLeft, Mods::none()),
        Command::HeaderMovePrev,
    );
    layer.bind(
        ContextId::TableHeaders,
        KeyChord::new(Key::ArrowRight, Mods::none()),
        Command::HeaderMoveNext,
    );
    layer.bind(
        ContextId::TableHeaders,
        KeyChord::new(Key::Char('j'), Mods::none()),
        Command::HeaderCycleSortOrder,
    );
    layer.bind(
        ContextId::TableHeaders,
        KeyChord::new(Key::Char('k'), Mods::none()),
        Command::HeaderCycleSortOrder,
    );
    layer.bind(
        ContextId::TableHeaders,
        KeyChord::new(Key::Enter, Mods::none()),
        Command::HeaderCycleSortOrder,
    );
    layer.bind(
        ContextId::TableHeaders,
        KeyChord::new(Key::Space, Mods::none()),
        Command::HeaderCycleSortOrder,
    );
    layer.bind(
        ContextId::TableHeaders,
        KeyChord::new(Key::Char('j'), Mods::ctrl()),
        Command::FocusTable,
    );
    layer.bind(
        ContextId::TableHeaders,
        KeyChord::new(Key::Char('k'), Mods::ctrl()),
        Command::FocusSearch,
    );

    // Sidebar Projects navigation
    layer.bind(
        ContextId::SidebarProjects,
        KeyChord::new(Key::Char('l'), Mods::ctrl()),
        Command::FocusTable,
    );
    layer.bind(
        ContextId::SidebarProjects,
        KeyChord::new(Key::Char('j'), Mods::ctrl()),
        Command::FocusSidebarTags,
    );
    layer.bind(
        ContextId::SidebarProjects,
        KeyChord::new(Key::Char('j'), Mods::none()),
        Command::SelectNextRow,
    );
    layer.bind(
        ContextId::SidebarProjects,
        KeyChord::new(Key::Char('k'), Mods::none()),
        Command::SelectPrevRow,
    );
    layer.bind(
        ContextId::SidebarProjects,
        KeyChord::new(Key::ArrowDown, Mods::none()),
        Command::SelectNextRow,
    );
    layer.bind(
        ContextId::SidebarProjects,
        KeyChord::new(Key::ArrowUp, Mods::none()),
        Command::SelectPrevRow,
    );
    layer.bind(
        ContextId::SidebarProjects,
        KeyChord::new(Key::Char('g'), Mods::none()),
        Command::SelectFirstRow,
    );
    layer.bind(
        ContextId::SidebarProjects,
        KeyChord::new(Key::Char('g'), Mods::shift()),
        Command::SelectLastRow,
    );
    layer.bind(
        ContextId::SidebarProjects,
        KeyChord::new(Key::Char('h'), Mods::none()),
        Command::CollapseProject,
    );
    layer.bind(
        ContextId::SidebarProjects,
        KeyChord::new(Key::Char('l'), Mods::none()),
        Command::ExpandProject,
    );
    layer.bind(
        ContextId::SidebarProjects,
        KeyChord::new(Key::ArrowLeft, Mods::none()),
        Command::CollapseProject,
    );
    layer.bind(
        ContextId::SidebarProjects,
        KeyChord::new(Key::ArrowRight, Mods::none()),
        Command::ExpandProject,
    );
    layer.bind(
        ContextId::SidebarProjects,
        KeyChord::new(Key::Enter, Mods::none()),
        Command::OpenSelectedTask,
    );
    layer.bind(
        ContextId::SidebarProjects,
        KeyChord::new(Key::Space, Mods::none()),
        Command::OpenSelectedTask,
    );

    // Sidebar Tags navigation
    layer.bind(
        ContextId::SidebarTags,
        KeyChord::new(Key::Char('l'), Mods::ctrl()),
        Command::FocusTable,
    );
    layer.bind(
        ContextId::SidebarTags,
        KeyChord::new(Key::Char('k'), Mods::ctrl()),
        Command::FocusSidebarProjects,
    );
    layer.bind(
        ContextId::SidebarTags,
        KeyChord::new(Key::Char('j'), Mods::none()),
        Command::SelectNextRow,
    );
    layer.bind(
        ContextId::SidebarTags,
        KeyChord::new(Key::Char('k'), Mods::none()),
        Command::SelectPrevRow,
    );
    layer.bind(
        ContextId::SidebarTags,
        KeyChord::new(Key::ArrowDown, Mods::none()),
        Command::SelectNextRow,
    );
    layer.bind(
        ContextId::SidebarTags,
        KeyChord::new(Key::ArrowUp, Mods::none()),
        Command::SelectPrevRow,
    );
    layer.bind(
        ContextId::SidebarTags,
        KeyChord::new(Key::Char('g'), Mods::none()),
        Command::SelectFirstRow,
    );
    layer.bind(
        ContextId::SidebarTags,
        KeyChord::new(Key::Char('g'), Mods::shift()),
        Command::SelectLastRow,
    );
    layer.bind(
        ContextId::SidebarTags,
        KeyChord::new(Key::Enter, Mods::none()),
        Command::OpenSelectedTask,
    );
    layer.bind(
        ContextId::SidebarTags,
        KeyChord::new(Key::Space, Mods::none()),
        Command::OpenSelectedTask,
    );

    // TextInput / FilterBar
    layer.bind(
        ContextId::TextInput,
        KeyChord::new(Key::Escape, Mods::none()),
        Command::BlurInput,
    );
    layer.bind(
        ContextId::TextInput,
        KeyChord::new(Key::Enter, Mods::none()),
        Command::ApplySearch,
    );
    layer.bind(
        ContextId::TextInput,
        KeyChord::new(Key::Char('l'), Mods::ctrl()),
        Command::FocusFilterNext,
    );
    layer.bind(
        ContextId::TextInput,
        KeyChord::new(Key::Char('h'), Mods::ctrl()),
        Command::FocusFilterPrev,
    );
    layer.bind(
        ContextId::TextInput,
        KeyChord::new(Key::Char('j'), Mods::ctrl()),
        Command::FocusTableHeaders,
    );

    // FilterBar (when focus is on dropdowns)
    layer.bind(
        ContextId::FilterBar,
        KeyChord::new(Key::Enter, Mods::none()),
        Command::ToggleDropdown,
    );
    layer.bind(
        ContextId::FilterBar,
        KeyChord::new(Key::Space, Mods::none()),
        Command::ToggleDropdown,
    );
    layer.bind(
        ContextId::FilterBar,
        KeyChord::new(Key::Char('j'), Mods::none()),
        Command::SelectNextOption,
    );
    layer.bind(
        ContextId::FilterBar,
        KeyChord::new(Key::Char('k'), Mods::none()),
        Command::SelectPrevOption,
    );
    layer.bind(
        ContextId::FilterBar,
        KeyChord::new(Key::ArrowDown, Mods::none()),
        Command::SelectNextOption,
    );
    layer.bind(
        ContextId::FilterBar,
        KeyChord::new(Key::ArrowUp, Mods::none()),
        Command::SelectPrevOption,
    );
    layer.bind(
        ContextId::FilterBar,
        KeyChord::new(Key::Char('l'), Mods::ctrl()),
        Command::FocusFilterNext,
    );
    layer.bind(
        ContextId::FilterBar,
        KeyChord::new(Key::Char('h'), Mods::ctrl()),
        Command::FocusFilterPrev,
    );
    layer.bind(
        ContextId::FilterBar,
        KeyChord::new(Key::Char('j'), Mods::ctrl()),
        Command::FocusTableHeaders,
    );
    layer.bind(
        ContextId::FilterBar,
        KeyChord::new(Key::Escape, Mods::none()),
        Command::BlurInput,
    );

    // Modal
    layer.bind(
        ContextId::Modal,
        KeyChord::new(Key::Escape, Mods::none()),
        Command::CloseModal,
    );
    layer.bind(
        ContextId::Modal,
        KeyChord::new(Key::Char('j'), Mods::none()),
        Command::ModalScrollDown,
    );
    layer.bind(
        ContextId::Modal,
        KeyChord::new(Key::Char('k'), Mods::none()),
        Command::ModalScrollUp,
    );
    layer.bind(
        ContextId::Modal,
        KeyChord::new(Key::ArrowDown, Mods::none()),
        Command::ModalScrollDown,
    );
    layer.bind(
        ContextId::Modal,
        KeyChord::new(Key::ArrowUp, Mods::none()),
        Command::ModalScrollUp,
    );
    layer.bind(
        ContextId::Modal,
        KeyChord::new(Key::Enter, Mods::ctrl()),
        Command::SaveModal,
    );

    layer
}
