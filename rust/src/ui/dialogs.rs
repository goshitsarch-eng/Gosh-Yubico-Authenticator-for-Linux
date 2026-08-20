use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk::prelude::*;

use crate::prefs::{APP_DEVELOPER, APP_ID, APP_ISSUES, APP_NAME, APP_VERSION, APP_WEBSITE};
use crate::service_icons::{all_services, extract_domain_from_text};
use crate::ui::state::AppState;
use gosh_authenticator_core::core::credential::decode_secret;

pub fn present_about(parent: &impl IsA<gtk::Widget>) {
    let about = adw::AboutDialog::builder()
        .application_name(APP_NAME)
        .application_icon(APP_ID)
        .developer_name(APP_DEVELOPER)
        .version(APP_VERSION)
        .developers(vec![APP_DEVELOPER.to_string()])
        .copyright("© Goshitsarch")
        .license_type(gtk::License::Gpl30)
        .website(APP_WEBSITE)
        .issue_url(APP_ISSUES)
        .comments("Manage OATH (TOTP/HOTP) credentials on YubiKey devices.\n\nSECURE HARDWARE, SECURE LOGIN")
        .build();
    about.present(Some(parent));
}

pub fn present_pin_dialog(
    parent: &impl IsA<gtk::Widget>,
    state: Rc<AppState>,
    error: Option<&str>,
    on_close: impl Fn() + 'static,
) {
    if state.pin_open.get() {
        return;
    }
    state.pin_open.set(true);

    let entry = gtk::PasswordEntry::builder()
        .show_peek_icon(true)
        .hexpand(true)
        .activates_default(true)
        .placeholder_text("YubiKey PIN")
        .build();

    let body = if let Some(error) = error {
        format!("{error}\n\nEnter the OATH password for this YubiKey.")
    } else {
        "This YubiKey is password protected. Enter the OATH PIN to continue.".to_string()
    };

    let dialog = adw::AlertDialog::builder()
        .heading("Unlock YubiKey")
        .body(&body)
        .default_response("unlock")
        .close_response("cancel")
        .extra_child(&entry)
        .build();
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("unlock", "Unlock");
    dialog.set_response_appearance("unlock", adw::ResponseAppearance::Suggested);
    dialog.set_response_enabled("unlock", false);

    entry.connect_changed({
        let dialog = dialog.clone();
        move |entry| {
            dialog.set_response_enabled("unlock", !entry.text().is_empty());
        }
    });

    let on_close = Rc::new(on_close);
    dialog.connect_response(None, {
        let state = Rc::clone(&state);
        let entry = entry.clone();
        let on_close = Rc::clone(&on_close);
        move |_, response| {
            state.pin_open.set(false);
            if response == "unlock" {
                let password = entry.text().to_string();
                if !password.is_empty() {
                    state.service.authenticate(password);
                }
            }
            on_close();
        }
    });

    dialog.present(Some(parent));
    entry.grab_focus();
}

pub fn present_touch_dialog(parent: &impl IsA<gtk::Widget>, state: Rc<AppState>) {
    if state.touch_open.get() {
        return;
    }
    state.touch_open.set(true);

    let dialog = adw::AlertDialog::builder()
        .heading("Touch required")
        .body("Touch your YubiKey to generate the code.")
        .close_response("cancel")
        .build();
    dialog.add_response("cancel", "Cancel");
    dialog.connect_response(None, {
        let state = Rc::clone(&state);
        move |_, _| {
            state.touch_open.set(false);
        }
    });
    dialog.present(Some(parent));
}

pub fn present_delete_dialog(
    parent: &impl IsA<gtk::Widget>,
    state: Rc<AppState>,
    display_name: String,
    id: gosh_authenticator_core::core::credential::CredentialId,
) {
    let dialog = adw::AlertDialog::builder()
        .heading("Delete credential?")
        .body(&format!(
            "Are you sure you want to delete {display_name}? This cannot be undone."
        ))
        .default_response("cancel")
        .close_response("cancel")
        .build();
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("delete", "Delete");
    dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
    dialog.connect_response(None, move |_, response| {
        if response == "delete" {
            state.service.delete_credential(id.clone());
        }
    });
    dialog.present(Some(parent));
}

