use super::state::ModalFocus;

pub(super) struct FocusMap;

impl FocusMap {
    pub(super) fn section_index(focus: ModalFocus) -> usize {
        match focus {
            ModalFocus::None
            | ModalFocus::StatusDropdown
            | ModalFocus::Description
            | ModalFocus::Project
            | ModalFocus::PriorityDropdown
            | ModalFocus::Due => 0,
            ModalFocus::TagsInput => 1,
            ModalFocus::AnnotationsInput => 3,
        }
    }

    pub(super) fn wants_input_focus(focus: ModalFocus) -> bool {
        matches!(
            focus,
            ModalFocus::Description
                | ModalFocus::Project
                | ModalFocus::Due
                | ModalFocus::TagsInput
                | ModalFocus::AnnotationsInput
        )
    }
}
