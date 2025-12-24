mod suggestion;

use crate::theme::ActiveTheme;
use gpui::prelude::*;
use std::sync::Arc;

pub use suggestion::Suggestion;

pub struct Input {
    id: gpui::ElementId,
    focus: gpui::FocusHandle,
    value: String,
    placeholder: gpui::SharedString,

    cursor_pos: usize,

    suggestions: Vec<Suggestion>,
    suggestions_open: bool,
    active_suggestion: usize,

    suggest: Option<Arc<dyn Fn(&str) -> Vec<Suggestion> + Send + Sync>>,
    on_change: Option<Arc<dyn Fn(&str, &mut gpui::Context<Self>) + Send + Sync>>,
    on_submit: Option<Arc<dyn Fn(&str, &mut gpui::Context<Self>) + Send + Sync>>,
}

impl Input {
    pub fn new(
        id: impl Into<gpui::ElementId>,
        cx: &mut gpui::Context<Self>,
        placeholder: impl Into<gpui::SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            focus: cx.focus_handle(),
            value: String::new(),
            placeholder: placeholder.into(),

            cursor_pos: 0,

            suggestions: vec![],
            suggestions_open: false,
            active_suggestion: 0,

            suggest: None,
            on_change: None,
            on_submit: None,
        }
    }

    pub fn with_suggest(mut self, f: Arc<dyn Fn(&str) -> Vec<Suggestion> + Send + Sync>) -> Self {
        self.suggest = Some(f);
        self
    }

    pub fn with_on_change(
        mut self,
        f: Arc<dyn Fn(&str, &mut gpui::Context<Self>) + Send + Sync>,
    ) -> Self {
        self.on_change = Some(f);
        self
    }

    pub fn with_on_submit(
        mut self,
        f: Arc<dyn Fn(&str, &mut gpui::Context<Self>) + Send + Sync>,
    ) -> Self {
        self.on_submit = Some(f);
        self
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, value: impl Into<String>, cx: &mut gpui::Context<Self>) {
        self.value = value.into();
        self.cursor_pos = self.value.len();
        self.refresh_suggestions(cx);
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut gpui::Context<Self>) {
        self.set_value("", cx);
    }

    pub fn focus(&self, window: &mut gpui::Window, cx: &mut gpui::Context<Self>) {
        window.focus(&self.focus);
        cx.notify();
    }

    fn word_start_before(&self, pos: usize) -> usize {
        if pos == 0 {
            return 0;
        }
        let bytes = self.value.as_bytes();
        let mut i = pos;

        while i > 0 && bytes[i - 1].is_ascii_whitespace() {
            i -= 1;
        }

        while i > 0 && !bytes[i - 1].is_ascii_whitespace() {
            i -= 1;
        }

        i
    }

    fn word_end_after(&self, pos: usize) -> usize {
        let len = self.value.len();
        if pos >= len {
            return len;
        }
        let bytes = self.value.as_bytes();
        let mut i = pos;

        while i < len && !bytes[i].is_ascii_whitespace() {
            i += 1;
        }

        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }

        i
    }

    fn move_left(&mut self) {
        if self.cursor_pos > 0 {
            let mut new_pos = self.cursor_pos - 1;
            while new_pos > 0 && !self.value.is_char_boundary(new_pos) {
                new_pos -= 1;
            }
            self.cursor_pos = new_pos;
        }
    }

    fn move_right(&mut self) {
        if self.cursor_pos < self.value.len() {
            let mut new_pos = self.cursor_pos + 1;
            while new_pos < self.value.len() && !self.value.is_char_boundary(new_pos) {
                new_pos += 1;
            }
            self.cursor_pos = new_pos;
        }
    }

    fn refresh_suggestions(&mut self, cx: &mut gpui::Context<Self>) {
        if let Some(suggest) = &self.suggest {
            self.suggestions = suggest(&self.value);
            self.active_suggestion = 0;
            self.suggestions_open = !self.suggestions.is_empty() && !self.value.is_empty();
            cx.notify();
        }
    }

    fn open_suggestions(&mut self, cx: &mut gpui::Context<Self>) {
        if let Some(suggest) = &self.suggest {
            self.suggestions = suggest(&self.value);
            self.active_suggestion = 0;
            self.suggestions_open = !self.suggestions.is_empty();
            cx.notify();
        }
    }

    fn accept_suggestion(&mut self, cx: &mut gpui::Context<Self>) {
        if !self.suggestions_open {
            return;
        }
        if let Some(s) = self.suggestions.get(self.active_suggestion).cloned() {
            self.value = s.insert.to_string();
            self.cursor_pos = self.value.len();
            self.suggestions_open = false;

            if let Some(on_change) = self.on_change.clone() {
                on_change(&self.value, cx);
            }
            cx.notify();
        }
    }

    fn click_suggestion(&mut self, index: usize, cx: &mut gpui::Context<Self>) {
        if !self.suggestions_open {
            return;
        }
        if index >= self.suggestions.len() {
            return;
        }
        self.active_suggestion = index;
        self.accept_suggestion(cx);
    }

    fn submit(&mut self, cx: &mut gpui::Context<Self>) {
        self.suggestions_open = false;
        if let Some(on_submit) = self.on_submit.clone() {
            on_submit(&self.value, cx);
        }
        cx.notify();
    }

    fn move_suggestion(&mut self, delta: isize, cx: &mut gpui::Context<Self>) {
        if !self.suggestions_open || self.suggestions.is_empty() {
            return;
        }
        let len = self.suggestions.len() as isize;
        let mut next = self.active_suggestion as isize + delta;
        if next < 0 {
            next = len - 1;
        }
        if next >= len {
            next = 0;
        }
        self.active_suggestion = next as usize;
        cx.notify();
    }

    fn insert_text(&mut self, text: &str, cx: &mut gpui::Context<Self>) {
        self.value.insert_str(self.cursor_pos, text);
        self.cursor_pos += text.len();

        if let Some(on_change) = self.on_change.clone() {
            on_change(&self.value, cx);
        }
        self.refresh_suggestions(cx);
        cx.notify();
    }

    fn delete_backward(&mut self, cx: &mut gpui::Context<Self>) {
        if self.cursor_pos == 0 {
            return;
        }

        let old_pos = self.cursor_pos;
        self.move_left();
        self.value.drain(self.cursor_pos..old_pos);

        if let Some(on_change) = self.on_change.clone() {
            on_change(&self.value, cx);
        }
        self.refresh_suggestions(cx);
        cx.notify();
    }

    fn delete_forward(&mut self, cx: &mut gpui::Context<Self>) {
        if self.cursor_pos >= self.value.len() {
            return;
        }

        let mut end = self.cursor_pos + 1;
        while end < self.value.len() && !self.value.is_char_boundary(end) {
            end += 1;
        }
        self.value.drain(self.cursor_pos..end);

        if let Some(on_change) = self.on_change.clone() {
            on_change(&self.value, cx);
        }
        self.refresh_suggestions(cx);
        cx.notify();
    }

    fn delete_word_backward(&mut self, cx: &mut gpui::Context<Self>) {
        if self.cursor_pos == 0 {
            return;
        }

        let word_start = self.word_start_before(self.cursor_pos);
        self.value.drain(word_start..self.cursor_pos);
        self.cursor_pos = word_start;

        if let Some(on_change) = self.on_change.clone() {
            on_change(&self.value, cx);
        }
        self.refresh_suggestions(cx);
        cx.notify();
    }

    fn delete_word_forward(&mut self, cx: &mut gpui::Context<Self>) {
        if self.cursor_pos >= self.value.len() {
            return;
        }

        let word_end = self.word_end_after(self.cursor_pos);
        self.value.drain(self.cursor_pos..word_end);

        if let Some(on_change) = self.on_change.clone() {
            on_change(&self.value, cx);
        }
        self.refresh_suggestions(cx);
        cx.notify();
    }

    fn handle_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        let ctrl = event.keystroke.modifiers.control;
        let shift = event.keystroke.modifiers.shift;

        match key {
            "enter" => {
                if self.suggestions_open {
                    self.accept_suggestion(cx);
                } else {
                    self.submit(cx);
                }
            }

            "escape" => {
                self.suggestions_open = false;
                cx.notify();
            }

            "tab" => {
                if shift {
                    if self.suggestions_open {
                        self.move_suggestion(-1, cx);
                    }
                } else {
                    if self.suggestions_open {
                        self.move_suggestion(1, cx);
                    } else {
                        self.open_suggestions(cx);
                    }
                }
            }

            "up" => self.move_suggestion(-1, cx),
            "down" => self.move_suggestion(1, cx),

            "left" => {
                if ctrl {
                    self.cursor_pos = self.word_start_before(self.cursor_pos);
                    cx.notify();
                } else {
                    self.move_left();
                    cx.notify();
                }
            }

            "right" => {
                if ctrl {
                    self.cursor_pos = self.word_end_after(self.cursor_pos);
                    cx.notify();
                } else {
                    self.move_right();
                    cx.notify();
                }
            }

            "home" => {
                self.cursor_pos = 0;
                cx.notify();
            }

            "end" => {
                self.cursor_pos = self.value.len();
                cx.notify();
            }

            "backspace" => {
                if ctrl {
                    self.delete_word_backward(cx);
                } else {
                    self.delete_backward(cx);
                }
            }

            "delete" => {
                if ctrl {
                    self.delete_word_forward(cx);
                } else {
                    self.delete_forward(cx);
                }
            }

            "w" if ctrl => {
                self.delete_word_backward(cx);
            }

            "a" if ctrl => {
                self.cursor_pos = 0;
                cx.notify();
            }

            "e" if ctrl => {
                self.cursor_pos = self.value.len();
                cx.notify();
            }

            "u" if ctrl => {
                self.value.drain(0..self.cursor_pos);
                self.cursor_pos = 0;
                if let Some(on_change) = self.on_change.clone() {
                    on_change(&self.value, cx);
                }
                self.refresh_suggestions(cx);
                cx.notify();
            }

            "k" if ctrl => {
                self.value.truncate(self.cursor_pos);
                if let Some(on_change) = self.on_change.clone() {
                    on_change(&self.value, cx);
                }
                self.refresh_suggestions(cx);
                cx.notify();
            }

            _ => {
                if let Some(ch) = event.keystroke.key_char.as_ref() {
                    if !ctrl && ch != "\n" && ch != "\r" && ch != "\t" {
                        self.insert_text(ch, cx);
                    }
                }
            }
        }
    }

    fn render_suggestions(&self, cx: &gpui::Context<Self>) -> impl IntoElement {
        if !self.suggestions_open {
            return gpui::div().into_any_element();
        }

        let theme = cx.theme();
        let items: Vec<gpui::AnyElement> = self
            .suggestions
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let is_active = i == self.active_suggestion;
                gpui::div()
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, _e, _w, cx| {
                            this.click_suggestion(i, cx);
                        }),
                    )
                    .cursor_pointer()
                    .px_2()
                    .py_1()
                    .when(is_active, |el| el.bg(theme.selection))
                    .text_color(if is_active {
                        theme.selection_foreground
                    } else {
                        theme.foreground
                    })
                    .child(s.label.clone())
                    .into_any_element()
            })
            .collect();

        gpui::div()
            .absolute()
            .top_full()
            .left_0()
            .right_0()
            .mt_1()
            .border_1()
            .border_color(theme.border)
            .bg(theme.panel)
            .rounded_md()
            .overflow_hidden()
            .children(items)
            .into_any_element()
    }
}

