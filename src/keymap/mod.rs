mod active_context;
mod chord;
mod command;
mod context;
pub mod defaults;
mod dispatcher;
mod keymap;

pub use active_context::FocusTarget;
pub use chord::{Key, KeyChord, Mods};
pub use command::Command;
pub use context::ContextId;
pub use dispatcher::CommandDispatcher;
pub use keymap::{KeymapLayer, KeymapStack};

// Legacy compatibility - will be removed after refactor
use gpui::Modifiers;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyBinding {
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    PageUp,
    PageDown,
    Home,
    End,
    Enter,
    Escape,
    Refresh,
    Focus,
}

impl KeyBinding {
    pub fn from_keystroke(key: &str, modifiers: &Modifiers) -> Option<Self> {
        if modifiers.platform {
            return match key {
                "r" | "R" => Some(Self::Refresh),
                "f" | "F" => Some(Self::Focus),
                _ => None,
            };
        }

        if modifiers.control || modifiers.alt || modifiers.shift {
            return None;
        }

        match key {
            "ArrowUp" => Some(Self::ArrowUp),
            "ArrowDown" => Some(Self::ArrowDown),
            "ArrowLeft" => Some(Self::ArrowLeft),
            "ArrowRight" => Some(Self::ArrowRight),
            "PageUp" => Some(Self::PageUp),
            "PageDown" => Some(Self::PageDown),
            "Home" => Some(Self::Home),
            "End" => Some(Self::End),
            "Enter" => Some(Self::Enter),
            "Escape" => Some(Self::Escape),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableAction {
    SelectNext,
    SelectPrevious,
    NextPage,
    PreviousPage,
    SelectFirst,
    SelectLast,
    ClearSelection,
    ExpandProject,
    CollapseProject,
}

impl TableAction {
    pub fn from_binding(binding: &KeyBinding) -> Option<Self> {
        match binding {
            KeyBinding::ArrowUp => Some(Self::SelectPrevious),
            KeyBinding::ArrowDown => Some(Self::SelectNext),
            KeyBinding::PageUp => Some(Self::PreviousPage),
            KeyBinding::PageDown => Some(Self::NextPage),
            KeyBinding::Home => Some(Self::SelectFirst),
            KeyBinding::End => Some(Self::SelectLast),
            KeyBinding::Escape => Some(Self::ClearSelection),
            KeyBinding::ArrowLeft => Some(Self::CollapseProject),
            KeyBinding::ArrowRight => Some(Self::ExpandProject),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalAction {
    Refresh,
    FocusSearch,
}

impl GlobalAction {
    pub fn from_binding(binding: &KeyBinding) -> Option<Self> {
        match binding {
            KeyBinding::Refresh => Some(Self::Refresh),
            KeyBinding::Focus => Some(Self::FocusSearch),
            _ => None,
        }
    }
}
