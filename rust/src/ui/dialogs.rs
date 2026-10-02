use super::{Dialog, Ui};
use dioxus::prelude::*;
use gosh_authenticator_core::{
    app::Command,
    service_icons::all_services,
    settings::{validated_domain, IconPreference, Settings, APP_NAME, APP_WEBSITE},
};
use zeroize::Zeroize;
#[component]
pub fn Dialogs() -> Element {
    let ui = use_context::<Ui>();
    let mut dialog = ui.dialog;
    let current = dialog();
    if current == Dialog::None {
        return rsx! {};
    }
    let title = match &current {
        Dialog::About => "About Gosh",
        Dialog::Licenses => "Licenses",
        Dialog::Unlock => "Unlock YubiKey",
        Dialog::Password => "OATH password",
        Dialog::Delete(_) => "Delete credential?",
        Dialog::Icon(_) => "Choose icon",
        Dialog::Touch => "Generating code",
        Dialog::Actions(_) => "Credential actions",
        Dialog::None => "",
    };
    rsx! {
        div { class: "modal-backdrop",
            div {
                class: "modal",
                role: "dialog",
                "aria-modal": "true",
                "aria-label": title,
                onmounted: move |_| {
                    spawn(async {
                        let _ = document::eval(
                                "document.querySelector('.modal input, .modal button')?.focus()",
                            )
                            .await;
                    });
                },
                onkeydown: move |e: KeyboardEvent| {
                    if e.key() == Key::Tab {
                        e.prevent_default();
                        let step = if e.modifiers().shift() { -1 } else { 1 };
                        spawn(async move {
                            let script = format!(
                                "const n=[...document.querySelectorAll(\".modal button:not([disabled]), .modal input:not([disabled]), .modal select:not([disabled]), .modal [tabindex='0']\")]; const i=n.indexOf(document.activeElement); n[(i+{step}+n.length)%n.length]?.focus();",
                            );
                            let _ = document::eval(&script).await;
                        });
                    }
                },
                div { class: "modal-heading",
                    h2 { "{title}" }
                    button {
                        title: "Close dialog",
                        "aria-label": "Close dialog",
                        disabled: ui.state.read().busy && current != Dialog::Touch,
                        onclick: move |_| {
                            if current == Dialog::Touch {
                                ui.runtime.read().cancel();
                            }
                            dialog.set(Dialog::None);
                        },
                        "×"
                    }
                }
                if ui.state.read().error {
                    if let Some(message) = ui.state.read().message.clone() {
                        p { class: "notice error", role: "alert", "{message}" }
                    }
                }
                match dialog() {
                    Dialog::About => rsx! {
                        About {}
                    },
                    Dialog::Licenses => rsx! {
                        LicenseDetails {}
                    },
                    Dialog::Unlock => rsx! {
                        Unlock {}
                    },
                    Dialog::Password => rsx! {
                        Password {}
                    },
                    Dialog::Delete(id) => rsx! {
                        DeleteCredential { id }
                    },
                    Dialog::Icon(id) => rsx! {
                        IconPicker { id }
                    },
                    Dialog::Touch => rsx! {
                        p {
                            "If your key lights up, touch its illuminated area. Code generation runs in the background; HOTP touch policy is not reported by the key's credential list."
                        }
                        p { class: "hint",
                            "Cancelling discards the result; the smart-card operation may take time to return. HOTP counter changes on the key cannot be undone."
                        }
                        button {
                            onclick: move |_| {
                                ui.runtime.read().cancel();
                                dialog.set(Dialog::None);
                            },
                            "Cancel request"
                        }
                    },
                    Dialog::Actions(id) => rsx! {
                        Actions { id }
                    },
                    Dialog::None => rsx! {},
                }
            }
        }
    }
}
#[component]
fn About() -> Element {
    let ui = use_context::<Ui>();
    let mut dialog = ui.dialog;
    rsx! {
        div { class: "about-mark", "G" }
        h3 { "{APP_NAME}" }
        p {
            "Version "
            {env!("CARGO_PKG_VERSION")}
        }
        p {
            "Secure hardware, secure login. An independent application, not affiliated with or endorsed by Yubico. Yubico and YubiKey are trademarks of Yubico AB."
        }
        p { "© Goshitsarch · GPL-3.0-or-later" }
        button { onclick: move |_| dialog.set(Dialog::Licenses), "License details" }
        div { class: "form-actions",
            button { onclick: move |_| ui.open_url(APP_WEBSITE), "Project website" }
            button { onclick: move |_| ui.open_url(&format!("{APP_WEBSITE}/issues")), "Report an issue" }
        }
    }
}
#[component]
fn LicenseDetails() -> Element {
    let ui = use_context::<Ui>();
    rsx! {
        p {
            "Gosh Yubico Authenticator is licensed under GPL-3.0-or-later. Dependency licenses are also included in every package."
        }
        pre { class: "license-text", tabindex: "0", {include_str!("../../../LICENSE")} }
        button {
            onclick: move |_| {
                spawn(async move {
                    if let Err(error) = gosh_authenticator_core::platform::open_third_party_licenses_async()
                        .await
                    {
                        ui.error(error);
                    }
                });
            },
            "Open third-party notices"
        }
    }
}
#[component]
fn Unlock() -> Element {
    let ui = use_context::<Ui>();
    let mut password = use_signal(String::new);
    let busy = ui.state.read().busy;
    rsx! {
        p { "Enter the OATH password for this YubiKey. Passwords are never saved." }
        label { class: "field",
            span { "OATH password" }
            input {
                r#type: "password",
                autocomplete: "off",
                value: password(),
                oninput: move |e| password.set(e.value()),
            }
        }
        button {
            class: "primary",
            disabled: busy || password.read().is_empty(),
            onclick: move |_| {
                ui.send(Command::Authenticate(zeroize::Zeroizing::new(password())));
                password.write().zeroize();
            },
            if busy {
                "Unlocking…"
            } else {
                "Unlock"
            }
        }
    }
}
#[component]
fn Password() -> Element {
    let ui = use_context::<Ui>();
    let mut password = use_signal(String::new);
    let mut confirmation = use_signal(String::new);
    let mut remove = use_signal(|| false);
    let valid = remove() || (password.read().chars().count() >= 4 && password() == confirmation());
    rsx! {
        p { "Protect this key with an OATH password, or explicitly remove its password protection." }
        label { class: "field",
            span { "New password" }
            input {
                r#type: "password",
                disabled: remove(),
                value: password(),
                oninput: move |e| password.set(e.value()),
            }
        }
        label { class: "field",
            span { "Confirm password" }
            input {
                r#type: "password",
                disabled: remove(),
                value: confirmation(),
                oninput: move |e| confirmation.set(e.value()),
            }
        }
        label { class: "check-row",
            input {
                r#type: "checkbox",
                checked: remove(),
                onchange: move |e| remove.set(e.checked()),
            }
            "Remove password protection"
        }
        if remove() {
            p { class: "validation", "Anyone with physical access will be able to use this key." }
        }
        button {
            class: if remove() { "danger" } else { "primary" },
            disabled: !valid || ui.state.read().busy,
            onclick: move |_| {
                ui.send(
                    Command::SetPassword(
                        zeroize::Zeroizing::new(
                            if remove() { String::new() } else { password() },
                        ),
                    ),
                );
                password.write().zeroize();
                confirmation.write().zeroize();
            },
            if remove() {
                "Remove protection"
            } else {
                "Save password"
            }
        }
    }
}
#[component]
fn DeleteCredential(id: gosh_authenticator_core::core::credential::CredentialId) -> Element {
    let ui = use_context::<Ui>();
    let mut dialog = ui.dialog;
    let name = ui
        .state
        .read()
        .credentials
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.display_name())
        .unwrap_or_else(|| "this credential".into());
    rsx! {
        p { "Permanently delete {name} from your YubiKey? This cannot be undone." }
        div { class: "form-actions",
            button { onclick: move |_| dialog.set(Dialog::None), "Cancel" }
            button {
                class: "danger",
                disabled: ui.state.read().busy,
                onclick: move |_| ui.send(Command::Delete(id.clone())),
                "Delete permanently"
            }
        }
    }
}
#[component]
fn Actions(id: gosh_authenticator_core::core::credential::CredentialId) -> Element {
    let ui = use_context::<Ui>();
    let mut dialog = ui.dialog;
    let code = ui
        .state
        .read()
        .credentials
        .iter()
        .find(|c| c.id == id)
        .and_then(|c| c.code.clone());
    let copy_id = id.clone();
    let generate_id = id.clone();
    let icon_id = id.clone();
    rsx! {
        div { class: "action-list",
            button {
                disabled: code.is_none() || ui.state.read().busy,
                onclick: move |_| {
                    ui.send(Command::Copy(copy_id.clone()));
                    dialog.set(Dialog::None);
                },
                "Copy code"
            }
            button {
                disabled: ui.state.read().busy,
                onclick: move |_| {
                    ui.send(Command::Calculate(generate_id.clone()));
                    dialog.set(Dialog::None);
                },
                "Generate code"
            }
            button { onclick: move |_| dialog.set(Dialog::Icon(icon_id.clone())), "Choose icon" }
            button {
                class: "danger",
                onclick: move |_| dialog.set(Dialog::Delete(id.clone())),
                "Delete credential…"
            }
        }
    }
}
#[component]
fn IconPicker(id: gosh_authenticator_core::core::credential::CredentialId) -> Element {
    let ui = use_context::<Ui>();
    let mut dialog = ui.dialog;
    let key = Settings::icon_key(&id.0);
    let pref = ui
        .state
        .read()
        .settings
        .icon_prefs
        .get(&key)
        .cloned()
        .unwrap_or_default();
    let mut selected = use_signal(|| pref.custom_icon_key);
    let mut domain = use_signal(|| pref.favicon_domain.unwrap_or_default());
    let save_key = key.clone();
    let reset_ui = ui;
    let download_ui = ui;
    rsx! {
        div { class: "icon-grid",
            for service in all_services() {
                button {
                    class: if selected.read().as_deref() == Some(service.key) { "selected" } else { "" },
                    "aria-pressed": selected.read().as_deref() == Some(service.key),
                    onclick: move |_| selected.set(Some(service.key.into())),
                    "{service.label}"
                }
            }
        }
        label { class: "field",
            span { "Optional favicon domain" }
            input {
                value: domain(),
                placeholder: "example.com",
                oninput: move |e| domain.set(e.value()),
            }
        }
        button {
            disabled: !ui.state.read().settings.allow_favicons || domain.read().is_empty()
                || ui.state.read().busy,
            onclick: move |_| match validated_domain(&domain()) {
                Ok(d) => download_ui.send(Command::FetchIcon(d)),
                Err(e) => download_ui.error(e.to_string()),
            },
            "Download favicon"
        }
        div { class: "form-actions",
            button {
                onclick: move |_| {
                    let mut settings = reset_ui.state.read().settings.clone();
                    settings.icon_prefs.remove(&key);
                    reset_ui.send(Command::UpdateSettings(settings));
                    dialog.set(Dialog::None);
                },
                "Reset icon"
            }
            button {
                class: "primary",
                onclick: move |_| {
                    let value = if domain.read().trim().is_empty() {
                        None
                    } else {
                        match validated_domain(&domain()) {
                            Ok(d) => Some(d),
                            Err(e) => {
                                ui.error(e.to_string());
                                return;
                            }
                        }
                    };
                    let mut settings = ui.state.read().settings.clone();
                    settings
                        .icon_prefs
                        .insert(
                            save_key.clone(),
                            IconPreference {
                                custom_icon_key: selected(),
                                favicon_domain: value,
                            },
                        );
                    ui.send(Command::UpdateSettings(settings));
                    dialog.set(Dialog::None);
                },
                "Save icon"
            }
        }
    }
}