impl gpui::Render for Input {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let is_focused = self.focus.is_focused(window);

        let show_placeholder = self.value.is_empty();

        let content = if show_placeholder {
            let cursor = if is_focused {
                gpui::div().w_px().h_4().bg(theme.accent).into_any_element()
            } else {
                gpui::div().into_any_element()
            };

            gpui::div()
                .id(self.id.clone())
                .flex()
                .flex_row()
                .items_center()
                .child(cursor)
                .child(
                    gpui::div()
                        .text_color(theme.muted)
                        .child(self.placeholder.clone()),
                )
                .into_any_element()
        } else {
            let before = &self.value[..self.cursor_pos];
            let after = &self.value[self.cursor_pos..];

            let cursor = if is_focused {
                gpui::div()
                    .id(self.id.clone())
                    .w_px()
                    .h_4()
                    .bg(theme.accent)
                    .into_any_element()
            } else {
                gpui::div().id(self.id.clone()).into_any_element()
            };

            gpui::div()
                .id(self.id.clone())
                .flex()
                .flex_row()
                .items_center()
                .child(
                    gpui::div()
                        .text_color(theme.foreground)
                        .child(before.to_string()),
                )
                .child(cursor)
                .child(
                    gpui::div()
                        .text_color(theme.foreground)
                        .child(after.to_string()),
                )
                .into_any_element()
        };

        let focus_handle = self.focus.clone();

        gpui::div()
            .id(self.id.clone())
            .key_context("Input")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::handle_key_down))
            .on_mouse_down(gpui::MouseButton::Left, move |_ev, window, _cx| {
                window.focus(&focus_handle);
            })
            .relative()
            .min_w(gpui::rems(12.))
            .border_1()
            .border_color(if is_focused {
                theme.accent
            } else {
                theme.border
            })
            .bg(theme.background)
            .rounded_md()
            .p_2()
            .cursor(gpui::CursorStyle::IBeam)
            .child(content)
            .child(self.render_suggestions(cx))
    }
}

impl gpui::Focusable for Input {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus.clone()
    }
}
