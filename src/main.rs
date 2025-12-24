use gpui::{
    prelude::*, App, Application, Context, CursorStyle, KeyDownEvent, SharedString, Window,
    WindowOptions,
};

struct HelloWorld {
    text: SharedString,
    focus_handle: gpui::FocusHandle,
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let input_content = if self.text.is_empty() {
            gpui::div()
                .text_color(gpui::rgb(0x888888))
                .child("Escribe aqui...")
        } else {
            gpui::div().child(self.text.clone())
        };

        gpui::div()
            .size_full()
            .bg(gpui::white())
            .flex()
            .flex_col()
            .gap_3()
            .justify_center()
            .items_center()
            .text_3xl()
            .child(
                gpui::div()
                    .min_w(gpui::rems(12.))
                    .border_1()
                    .border_color(gpui::black())
                    .bg(gpui::white())
                    .p_2()
                    .cursor(CursorStyle::IBeam)
                    .track_focus(&self.focus_handle)
                    .on_key_down(cx.listener(Self::on_key_down))
                    .child(input_content),
            )
            .child(format!("Hello, {}!", &self.text))
    }
}

impl HelloWorld {
    fn on_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut next = self.text.to_string();
        match event.keystroke.key.as_str() {
            "backspace" | "delete" => {
                next.pop();
            }
            _ => {
                let Some(ch) = event.keystroke.key_char.as_ref() else {
                    return;
                };
                if ch == "\n" || ch == "\r" {
                    return;
                }
                next.push_str(ch);
            }
        }

        if next != self.text.as_str() {
            self.text = next.into();
            cx.notify();
        }
    }
}

fn main() {
    let app = Application::new();

    app.run(|app: &mut App| {
        app.open_window(
            WindowOptions::default(),
            |_window: &mut Window, app: &mut App| {
                app.new(|cx: &mut Context<'_, HelloWorld>| HelloWorld {
                    text: SharedString::from("World"),
                    focus_handle: cx.focus_handle(),
                })
            },
        )
        .unwrap();
    });
}
