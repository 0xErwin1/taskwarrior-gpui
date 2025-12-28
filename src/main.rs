use crate::app::App;

mod app;
mod components;
mod keymap;
mod models;
mod task;
mod theme;
mod ui;
mod view;

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    log::info!("Starting Task Warrior GPUI");
    App::run();
}
