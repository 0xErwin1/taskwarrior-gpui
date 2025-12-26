use std::sync::Arc;

use gpui::prelude::*;

use crate::components::button::{Button, ButtonVariants};
use crate::theme::ActiveTheme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModalAction {
    Close,
    Cancel,
    Save,
}

#[derive(Clone, Debug, Default)]
pub struct ModalState {
    pub open: bool,
    pub last_action: Option<ModalAction>,
}

pub struct Modal {
    id: gpui::ElementId,
    focus: gpui::FocusHandle,
    title: Option<gpui::SharedString>,
    content: Vec<Arc<dyn Fn() -> gpui::AnyElement + Send + Sync>>,
    footer: Vec<Arc<dyn Fn() -> gpui::AnyElement + Send + Sync>>,
    width: Option<gpui::Pixels>,
    open: bool,
    close_on_backdrop: bool,
    last_action: Option<ModalAction>,
    style: gpui::StyleRefinement,
    on_close: Option<Arc<dyn Fn(ModalAction, &mut gpui::Context<Self>) + Send + Sync>>,
    on_save: Option<Arc<dyn Fn(&mut gpui::Context<Self>) + Send + Sync>>,
    on_cancel: Option<Arc<dyn Fn(&mut gpui::Context<Self>) + Send + Sync>>,
}

impl Modal {
    pub fn new(id: impl Into<gpui::ElementId>, cx: &mut gpui::Context<Self>) -> Self {
        Self {
            id: id.into(),
            focus: cx.focus_handle(),
            title: None,
            content: Vec::new(),
            footer: Vec::new(),
            width: Some(gpui::px(520.0)),
            open: false,
            close_on_backdrop: true,
            last_action: None,
            style: gpui::StyleRefinement::default(),
            on_close: None,
            on_save: None,
            on_cancel: None,
        }
    }

