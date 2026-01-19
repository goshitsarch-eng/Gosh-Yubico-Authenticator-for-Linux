use glib::clone;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use libadwaita::subclass::prelude::*;

use crate::models::{CredentialList, CredentialObject};
use crate::services::{Event, TimerService, YubiKeyService};
use crate::widgets::{AddCredentialDialog, CredentialRow, PasswordDialog};

mod imp {
    use super::*;
    use std::cell::{Cell, OnceCell};

    #[derive(Default, gtk4::CompositeTemplate)]
    #[template(resource = "/com/github/gosh/authenticator/ui/window.ui")]
    pub struct GoshWindow {
        #[template_child]
        pub refresh_button: TemplateChild<gtk4::Button>,
        #[template_child]
        pub add_button: TemplateChild<gtk4::Button>,
        #[template_child]
        pub menu_button: TemplateChild<gtk4::MenuButton>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub main_stack: TemplateChild<gtk4::Stack>,
        #[template_child]
        pub credentials_list: TemplateChild<gtk4::ListBox>,
        #[template_child]
        pub add_first_button: TemplateChild<gtk4::Button>,
        #[template_child]
        pub loading_label: TemplateChild<gtk4::Label>,

        pub credentials: OnceCell<CredentialList>,
        pub yubikey_service: OnceCell<YubiKeyService>,
        pub timer_service: OnceCell<TimerService>,
        pub connected: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GoshWindow {
        const NAME: &'static str = "GoshWindow";
        type Type = super::GoshWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            CredentialRow::ensure_type();
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk4::template_callbacks]
    impl GoshWindow {
        #[template_callback]
        fn on_refresh_clicked(&self) {
            let obj = self.obj();
            if obj.imp().connected.get() {
                obj.refresh();
            } else {
                obj.try_connect();
            }
        }

        #[template_callback]
        fn on_add_clicked(&self) {
            self.obj().show_add_dialog();
        }

        #[template_callback]
        fn on_add_first_clicked(&self) {
            self.obj().show_add_dialog();
        }
    }

    impl ObjectImpl for GoshWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.setup_credentials_list();
            obj.setup_services();
        }

        fn dispose(&self) {
            if let Some(service) = self.yubikey_service.get() {
                service.shutdown();
            }
            if let Some(timer) = self.timer_service.get() {
                timer.stop();
            }
        }
    }

    impl WidgetImpl for GoshWindow {}
    impl WindowImpl for GoshWindow {}
    impl ApplicationWindowImpl for GoshWindow {}
    impl AdwApplicationWindowImpl for GoshWindow {}
}

