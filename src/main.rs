mod application;
mod core;
mod models;
mod services;
mod widgets;

use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;

use application::GoshApplication;

const APP_ID: &str = "com.github.gosh.authenticator";

fn main() -> glib::ExitCode {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    log::info!("Starting Gosh Authenticator v{}", env!("CARGO_PKG_VERSION"));

    // Load resources
    gio::resources_register_include!("gosh-authenticator.gresource")
        .expect("Failed to register resources");

    // Create and run the application
    let app = GoshApplication::new(APP_ID, gio::ApplicationFlags::empty());
    app.run()
}
