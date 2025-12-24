use std::sync::Arc;

use gpui::prelude::*;

use crate::components::button::Button;
use crate::theme::ActiveTheme;

#[derive(Clone, Debug)]
pub struct DropdownItem {
    pub label: gpui::SharedString,
}

impl DropdownItem {
    pub fn new(label: impl Into<gpui::SharedString>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

pub struct Dropdown {
    id: gpui::ElementId,
    button: Option<Button>,
    items: Vec<DropdownItem>,
    open: bool,
    selected_index: Option<usize>,
    disabled: bool,
    loading: bool,
    placeholder: gpui::SharedString,
    on_select: Option<Arc<dyn Fn(usize, &DropdownItem, &mut gpui::Context<Self>) + Send + Sync>>,
}

impl Dropdown {
    pub fn new(id: impl Into<gpui::ElementId>) -> Self {
        Self {
            id: id.into(),
            button: None,
            items: Vec::new(),
            open: false,
            selected_index: None,
            disabled: false,
            loading: false,
            placeholder: "Seleccionar".into(),
            on_select: None,
        }
    }

    pub fn button(mut self, button: Button) -> Self {
        self.button = Some(button);
        self
    }

    pub fn item(mut self, item: DropdownItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items<I>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = DropdownItem>,
    {
        self.items = items.into_iter().collect();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<gpui::SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn selected_index(mut self, index: usize) -> Self {
        self.selected_index = Some(index);
        self
    }

    pub fn selected_index_value(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn selected_item(&self) -> Option<&DropdownItem> {
        self.selected_index.and_then(|index| self.items.get(index))
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn on_select(
        mut self,
        handler: Arc<dyn Fn(usize, &DropdownItem, &mut gpui::Context<Self>) + Send + Sync>,
    ) -> Self {
        self.on_select = Some(handler);
        self
    }

    pub fn selected_label(&self) -> Option<gpui::SharedString> {
        self.selected_index
            .and_then(|index| self.items.get(index).map(|item| item.label.clone()))
    }

    fn toggle_open(&mut self, cx: &mut gpui::Context<Self>) {
        if self.disabled || self.loading || self.items.is_empty() {
            return;
        }
        self.open = !self.open;
        cx.notify();
    }

    fn select_item(&mut self, index: usize, cx: &mut gpui::Context<Self>) {
        if self.disabled || self.loading {
            return;
        }
        let item = match self.items.get(index) {
            Some(item) => item,
            None => return,
        };
        self.selected_index = Some(index);
        self.open = false;
        if let Some(on_select) = self.on_select.clone() {
            on_select(index, item, cx);
        }
        cx.notify();
    }

    fn handle_trigger_mouse_down(
        &mut self,
        _event: &gpui::MouseDownEvent,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        self.toggle_open(cx);
    }

    fn handle_mouse_down_out(
        &mut self,
        _event: &gpui::MouseDownEvent,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        if self.open {
            self.open = false;
            cx.notify();
        }
    }

    fn render_menu(&self, cx: &gpui::Context<Self>) -> gpui::AnyElement {
        if !self.open || self.items.is_empty() {
            return gpui::div().into_any_element();
        }

        let theme = cx.theme();
        let is_disabled = self.disabled || self.loading;
        let items: Vec<gpui::AnyElement> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let is_selected = self.selected_index == Some(index);
                let mut row = gpui::div()
                    .id(index)
                    .px_2()
                    .py_1()
                    .text_color(if is_selected {
                        theme.selection_foreground
                    } else {
                        theme.foreground
                    })
                    .when(is_selected, |el| el.bg(theme.selection))
                    .hover(|s: gpui::StyleRefinement| s.bg(theme.selection))
                    .child(item.label.clone());

                if is_disabled {
                    row = row.text_color(theme.muted).cursor_not_allowed();
                } else {
                    row = row.cursor_pointer().on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(move |this, _event, _window, cx| {
                            this.select_item(index, cx);
                        }),
                    );
                }

                row.into_any_element()
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

impl gpui::Render for Dropdown {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let is_disabled = self.disabled || self.loading;
        let label = self
            .selected_label()
            .unwrap_or_else(|| self.placeholder.clone());

        let mut trigger = if let Some(button) = self.button.clone() {
            let mut button = button;
            button.on_click = None;
            button.label = Some(label.clone());
            button.disabled(is_disabled).loading(self.loading)
        } else {
            Button::label(self.id.clone(), label)
                .disabled(is_disabled)
                .loading(self.loading)
        };

        if is_disabled || self.items.is_empty() {
            trigger = trigger.disabled(true);
        }

        let mut trigger_wrap = gpui::div().child(trigger);
        if !is_disabled && !self.items.is_empty() {
            trigger_wrap = trigger_wrap.on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(Self::handle_trigger_mouse_down),
            );
        }

        let mut container = gpui::div()
            .id(self.id.clone())
            .relative()
            .flex()
            .flex_col()
            .child(trigger_wrap)
            .child(self.render_menu(cx));

        if self.open {
            container = container.on_mouse_down_out(cx.listener(Self::handle_mouse_down_out));
        }

        container
    }
}