    pub fn title(mut self, title: impl Into<gpui::SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn width(mut self, width: gpui::Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn close_on_backdrop(mut self, close: bool) -> Self {
        self.close_on_backdrop = close;
        self
    }

    pub fn child<E>(mut self, child: E) -> Self
    where
        E: gpui::IntoElement + Clone + Send + Sync + 'static,
    {
        let element = child.clone();
        self.content
            .push(Arc::new(move || element.clone().into_any_element()));
        self
    }

    pub fn children<E>(mut self, children: impl IntoIterator<Item = E>) -> Self
    where
        E: gpui::IntoElement + Clone + Send + Sync + 'static,
    {
        self.content.extend(
            children
                .into_iter()
                .map(|child| {
                    let element = child.clone();
                    Arc::new(move || element.clone().into_any_element())
                        as Arc<dyn Fn() -> gpui::AnyElement + Send + Sync>
                })
                .collect::<Vec<_>>(),
        );
        self
    }

    pub fn footer_child<E>(mut self, child: E) -> Self
    where
        E: gpui::IntoElement + Clone + Send + Sync + 'static,
    {
        let element = child.clone();
        self.footer
            .push(Arc::new(move || element.clone().into_any_element()));
        self
    }

    pub fn footer_children<E>(mut self, children: impl IntoIterator<Item = E>) -> Self
    where
        E: gpui::IntoElement + Clone + Send + Sync + 'static,
    {
        self.footer.extend(
            children
                .into_iter()
                .map(|child| {
                    let element = child.clone();
                    Arc::new(move || element.clone().into_any_element())
                        as Arc<dyn Fn() -> gpui::AnyElement + Send + Sync>
                })
                .collect::<Vec<_>>(),
        );
        self
    }

    pub fn on_close(
        mut self,
        handler: Arc<dyn Fn(ModalAction, &mut gpui::Context<Self>) + Send + Sync>,
    ) -> Self {
        self.on_close = Some(handler);
        self
    }

    pub fn on_save(mut self, handler: Arc<dyn Fn(&mut gpui::Context<Self>) + Send + Sync>) -> Self {
        self.on_save = Some(handler);
        self
    }

    pub fn on_cancel(
        mut self,
        handler: Arc<dyn Fn(&mut gpui::Context<Self>) + Send + Sync>,
    ) -> Self {
        self.on_cancel = Some(handler);
        self
    }

    pub fn set_open(&mut self, open: bool, cx: &mut gpui::Context<Self>) {
        if self.open == open {
            return;
        }
        self.open = open;
        cx.notify();
    }

    pub fn open(&mut self, cx: &mut gpui::Context<Self>) {
        self.set_open(true, cx);
    }

    pub fn close(&mut self, cx: &mut gpui::Context<Self>) {
        self.close_with_action(ModalAction::Close, cx);
    }

    pub fn toggle(&mut self, cx: &mut gpui::Context<Self>) {
        let next = !self.open;
        self.set_open(next, cx);
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn focus_handle(&self) -> &gpui::FocusHandle {
        &self.focus
    }

    pub fn state(&self) -> ModalState {
        ModalState {
            open: self.open,
            last_action: self.last_action,
        }
    }

    pub fn last_action(&self) -> Option<ModalAction> {
        self.last_action
    }

    pub fn take_last_action(&mut self) -> Option<ModalAction> {
        self.last_action.take()
    }

    fn close_with_action(&mut self, action: ModalAction, cx: &mut gpui::Context<Self>) {
        self.last_action = Some(action);

        if let Some(on_close) = self.on_close.clone() {
            on_close(action, cx);
        }

        self.open = false;
        cx.notify();
    }

    fn handle_backdrop_mouse_down(
        &mut self,
        _event: &gpui::MouseDownEvent,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        if self.close_on_backdrop {
            self.close_with_action(ModalAction::Close, cx);
        }
    }

    fn handle_cancel(&mut self, cx: &mut gpui::Context<Self>) {
        if let Some(on_cancel) = self.on_cancel.clone() {
            on_cancel(cx);
        }
        self.close_with_action(ModalAction::Cancel, cx);
    }

    fn handle_save(&mut self, cx: &mut gpui::Context<Self>) {
        if let Some(on_save) = self.on_save.clone() {
            on_save(cx);
        }
        self.close_with_action(ModalAction::Save, cx);
    }
}

impl gpui::Styled for Modal {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl gpui::Render for Modal {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        if !self.open {
            return gpui::Empty.into_any_element();
        }

        let theme = cx.theme();

        let title = self.title.clone().map(|title| {
            gpui::div()
                .px_3()
                .py_2()
                .text_color(theme.foreground)
                .child(title)
                .into_any_element()
        });

        let body: Vec<gpui::AnyElement> = self
            .content
            .iter()
            .enumerate()
            .map(|(ix, build)| gpui::div().id(ix).child(build()).into_any_element())
            .collect();

        let mut footer: Vec<gpui::AnyElement> = self.footer.iter().map(|build| build()).collect();

        if footer.is_empty() {
            let mut actions = Vec::new();
            if self.on_cancel.is_some() {
                actions.push(
                    Button::label((self.id.clone(), "cancel"), "Cancel")
                        .text()
                        .on_click(cx.listener(|this, _e, _w, cx| {
                            this.handle_cancel(cx);
                        }))
                        .into_any_element(),
                );
            }
            if self.on_save.is_some() {
                actions.push(
                    Button::label((self.id.clone(), "save"), "Save")
                        .primary()
                        .on_click(cx.listener(|this, _e, _w, cx| {
                            this.handle_save(cx);
                        }))
                        .into_any_element(),
                );
            }

            if !actions.is_empty() {
                footer = actions;
            }
        }

        let footer = if footer.is_empty() {
            gpui::div().into_any_element()
        } else {
            gpui::div()
                .px_3()
                .py_2()
                .flex()
                .flex_row()
                .justify_end()
                .gap_2()
                .children(footer)
                .into_any_element()
        };

        let mut panel = gpui::div()
            .id(gpui::ElementId::Name(format!("{}-panel", self.id).into()))
            .flex()
            .flex_col()
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border)
            .rounded_md()
            .children(title)
            .child(
                gpui::div()
                    .px_3()
                    .py_2()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(body),
            )
            .child(footer)
            .block_mouse_except_scroll()
            .track_focus(&self.focus)
            .on_key_down(
                cx.listener(|this, event: &gpui::KeyDownEvent, _window, cx| {
                    if event.keystroke.key.as_str() == "escape" {
                        this.close_with_action(ModalAction::Cancel, cx);
                    }
                }),
            );

        if let Some(width) = self.width {
            panel = panel.w(width);
        }

        let panel_wrap = if self.close_on_backdrop {
            gpui::div()
                .child(panel)
                .on_mouse_down_out(cx.listener(|this, event, window, cx| {
                    this.handle_backdrop_mouse_down(event, window, cx);
                }))
        } else {
            gpui::div().child(panel)
        };

        gpui::div()
            .id(self.id.clone())
            .size_full()
            .absolute()
            .top_0()
            .left_0()
            .occlude()
            .child(
                gpui::div()
                    .size_full()
                    .bg(gpui::rgba(0x00000080))
                    .absolute()
                    .top_0()
                    .left_0(),
            )
            .child(
                gpui::div()
                    .size_full()
                    .absolute()
                    .top_0()
                    .left_0()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .child(panel_wrap),
            )
            .into_any_element()
    }
}