glib::wrapper! {
    pub struct GoshWindow(ObjectSubclass<imp::GoshWindow>)
        @extends adw::ApplicationWindow, gtk4::ApplicationWindow, gtk4::Window, gtk4::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl GoshWindow {
    pub fn new<P: IsA<gtk4::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    fn setup_credentials_list(&self) {
        let imp = self.imp();

        let credentials = CredentialList::new();
        imp.credentials.set(credentials.clone()).unwrap();

        // Set up the list box with a factory
        imp.credentials_list.bind_model(
            Some(&credentials),
            clone!(
                #[weak(rename_to = window)]
                self,
                #[upgrade_or_panic]
                move |item| {
                    let credential = item
                        .downcast_ref::<CredentialObject>()
                        .expect("Expected CredentialObject");

                    let row = CredentialRow::new();
                    row.bind(credential);

                    // Connect row signals
                    row.connect_copy_clicked(clone!(
                        #[weak]
                        window,
                        move |row| {
                            if let Some(cred) = row.credential() {
                                window.copy_code(&cred);
                            }
                        }
                    ));

                    row.connect_refresh_clicked(clone!(
                        #[weak]
                        window,
                        move |row| {
                            if let Some(cred) = row.credential() {
                                window.calculate_credential(&cred);
                            }
                        }
                    ));

                    row.connect_delete_clicked(clone!(
                        #[weak]
                        window,
                        move |row| {
                            if let Some(cred) = row.credential() {
                                window.confirm_delete(&cred);
                            }
                        }
                    ));

                    row.upcast::<gtk4::Widget>()
                }
            ),
        );
    }

    fn setup_services(&self) {
        let imp = self.imp();

        // Create YubiKey service
        let service = YubiKeyService::new();
        let event_rx = service.event_receiver();
        imp.yubikey_service.set(service).unwrap();

        // Handle events from the YubiKey service
        glib::spawn_future_local(clone!(
            #[weak(rename_to = window)]
            self,
            async move {
                while let Ok(event) = event_rx.recv().await {
                    window.handle_event(event);
                }
            }
        ));

        // Create timer service for TOTP countdown
        let timer = TimerService::new(30);
        timer.start(
            clone!(
                #[weak(rename_to = window)]
                self,
                move || {
                    // Period changed - refresh credentials
                    window.refresh();
                }
            ),
            clone!(
                #[weak(rename_to = window)]
                self,
                move |remaining, progress| {
                    // Update countdown on all rows
                    window.update_countdown(remaining, progress);
                }
            ),
        );
        imp.timer_service.set(timer).unwrap();

        // Initial connection attempt
        self.try_connect();
    }

    fn service(&self) -> &YubiKeyService {
        self.imp().yubikey_service.get().unwrap()
    }

    fn credentials(&self) -> &CredentialList {
        self.imp().credentials.get().unwrap()
    }

    fn handle_event(&self, event: Event) {
        let imp = self.imp();

        match event {
            Event::Ready => {
                log::debug!("YubiKey service ready");
            }

            Event::Connected { version, device_id } => {
                log::info!(
                    "Connected to YubiKey v{}.{}.{} (ID: {:02X?})",
                    version.0,
                    version.1,
                    version.2,
                    device_id
                );
                imp.connected.set(true);
                imp.add_button.set_sensitive(true);
            }

            Event::Disconnected => {
                log::info!("YubiKey disconnected");
                imp.connected.set(false);
                imp.add_button.set_sensitive(false);
                self.credentials().set_credentials(vec![]);
                self.show_no_device();
            }

            Event::AuthenticationRequired => {
                self.show_password_dialog();
            }

            Event::AuthenticationSuccessful => {
                self.show_toast("Unlocked successfully");
            }

            Event::AuthenticationFailed(msg) => {
                self.show_toast(&format!("Authentication failed: {}", msg));
                self.show_password_dialog();
            }

            Event::CredentialsUpdated(credentials) => {
                if credentials.is_empty() {
                    self.show_no_credentials();
                } else {
                    self.credentials().set_credentials(credentials);
                    self.show_credentials();
                }
            }

            Event::CredentialCalculated { id, code, .. } => {
                self.credentials().update_code(&id, Some(code));
            }

            Event::TouchRequired(_id) => {
                self.show_touch_required();
                self.show_toast("Touch your YubiKey");
            }

            Event::CredentialAdded(credential) => {
                self.show_toast(&format!("Added {}", credential.display_name()));
            }

            Event::CredentialDeleted(_id) => {
                self.show_toast("Credential deleted");
            }

            Event::Error(msg) => {
                log::error!("YubiKey error: {}", msg);

                // Show user-friendly messages for common errors
                if msg.contains("resource manager") || msg.contains("not running") {
                    self.show_toast("PC/SC service not running. Start pcscd.");
                } else if msg.contains("No YubiKey") || msg.contains("No device") {
                    // Don't show toast for "no device" - the UI state is enough
                } else {
                    self.show_toast(&format!("Error: {}", msg));
                }

                self.show_no_device();
            }
        }
    }

    fn show_loading(&self, message: &str) {
        self.imp().loading_label.set_text(message);
        self.imp().main_stack.set_visible_child_name("loading");
    }

    fn show_no_device(&self) {
        self.imp().main_stack.set_visible_child_name("no-device");
    }

    fn show_no_credentials(&self) {
        self.imp()
            .main_stack
            .set_visible_child_name("no-credentials");
    }

    fn show_credentials(&self) {
        self.imp().main_stack.set_visible_child_name("credentials");
    }

    fn show_touch_required(&self) {
        self.imp()
            .main_stack
            .set_visible_child_name("touch-required");
    }

    fn show_toast(&self, message: &str) {
        let toast = adw::Toast::new(message);
        self.imp().toast_overlay.add_toast(toast);
    }

    fn refresh(&self) {
        if self.imp().connected.get() {
            self.service().refresh();
        }
        // Don't auto-reconnect on timer - only on user action
    }

    fn try_connect(&self) {
        if !self.imp().connected.get() {
            self.show_loading("Connecting to YubiKey...");
            self.service().connect();
        }
    }

    fn update_countdown(&self, remaining: u32, progress: f64) {
        // Update all credential rows with the new countdown
        let list = &self.imp().credentials_list;
        let mut i = 0;
        while let Some(row) = list.row_at_index(i) {
            if let Some(cred_row) = row.downcast_ref::<CredentialRow>() {
                cred_row.update_countdown(remaining, progress);
            }
            i += 1;
        }
    }

    fn copy_code(&self, credential: &CredentialObject) {
        if let Some(code) = credential.code() {
            // Remove spaces from code for clipboard
            let clean_code = code.replace(' ', "");
            let clipboard = self.clipboard();
            clipboard.set_text(&clean_code);

            let name = credential.issuer().unwrap_or_else(|| credential.account());
            self.show_toast(&format!("Copied code for {}", name));
        }
    }

    fn calculate_credential(&self, credential: &CredentialObject) {
        self.service().calculate(credential.id());
    }

    fn show_add_dialog(&self) {
        let dialog = AddCredentialDialog::new();

        dialog.connect_credential_added(clone!(
            #[weak(rename_to = window)]
            self,
            move |dialog| {
                match dialog.get_credential() {
                    Ok(cred) => {
                        window.service().add_credential(cred);
                        dialog.close();
                    }
                    Err(e) => {
                        window.show_toast(&e);
                    }
                }
            }
        ));

        dialog.present(Some(self));
    }

    fn show_password_dialog(&self) {
        let dialog = PasswordDialog::new();

        glib::spawn_future_local(clone!(
            #[weak(rename_to = window)]
            self,
            #[strong]
            dialog,
            async move {
                if let Some(password) = dialog.run(&window).await {
                    window.service().authenticate(password);
                } else {
                    window.show_no_device();
                }
            }
        ));
    }

    fn confirm_delete(&self, credential: &CredentialObject) {
        let dialog = adw::MessageDialog::builder()
            .heading("Delete Credential?")
            .body(format!(
                "Are you sure you want to delete \"{}\"? This cannot be undone.",
                credential.display_name()
            ))
            .transient_for(self)
            .modal(true)
            .build();

        dialog.add_response("cancel", "Cancel");
        dialog.add_response("delete", "Delete");
        dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
        dialog.set_default_response(Some("cancel"));
        dialog.set_close_response("cancel");

        let id = credential.id();
        dialog.connect_response(
            None,
            clone!(
                #[weak(rename_to = window)]
                self,
                move |_, response| {
                    if response == "delete" {
                        window.service().delete_credential(id.clone());
                    }
                }
            ),
        );

        dialog.present();
    }
}

impl Default for GoshWindow {
    fn default() -> Self {
        GoshWindow::new(&gtk4::Application::default())
    }
}
