use super::form::TaskForm;

/// History stack for undo/redo functionality
#[derive(Debug, Clone)]
pub(super) struct FormHistory {
    states: Vec<TaskForm>,
    current_index: usize,
    max_size: usize,
}

impl Default for FormHistory {
    fn default() -> Self {
        Self {
            states: Vec::new(),
            current_index: 0,
            max_size: 50,
        }
    }
}

impl FormHistory {
    pub(super) fn clear(&mut self) {
        self.states.clear();
        self.current_index = 0;
    }

    pub(super) fn push(&mut self, state: TaskForm) {
        if self.current_index < self.states.len() {
            self.states.truncate(self.current_index);
        }

        self.states.push(state);

        if self.states.len() > self.max_size {
            self.states.remove(0);
        } else {
            self.current_index = self.states.len();
        }
    }

    pub(super) fn undo(&mut self) -> Option<&TaskForm> {
        if self.current_index > 1 {
            self.current_index -= 1;
            self.states.get(self.current_index - 1)
        } else {
            None
        }
    }

    pub(super) fn redo(&mut self) -> Option<&TaskForm> {
        if self.current_index < self.states.len() {
            self.current_index += 1;
            self.states.get(self.current_index - 1)
        } else {
            None
        }
    }

    pub(super) fn can_undo(&self) -> bool {
        self.current_index > 1
    }

    pub(super) fn can_redo(&self) -> bool {
        self.current_index < self.states.len()
    }
}
