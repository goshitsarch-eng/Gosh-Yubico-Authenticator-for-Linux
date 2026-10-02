mod credentials;
mod dialogs;
mod forms;
mod preferences;
use dioxus::prelude::*;
use gosh_authenticator_core::{
    app::{unix_time, Command, Connection, Event, Outcome, State},
    core::credential::CredentialId,
    services::Runtime,
    settings::{SettingsUpdate, WindowState},
};
#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    Credentials,
    KeyInfo,
    Settings,
    Add,
}
#[derive(Clone, PartialEq)]
pub enum Dialog {
    None,
    About,
    Licenses,
    Unlock,
    Password,
    Delete(CredentialId),
    Icon(CredentialId),
    Touch,
    Actions(CredentialId),
}
impl Dialog {
    fn is_invalidated_by(&self, event: &Event, state: &State) -> bool {
        if matches!(self, Self::None | Self::About | Self::Licenses) {
            return false;
        }
        match event {
            Event::Disconnected => true,
            Event::Connected(info, _) => {
                state.device.as_ref().map(|device| device.id) != Some(info.id)
            }
            _ => false,
        }
    }
}
#[derive(Clone, Copy)]
pub struct Ui {
    pub state: Signal<State>,
    pub page: Signal<Page>,
    pub dialog: Signal<Dialog>,
    pub dialog_epoch: Signal<u64>,
    pub form: Signal<forms::AddForm>,
    pub now: Signal<u64>,
    pub runtime: Signal<Runtime>,
    pub quitting: Signal<bool>,
    pub focus_search: Signal<bool>,
}
impl Ui {
    pub fn send(&self, command: Command) {
        if let Err(e) = self.runtime.read().send(command) {
            self.error(e);
        }
    }
    pub fn error(&self, text: String) {
        let mut state = self.state;
        state.write().message = Some(text);
        state.write().error = true;
    }
    pub fn open_url(&self, url: &str) {
        let ui = *self;
        let url = url.to_string();
        spawn(async move {
            if let Err(error) = gosh_authenticator_core::platform::open_url_async(url).await {
                ui.error(error);
            }
        });
    }
    pub fn quit(&self) {
        if (self.quitting)() {
            return;
        }
        let mut quitting = self.quitting;
        quitting.set(true);
        let runtime = self.runtime.read().clone();
        let desktop = dioxus_desktop::window();
        spawn(async move {
            runtime.shutdown().await;
            desktop.set_close_behavior(dioxus_desktop::WindowCloseBehaviour::WindowCloses);
            desktop.close();
        });
    }
    pub fn action(&self, id: &str) {
        let mut page = self.page;
        let mut dialog = self.dialog;
        match id {
            "new" => {
                page.set(Page::Add);
            }
            "import" => {
                page.set(Page::Add);
                let ui = *self;
                spawn(async move {
                    if let Some(path) = gosh_authenticator_core::platform::pick_qr_image().await {
                        ui.send(Command::Import(path));
                    }
                });
            }
            "refresh" => self.send(Command::Refresh),
            "connect" => self.send(Command::Connect),
            "lock" => self.send(Command::Disconnect),
            "settings" => page.set(Page::Settings),
            "about" => dialog.set(Dialog::About),
            "unlock" => dialog.set(Dialog::Unlock),
            "quit" => self.quit(),
            "search" => {
                let mut focus_search = self.focus_search;
                focus_search.set(true);
                page.set(Page::Credentials);
            }
            _ => {}
        }
    }
}
#[component]
pub fn App() -> Element {
    let startup = use_context::<crate::Startup>();
    let mut state = use_signal(|| {
        let mut s = State::new(startup.settings.clone());
        if let Some(error) = startup.error.clone() {
            s.message = Some(error);
            s.error = true;
        }
        s
    });
    let mut page = use_signal(|| Page::Credentials);
    let mut dialog = use_signal(|| Dialog::None);
    let mut dialog_epoch = use_signal(|| 0_u64);
    let mut form = use_signal(forms::AddForm::default);
    let mut now = use_signal(unix_time);
    let mut pending_window = use_signal(|| None::<WindowState>);
    let ui = Ui {
        state,
        page,
        dialog,
        dialog_epoch,
        form,
        now,
        runtime: use_signal(|| startup.runtime.clone()),
        quitting: use_signal(|| false),
        focus_search: use_signal(|| false),
    };
    use_context_provider(|| ui);
    let theme_mode = use_memo(move || state.read().settings.theme_mode);
    use_effect(move || gosh_authenticator_core::platform::apply_theme(*theme_mode.read()));
    let events = startup.runtime.events.clone();
    use_future(move || {
        let events = events.clone();
        async move {
            while let Ok(event) = events.recv().await {
                if dialog.read().is_invalidated_by(&event, &state.read()) {
                    dialog.set(Dialog::None);
                    // Force a fresh component if the next event immediately
                    // opens Unlock again before Dioxus renders the closed state.
                    dialog_epoch += 1;
                }
                match event {
                    Event::Imported(new) => {
                        form.set(forms::AddForm::from_credential(&new));
                        page.set(Page::Add);
                    }
                    Event::Touch(_) => dialog.set(Dialog::Touch),
                    Event::Connected(info, locked) => {
                        let prompt = state.read().settings.require_pin_on_launch;
                        state.write().apply(Event::Connected(info, locked));
                        if locked && prompt {
                            dialog.set(Dialog::Unlock);
                        }
                    }
                    Event::Success(msg) => {
                        if msg == Outcome::Added {
                            form.set(forms::AddForm::default());
                            page.set(Page::Credentials);
                        }
                        if matches!(
                            msg,
                            Outcome::Unlocked
                                | Outcome::PasswordChanged
                                | Outcome::PasswordRemoved
                                | Outcome::Deleted
                        ) {
                            dialog.set(Dialog::None);
                        }
                        state.write().apply(Event::Success(msg));
                    }
                    Event::Credentials(c) => {
                        state.write().apply(Event::Credentials(c));
                        if dialog() == Dialog::Touch {
                            dialog.set(Dialog::None);
                        }
                    }
                    Event::Cancelled => {
                        dialog.set(Dialog::None);
                        state.write().apply(Event::Cancelled);
                    }
                    Event::Error(error) => {
                        if dialog() == Dialog::Touch {
                            dialog.set(Dialog::None);
                        }
                        state.write().apply(Event::Error(error));
                    }
                    other => state.write().apply(other),
                }
            }
        }
    });
    let timer_ui = ui;
    use_future(move || {
        let ui = timer_ui;
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                now.set(unix_time());
                if let Some(window) = pending_window.write().take() {
                    ui.send(Command::UpdateSettings(SettingsUpdate::Window(window)));
                }
            }
        }
    });
    let native_ui = ui;
    dioxus_desktop::use_muda_event_handler(move |event| native_ui.action(event.id.as_ref()));
    let desktop = dioxus_desktop::use_window();
    let window_ui = ui;
    dioxus_desktop::use_wry_event_handler(move |event, _| {
        use dioxus_desktop::tao::event::{Event as TaoEvent, WindowEvent};
        if let TaoEvent::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::Resized(size) => {
                    let logical = size.to_logical::<f64>(desktop.window.scale_factor());
                    if logical.width >= 360. && logical.height >= 400. {
                        pending_window.set(Some(WindowState {
                            width: if desktop.window.is_maximized() {
                                window_ui.state.read().settings.window.width
                            } else {
                                logical.width
                            },
                            height: if desktop.window.is_maximized() {
                                window_ui.state.read().settings.window.height
                            } else {
                                logical.height
                            },
                            maximized: desktop.window.is_maximized(),
                        }));
                    }
                }
                WindowEvent::CloseRequested => {
                    if let Some(window) = pending_window.write().take() {
                        window_ui.send(Command::UpdateSettings(SettingsUpdate::Window(window)));
                    }
                    window_ui.quit();
                }
                WindowEvent::DroppedFile(path) => window_ui.send(Command::Import(path.clone())),
                _ => {}
            }
        }
    });
    let keyboard_ui = ui;
    let theme = state.read().settings.theme_mode.css();
    let selected = page();
    let busy = state.read().busy;
    rsx! {
        style { {include_str!("style.css")} }
        div {
            class: "app {theme}",
            onkeydown: move |event: KeyboardEvent| {
                let modifiers = event.modifiers();
                let command = if cfg!(target_os = "macos") {
                    modifiers.meta()
                } else {
                    modifiers.ctrl()
                };
                if command {
                    let action = match event.key() {
                        Key::Character(ref c) => {
                            match c.to_lowercase().as_str() {
                                "n" => Some("new"),
                                "i" => Some("import"),
                                "r" => Some("refresh"),
                                "f" => Some("search"),
                                "l" => Some("lock"),
                                "," => Some("settings"),
                                "q" => Some("quit"),
                                _ => None,
                            }
                        }
                        _ => None,
                    };
                    if let Some(id) = action {
                        event.prevent_default();
                        keyboard_ui.action(id);
                    }
                }
                if event.key() == Key::Escape {
                    if dialog() != Dialog::None {
                        if dialog() == Dialog::Touch {
                            keyboard_ui.runtime.read().cancel();
                        }
                        if !keyboard_ui.state.read().busy || dialog() == Dialog::Touch {
                            dialog.set(Dialog::None);
                        }
                    } else if page() == Page::Add {
                        if keyboard_ui.state.read().busy {
                            keyboard_ui.runtime.read().cancel();
                        }
                        page.set(Page::Credentials);
                    }
                }
            },
            aside { class: "sidebar",
                div { class: "brand",
                    div { class: "brand-mark", "G" }
                    div {
                        strong { "Gosh" }
                        small { "Yubico Authenticator" }
                    }
                }
                nav { "aria-label": "Main navigation",
                    for (label , target , icon) in [
                        ("Credentials", Page::Credentials, "⌘"),
                        ("Key Info", Page::KeyInfo, "◈"),
                        ("Settings", Page::Settings, "⚙"),
                    ]
                    {
                        button {
                            class: if selected == target { "nav-item active" } else { "nav-item" },
                            "aria-label": label,
                            "aria-current": if selected == target { "page" } else { "false" },
                            onclick: move |_| page.set(target),
                            span { class: "nav-icon", "{icon}" }
                            "{label}"
                        }
                    }
                }
                div { class: "sidebar-foot",
                    span { class: if state.read().connection == Connection::Ready { "status-dot ready" } else { "status-dot" } }
                    small {
                        match state.read().connection {
                            Connection::Connecting => "Connecting…",
                            Connection::Disconnected => "No YubiKey connected",
                            Connection::Locked => "YubiKey locked",
                            Connection::Ready => "YubiKey connected",
                        }
                    }
                }
            }
            main { class: "main",
                header { class: "toolbar",
                    div {
                        h1 {
                            match selected {
                                Page::Credentials => "Credentials",
                                Page::KeyInfo => "Key Info",
                                Page::Settings => "Settings",
                                Page::Add => "Add credential",
                            }
                        }
                        p {
                            match selected {
                                Page::Credentials => "Secure hardware. Simple access.",
                                Page::KeyInfo => "Your hardware and OATH credentials",
                                Page::Settings => "Make Gosh work your way",
                                Page::Add => "Store a new OATH credential securely on your key",
                            }
                        }
                    }
                    div { class: "toolbar-actions",
                        if selected == Page::Credentials {
                            button {
                                title: "Refresh (Ctrl/Cmd+R)",
                                "aria-label": "Refresh",
                                disabled: busy,
                                onclick: move |_| ui.action("refresh"),
                                "↻"
                            }
                            button {
                                class: "primary",
                                onclick: move |_| ui.action("new"),
                                "+ Add credential"
                            }
                        }
                        button {
                            title: "About Gosh Yubico Authenticator",
                            "aria-label": "About",
                            onclick: move |_| ui.action("about"),
                            svg {
                                width: 18,
                                height: 18,
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: 2,
                                "aria-hidden": "true",
                                circle { cx: 12, cy: 12, r: 9 }
                                path { d: "M12 11v6M12 7v1" }
                            }
                        }
                    }
                }
                if let Some(message) = state.read().message.clone() {
                    div {
                        class: if state.read().error { "notice error" } else { "notice" },
                        role: if state.read().error { "alert" } else { "status" },
                        "{message}"
                        button {
                            title: "Dismiss message",
                            "aria-label": "Dismiss message",
                            onclick: move |_| state.write().message = None,
                            "×"
                        }
                    }
                }
                div { class: "page",
                    match selected {
                        Page::Credentials => rsx! {
                            credentials::Credentials {}
                        },
                        Page::KeyInfo => rsx! {
                            credentials::KeyInfo {}
                        },
                        Page::Settings => rsx! {
                            preferences::Preferences {}
                        },
                        Page::Add => rsx! {
                            forms::AddCredential {}
                        },
                    }
                }
                footer { class: "app-footer",
                    span { "Credentials stay on your YubiKey" }
                    span {
                        "Version "
                        {env!("CARGO_PKG_VERSION")}
                    }
                }
            }
            dialogs::Dialogs {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gosh_authenticator_core::{app::DeviceInfo, settings::Settings};

    fn connected(id: [u8; 8]) -> Event {
        Event::Connected(
            DeviceInfo {
                version: (5, 7, 1),
                id,
                has_password: false,
            },
            false,
        )
    }

    fn device_dialogs() -> [Dialog; 6] {
        let id = CredentialId(b"issuer:account".to_vec());
        [
            Dialog::Unlock,
            Dialog::Password,
            Dialog::Delete(id.clone()),
            Dialog::Icon(id.clone()),
            Dialog::Touch,
            Dialog::Actions(id),
        ]
    }

    #[test]
    fn key_removal_and_replacement_invalidate_every_device_dialog() {
        let mut state = State::new(Settings::default());
        state.apply(connected(*b"key-A-id"));
        for dialog in device_dialogs() {
            assert!(dialog.is_invalidated_by(&Event::Disconnected, &state));
            // Key B may have the same credential ID as the dialog for key A.
            assert!(dialog.is_invalidated_by(&connected(*b"key-B-id"), &state));
        }
        state.apply(Event::Disconnected);
        for dialog in device_dialogs() {
            assert!(dialog.is_invalidated_by(&connected(*b"key-B-id"), &state));
        }
    }

    #[test]
    fn same_key_metadata_updates_and_global_dialogs_remain_open() {
        let mut state = State::new(Settings::default());
        state.apply(connected(*b"key-A-id"));
        for dialog in device_dialogs() {
            assert!(!dialog.is_invalidated_by(&connected(*b"key-A-id"), &state));
            assert!(!dialog.is_invalidated_by(&Event::Busy(false), &state));
        }
        for dialog in [Dialog::None, Dialog::About, Dialog::Licenses] {
            assert!(!dialog.is_invalidated_by(&Event::Disconnected, &state));
            assert!(!dialog.is_invalidated_by(&connected(*b"key-B-id"), &state));
        }
    }
}
