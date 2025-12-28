use gpui::{Context, IntoElement, MouseButton, Render, Window, div, prelude::*, rems};

use crate::components::label::Label;
use crate::theme::{ActiveTheme, Theme};
use crate::ui::divider_v;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncState {
    Idle,
    Syncing,
    Success,
    Error,
}

impl Default for SyncState {
    fn default() -> Self {
        Self::Idle
    }
}

pub struct StatusBar {
    sync_state: SyncState,
    last_sync_message: String,
    error_message: Option<String>,
}

impl StatusBar {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            sync_state: SyncState::default(),
            last_sync_message: String::new(),
            error_message: None,
        }
    }

    pub fn set_sync_state(&mut self, state: SyncState, cx: &mut Context<Self>) {
        self.sync_state = state;
        cx.notify();
    }

    pub fn set_last_sync_message(&mut self, message: String, cx: &mut Context<Self>) {
        self.last_sync_message = message;
        cx.notify();
    }

    pub fn clear_message(&mut self, cx: &mut Context<Self>) {
        self.last_sync_message.clear();
        self.sync_state = SyncState::Idle;
        cx.notify();
    }

    pub fn set_error(&mut self, message: String, cx: &mut Context<Self>) {
        self.error_message = Some(message);
        cx.notify();
    }

    pub fn clear_error(&mut self, cx: &mut Context<Self>) {
        self.error_message = None;
        cx.notify();
    }

    fn sync_icon(&self) -> &'static str {
        match self.sync_state {
            SyncState::Idle => "↻",
            SyncState::Syncing => "◌",
            SyncState::Success => "✓",
            SyncState::Error => "✕",
        }
    }
}

impl Render for StatusBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let is_syncing = self.sync_state == SyncState::Syncing;

        let sync_color = match self.sync_state {
            SyncState::Success => theme.success,
            SyncState::Error => theme.error,
            SyncState::Syncing => theme.info,
            SyncState::Idle => theme.muted,
        };

        let sync_button = div()
            .flex()
            .items_center()
            .gap_1()
            .px_2()
            .py_1()
            .rounded_md()
            .text_sm()
            .text_color(sync_color)
            .when(!is_syncing, |d| {
                d.cursor_pointer()
                    .hover(|s| s.bg(theme.hover))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|_this, _event, _window, cx| {
                            cx.emit(StatusBarEvent::SyncRequested);
                        }),
                    )
            })
            .when(is_syncing, |d| d.cursor_not_allowed())
            .child(Label::new(self.sync_icon()).text_color(sync_color))
            .child(Label::new("Sync").text_color(sync_color));

        let status_text = if !self.last_sync_message.is_empty() {
            Label::new(self.last_sync_message.clone()).text_color(theme.muted)
        } else {
            Label::new("")
        };

        let error_banner = if let Some(ref error) = self.error_message {
            Some(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .px_3()
                    .py_2()
                    .bg(Theme::alpha(theme.error, 0.12))
                    .border_1()
                    .border_color(theme.error)
                    .rounded_md()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Label::new("✕").text_color(theme.error).text_sm())
                            .child(Label::new(error.clone()).text_color(theme.error).text_sm()),
                    )
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .rounded_sm()
                            .cursor_pointer()
                            .hover(|s| s.bg(Theme::alpha(theme.error, 0.2)))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _event, _window, cx| {
                                    this.clear_error(cx);
                                }),
                            )
                            .child(Label::new("✕").text_color(theme.error).text_sm()),
                    ),
            )
        } else {
            None
        };

        let status_bar_content = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .h(rems(2.0))
            .px_3()
            .py_1()
            .bg(theme.panel)
            .border_t_1()
            .border_color(theme.divider)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .text_xs()
                    .child(status_text),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(divider_v(&theme).h(rems(1.0)))
                    .child(sync_button),
            );

        if let Some(error) = error_banner {
            div()
                .flex()
                .flex_col()
                .w_full()
                .gap_2()
                .p_2()
                .child(error)
                .child(status_bar_content)
        } else {
            status_bar_content
        }
    }
}

pub enum StatusBarEvent {
    SyncRequested,
}

impl gpui::EventEmitter<StatusBarEvent> for StatusBar {}
