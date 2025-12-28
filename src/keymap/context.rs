#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextId {
    Global,
    Table,
    TableHeaders,
    SidebarProjects,
    SidebarTags,
    Modal,
    FilterBar,
    TextInput,
}

impl ContextId {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "global" | "Global" => Some(Self::Global),
            "table" | "Table" => Some(Self::Table),
            "tableheaders" | "TableHeaders" => Some(Self::TableHeaders),
            "sidebarprojects" | "SidebarProjects" => Some(Self::SidebarProjects),
            "sidebartags" | "SidebarTags" => Some(Self::SidebarTags),
            "modal" | "Modal" => Some(Self::Modal),
            "filterbar" | "FilterBar" => Some(Self::FilterBar),
            "textinput" | "TextInput" => Some(Self::TextInput),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Global => "Global",
            Self::Table => "Table",
            Self::TableHeaders => "TableHeaders",
            Self::SidebarProjects => "SidebarProjects",
            Self::SidebarTags => "SidebarTags",
            Self::Modal => "Modal",
            Self::FilterBar => "FilterBar",
            Self::TextInput => "TextInput",
        }
    }
}
