mod clipboard;
mod prefs;
mod qr;
mod service_icons;
mod ui;

use std::path::PathBuf;

use adw::prelude::*;
use gtk::prelude::*;

use crate::prefs::{apply_theme, Prefs, APP_ID};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .format_timestamp_secs()
        .init();

    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_startup(|_| {
        load_css();
        add_icon_search_paths();
        apply_theme(Prefs::load().theme_mode);
    });
    app.connect_activate(ui::build_ui);
    std::process::exit(app.run().into());
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(include_str!("style.css"));
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn add_icon_search_paths() {
    let Some(display) = gtk::gdk::Display::default() else {
        return;
    };
    let theme = gtk::IconTheme::for_display(&display);
    let mut paths = Vec::new();

    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        paths.push(PathBuf::from(manifest).join("../data/icons"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.join("../share/icons"));
            paths.push(dir.join("share/icons"));
        }
    }
    paths.push(PathBuf::from("/usr/share/icons"));
    paths.push(PathBuf::from("/usr/local/share/icons"));

    for path in paths {
        if path.exists() {
            theme.add_search_path(path);
        }
    }
}
