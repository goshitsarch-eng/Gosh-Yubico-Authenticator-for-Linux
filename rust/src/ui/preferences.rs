use super::{Dialog, Ui};
use dioxus::prelude::*;
use gosh_authenticator_core::{
    app::{Command, Connection},
    settings::ThemeMode,
};
#[component]
pub fn Preferences() -> Element {
    let ui = use_context::<Ui>();
    let mut dialog = ui.dialog;
    let settings_path = use_context::<crate::Startup>().settings_path;
    let settings = ui.state.read().settings.clone();
    let ready = ui.state.read().connection == Connection::Ready;
    let theme_ui = ui;
    let clip_ui = ui;
    let prompt_ui = ui;
    let favicon_ui = ui;
    rsx! {
        section { class: "card",
            h2 { "Appearance" }
            label { class: "setting-row",
                span {
                    strong { "Theme" }
                    small { "Change appearance instantly" }
                }
                select {
                    "aria-label": "Theme",
                    value: settings.theme_mode.css(),
                    onchange: move |e| {
                        let mut s = theme_ui.state.read().settings.clone();
                        s.theme_mode = match e.value().as_str() {
                            "light" => ThemeMode::Light,
                            "dark" => ThemeMode::Dark,
                            _ => ThemeMode::System,
                        };
                        theme_ui.send(Command::UpdateSettings(s));
                    },
                    option { value: "system", selected: settings.theme_mode == ThemeMode::System, "Follow System" }
                    option { value: "light", selected: settings.theme_mode == ThemeMode::Light, "Light" }
                    option { value: "dark", selected: settings.theme_mode == ThemeMode::Dark, "Dark" }
                }
            }
        }
        section { class: "card",
            h2 { "Security" }
            label { class: "setting-row",
                span {
                    strong { "Clear clipboard" }
                    small { "Only clears a code while the clipboard still contains that code" }
                }
                select {
                    "aria-label": "Clear clipboard",
                    value: "{settings.clipboard_timeout_seconds}",
                    onchange: move |e| {
                        match e.value().parse() {
                            Ok(seconds) => {
                                let mut s = clip_ui.state.read().settings.clone();
                                s.clipboard_timeout_seconds = seconds;
                                clip_ui.send(Command::UpdateSettings(s));
                            }
                            Err(_) => clip_ui.error("Invalid clipboard timeout".into()),
                        }
                    },
                    for seconds in [10, 20, 30, 60, 120] {
                        option { value: "{seconds}", selected: settings.clipboard_timeout_seconds == seconds, "{seconds} seconds" }
                    }
                }
            }
            label { class: "setting-row",
                span {
                    strong { "Prompt to unlock on launch" }
                    small { "Automatically opens the password prompt for a protected key" }
                }
                input {
                    r#type: "checkbox",
                    "aria-label": "Prompt to unlock on launch",
                    checked: settings.require_pin_on_launch,
                    onchange: move |e| {
                        let mut s = prompt_ui.state.read().settings.clone();
                        s.require_pin_on_launch = e.checked();
                        prompt_ui.send(Command::UpdateSettings(s));
                    },
                }
            }
            div { class: "setting-row",
                span {
                    strong { "OATH password" }
                    small { "Set, change, or remove protection on your hardware key" }
                }
                button {
                    disabled: !ready || ui.state.read().busy,
                    onclick: move |_| dialog.set(Dialog::Password),
                    "Change password…"
                }
            }
            p { class: "hint",
                "The launch preference cannot protect a key that has no OATH password. Set a password on the key to require authentication."
            }
        }
        section { class: "card",
            h2 { "Service icons" }
            label { class: "setting-row",
                span {
                    strong { "Allow optional favicon downloads" }
                    small { "Only a domain you explicitly request is sent to Google's favicon service" }
                }
                input {
                    r#type: "checkbox",
                    "aria-label": "Allow optional favicon downloads",
                    checked: settings.allow_favicons,
                    onchange: move |e| {
                        let mut s = favicon_ui.state.read().settings.clone();
                        s.allow_favicons = e.checked();
                        favicon_ui.send(Command::UpdateSettings(s));
                    },
                }
            }
            p { class: "hint",
                "Built-in service avatars work offline. Use Choose icon on a credential to request a favicon. No account domains are sent automatically."
            }
        }
        section { class: "card",
            h2 { "About Gosh" }
            p { "A hardware-first OATH authenticator for Windows, macOS and Linux." }
            button { onclick: move |_| ui.action("about"), "About Gosh Yubico Authenticator" }
            button {
                onclick: move |_| {
                    {
                        let path = settings_path.clone();
                        spawn(async move {
                            if let Err(e) = gosh_authenticator_core::platform::show_in_folder_async(
                                    path,
                                )
                                .await
                            {
                                ui.error(e);
                            }
                        });
                    }
                },
                "Show settings folder"
            }
        }
    }
}
