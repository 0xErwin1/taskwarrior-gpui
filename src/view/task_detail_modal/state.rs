use std::collections::HashMap;

use gpui::SharedString;
use uuid::Uuid;

use crate::task::TaskDetailVm;

use super::annotations::AnnotationState;
use super::form::{FieldId, TaskForm};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ModalMode {
    View,
    Edit,
}

impl Default for ModalMode {
    fn default() -> Self {
        Self::View
    }
}

/// State within Edit mode - determines how keyboard input is handled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditState {
    /// Navigating between fields with j/k, not typing
    #[default]
    Navigating,
    /// Actively typing in an input field
    Editing,
    /// A dropdown menu is open
    DropdownOpen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalFocus {
    None,
    Description,
    Project,
    StatusDropdown,
    PriorityDropdown,
    Due,
    TagsInput,
    AnnotationsInput,
}

impl Default for ModalFocus {
    fn default() -> Self {
        Self::StatusDropdown
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InlineEditTarget {
    Tag(usize),
    Annotation(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ConfirmAction {
    DeleteTag { index: usize, text: String },
    DeleteAnnotation { index: usize, text: String },
    DiscardUnsavedChanges,
    DiscardUnsavedChangesAndClose,
}

#[derive(Debug, Clone)]
pub(super) struct TaskModalState {
    pub(super) open: bool,
    pub(super) task_id: Option<Uuid>,
    pub(super) loading: bool,
    pub(super) original: Option<TaskDetailVm>,
    pub(super) form: TaskForm,
    pub(super) errors: HashMap<FieldId, SharedString>,
    pub(super) mode: ModalMode,
    pub(super) is_create: bool,
    pub(super) edit_state: EditState,
    pub(super) annotations: AnnotationState,
    pub(super) error: Option<SharedString>,
    pub(super) modal_focus: ModalFocus,

    /// Which tag is selected for h/l navigation (None = input focused)
    pub(super) tag_selected: Option<usize>,
    /// Which annotation is selected for h/l navigation (None = input focused)
    pub(super) annotation_selected: Option<usize>,
    /// Currently editing an existing item (not creating new)
    pub(super) inline_edit: Option<InlineEditTarget>,
    /// Pending confirmation action
    pub(super) pending_confirm: Option<ConfirmAction>,
}

impl Default for TaskModalState {
    fn default() -> Self {
        Self {
            open: false,
            task_id: None,
            loading: false,
            original: None,
            form: TaskForm::default(),
            errors: HashMap::new(),
            mode: ModalMode::default(),
            is_create: false,
            edit_state: EditState::default(),
            annotations: AnnotationState::default(),
            error: None,
            modal_focus: ModalFocus::default(),
            tag_selected: None,
            annotation_selected: None,
            inline_edit: None,
            pending_confirm: None,
        }
    }
}
