use std::cell::Cell;
use std::rc::Rc;

use adw::prelude::*;
use gtk::gio;
use gtk::prelude::*;

use crate::qr::{parse_otpauth_uri, scan_qr_file, OtpAuthCredential};
use crate::ui::state::AppState;
use gosh_authenticator_core::core::credential::{decode_secret, NewCredential};
use gosh_authenticator_core::core::yubikey::{Algorithm, OathType};

pub struct AddPage {
    pub page: adw::NavigationPage,
    issuer: adw::EntryRow,
    account: adw::EntryRow,
    secret: adw::PasswordEntryRow,
    type_row: adw::ComboRow,
    algorithm_row: adw::ComboRow,
    digits_row: adw::ComboRow,
    touch_row: adw::SwitchRow,
    submitting: Cell<bool>,
}

impl AddPage {
    pub fn new(
        state: Rc<AppState>,
        window: adw::ApplicationWindow,
        toast: adw::ToastOverlay,
        nav: adw::NavigationView,
    ) -> Self {
        let issuer = adw::EntryRow::builder().title("Issuer").build();
        let account = adw::EntryRow::builder().title("Account").build();
        let secret = adw::PasswordEntryRow::builder()
            .title("Secret (Base32)")
            .build();

        let type_row = adw::ComboRow::builder()
            .title("Type")
            .model(&gtk::StringList::new(&["TOTP", "HOTP"]))
            .build();
        let algorithm_row = adw::ComboRow::builder()
            .title("Algorithm")
            .subtitle("Hash algorithm used to generate codes")
            .model(&gtk::StringList::new(&["SHA1", "SHA256", "SHA512"]))
            .build();
        let digits_row = adw::ComboRow::builder()
            .title("Digits")
            .model(&gtk::StringList::new(&["6", "7", "8"]))
            .build();
        let touch_row = adw::SwitchRow::builder()
            .title("Require touch")
            .subtitle("Physical touch is required to generate a code")
            .build();

        let identity = adw::PreferencesGroup::builder()
            .title("Identity")
            .build();
        identity.add(&issuer);
        identity.add(&account);

        let security = adw::PreferencesGroup::builder().title("Secret").build();
        security.add(&secret);
        security.add(&type_row);
        security.add(&algorithm_row);
        security.add(&digits_row);
        security.add(&touch_row);

        let info = adw::PreferencesGroup::builder()
            .description("Credentials are stored securely on your YubiKey. Removing the app will not delete credentials from the hardware key.")
            .build();

        let qr_button = gtk::Button::builder()
            .label("Scan QR from file")
            .css_classes(["suggested-action"])
            .build();
        let save_button = gtk::Button::builder()
            .label("Add Credential")
            .css_classes(["suggested-action"])
            .sensitive(false)
            .build();

        let page_box = gtk::Box::new(gtk::Orientation::Vertical, 18);
        page_box.set_margin_top(18);
        page_box.set_margin_bottom(24);
        page_box.set_margin_start(18);
        page_box.set_margin_end(18);
        page_box.append(&qr_button);
        page_box.append(&identity);
        page_box.append(&security);
        page_box.append(&info);
        page_box.append(&save_button);

        let clamp = adw::Clamp::builder()
            .maximum_size(560)
            .child(&page_box)
            .build();
        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&clamp)
            .build();

        let header = adw::HeaderBar::new();
        let view = adw::ToolbarView::new();
        view.add_top_bar(&header);
        view.set_content(Some(&scrolled));

        let page = adw::NavigationPage::builder()
            .child(&view)
            .title("Add Credential")
            .tag("add")
            .build();

        let form = Rc::new(Self {
            page: page.clone(),
            issuer: issuer.clone(),
            account: account.clone(),
            secret: secret.clone(),
            type_row: type_row.clone(),
            algorithm_row: algorithm_row.clone(),
            digits_row: digits_row.clone(),
            touch_row: touch_row.clone(),
            submitting: Cell::new(false),
        });

        let validate = {
            let form = Rc::clone(&form);
            let save_button = save_button.clone();
            move || {
                let account_ok = !form.account.text().trim().is_empty();
                let secret_ok = crate::ui::dialogs::secret_is_valid(&form.secret.text());
                save_button.set_sensitive(account_ok && secret_ok && !form.submitting.get());
            }
        };
        let validate = Rc::new(validate);
        account.connect_changed({
            let validate = Rc::clone(&validate);
            move |_| validate()
        });
        secret.connect_changed({
            let validate = Rc::clone(&validate);
            move |_| validate()
        });

