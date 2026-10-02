use crate::settings::{Settings, SettingsStore, ThemeMode, APP_NAME};
use dioxus_desktop::{
    muda::{
        accelerator::{Accelerator, Code, Modifiers},
        Menu, MenuItem, PredefinedMenuItem, Submenu,
    },
    Config, LogicalSize, WindowBuilder,
};
pub fn command_modifier() -> Modifiers {
    if cfg!(target_os = "macos") {
        Modifiers::META
    } else {
        Modifiers::CONTROL
    }
}
fn native_theme(mode: ThemeMode) -> Option<dioxus_desktop::tao::window::Theme> {
    use dioxus_desktop::tao::window::Theme;
    match mode {
        ThemeMode::System => None,
        ThemeMode::Light => Some(Theme::Light),
        ThemeMode::Dark => Some(Theme::Dark),
    }
}
pub fn apply_theme(mode: ThemeMode) {
    dioxus_desktop::window()
        .window
        .set_theme(native_theme(mode));
}
pub fn menu() -> Result<Menu, dioxus_desktop::muda::Error> {
    let menu = Menu::new();
    let app = Submenu::new(
        if cfg!(target_os = "macos") {
            APP_NAME
        } else {
            "Application"
        },
        true,
    );
    let accel = |code| Some(Accelerator::new(Some(command_modifier()), code));
    app.append_items(&[
        &MenuItem::with_id("about", "About Gosh Yubico Authenticator", true, None),
        &MenuItem::with_id("settings", "Settings…", true, accel(Code::Comma)),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id("quit", "Quit", true, accel(Code::KeyQ)),
    ])?;
    let credentials = Submenu::new("Credentials", true);
    credentials.append_items(&[
        &MenuItem::with_id("search", "Search credentials", true, accel(Code::KeyF)),
        &MenuItem::with_id("new", "Add credential…", true, accel(Code::KeyN)),
        &MenuItem::with_id("import", "Import QR image…", true, accel(Code::KeyI)),
        &MenuItem::with_id("refresh", "Refresh", true, accel(Code::KeyR)),
        &MenuItem::with_id("lock", "Lock / disconnect", true, accel(Code::KeyL)),
    ])?;
    menu.append_items(&[&app, &credentials])?;
    #[cfg(target_os = "macos")]
    {
        let edit = Submenu::new("Edit", true);
        edit.append_items(&[
            &PredefinedMenuItem::undo(None),
            &PredefinedMenuItem::redo(None),
            &PredefinedMenuItem::cut(None),
            &PredefinedMenuItem::copy(None),
            &PredefinedMenuItem::paste(None),
            &PredefinedMenuItem::select_all(None),
        ])?;
        menu.append(&edit)?;
    }
    Ok(menu)
}
pub fn config(settings: &Settings, store: &SettingsStore) -> Result<Config, String> {
    let browser_data = store
        .path
        .parent()
        .ok_or("Invalid settings path")?
        .join("webview");
    let window = WindowBuilder::new()
        .with_title(APP_NAME)
        .with_theme(native_theme(settings.theme_mode))
        .with_inner_size(LogicalSize::new(
            settings.window.width,
            settings.window.height,
        ))
        .with_min_inner_size(LogicalSize::new(360., 400.))
        .with_maximized(settings.window.maximized)
        .with_always_on_top(false);
    let mut config = Config::new()
        .with_close_behaviour(dioxus_desktop::WindowCloseBehaviour::WindowHides)
        .with_window(window)
        .with_menu(menu().map_err(|e| e.to_string())?)
        .with_data_directory(browser_data)
        .with_disable_context_menu(true);
    #[cfg(target_os = "linux")]
    {
        use dioxus_desktop::tao::{
            event_loop::EventLoopBuilder, platform::unix::EventLoopBuilderExtUnix,
        };
        let mut event_loop = EventLoopBuilder::with_user_event();
        event_loop.with_app_id(crate::settings::APP_ID);
        config = config.with_event_loop(event_loop.build());
    }
    let rgba = image::load_from_memory(include_bytes!(
        "../../../data/icons/hicolor/128x128/apps/com.goshapps.YubicoAuthenticator.png"
    ))
    .map_err(|e| e.to_string())?
    .to_rgba8();
    let (width, height) = rgba.dimensions();
    let icon = dioxus_desktop::tao::window::Icon::from_rgba(rgba.into_raw(), width, height)
        .map_err(|e| e.to_string())?;
    config = config.with_icon(icon);
    Ok(config.with_custom_index(format!("<!DOCTYPE html><html><head><title>{APP_NAME}</title><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><meta name=\"color-scheme\" content=\"light dark\"></head><body><div id=\"main\"></div></body></html>")))
}
pub async fn pick_qr_image() -> Option<std::path::PathBuf> {
    let desktop = dioxus_desktop::window();
    rfd::AsyncFileDialog::new()
        .set_parent(desktop.window.as_ref())
        .set_title("Import an OTP QR image")
        .add_filter("QR images", &["png", "jpg", "jpeg", "webp"])
        .pick_file()
        .await
        .map(|f| f.path().to_path_buf())
}
pub fn open_url(url: &str) -> Result<(), String> {
    let parsed = url::Url::parse(url).map_err(|_| "Invalid website URL")?;
    if parsed.scheme() != "https" {
        return Err("Only HTTPS links are allowed".into());
    }
    open::that(parsed.as_str()).map_err(|e| e.to_string())
}

/// Opens a user-selected local file with the OS default application.
pub fn open_file(path: &std::path::Path) -> Result<(), String> {
    let path = path.canonicalize().map_err(|e| e.to_string())?;
    open::that(path).map_err(|e| e.to_string())
}
/// Opens the containing directory in Explorer, Finder, or the desktop file manager.
pub fn show_in_folder(path: &std::path::Path) -> Result<(), String> {
    let directory = path.parent().ok_or("Path has no parent folder")?;
    std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    open_file(directory)
}

async fn native_action(
    action: impl FnOnce() -> Result<(), String> + Send + 'static,
) -> Result<(), String> {
    let (sender, receiver) = async_channel::bounded(1);
    std::thread::Builder::new()
        .name("gosh-os-action".into())
        .spawn(move || {
            let _ = sender.send_blocking(action());
        })
        .map_err(|e| e.to_string())?;
    receiver
        .recv()
        .await
        .map_err(|_| "Native integration service stopped".to_string())?
}
pub async fn open_url_async(url: String) -> Result<(), String> {
    native_action(move || open_url(&url)).await
}
pub async fn show_in_folder_async(path: std::path::PathBuf) -> Result<(), String> {
    native_action(move || show_in_folder(&path)).await
}
/// Exports only the trusted, compiled notice document to the OS cache directory.
pub async fn open_third_party_licenses_async() -> Result<(), String> {
    native_action(|| {
        use std::io::Write;
        let directory = crate::settings::cache_dir()
            .map_err(|e| e.to_string())?
            .join("licenses");
        std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
        let path = directory.join("THIRD_PARTY_LICENSES.html");
        let mut temporary =
            tempfile::NamedTempFile::new_in(&directory).map_err(|e| e.to_string())?;
        temporary
            .write_all(include_bytes!("../../../THIRD_PARTY_LICENSES.html"))
            .map_err(|e| e.to_string())?;
        temporary.persist(&path).map_err(|e| e.to_string())?;
        open_file(&path)
    })
    .await
}
