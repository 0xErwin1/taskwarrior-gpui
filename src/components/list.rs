use std::sync::Arc;

use gpui::prelude::*;

use crate::theme::ActiveTheme;

#[derive(Clone, Debug)]
pub struct ListItem {
    pub label: gpui::SharedString,
    pub disabled: bool,
}

impl ListItem {
    pub fn new(label: impl Into<gpui::SharedString>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

pub struct List {
    id: gpui::ElementId,
    items: Vec<ListItem>,
    selected_index: Option<usize>,
    height: Option<gpui::Pixels>,
    on_click: Option<Arc<dyn Fn(usize, &ListItem, &mut gpui::Context<Self>) + Send + Sync>>,
    on_hover: Option<Arc<dyn Fn(usize, bool, &ListItem, &mut gpui::Context<Self>) + Send + Sync>>,
}

impl List {
    pub fn new(id: impl Into<gpui::ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            selected_index: None,
            height: None,
            on_click: None,
            on_hover: None,
        }
    }

    pub fn item(mut self, item: ListItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items<I>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = ListItem>,
    {
        self.items = items.into_iter().collect();
        self
    }

    pub fn selected_index(mut self, index: usize) -> Self {
        self.selected_index = Some(index);
        self
    }

    pub fn selected_index_value(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn selected_item(&self) -> Option<&ListItem> {
        self.selected_index.and_then(|index| self.items.get(index))
    }

    pub fn height(mut self, height: gpui::Pixels) -> Self {
        self.height = Some(height);
        self
    }

    pub fn on_click(
        mut self,
        handler: Arc<dyn Fn(usize, &ListItem, &mut gpui::Context<Self>) + Send + Sync>,
    ) -> Self {
        self.on_click = Some(handler);
        self
    }

    pub fn on_hover(
        mut self,
        handler: Arc<dyn Fn(usize, bool, &ListItem, &mut gpui::Context<Self>) + Send + Sync>,
    ) -> Self {
        self.on_hover = Some(handler);
        self
    }

    fn click_item(&mut self, index: usize, cx: &mut gpui::Context<Self>) {
        let item = match self.items.get(index) {
            Some(item) => item,
            None => return,
        };

        if item.disabled {
            return;
        }

        self.selected_index = Some(index);

        if let Some(on_click) = self.on_click.clone() {
            on_click(index, item, cx);
        }

        cx.notify();
    }

    fn hover_item(&mut self, index: usize, hovering: bool, cx: &mut gpui::Context<Self>) {
        let item = match self.items.get(index) {
            Some(item) => item,
            None => return,
        };

        if item.disabled {
            return;
        }

        if let Some(on_hover) = self.on_hover.clone() {
            on_hover(index, hovering, item, cx);
        }
    }
}

impl gpui::Render for List {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();

        let items: Vec<gpui::AnyElement> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let is_selected = self.selected_index == Some(index);

                let mut row = gpui::div()
                    .id(index)
                    .w_full()
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

                if item.disabled {
                    row = row.text_color(theme.muted).cursor_not_allowed();
                } else {
                    row = row
                        .cursor_pointer()
                        .on_mouse_down(
                            gpui::MouseButton::Left,
                            cx.listener(move |this, _event, _window, cx| {
                                this.click_item(index, cx);
                            }),
                        )
                        .on_hover(cx.listener(move |this, hovering, _window, cx| {
                            this.hover_item(index, *hovering, cx);
                        }));
                }

                row.into_any_element()
            })
            .collect();

        let mut container = gpui::div()
            .id(self.id.clone())
            .flex()
            .flex_col()
            .w_full()
            .border_1()
            .border_color(theme.border)
            .bg(theme.panel)
            .rounded_md()
            .overflow_y_scroll()
            .scrollbar_width(gpui::px(6.0))
            .children(items);

        if let Some(height) = self.height {
            container = container.h(height);
        }

        container
    }
}