        save_button.connect_clicked({
            let form = Rc::clone(&form);
            let state = Rc::clone(&state);
            let toast = toast.clone();
            let nav = nav.clone();
            let validate = Rc::clone(&validate);
            move |_| {
                if form.submitting.get() {
                    return;
                }
                let account = form.account.text().trim().to_string();
                if account.is_empty() {
                    toast.add_toast(adw::Toast::new("Account is required"));
                    return;
                }
                let secret = form.secret.text().to_string();
                let secret_bytes = match decode_secret(&secret) {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        toast.add_toast(adw::Toast::new(&err));
                        return;
                    }
                };
                let issuer = form.issuer.text().trim().to_string();
                let oath_type = if form.type_row.selected() == 1 {
                    OathType::Hotp
                } else {
                    OathType::Totp
                };
                let algorithm = match form.algorithm_row.selected() {
                    1 => Algorithm::Sha256,
                    2 => Algorithm::Sha512,
                    _ => Algorithm::Sha1,
                };
                let digits = match form.digits_row.selected() {
                    1 => 7,
                    2 => 8,
                    _ => 6,
                };
                form.submitting.set(true);
                validate();
                state.service.add_credential(NewCredential {
                    issuer: if issuer.is_empty() { None } else { Some(issuer) },
                    account,
                    secret: secret_bytes,
                    oath_type,
                    algorithm,
                    digits,
                    require_touch: form.touch_row.is_active(),
                    initial_counter: if oath_type == OathType::Hotp {
                        Some(0)
                    } else {
                        None
                    },
                });
                nav.pop();
                form.submitting.set(false);
                form.clear();
                validate();
            }
        });

        qr_button.connect_clicked({
            let form = Rc::clone(&form);
            let window = window.clone();
            let toast = toast.clone();
            let validate = Rc::clone(&validate);
            move |_| {
                let dialog = gtk::FileDialog::builder()
                    .title("Select QR Code Image")
                    .build();
                let filter = gtk::FileFilter::new();
                filter.set_name(Some("Images"));
                filter.add_mime_type("image/png");
                filter.add_mime_type("image/jpeg");
                filter.add_mime_type("image/webp");
                filter.add_pattern("*.png");
                filter.add_pattern("*.jpg");
                filter.add_pattern("*.jpeg");
                filter.add_pattern("*.webp");
                let filters = gio::ListStore::new::<gtk::FileFilter>();
                filters.append(&filter);
                dialog.set_filters(Some(&filters));
                dialog.set_default_filter(Some(&filter));

                let form = Rc::clone(&form);
                let toast = toast.clone();
                let validate = Rc::clone(&validate);
                dialog.open(Some(&window), None::<&gio::Cancellable>, move |result| {
                    match result {
                        Ok(file) => {
                            if let Some(path) = file.path() {
                                match scan_qr_file(&path) {
                                    Ok(cred) => {
                                        form.apply_otpauth(&cred);
                                        validate();
                                        toast.add_toast(adw::Toast::new("QR code imported"));
                                    }
                                    Err(err) => toast.add_toast(adw::Toast::new(&err.0)),
                                }
                            }
                        }
                        Err(err) if err.matches(gtk::DialogError::Dismissed) => {}
                        Err(err) => toast.add_toast(adw::Toast::new(&err.to_string())),
                    }
                });
            }
        });

        // Allow pasting otpauth URIs into the secret field as a convenience.
        secret.connect_changed({
            let form = Rc::clone(&form);
            let validate = Rc::clone(&validate);
            move |row| {
                let text = row.text().to_string();
                if text.starts_with("otpauth://") {
                    if let Ok(cred) = parse_otpauth_uri(&text) {
                        form.apply_otpauth(&cred);
                        validate();
                    }
                }
            }
        });

        (*form).clone_handles()
    }

    fn clone_handles(&self) -> Self {
        Self {
            page: self.page.clone(),
            issuer: self.issuer.clone(),
            account: self.account.clone(),
            secret: self.secret.clone(),
            type_row: self.type_row.clone(),
            algorithm_row: self.algorithm_row.clone(),
            digits_row: self.digits_row.clone(),
            touch_row: self.touch_row.clone(),
            submitting: Cell::new(false),
        }
    }

    fn apply_otpauth(&self, cred: &OtpAuthCredential) {
        self.issuer
            .set_text(cred.issuer.as_deref().unwrap_or_default());
        self.account.set_text(&cred.account);
        self.secret.set_text(&cred.secret);
        self.type_row
            .set_selected(if cred.oath_type == OathType::Hotp { 1 } else { 0 });
        self.algorithm_row.set_selected(match cred.algorithm {
            Algorithm::Sha256 => 1,
            Algorithm::Sha512 => 2,
            Algorithm::Sha1 => 0,
        });
        self.digits_row.set_selected(match cred.digits {
            7 => 1,
            8 => 2,
            _ => 0,
        });
    }

    fn clear(&self) {
        self.issuer.set_text("");
        self.account.set_text("");
        self.secret.set_text("");
        self.type_row.set_selected(0);
        self.algorithm_row.set_selected(0);
        self.digits_row.set_selected(0);
        self.touch_row.set_active(false);
    }
}
