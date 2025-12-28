use super::ContextId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusTarget {
    App,
    Table,
    TableHeaders,
    SidebarProjects,
    SidebarTags,
}

impl FocusTarget {
    pub fn to_context(&self) -> ContextId {
        match self {
            Self::App => ContextId::Global,
            Self::Table => ContextId::Table,
            Self::TableHeaders => ContextId::TableHeaders,
            Self::SidebarProjects => ContextId::SidebarProjects,
            Self::SidebarTags => ContextId::SidebarTags,
        }
    }

    pub fn is_sidebar(&self) -> bool {
        matches!(self, Self::SidebarProjects | Self::SidebarTags)
    }
}

impl Default for FocusTarget {
    fn default() -> Self {
        Self::App
    }
}
