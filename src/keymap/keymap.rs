use std::collections::HashMap;

use super::{Command, ContextId, KeyChord};

#[derive(Debug, Clone)]
pub struct KeymapLayer {
    bindings: HashMap<ContextId, HashMap<KeyChord, Command>>,
}

impl KeymapLayer {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn bind(&mut self, context: ContextId, chord: KeyChord, command: Command) {
        self.bindings
            .entry(context)
            .or_insert_with(HashMap::new)
            .insert(chord, command);
    }

    pub fn resolve(&self, context: ContextId, chord: &KeyChord) -> Option<Command> {
        self.bindings
            .get(&context)
            .and_then(|map| map.get(chord))
            .copied()
    }
}

impl Default for KeymapLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct KeymapStack {
    layers: Vec<KeymapLayer>,
}

impl KeymapStack {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    pub fn push_layer(&mut self, layer: KeymapLayer) {
        self.layers.push(layer);
    }

    pub fn pop_layer(&mut self) -> Option<KeymapLayer> {
        self.layers.pop()
    }

    pub fn resolve(&self, context: ContextId, chord: &KeyChord) -> Option<Command> {
        for layer in self.layers.iter().rev() {
            if let Some(cmd) = layer.resolve(context, chord) {
                return Some(cmd);
            }
        }

        if context != ContextId::Global {
            for layer in self.layers.iter().rev() {
                if let Some(cmd) = layer.resolve(ContextId::Global, chord) {
                    return Some(cmd);
                }
            }
        }

        None
    }
}

impl Default for KeymapStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keymap::{Key, Mods};

    #[test]
    fn test_layer_bind_and_resolve() {
        let mut layer = KeymapLayer::new();
        let chord = KeyChord::new(Key::Char('j'), Mods::none());

        layer.bind(ContextId::Table, chord, Command::SelectNextRow);

        assert_eq!(
            layer.resolve(ContextId::Table, &chord),
            Some(Command::SelectNextRow)
        );
        assert_eq!(layer.resolve(ContextId::Global, &chord), None);
    }

    #[test]
    fn test_stack_fallback_to_global() {
        let mut stack = KeymapStack::new();
        let mut layer = KeymapLayer::new();

        let chord = KeyChord::new(Key::Char('r'), Mods::platform());
        layer.bind(ContextId::Global, chord, Command::Sync);

        stack.push_layer(layer);

        assert_eq!(stack.resolve(ContextId::Table, &chord), Some(Command::Sync));
    }

    #[test]
    fn test_stack_user_override() {
        let mut stack = KeymapStack::new();

        let mut default_layer = KeymapLayer::new();
        let chord = KeyChord::new(Key::Char('j'), Mods::none());
        default_layer.bind(ContextId::Table, chord, Command::SelectNextRow);
        stack.push_layer(default_layer);

        let mut user_layer = KeymapLayer::new();
        user_layer.bind(ContextId::Table, chord, Command::SelectPrevRow);
        stack.push_layer(user_layer);

        assert_eq!(
            stack.resolve(ContextId::Table, &chord),
            Some(Command::SelectPrevRow)
        );
    }
}
