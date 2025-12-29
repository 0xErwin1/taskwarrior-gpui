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
    let mut logger = env_logger::Builder::from_default_env();
    if cfg!(debug_assertions) {
        logger.filter_level(log::LevelFilter::Debug);
    } else {
        logger.filter_level(log::LevelFilter::Error);
    }
    logger.init();

    log::info!("Starting Task Warrior GPUI");
    App::run();
}
