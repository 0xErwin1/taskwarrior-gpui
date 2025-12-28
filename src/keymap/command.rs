#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
    // Navigation
    SelectNextRow,
    SelectPrevRow,
    SelectFirstRow,
    SelectLastRow,
    NextPage,
    PrevPage,
    ClearSelection,

    // Actions
    OpenSelectedTask,
    Sync,

    // Focus
    FocusSearch,
    FocusTable,
    FocusTableHeaders,
    FocusSidebar,
    FocusSidebarProjects,
    FocusSidebarTags,
    BlurInput,

    // Modal
    CloseModal,
    SaveModal,

    // Filter
    ApplySearch,
    ClearFilters,
    ClearAllFilters,
    ClearProjectFilter,
    ClearTagFilter,
    ClearSearchAndDropdowns,
    FocusFilterNext,
    FocusFilterPrev,
    ToggleDropdown,
    SelectNextOption,
    SelectPrevOption,

    // Projects
    ExpandProject,
    CollapseProject,

    // Table Headers
    HeaderMoveNext,
    HeaderMovePrev,
    HeaderCycleSortOrder,
}

impl Command {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "SelectNextRow" => Some(Self::SelectNextRow),
            "SelectPrevRow" => Some(Self::SelectPrevRow),
            "SelectFirstRow" => Some(Self::SelectFirstRow),
            "SelectLastRow" => Some(Self::SelectLastRow),
            "NextPage" => Some(Self::NextPage),
            "PrevPage" => Some(Self::PrevPage),
            "ClearSelection" => Some(Self::ClearSelection),
            "OpenSelectedTask" => Some(Self::OpenSelectedTask),
            "Sync" => Some(Self::Sync),
            "FocusSearch" => Some(Self::FocusSearch),
            "FocusTable" => Some(Self::FocusTable),
            "FocusTableHeaders" => Some(Self::FocusTableHeaders),
            "FocusSidebar" => Some(Self::FocusSidebar),
            "FocusSidebarProjects" => Some(Self::FocusSidebarProjects),
            "FocusSidebarTags" => Some(Self::FocusSidebarTags),
            "BlurInput" => Some(Self::BlurInput),
            "CloseModal" => Some(Self::CloseModal),
            "SaveModal" => Some(Self::SaveModal),
            "ApplySearch" => Some(Self::ApplySearch),
            "ClearFilters" => Some(Self::ClearFilters),
            "ClearAllFilters" => Some(Self::ClearAllFilters),
            "ClearProjectFilter" => Some(Self::ClearProjectFilter),
            "ClearTagFilter" => Some(Self::ClearTagFilter),
            "ClearSearchAndDropdowns" => Some(Self::ClearSearchAndDropdowns),
            "FocusFilterNext" => Some(Self::FocusFilterNext),
            "FocusFilterPrev" => Some(Self::FocusFilterPrev),
            "ToggleDropdown" => Some(Self::ToggleDropdown),
            "SelectNextOption" => Some(Self::SelectNextOption),
            "SelectPrevOption" => Some(Self::SelectPrevOption),
            "ExpandProject" => Some(Self::ExpandProject),
            "CollapseProject" => Some(Self::CollapseProject),
            "HeaderMoveNext" => Some(Self::HeaderMoveNext),
            "HeaderMovePrev" => Some(Self::HeaderMovePrev),
            "HeaderCycleSortOrder" => Some(Self::HeaderCycleSortOrder),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SelectNextRow => "SelectNextRow",
            Self::SelectPrevRow => "SelectPrevRow",
            Self::SelectFirstRow => "SelectFirstRow",
            Self::SelectLastRow => "SelectLastRow",
            Self::NextPage => "NextPage",
            Self::PrevPage => "PrevPage",
            Self::ClearSelection => "ClearSelection",
            Self::OpenSelectedTask => "OpenSelectedTask",
            Self::Sync => "Sync",
            Self::FocusSearch => "FocusSearch",
            Self::FocusTable => "FocusTable",
            Self::FocusTableHeaders => "FocusTableHeaders",
            Self::FocusSidebar => "FocusSidebar",
            Self::FocusSidebarProjects => "FocusSidebarProjects",
            Self::FocusSidebarTags => "FocusSidebarTags",
            Self::BlurInput => "BlurInput",
            Self::CloseModal => "CloseModal",
            Self::SaveModal => "SaveModal",
            Self::ApplySearch => "ApplySearch",
            Self::ClearFilters => "ClearFilters",
            Self::ClearAllFilters => "ClearAllFilters",
            Self::ClearProjectFilter => "ClearProjectFilter",
            Self::ClearTagFilter => "ClearTagFilter",
            Self::ClearSearchAndDropdowns => "ClearSearchAndDropdowns",
            Self::FocusFilterNext => "FocusFilterNext",
            Self::FocusFilterPrev => "FocusFilterPrev",
            Self::ToggleDropdown => "ToggleDropdown",
            Self::SelectNextOption => "SelectNextOption",
            Self::SelectPrevOption => "SelectPrevOption",
            Self::ExpandProject => "ExpandProject",
            Self::CollapseProject => "CollapseProject",
            Self::HeaderMoveNext => "HeaderMoveNext",
            Self::HeaderMovePrev => "HeaderMovePrev",
            Self::HeaderCycleSortOrder => "HeaderCycleSortOrder",
        }
    }
}