pub fn present_change_password_dialog(parent: &impl IsA<gtk::Widget>, state: Rc<AppState>) {
    let password = adw::PasswordEntryRow::builder()
        .title("New PIN")
        .build();
    let confirm = adw::PasswordEntryRow::builder()
        .title("Confirm PIN")
        .build();
    let remove = adw::SwitchRow::builder()
        .title("Remove password protection")
        .subtitle("Anyone with physical access can use the key")
        .build();

    let group = adw::PreferencesGroup::new();
    group.add(&password);
    group.add(&confirm);
    group.add(&remove);

    let dialog = adw::AlertDialog::builder()
        .heading("Change YubiKey PIN")
        .body("Set a new OATH password, or remove password protection.")
        .default_response("save")
        .close_response("cancel")
        .extra_child(&group)
        .build();
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("save", "Save");
    dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);

    let sync_enabled = {
        let dialog = dialog.clone();
        let password = password.clone();
        let confirm = confirm.clone();
        let remove = remove.clone();
        move || {
            if remove.is_active() {
                dialog.set_response_enabled("save", true);
            } else {
                let p = password.text();
                let c = confirm.text();
                dialog.set_response_enabled("save", p.len() >= 4 && p == c);
            }
        }
    };
    let sync_enabled = Rc::new(sync_enabled);
    sync_enabled();
    password.connect_changed({
        let sync_enabled = Rc::clone(&sync_enabled);
        move |_| sync_enabled()
    });
    confirm.connect_changed({
        let sync_enabled = Rc::clone(&sync_enabled);
        move |_| sync_enabled()
    });
    remove.connect_active_notify({
        let sync_enabled = Rc::clone(&sync_enabled);
        let password = password.clone();
        let confirm = confirm.clone();
        move |row| {
            password.set_sensitive(!row.is_active());
            confirm.set_sensitive(!row.is_active());
            sync_enabled();
        }
    });

    dialog.connect_response(None, move |_, response| {
        if response != "save" {
            return;
        }
        if remove.is_active() {
            state.service.set_password(String::new());
        } else {
            let value = password.text().to_string();
            state.service.set_password(value);
        }
    });
    dialog.present(Some(parent));
}

pub fn present_icon_picker(
    parent: &impl IsA<gtk::Widget>,
    state: Rc<AppState>,
    credential_id: Vec<u8>,
    current_domain: Option<String>,
    on_changed: impl Fn() + 'static,
) {
    let flow = gtk::FlowBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .max_children_per_line(4)
        .min_children_per_line(2)
        .column_spacing(8)
        .row_spacing(8)
        .build();

    let chosen_key = Rc::new(RefCell::new(None::<String>));
    for service in all_services() {
        let button = gtk::Button::builder()
            .label(service.label)
            .hexpand(true)
            .build();
        button.connect_clicked({
            let chosen_key = Rc::clone(&chosen_key);
            let key = service.key.to_string();
            move |_| {
                *chosen_key.borrow_mut() = Some(key.clone());
            }
        });
        flow.append(&button);
    }

    let domain = adw::EntryRow::builder()
        .title("Favicon domain")
        .text(&current_domain.unwrap_or_default())
        .build();

    let group = adw::PreferencesGroup::new();
    group.set_title("Choose icon");
    group.add(&domain);

    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 12);
    box_.append(&group);
    let scrolled = gtk::ScrolledWindow::builder()
        .min_content_height(220)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&flow)
        .build();
    box_.append(&scrolled);

    let dialog = adw::AlertDialog::builder()
        .heading("Choose icon")
        .default_response("save")
        .close_response("cancel")
        .extra_child(&box_)
        .build();
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("reset", "Reset");
    dialog.add_response("save", "Save");
    dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);

    let on_changed = Rc::new(on_changed);
    dialog.connect_response(None, move |_, response| {
        let key = crate::prefs::Prefs::icon_key(&credential_id);
        {
            let mut prefs = state.prefs.borrow_mut();
            match response {
                "reset" => {
                    prefs.icon_prefs.remove(&key);
                    prefs.save();
                }
                "save" => {
                    let mut pref = crate::prefs::IconPreference::default();
                    pref.custom_icon_key = chosen_key.borrow().clone();
                    let domain_text = domain.text().to_string();
                    pref.favicon_domain = extract_domain_from_text(&domain_text)
                        .or_else(|| {
                            if domain_text.trim().is_empty() {
                                None
                            } else {
                                Some(domain_text.trim().to_string())
                            }
                        });
                    if pref.custom_icon_key.is_none() && pref.favicon_domain.is_none() {
                        prefs.icon_prefs.remove(&key);
                    } else {
                        prefs.icon_prefs.insert(key, pref);
                    }
                    prefs.save();
                }
                _ => return,
            }
        }
        on_changed();
    });
    dialog.present(Some(parent));
}

pub fn secret_is_valid(secret: &str) -> bool {
    match decode_secret(secret) {
        Ok(bytes) => !bytes.is_empty(),
        Err(_) => false,
    }
}
