#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod ui;
use gosh_authenticator_core::{
    services::Runtime,
    settings::{Settings, SettingsStore, APP_NAME},
};
#[derive(Clone)]
struct Startup {
    settings: Settings,
    runtime: Runtime,
    error: Option<String>,
    settings_path: std::path::PathBuf,
}
fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let mut args = std::env::args().skip(1);
    let mut settings_path = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--version" => {
                println!("{APP_NAME} {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--help" | "-h" => {
                println!("{APP_NAME}\nUsage: gosh-authenticator [--settings PATH] [--version] [--help]\nSecrets remain on the YubiKey; --settings selects a diagnostic configuration file.");
                return;
            }
            "--settings" => match args.next() {
                Some(path) => settings_path = Some(path.into()),
                None => {
                    eprintln!("--settings requires a file path");
                    std::process::exit(2);
                }
            },
            _ => {
                eprintln!("Unknown option: {arg}");
                std::process::exit(2);
            }
        }
    }
    let store = match settings_path {
        Some(path) => SettingsStore { path },
        None => match SettingsStore::standard() {
            Ok(store) => store,
            Err(e) => {
                eprintln!("Cannot start: {e}");
                std::process::exit(1);
            }
        },
    };
    let (settings, error) = match store.load() {
        Ok(settings) => (settings, None),
        Err(e) => (
            Settings::default(),
            Some(format!(
                "{e}. The original file is preserved; preferences cannot overwrite it."
            )),
        ),
    };
    let config = match gosh_authenticator_core::platform::config(&settings, &store) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Cannot create desktop window: {e}");
            std::process::exit(1);
        }
    };
    let settings_path = store.path.clone();
    let runtime = Runtime::start(store);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .with_context(Startup {
            settings,
            runtime,
            error,
            settings_path,
        })
        .launch(ui::App);
}
