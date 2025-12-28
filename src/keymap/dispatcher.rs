use super::Command;

pub trait CommandDispatcher: Sized {
    fn dispatch(&mut self, command: Command, cx: &mut gpui::Context<Self>) -> bool;
}
