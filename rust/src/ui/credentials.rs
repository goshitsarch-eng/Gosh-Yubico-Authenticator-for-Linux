use super::{Dialog, Ui};
use dioxus::prelude::*;
use gosh_authenticator_core::{
    app::{Command, Connection},
    core::credential::Credential,
    service_icons::{get_service_by_key, guess_service},
    settings::Settings,
};
#[component]
pub fn Credentials() -> Element {
    let ui = use_context::<Ui>();
    let mut state = ui.state;
    let connection = state.read().connection;
    let filtered = state.read().filtered();
    let total = state.read().credentials.len();
    let busy = state.read().busy;
    rsx! {
        label { class: "search-field",
            span { "⌕" }
            input {
                id: "search",
                r#type: "search",
                placeholder: "Search accounts or services…",
                "aria-label": "Search credentials",
                value: state.read().search.clone(),
                oninput: move |e| state.write().search = e.value(),
            }
        }
        if connection != Connection::Ready {
            section { class: "empty-state",
                div { class: "empty-icon", "◈" }
                h2 {
                    if connection == Connection::Locked {
                        "Unlock your YubiKey"
                    } else {
                        "No YubiKey connected"
                    }
                }
                p {
                    if connection == Connection::Locked {
                        "Enter this key's OATH password to access its credentials."
                    } else {
                        "Insert your YubiKey to access your authentication codes. Your secrets stay safely on your hardware."
                    }
                }
                button {
                    class: "primary",
                    disabled: busy,
                    onclick: move |_| {
                        ui.action(if connection == Connection::Locked { "unlock" } else { "connect" })
                    },
                    if busy {
                        "Connecting…"
                    } else if connection == Connection::Locked {
                        "Unlock YubiKey"
                    } else {
                        "Retry connection"
                    }
                }
            }
        } else if filtered.is_empty() {
            section { class: "empty-state",
                div { class: "empty-icon", "⌘" }
                h2 {
                    if total == 0 {
                        "Ready for your first credential"
                    } else {
                        "No matching credentials"
                    }
                }
                p {
                    if total == 0 {
                        "Add an account manually or import an OTP QR image."
                    } else {
                        "Try a different service or account name."
                    }
                }
                if total == 0 {
                    button { class: "primary", onclick: move |_| ui.action("new"), "Add credential" }
                } else {
                    button { onclick: move |_| state.write().search.clear(), "Clear search" }
                }
            }
        } else {
            div { class: "credential-list",
                for credential in filtered {
                    CredentialRow {
                        key: "{Settings::icon_key(&credential.id.0)}",
                        credential,
                    }
                }
            }
        }
    }
}
#[component]
fn CredentialRow(credential: Credential) -> Element {
    let ui = use_context::<Ui>();
    let mut dialog = ui.dialog;
    let pref = ui
        .state
        .read()
        .settings
        .icon_prefs
        .get(&Settings::icon_key(&credential.id.0))
        .cloned();
    let favicon = pref
        .as_ref()
        .and_then(|p| p.favicon_domain.as_ref())
        .and_then(|domain| gosh_authenticator_core::settings::validated_domain(domain).ok())
        .and_then(|domain| ui.state.read().icons.get(&domain).cloned());
    let service = pref
        .and_then(|p| p.custom_icon_key)
        .and_then(|key| get_service_by_key(&key))
        .or_else(|| guess_service(credential.issuer.as_deref(), &credential.account));
    let color = service
        .map(|s| format!("#{:02x}{:02x}{:02x}", s.color.0, s.color.1, s.color.2))
        .unwrap_or_else(|| "#3979dc".into());
    let title = credential
        .issuer
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| credential.account.clone());
    let letter = title
        .chars()
        .next()
        .unwrap_or('G')
        .to_uppercase()
        .to_string();
    let name = credential.display_name();
    let id = credential.id.clone();
    let expired = credential.valid_until.is_some_and(|t| t <= (ui.now)());
    let code = credential.code.clone().filter(|_| !expired);
    let busy = ui.state.read().busy;
    let remaining = credential
        .valid_until
        .map(|t| t.saturating_sub((ui.now)()))
        .unwrap_or(0);
    let keyboard_ui = ui;
    let keyboard_id = id.clone();
    let context_id = id.clone();
    let copy_id = id.clone();
    let generate_id = id.clone();
    let options_id = id.clone();
    rsx! {
        article {
            class: "credential",
            tabindex: 0,
            "aria-label": "{name}",
            oncontextmenu: move |e| {
                e.prevent_default();
                dialog.set(Dialog::Actions(context_id.clone()));
            },
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Enter {
                    keyboard_ui.send(Command::Copy(keyboard_id.clone()));
                }
                if e.key() == Key::F10 && e.modifiers().shift() {
                    e.prevent_default();
                    dialog.set(Dialog::Actions(keyboard_id.clone()));
                }
            },
            div { class: "avatar", style: "background:{color}",
                if let Some(src) = favicon {
                    img {
                        src,
                        alt: "{title} icon",
                        width: 40,
                        height: 40,
                    }
                } else {
                    "{letter}"
                }
            }
            div { class: "credential-name",
                strong { "{title}" }
                small { "{credential.account}" }
                if credential.touch_required {
                    span { class: "badge", "Touch required" }
                }
                if credential.is_hotp() {
                    span { class: "badge", "HOTP" }
                }
            }
            div { class: "credential-code",
                if let Some(code) = code {
                    button {
                        class: "code",
                        title: "Copy code",
                        "aria-label": "Copy code for {name}",
                        disabled: busy,
                        onclick: move |_| ui.send(Command::Copy(copy_id.clone())),
                        "{code}"
                    }
                    if credential.is_totp() {
                        span {
                            class: "countdown",
                            "aria-label": "{remaining} seconds remaining",
                            "{remaining}s"
                        }
                    }
                } else {
                    span { class: "code muted", "••• •••" }
                }
            }
            button {
                title: if credential.is_hotp() { "Generate next HOTP code (advances counter)" } else { "Generate code" },
                "aria-label": "Generate code for {name}",
                disabled: busy,
                onclick: move |_| ui.send(Command::Calculate(generate_id.clone())),
                "↻"
            }
            button {
                title: "Credential actions",
                "aria-label": "Actions for {name}",
                onclick: move |_| dialog.set(Dialog::Actions(options_id.clone())),
                "⋯"
            }
        }
    }
}
#[component]
pub fn KeyInfo() -> Element {
    let ui = use_context::<Ui>();
    let state = ui.state.read();
    rsx! {
        if let Some(device) = state.device.clone() {
            section { class: "card",
                h2 { "YubiKey OATH" }
                dl {
                    dt { "OATH firmware" }
                    dd { "{device.version.0}.{device.version.1}.{device.version.2}" }
                    dt { "Connection" }
                    dd {
                        if state.connection == Connection::Locked {
                            "Locked"
                        } else {
                            "Connected via PC/SC"
                        }
                    }
                    dt { "Password protection" }
                    dd {
                        if device.has_password {
                            "Enabled"
                        } else {
                            "Not enabled"
                        }
                    }
                    dt { "Credentials" }
                    dd { "{state.credentials.len()}" }
                    dt { "TOTP" }
                    dd { "{state.credentials.iter().filter(|c|c.is_totp()).count()}" }
                    dt { "HOTP" }
                    dd { "{state.credentials.iter().filter(|c|c.is_hotp()).count()}" }
                }
                p { class: "hint",
                    "This is the OATH applet firmware. The device's product model is not inferred from this number."
                }
                button { onclick: move |_| ui.action("lock"), "Lock / disconnect" }
            }
        } else {
            section { class: "empty-state",
                div { class: "empty-icon", "◈" }
                h2 { "No YubiKey connected" }
                p { "Insert your YubiKey to view device information." }
                button { class: "primary", onclick: move |_| ui.action("connect"), "Retry connection" }
            }
        }
    }
}
