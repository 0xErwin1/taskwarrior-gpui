use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Char(char),
    Enter,
    Escape,
    Backspace,
    Delete,
    Tab,
    Space,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    PageUp,
    PageDown,
    Home,
    End,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

impl Key {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "enter" | "return" => Some(Self::Enter),
            "esc" | "escape" => Some(Self::Escape),
            "backspace" => Some(Self::Backspace),
            "delete" | "del" => Some(Self::Delete),
            "tab" => Some(Self::Tab),
            "space" => Some(Self::Space),
            "up" | "arrowup" => Some(Self::ArrowUp),
            "down" | "arrowdown" => Some(Self::ArrowDown),
            "left" | "arrowleft" => Some(Self::ArrowLeft),
            "right" | "arrowright" => Some(Self::ArrowRight),
            "pageup" => Some(Self::PageUp),
            "pagedown" => Some(Self::PageDown),
            "home" => Some(Self::Home),
            "end" => Some(Self::End),
            "f1" => Some(Self::F1),
            "f2" => Some(Self::F2),
            "f3" => Some(Self::F3),
            "f4" => Some(Self::F4),
            "f5" => Some(Self::F5),
            "f6" => Some(Self::F6),
            "f7" => Some(Self::F7),
            "f8" => Some(Self::F8),
            "f9" => Some(Self::F9),
            "f10" => Some(Self::F10),
            "f11" => Some(Self::F11),
            "f12" => Some(Self::F12),
            s if s.len() == 1 => s.chars().next().map(Self::Char),
            _ => None,
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Char(c) => write!(f, "{}", c),
            Self::Enter => write!(f, "Enter"),
            Self::Escape => write!(f, "Esc"),
            Self::Backspace => write!(f, "Backspace"),
            Self::Delete => write!(f, "Del"),
            Self::Tab => write!(f, "Tab"),
            Self::Space => write!(f, "Space"),
            Self::ArrowUp => write!(f, "Up"),
            Self::ArrowDown => write!(f, "Down"),
            Self::ArrowLeft => write!(f, "Left"),
            Self::ArrowRight => write!(f, "Right"),
            Self::PageUp => write!(f, "PageUp"),
            Self::PageDown => write!(f, "PageDown"),
            Self::Home => write!(f, "Home"),
            Self::End => write!(f, "End"),
            Self::F1 => write!(f, "F1"),
            Self::F2 => write!(f, "F2"),
            Self::F3 => write!(f, "F3"),
            Self::F4 => write!(f, "F4"),
            Self::F5 => write!(f, "F5"),
            Self::F6 => write!(f, "F6"),
            Self::F7 => write!(f, "F7"),
            Self::F8 => write!(f, "F8"),
            Self::F9 => write!(f, "F9"),
            Self::F10 => write!(f, "F10"),
            Self::F11 => write!(f, "F11"),
            Self::F12 => write!(f, "F12"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Mods {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub platform: bool,
}

impl Mods {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            ..Default::default()
        }
    }

    pub fn alt() -> Self {
        Self {
            alt: true,
            ..Default::default()
        }
    }

    pub fn shift() -> Self {
        Self {
            shift: true,
            ..Default::default()
        }
    }

    pub fn platform() -> Self {
        Self {
            platform: true,
            ..Default::default()
        }
    }
}

impl fmt::Display for Mods {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.platform {
            parts.push("Cmd");
        }
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        write!(f, "{}", parts.join("+"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyChord {
    pub key: Key,
    pub mods: Mods,
}

impl KeyChord {
    pub fn new(key: Key, mods: Mods) -> Self {
        Self { key, mods }
    }

    pub fn from_gpui(event: &gpui::KeyDownEvent) -> Option<Self> {
        let key = Self::key_from_gpui(&event.keystroke.key)?;
        let mods = Mods {
            ctrl: event.keystroke.modifiers.control,
            alt: event.keystroke.modifiers.alt,
            shift: event.keystroke.modifiers.shift,
            platform: event.keystroke.modifiers.platform,
        };
        Some(Self { key, mods })
    }

    fn key_from_gpui(key_str: &str) -> Option<Key> {
        let normalized = key_str.to_lowercase();
        match normalized.as_str() {
            "enter" | "return" => Some(Key::Enter),
            "escape" | "esc" => Some(Key::Escape),
            "backspace" => Some(Key::Backspace),
            "delete" | "del" => Some(Key::Delete),
            "tab" => Some(Key::Tab),
            " " | "space" => Some(Key::Space),
            "arrowup" | "up" => Some(Key::ArrowUp),
            "arrowdown" | "down" => Some(Key::ArrowDown),
            "arrowleft" | "left" => Some(Key::ArrowLeft),
            "arrowright" | "right" => Some(Key::ArrowRight),
            "pageup" => Some(Key::PageUp),
            "pagedown" => Some(Key::PageDown),
            "home" => Some(Key::Home),
            "end" => Some(Key::End),
            "f1" => Some(Key::F1),
            "f2" => Some(Key::F2),
            "f3" => Some(Key::F3),
            "f4" => Some(Key::F4),
            "f5" => Some(Key::F5),
            "f6" => Some(Key::F6),
            "f7" => Some(Key::F7),
            "f8" => Some(Key::F8),
            "f9" => Some(Key::F9),
            "f10" => Some(Key::F10),
            "f11" => Some(Key::F11),
            "f12" => Some(Key::F12),
            s if s.len() == 1 => s.chars().next().map(Key::Char),
            _ => {
                if key_str.len() == 1 {
                    key_str.chars().next().map(Key::Char)
                } else {
                    None
                }
            }
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('+').collect();
        if parts.is_empty() {
            return None;
        }

        let mut mods = Mods::none();
        let key_str = parts.last()?;

        for part in &parts[..parts.len() - 1] {
            match part.to_lowercase().as_str() {
                "ctrl" => mods.ctrl = true,
                "alt" => mods.alt = true,
                "shift" => mods.shift = true,
                "cmd" | "super" | "platform" => mods.platform = true,
                _ => return None,
            }
        }

        let key = Key::from_str(key_str)?;
        Some(Self { key, mods })
    }
}

impl fmt::Display for KeyChord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.mods.ctrl || self.mods.alt || self.mods.shift || self.mods.platform {
            write!(f, "{}+{}", self.mods, self.key)
        } else {
            write!(f, "{}", self.key)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_key() {
        let chord = KeyChord::parse("j").unwrap();
        assert_eq!(chord.key, Key::Char('j'));
        assert_eq!(chord.mods, Mods::none());
    }

    #[test]
    fn test_parse_ctrl_key() {
        let chord = KeyChord::parse("ctrl+f").unwrap();
        assert_eq!(chord.key, Key::Char('f'));
        assert!(chord.mods.ctrl);
    }

    #[test]
    fn test_parse_platform_key() {
        let chord = KeyChord::parse("cmd+r").unwrap();
        assert_eq!(chord.key, Key::Char('r'));
        assert!(chord.mods.platform);
    }

    #[test]
    fn test_parse_special_key() {
        let chord = KeyChord::parse("enter").unwrap();
        assert_eq!(chord.key, Key::Enter);
    }

    #[test]
    fn test_to_string() {
        let chord = KeyChord::new(Key::Char('f'), Mods::ctrl());
        assert_eq!(chord.to_string(), "Ctrl+f");
    }
}
