#![allow(dead_code)]

mod core;
mod services;
mod ui;

use iced::{application, Font, Size, Task};
use ui::app::App;

fn main() -> iced::Result {
    env_logger::init();

    application("Gosh Authenticator", App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .window_size(Size::new(420.0, 640.0))
        .default_font(Font::DEFAULT)
        .run_with(App::new)
}
