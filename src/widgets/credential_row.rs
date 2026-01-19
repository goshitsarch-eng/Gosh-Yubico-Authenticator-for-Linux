use glib::clone;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

use crate::models::CredentialObject;
use crate::widgets::CountdownIndicator;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default, gtk4::CompositeTemplate)]
    #[template(resource = "/com/github/gosh/authenticator/ui/credential-row.ui")]
    pub struct CredentialRow {
        #[template_child]
        pub countdown_indicator: TemplateChild<CountdownIndicator>,
        #[template_child]
        pub issuer_label: TemplateChild<gtk4::Label>,
        #[template_child]
        pub account_label: TemplateChild<gtk4::Label>,
        #[template_child]
        pub code_label: TemplateChild<gtk4::Label>,
        #[template_child]
        pub copy_button: TemplateChild<gtk4::Button>,
        #[template_child]
        pub refresh_hotp_button: TemplateChild<gtk4::Button>,
        #[template_child]
        pub delete_button: TemplateChild<gtk4::Button>,

        pub credential: RefCell<Option<CredentialObject>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CredentialRow {
        const NAME: &'static str = "GoshCredentialRow";
        type Type = super::CredentialRow;
        type ParentType = gtk4::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            CountdownIndicator::ensure_type();
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk4::template_callbacks]
    impl CredentialRow {
        #[template_callback]
        fn on_copy_clicked(&self) {
            self.obj().emit_by_name::<()>("copy-clicked", &[]);
        }

        #[template_callback]
        fn on_refresh_clicked(&self) {
            self.obj().emit_by_name::<()>("refresh-clicked", &[]);
        }

        #[template_callback]
        fn on_delete_clicked(&self) {
            self.obj().emit_by_name::<()>("delete-clicked", &[]);
        }
    }

    impl ObjectImpl for CredentialRow {
        fn signals() -> &'static [glib::subclass::Signal] {
            use std::sync::OnceLock;
            static SIGNALS: OnceLock<Vec<glib::subclass::Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    glib::subclass::Signal::builder("copy-clicked").build(),
                    glib::subclass::Signal::builder("refresh-clicked").build(),
                    glib::subclass::Signal::builder("delete-clicked").build(),
                ]
            })
        }

        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for CredentialRow {}
    impl ListBoxRowImpl for CredentialRow {}
}

glib::wrapper! {
    pub struct CredentialRow(ObjectSubclass<imp::CredentialRow>)
        @extends gtk4::ListBoxRow, gtk4::Widget;
}

impl CredentialRow {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn bind(&self, credential: &CredentialObject) {
        let imp = self.imp();

        // Store the credential
        imp.credential.replace(Some(credential.clone()));

        // Update labels
        self.update_display(credential);

        // Bind code changes
        credential.connect_notify_local(
            Some("code"),
            clone!(
                #[weak(rename_to = row)]
                self,
                move |cred, _| {
                    row.update_code(cred);
                }
            ),
        );

        // Show/hide HOTP refresh button
        imp.refresh_hotp_button.set_visible(credential.is_hotp());

        // Show/hide countdown for TOTP
        imp.countdown_indicator
            .set_visible(credential.is_totp() && !credential.touch_required());
    }

    pub fn unbind(&self) {
        self.imp().credential.replace(None);
    }

    fn update_display(&self, credential: &CredentialObject) {
        let imp = self.imp();

        // Set issuer (or hide if none)
        if let Some(issuer) = credential.issuer() {
            imp.issuer_label.set_text(&issuer);
            imp.issuer_label.set_visible(true);
        } else {
            imp.issuer_label.set_visible(false);
        }

        // Set account
        imp.account_label.set_text(&credential.account());

        // Set code
        self.update_code(credential);
    }

    fn update_code(&self, credential: &CredentialObject) {
        let imp = self.imp();

        if let Some(code) = credential.code() {
            imp.code_label.set_text(&code);
            imp.copy_button.set_sensitive(true);
        } else if credential.touch_required() {
            imp.code_label.set_text("Touch");
            imp.copy_button.set_sensitive(false);
        } else {
            imp.code_label.set_text("--- ---");
            imp.copy_button.set_sensitive(false);
        }
    }

    pub fn update_countdown(&self, remaining: u32, progress: f64) {
        self.imp().countdown_indicator.update(remaining, progress);
    }

    pub fn credential(&self) -> Option<CredentialObject> {
        self.imp().credential.borrow().clone()
    }

    pub fn connect_copy_clicked<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_local("copy-clicked", true, move |values| {
            let row = values[0].get::<Self>().unwrap();
            f(&row);
            None
        })
    }

    pub fn connect_refresh_clicked<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_local("refresh-clicked", true, move |values| {
            let row = values[0].get::<Self>().unwrap();
            f(&row);
            None
        })
    }

    pub fn connect_delete_clicked<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_local("delete-clicked", true, move |values| {
            let row = values[0].get::<Self>().unwrap();
            f(&row);
            None
        })
    }
}

impl Default for CredentialRow {
    fn default() -> Self {
        Self::new()
    }
}
