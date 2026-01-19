use glib::clone;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use libadwaita::subclass::prelude::*;

use crate::core::credential::{decode_secret, NewCredential};
use crate::core::yubikey::{Algorithm, OathType};

mod imp {
    use super::*;

    #[derive(Default, gtk4::CompositeTemplate)]
    #[template(resource = "/com/github/gosh/authenticator/ui/add-credential-dialog.ui")]
    pub struct AddCredentialDialog {
        #[template_child]
        pub cancel_button: TemplateChild<gtk4::Button>,
        #[template_child]
        pub add_button: TemplateChild<gtk4::Button>,
        #[template_child]
        pub issuer_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub account_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub secret_entry: TemplateChild<adw::PasswordEntryRow>,
        #[template_child]
        pub type_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub algorithm_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub digits_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub touch_switch: TemplateChild<adw::SwitchRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AddCredentialDialog {
        const NAME: &'static str = "GoshAddCredentialDialog";
        type Type = super::AddCredentialDialog;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk4::template_callbacks]
    impl AddCredentialDialog {
        #[template_callback]
        fn on_cancel_clicked(&self) {
            self.obj().close();
        }

        #[template_callback]
        fn on_add_clicked(&self) {
            self.obj().emit_by_name::<()>("credential-added", &[]);
        }
    }

    impl ObjectImpl for AddCredentialDialog {
        fn signals() -> &'static [glib::subclass::Signal] {
            use std::sync::OnceLock;
            static SIGNALS: OnceLock<Vec<glib::subclass::Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![glib::subclass::Signal::builder("credential-added").build()]
            })
        }

        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            // Connect entry changes to validate and enable add button
            // Use connect_notify_local for AdwEntryRow as it uses property notification
            self.account_entry.connect_notify_local(Some("text"), clone!(
                #[weak]
                obj,
                move |_, _| obj.validate()
            ));

            self.secret_entry.connect_notify_local(Some("text"), clone!(
                #[weak]
                obj,
                move |_, _| obj.validate()
            ));
        }
    }

    impl WidgetImpl for AddCredentialDialog {}
    impl WindowImpl for AddCredentialDialog {}
    impl AdwWindowImpl for AddCredentialDialog {}
}

glib::wrapper! {
    pub struct AddCredentialDialog(ObjectSubclass<imp::AddCredentialDialog>)
        @extends adw::Window, gtk4::Window, gtk4::Widget;
}

impl AddCredentialDialog {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    /// Validate the form and enable/disable the add button
    fn validate(&self) {
        let imp = self.imp();

        let account = imp.account_entry.text();
        let secret = imp.secret_entry.text();

        // Basic validation
        let account_valid = !account.is_empty();
        let secret_valid = !secret.is_empty() && decode_secret(&secret).is_ok();

        imp.add_button.set_sensitive(account_valid && secret_valid);

        // Show error styling if secret is invalid
        if !secret.is_empty() && decode_secret(&secret).is_err() {
            imp.secret_entry.add_css_class("error");
        } else {
            imp.secret_entry.remove_css_class("error");
        }
    }

    /// Get the credential from the form
    pub fn get_credential(&self) -> Result<NewCredential, String> {
        let imp = self.imp();

        let issuer = {
            let text = imp.issuer_entry.text();
            if text.is_empty() {
                None
            } else {
                Some(text.to_string())
            }
        };

        let account = imp.account_entry.text().to_string();
        if account.is_empty() {
            return Err("Account name is required".into());
        }

        let secret_text = imp.secret_entry.text();
        let secret =
            decode_secret(&secret_text).map_err(|e| format!("Invalid secret: {}", e))?;

        let oath_type = match imp.type_combo.selected() {
            0 => OathType::Totp,
            1 => OathType::Hotp,
            _ => OathType::Totp,
        };

        let algorithm = match imp.algorithm_combo.selected() {
            0 => Algorithm::Sha1,
            1 => Algorithm::Sha256,
            2 => Algorithm::Sha512,
            _ => Algorithm::Sha1,
        };

        let digits = match imp.digits_combo.selected() {
            0 => 6,
            1 => 7,
            2 => 8,
            _ => 6,
        };

        let require_touch = imp.touch_switch.is_active();

        Ok(NewCredential {
            issuer,
            account,
            secret,
            oath_type,
            algorithm,
            digits,
            require_touch,
            initial_counter: if oath_type == OathType::Hotp {
                Some(0)
            } else {
                None
            },
        })
    }

    /// Reset the form
    pub fn reset(&self) {
        let imp = self.imp();
        imp.issuer_entry.set_text("");
        imp.account_entry.set_text("");
        imp.secret_entry.set_text("");
        imp.type_combo.set_selected(0);
        imp.algorithm_combo.set_selected(0);
        imp.digits_combo.set_selected(0);
        imp.touch_switch.set_active(false);
        imp.add_button.set_sensitive(false);
    }

    pub fn connect_credential_added<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_local("credential-added", true, move |values| {
            let dialog = values[0].get::<Self>().unwrap();
            f(&dialog);
            None
        })
    }

    pub fn present(&self, parent: Option<&impl IsA<gtk4::Widget>>) {
        if let Some(parent) = parent {
            if let Some(window) = parent.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
                self.set_transient_for(Some(&window));
            }
        }
        self.set_modal(true);
        gtk4::prelude::GtkWindowExt::present(self);
    }
}

impl Default for AddCredentialDialog {
    fn default() -> Self {
        Self::new()
    }
}
